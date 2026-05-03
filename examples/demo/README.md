# Dioxus Storage Demo

A comprehensive demo showcasing all storage layers in the `dioxus-storage` workspace.

## Features

### 📦 LocalStorage
- Theme selector (light / dark / auto)
- Username persistence across sessions
- Counter with increment / decrement

### ⏱️ SessionStorage
- Session token generation
- Temporary notes (lost when the tab closes)

### 🗄️ IndexedDB with Indexes
- Full CRUD on an IndexedDB object store
- **Index-based filtering** — query tasks by priority using a `priority_idx` index
- Reactive hooks that sync UI with storage

### 🖱️ Cursor Demo (New in v0.0.3)
- Iterate large datasets **without loading everything into memory**
- Three iteration modes:
  - **Forward** — `CursorDirection::Next` with `cursor.next().await`
  - **Backward** — `CursorDirection::Prev`
  - **Stream API** — `cursor.into_stream()` + `futures::StreamExt::collect()`

## Quick Start

### Prerequisites

```bash
# Dioxus CLI
cargo install dioxus-cli

# WASM target
rustup target add wasm32-unknown-unknown
```

### Run

```bash
cd examples/demo

# Serve with hot reload
dx serve --platform web

# Or build for production
dx build --platform web --release
```

Open the URL printed by `dx` (usually `http://localhost:8080`).

## What to Try

### LocalStorage
1. Change the theme dropdown — the value is written to `localStorage` instantly.
2. Type a username and refresh the page — it persists.
3. Click `-` / `+` on the counter and refresh — the count is preserved.

### SessionStorage
1. Click **Generate** to create a session token.
2. Type notes in the textarea.
3. Open the same page in a new tab — the token and notes are gone (per-session only).

### IndexedDB
1. Fill in the form and click **➕ Add Task** — tasks are stored in IndexedDB.
2. Use the **Filter by Priority** buttons — this queries the `priority_idx` index directly.
3. Toggle completion or delete tasks — changes are persisted immediately.

### Cursor Demo
1. The demo seeds 5 items (`Item 1` … `Item 5`) into a dedicated IndexedDB store on load.
2. Click **⏩ Iterate Forward** — walks the cursor in `Next` order and prints the sequence.
3. Click **⏪ Iterate Backward** — walks the cursor in `Prev` order.
4. Click **🌊 Stream API** — converts the cursor into a `futures::Stream` and collects all labels.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Demo App (Dioxus)                         │
│                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ LocalStorage│  │SessionStorage│  │    IndexedDB        │  │
│  │  (sync)     │  │   (sync)     │  │  - CRUD             │  │
│  │  - theme    │  │  - token     │  │  - Index queries    │  │
│  │  - counter  │  │  - notes     │  │  - Cursor iteration │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Code Snippets

### Opening a cursor

```rust
use dioxus_indexeddb::prelude::*;

let collection: Collection<Item> = db.collection("items");

// Forward iteration
let mut cursor = collection
    .open_cursor(None, Some(CursorDirection::Next))
    .await?;

while let Some(item) = cursor.next().await? {
    println!("{}", item.label);
}
```

### Cursor as a Stream

```rust
use futures::StreamExt;

let cursor = collection
    .open_cursor(None, Some(CursorDirection::Next))
    .await?;

let labels: Vec<String> = cursor
    .into_stream()
    .filter_map(|r| async move { r.ok().map(|i| i.label) })
    .collect()
    .await;
```

### Creating a store with an index

```rust
let config = DatabaseConfig::new("demo_db_v2", 1)
    .with_store_and_indexes(
        "tasks",
        "id",
        vec![
            IndexConfig::new("priority_idx", "priority", false),
        ]
    );

let db = Database::open(config).await?;
```

## File Structure

```
demo/
├── Cargo.toml
└── src/
    └── main.rs          # All demo components (LocalStorage, SessionStorage,
                         # IndexedDB, CursorDemo) + inline CSS
```

## Troubleshooting

### "can't have any crate-types set"
Make sure `Cargo.toml` does **not** contain `crate-type` for the binary target.

### "prelude not found"
These crates are gated with `#![cfg(target_arch = "wasm32")]`. Always compile with:
```bash
cargo check --target wasm32-unknown-unknown
dx serve --platform web
```

## License

Same as the main project: MIT OR Apache-2.0
