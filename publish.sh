#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🔍 Formatting code..."
cargo fmt

echo "🔍 Running clippy..."
cargo clippy --package dioxus-indexeddb
cargo clippy --package dioxus-client-storage
cargo clippy --package dioxus-storage-sync

echo "🧪 Running tests..."
cargo test --workspace --exclude dioxus-client-storage-demo --exclude sync-demo

echo "📦 Verifying dioxus-indexeddb..."
cd "$SCRIPT_DIR/dioxus-indexeddb"
cargo publish --dry-run

echo "📦 Verifying dioxus-client-storage..."
cd "$SCRIPT_DIR/dioxus-client-storage"
cargo publish --dry-run

echo "📦 Verifying dioxus-storage-sync..."
cd "$SCRIPT_DIR/dioxus-storage-sync"
cargo publish --dry-run

echo ""
echo "========================================"
echo "🚀 Ready to publish!"
echo "========================================"
echo ""
echo "The following crates will be published:"
echo "  1. dioxus-indexeddb"
echo "  2. dioxus-client-storage (depends on dioxus-indexeddb)"
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
cargo publish

echo "⏳ Waiting for crates.io to index dioxus-indexeddb..."
sleep 45

echo "📦 Publishing dioxus-client-storage..."
cd "$SCRIPT_DIR/dioxus-client-storage"
cargo publish

echo "⏳ Waiting for crates.io to index dioxus-client-storage..."
sleep 45

echo "📦 Publishing dioxus-storage-sync..."
cd "$SCRIPT_DIR/dioxus-storage-sync"
cargo publish

echo ""
echo "✅ All 3 crates published successfully!"
echo ""
echo "Published:"
echo "  - dioxus-indexeddb"
echo "  - dioxus-client-storage"
echo "  - dioxus-storage-sync"
