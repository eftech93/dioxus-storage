#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# =============================================================================
# Configuration
# =============================================================================

CRATES=(
    "dioxus-indexeddb"
    "dioxus-client-storage"
    "dioxus-storage-sync"
)

# Publish order (respects internal dependencies)
PUBLISH_ORDER=(
    "dioxus-indexeddb"
    "dioxus-client-storage"
    "dioxus-storage-sync"
)

# Seconds to wait between publishes for crates.io indexing
INDEXING_WAIT=45

# =============================================================================
# Helpers
# =============================================================================

get_version() {
    local crate_dir="$1"
    grep '^version' "$crate_dir/Cargo.toml" | head -1 | sed 's/.*= *"\(.*\)".*/\1/'
}

get_workspace_version() {
    grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/.*= *"\(.*\)".*/\1/'
}

# =============================================================================
# Version Consistency Check
# =============================================================================

echo "========================================"
echo "🔍 Version Consistency Check"
echo "========================================"
echo ""

WORKSPACE_VERSION=$(get_workspace_version)
echo "Workspace version: $WORKSPACE_VERSION"
echo ""

ALL_VERSIONS=()
MISMATCHED=()

for crate in "${CRATES[@]}"; do
    crate_version=$(get_version "$PROJECT_ROOT/$crate")
    ALL_VERSIONS+=("$crate_version")
    echo "  $crate: $crate_version"

    if [ "$crate_version" != "$WORKSPACE_VERSION" ]; then
        MISMATCHED+=("$crate")
    fi
done

echo ""

# Check if all crate versions match each other
UNIQUE_VERSIONS=$(printf "%s\n" "${ALL_VERSIONS[@]}" | sort -u | wc -l | tr -d ' ')

if [ "$UNIQUE_VERSIONS" -ne 1 ]; then
    echo "❌ ERROR: Crate versions are not in sync!"
    echo ""
    echo "All crates in this workspace must share the same version number."
    echo "Please bump every crate to the same version before publishing."
    echo ""
    echo "You can use the following command to sync all versions:"
    echo "  sed -i '' 's/^version = \"[^\"]*\"/version = \"X.Y.Z\"/' \"\""
    for crate in "${CRATES[@]}"; do
        echo "    $crate/Cargo.toml"
    done
    echo "    Cargo.toml"
    echo ""
    exit 1
fi

# Check if all crate versions match the workspace version
if [ "${#MISMATCHED[@]}" -gt 0 ]; then
    echo "❌ ERROR: The following crates do not match the workspace version ($WORKSPACE_VERSION):"
    for crate in "${MISMATCHED[@]}"; do
        echo "  - $crate"
    done
    echo ""
    echo "Please ensure workspace Cargo.toml and all crate Cargo.toml files use the same version."
    exit 1
fi

TARGET_VERSION="${ALL_VERSIONS[0]}"
TAG="v$TARGET_VERSION"

echo "✅ All versions in sync: $TARGET_VERSION"
echo ""

# =============================================================================
# Git Tag Check
# =============================================================================

echo "========================================"
echo "🏷️  Git Tag Check"
echo "========================================"
echo ""

if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "❌ ERROR: Git tag '$TAG' already exists!"
    echo ""
    echo "If you want to re-publish, first delete the tag:"
    echo "  git tag -d $TAG"
    echo "  git push origin :refs/tags/$TAG"
    echo ""
    exit 1
fi

if git ls-remote --tags origin "$TAG" | grep -q "$TAG"; then
    echo "❌ ERROR: Git tag '$TAG' already exists on origin!"
    echo ""
    echo "If you want to re-publish, first delete the remote tag:"
    echo "  git push origin :refs/tags/$TAG"
    echo ""
    exit 1
fi

echo "✅ Tag '$TAG' is available"
echo ""

# =============================================================================
# Working Tree Check
# =============================================================================

