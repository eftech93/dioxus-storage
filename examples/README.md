# Dioxus Storage Examples

This directory contains example applications demonstrating the dioxus-storage crates.

## Available Examples

### 1. Demo (`demo/`)

A comprehensive demo showing:
- **LocalStorage** - Persistent key-value storage
- **SessionStorage** - Per-session storage
- **IndexedDB with Indexes** - Structured storage with indexed queries

Features demonstrated:
- Creating databases with indexes
- CRUD operations on IndexedDB
- Filtering by index (priority-based task filtering)
- Reactive storage hooks

## Running the Examples

### Prerequisites

1. Install Dioxus CLI:
```bash
cargo install dioxus-cli
```

2. Install wasm32 target:
```bash
rustup target add wasm32-unknown-unknown
```

### Run the Demo

```bash
cd examples/demo

# Serve with hot reload
dx serve --platform web

# Or build for production
dx build --platform web --release
```

Then open http://localhost:8080 in your browser.

### Demo Features

#### IndexedDB with Indexes
The demo showcases the new index support:

1. **Creating a store with an index**:
```rust
let config = DatabaseConfig::new("demo_db_v2", 1)
    .with_store_and_indexes(
        "tasks", 
        "id",
        vec![
            IndexConfig::new("priority_idx", "priority", false),
        ]
    );
```

2. **Querying by index**:
```rust
// Get all high priority tasks using the index
let high_priority_tasks = collection
    .get_by_index("priority_idx", "high")
    .await?;
```

3. **Visual priority indicators**:
- 🔴 High priority (red border)
- 🟡 Medium priority (yellow border)
- 🟢 Low priority (green border)

#### LocalStorage Demo
- Theme selector (light/dark/auto)
- Username persistence
- Counter with increment/decrement

#### SessionStorage Demo
- Session token generation
- Temporary notes (lost when tab closes)

## Example Code

### Basic IndexedDB Usage

```rust
use dioxus_indexeddb::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
struct Task {
    id: String,
    title: String,
    priority: String,
}

async fn example() -> Result<(), IndexedDbError> {
    // Open database with index
    let db = Database::open(
        DatabaseConfig::new("my_app", 1)
            .with_store_and_indexes(
                "tasks", 
                "id",
                vec![
                    IndexConfig::new("priority_idx", "priority", false),
                ]
            )
    ).await?;
    
    let collection = db.collection::<Task>("tasks");
    
    // Add task
    let task = Task {
        id: "1".to_string(),
        title: "Important task".to_string(),
        priority: "high".to_string(),
    };
    collection.put(&task.id, &task).await?;
    
    // Query by index
    let high_priority = collection
        .get_by_index("priority_idx", "high")
        .await?;
    
    println!("Found {} high priority tasks", high_priority.len());
    
    Ok(())
}
```

### Using with Dioxus Components

```rust
#[component]
fn TaskList() -> Element {
    let db = use_db(
        DatabaseConfig::new("my_app", 1)
            .with_store_and_indexes("tasks", "id", vec![
                IndexConfig::new("priority_idx", "priority", false),
            ])
    );
    
    let tasks = use_collection::<Task>(db, "tasks");
    
    rsx! {
        div {
            for task in tasks.read().iter() {
                p { "{task.title}" }
            }
        }
    }
}
```

## Building All Examples

```bash
# From the workspace root
cargo build --workspace

# Check all examples compile
cargo check --workspace --exclude dioxus-client-storage-demo --exclude sync-demo

# Check examples with WASM target
cargo check -p dioxus-client-storage-demo --target wasm32-unknown-unknown
cargo check -p sync-demo --target wasm32-unknown-unknown
```

## Troubleshooting

### "can't have any crate-types set" error

If you see this error, make sure the example's `Cargo.toml` doesn't have `crate-type` for the binary:

```toml
# Wrong:
[[bin]]
name = "demo"
crate-type = ["cdylib", "bin"]  # <- Remove this line

# Correct:
[[bin]]
name = "demo"
```

### "prelude not found" errors

Make sure you're compiling with the WASM target:
```bash
cargo check --target wasm32-unknown-unknown
```

The crates use `#![cfg(target_arch = "wasm32")]` which makes them empty on non-WASM targets.

## License

Same as the main project: MIT OR Apache-2.0
