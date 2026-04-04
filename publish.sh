#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🔍 Formatting code..."
cargo fmt

echo "🔍 Running clippy (WASM target)..."
cargo clippy --target wasm32-unknown-unknown --package dioxus-indexeddb
cargo clippy --target wasm32-unknown-unknown --package dioxus-storage
cargo clippy --target wasm32-unknown-unknown --package dioxus-storage-sync

echo "🧪 Running tests..."
cargo test --workspace

echo "📦 Verifying dioxus-indexeddb..."
cd "$SCRIPT_DIR/dioxus-indexeddb"
cargo publish --dry-run --allow-dirty --target wasm32-unknown-unknown

echo "📦 Verifying dioxus-storage..."
cd "$SCRIPT_DIR/dioxus-storage"
cargo publish --dry-run --allow-dirty --target wasm32-unknown-unknown

echo "📦 Verifying dioxus-storage-sync..."
cd "$SCRIPT_DIR/dioxus-storage-sync"
cargo publish --dry-run --allow-dirty --target wasm32-unknown-unknown

echo ""
echo "========================================"
echo "🚀 Ready to publish!"
echo "========================================"
echo ""
echo "The following crates will be published:"
echo "  1. dioxus-indexeddb"
echo "  2. dioxus-storage (depends on dioxus-indexeddb)"
echo "  3. dioxus-storage-sync (depends on dioxus-indexeddb)"
echo ""
read -p "Continue with publish? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Publish cancelled"
    exit 1
fi

echo "📦 Publishing dioxus-indexeddb..."
cd "$SCRIPT_DIR/dioxus-indexeddb"
cargo publish --allow-dirty

echo "⏳ Waiting for crates.io to index dioxus-indexeddb..."
sleep 45

echo "📦 Publishing dioxus-storage..."
cd "$SCRIPT_DIR/dioxus-storage"
cargo publish --allow-dirty

echo "⏳ Waiting for crates.io to index dioxus-storage..."
sleep 45

echo "📦 Publishing dioxus-storage-sync..."
cd "$SCRIPT_DIR/dioxus-storage-sync"
cargo publish --allow-dirty

echo ""
echo "✅ All 3 crates published successfully!"
echo ""
echo "Published:"
echo "  - dioxus-indexeddb"
echo "  - dioxus-storage"
echo "  - dioxus-storage-sync"