echo "========================================"
echo "🌳 Working Tree Check"
echo "========================================"
echo ""

if ! git diff-index --quiet HEAD --; then
    echo "❌ ERROR: Working directory is not clean!"
    echo ""
    echo "Please commit or stash all changes before publishing:"
    echo "  git add -A"
    echo "  git commit -m 'Prepare release v$TARGET_VERSION'"
    echo ""
    exit 1
fi

echo "✅ Working tree is clean"
echo ""

# =============================================================================
# Pre-flight Checks
# =============================================================================

echo "========================================"
echo "🔍 Pre-flight Checks"
echo "========================================"
echo ""

echo "🔍 Formatting code..."
cargo fmt
echo ""

echo "🔍 Checking formatting..."
cargo fmt --check
echo ""

echo "🔍 Running clippy..."
for crate in "${CRATES[@]}"; do
    echo "  → $crate"
    cargo clippy --package "$crate" -- -D warnings
done
echo ""

echo "🧪 Running tests..."
cargo test --workspace
echo ""

echo "📚 Checking documentation..."
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace
echo ""

# =============================================================================
# Publish Dry Run
# =============================================================================

echo "========================================"
echo "📦 Publish Dry Run"
echo "========================================"
echo ""

for crate in "${PUBLISH_ORDER[@]}"; do
    echo "📦 Dry-run $crate..."
    cd "$PROJECT_ROOT/$crate"
    cargo publish --dry-run
    echo ""
done

echo "✅ All dry-runs passed"
echo ""

# =============================================================================
# Confirm Publish
# =============================================================================

echo "========================================"
echo "🚀 Ready to Publish!"
echo "========================================"
echo ""
echo "Author:    Esteban Puello <eftech93@gmail.com>"
echo "Version:   $TARGET_VERSION"
echo "Tag:       $TAG"
echo "Crates:"
for crate in "${PUBLISH_ORDER[@]}"; do
    echo "  - $crate"
done
echo ""
echo "Order:"
i=1
for crate in "${PUBLISH_ORDER[@]}"; do
    deps=""
    case "$crate" in
        dioxus-client-storage)
            deps=" (depends on dioxus-indexeddb)"
            ;;
        dioxus-storage-sync)
            deps=" (depends on dioxus-indexeddb)"
            ;;
    esac
    echo "  $i. $crate$deps"
    ((i++))
done
echo ""
read -p "Continue with publish? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Publish cancelled"
    exit 1
fi

# =============================================================================
# Publish
# =============================================================================

echo ""
echo "========================================"
echo "🚀 Publishing"
echo "========================================"
echo ""

for crate in "${PUBLISH_ORDER[@]}"; do
    echo "📦 Publishing $crate v$TARGET_VERSION..."
    cd "$PROJECT_ROOT/$crate"
    cargo publish

    if [ "$crate" != "${PUBLISH_ORDER[-1]}" ]; then
        echo "⏳ Waiting ${INDEXING_WAIT}s for crates.io to index $crate..."
        sleep "$INDEXING_WAIT"
    fi
    echo ""
done

# =============================================================================
# Git Tag
# =============================================================================

echo "========================================"
echo "🏷️  Creating Git Tag"
echo "========================================"
echo ""

git tag -a "$TAG" -m "Release $TAG"
git push origin "$TAG"

echo "✅ Created and pushed tag $TAG"
echo ""

# =============================================================================
# Summary
# =============================================================================

echo "========================================"
echo "✅ Publish Complete!"
echo "========================================"
echo ""
echo "Published version: $TARGET_VERSION"
echo "Git tag:           $TAG"
echo ""
echo "Crates:"
for crate in "${PUBLISH_ORDER[@]}"; do
    echo "  ✓ $crate"
done
echo ""
echo "View on crates.io:"
for crate in "${PUBLISH_ORDER[@]}"; do
    echo "  https://crates.io/crates/$crate"
done
echo ""
