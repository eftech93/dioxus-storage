# dioxus-storage-sync

[![Crates.io](https://img.shields.io/crates/v/dioxus-storage-sync.svg)](https://crates.io/crates/dioxus-storage-sync)
[![Docs.rs](https://docs.rs/dioxus-storage-sync/badge.svg)](https://docs.rs/dioxus-storage-sync)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

Two-way synchronization between local IndexedDB and backend API for Dioxus applications.

## Features

- 🔥 **Hot Sync** - On-demand fetching with local cache
- 🌙 **Background Sync** - Periodic synchronization
- 📊 **Conflict Resolution** - Handle simultaneous updates
- 🔄 **Bidirectional** - Push local changes to server
- 🪝 **Reactive hooks** - `use_sync` for sync state
- ⚡ **Optimistic UI** - Show cached data instantly

## Installation

```toml
[dependencies]
dioxus-storage-sync = "0.0.1"
```

## Quick Start

```rust
use dioxus::prelude::*;
use dioxus_storage_sync::prelude::*;
use serde::{Serialize, Deserialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
    id: String,
    name: String,
    price: f64,
}

impl Syncable for Product {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn version(&self) -> u64 {
        0
    }
}

#[component]
fn ProductList() -> Element {
    let config = SyncConfig::new("https://api.example.com")
        .with_collection("products")
        .with_hot_sync(true)
        .with_background_sync(Duration::from_secs(30));
    
    let sync = use_sync::<Product>(config);
    
    rsx! {
        div {
            // Sync status
            if sync.status.read().is_syncing {
                "🔄 Syncing..."
            }
            
            // Product list
            ul {
                for product in sync.data.read().iter() {
                    li { "{product.name} - ${product.price}" }
                }
            }
            
            // Manual sync button
            button {
                onclick: move |_| sync.sync_now(),
                "🔄 Sync Now"
            }
        }
    }
}
```

## Sync Modes

### Hot Sync (Default)

Returns cached data immediately, fetches fresh data in background:

```rust
let config = SyncConfig::new("https://api.example.com")
    .with_hot_sync(true)
    .with_cache_duration(Duration::from_secs(300)); // 5 minutes
```

### Background Sync

Automatically syncs data periodically:

```rust
let config = SyncConfig::new("https://api.example.com")
    .with_background_sync(Duration::from_secs(30));
```

### Manual Sync

Full control over when to sync:

```rust
let engine = SyncEngine::new(collection, config);

// Pull from server
let result = engine.pull().await?;

// Push to server
let result = engine.push().await?;

// Bidirectional sync
let result = engine.sync().await?;
```

## Configuration

```rust
use dioxus_storage_sync::prelude::*;

let config = SyncConfig::new("https://api.example.com")
    // Sync mode
    .with_mode(SyncMode::Hot)
    
    // Endpoints
    .with_endpoint("products", "/api/products")
    
    // Timing
    .with_retry_count(3)
    .with_timeout(Duration::from_secs(30))
    
    // Conflict resolution
    .with_conflict_resolution(ConflictResolution::ServerWins);
```

## Conflict Resolution Strategies

| Strategy | Behavior |
|----------|----------|
| `ServerWins` | Always use server version |
| `ClientWins` | Always use local version |
| `LastWriteWins` | Use most recent timestamp |
| `Manual` | Custom merge logic |

```rust
let config = SyncConfig::new("https://api.example.com")
    .with_conflict_resolution(ConflictResolution::Manual)
    .with_conflict_handler(|local, remote| {
        // Custom merge logic
        merge_documents(local, remote)
    });
```

## Complete Example

```rust
use dioxus::prelude::*;
use dioxus_storage_sync::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    id: String,
    title: String,
    completed: bool,
    updated_at: u64,
}

impl Syncable for Task {
    fn id(&self) -> String { self.id.clone() }
    fn version(&self) -> u64 { self.updated_at }
}

#[component]
fn TaskApp() -> Element {
    let config = SyncConfig::new("http://localhost:3001/api")
        .with_collection("tasks")
        .with_hot_sync(true);
    
    let sync = use_sync::<Task>(config);
    let mut new_title = use_signal(String::new);
    
    rsx! {
        div {
            h1 { "Tasks" }
            
            // Add task
            input {
                value: "{new_title.read()}",
                oninput: move |e| new_title.set(e.value()),
            }
            button {
                onclick: move |_| {
                    let task = Task {
                        id: uuid::Uuid::new_v4().to_string(),
                        title: new_title.read().clone(),
                        completed: false,
                        updated_at: js_sys::Date::now() as u64,
                    };
                    sync.add_local(task);
                    new_title.set(String::new());
                },
                "Add Task"
            }
            
            // List tasks
            ul {
                for task in sync.data.read().iter() {
                    li {
                        input {
                            r#type: "checkbox",
                            checked: task.completed,
                            onchange: move |_| {
                                sync.update_local(&task.id, |t| {
                                    t.completed = !t.completed;
                                });
                            }
                        }
                        "{task.title}"
                    }
                }
            }
        }
    }
}
```

## API Overview

### SyncEngine

| Method | Description |
|--------|-------------|
| `pull()` | Fetch changes from server |
| `push()` | Push local changes to server |
| `sync()` | Bidirectional sync |
| `hot_sync(query)` | Query with cache fallback |
| `start_background()` | Start periodic sync |
| `stop_background()` | Stop periodic sync |

### SyncHandle (from use_sync hook)

| Property/Method | Description |
|-----------------|-------------|
| `data: Signal<Vec<T>>` | Access synced data |
| `status: Signal<SyncStatus>` | Check sync status |
| `sync_now()` | Trigger manual sync |
| `add_local(item)` | Add item locally |
| `update_local(id, f)` | Update item locally |
| `delete_local(id)` | Delete item locally |

## License

MIT OR Apache-2.0
