# dioxus-storage-sync

Two-way synchronization between local IndexedDB and backend API.

## Installation

```toml
[dependencies]
dioxus-storage-sync = "0.0.1"
```

## Features

- 🔥 **Hot Sync** - On-demand fetching with local cache
- 🌙 **Background Sync** - Periodic synchronization
- 📊 **Conflict Resolution** - Handle simultaneous updates
- 🔄 **Bidirectional** - Push local changes to server
- 📴 **Offline Queue** - Queue mutations when offline and replay when restored

## Basic Usage

```rust
use dioxus_storage_sync::prelude::*;
use serde::{Serialize, Deserialize};

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
        0 // Or use a timestamp/version field
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
            button {
                onclick: move |_| {
                    sync.start_background_sync();
                },
                "Start Background Sync"
            }
            
            ul {
                for product in sync.data.read().iter() {
                    li { "{product.name} - ${product.price}" }
                }
            }
        }
    }
}
```

## Sync Modes

### Hot Sync

Checks local cache first, fetches from backend only if needed:

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

// Pull all changes from server
let result = engine.pull().await?;

// Push local changes to server
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
    .with_endpoint("users", "/api/users")
    
    // Timing
    .with_retry_count(3)
    .with_timeout(Duration::from_secs(30))
    
    // Conflict resolution
    .with_conflict_resolution(ConflictResolution::ServerWins);
```

## Conflict Resolution Strategies

| Strategy | Description |
|----------|-------------|
| `ServerWins` | Always use server version |
| `ClientWins` | Always use local version |
| `LastWriteWins` | Use the most recent timestamp |
| `Manual` | Call conflict handler callback |

```rust
let config = SyncConfig::new("https://api.example.com")
    .with_conflict_resolution(ConflictResolution::Manual)
    .with_conflict_handler(|local, remote| {
        // Custom merge logic
        merge_documents(local, remote)
    });
```

## Offline Queue

The sync manager automatically detects when the browser goes offline and queues mutations. When connectivity is restored, the queue is replayed automatically during the next background sync.

### How It Works

- **Save/Delete** operations check the browser's online status.
- If **offline**, the operation is stored in a dedicated IndexedDB queue.
- When the browser comes **back online**, the queue is replayed against the backend.
- **Conflicts** during replay are handled according to the configured `ConflictResolution`.

### Manual Replay

```rust
let manager = use_sync_manager(collection, config);

// Manually trigger queue replay
let result = manager.replay_queue().await?;
println!("Replayed: {}, Failed: {}, Conflicts: {}",
    result.success, result.failed, result.conflicts);
```

### Queue Status in UI

```rust
#[component]
fn QueueStatus() -> Element {
    let manager = use_sync_manager(collection, config);
    let status = manager.status.read();

    rsx! {
        div { class: "queue-status",
            if !status.is_online {
                span { "📴 Offline — {} operations queued", status.queue_pending }
            } else if status.queue_replaying {
                span { "🔄 Replaying queue..." }
            } else if status.queue_pending > 0 {
                span { "⏳ {} operations pending", status.queue_pending }
            } else {
                span { "✅ Queue empty" }
            }
        }
    }
}
```

## Sync Status

Monitor sync operations in real-time:

```rust
#[component]
fn SyncStatus() -> Element {
    let sync = use_sync::<Product>(config);
    let status = sync.status.read();
    
    rsx! {
        div { class: "sync-status",
            if status.is_syncing {
                span { "🔄 Syncing..." }
            } else if let Some(error) = &status.error {
                span { class: "error", "❌ {error}" }
            } else if let Some(result) = &status.last_result {
                span { "✅ Last sync: {result.items_synced} items" }
            }
        }
    }
}
```

## Complete Example

```rust
use dioxus::prelude::*;
use dioxus_storage_sync::prelude::*;
use serde::{Serialize, Deserialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    id: String,
    title: String,
    completed: bool,
    updated_at: u64,
}

impl Syncable for Task {
    fn id(&self) -> String {
        self.id.clone()
    }
    
    fn version(&self) -> u64 {
        self.updated_at
    }
    
    fn is_dirty(&self) -> bool {
        // Check if local changes need to be synced
        true
    }
    
    fn mark_synced(&mut self) {
        // Mark as synced
    }
}

#[component]
fn TaskApp() -> Element {
    let config = SyncConfig::new("http://localhost:3001/api")
        .with_collection("tasks")
        .with_hot_sync(true)
        .with_background_sync(Duration::from_secs(60));
    
    let sync = use_sync::<Task>(config);
    let mut new_task_title = use_signal(String::new);
    
    rsx! {
        div { class: "task-app",
            h1 { "Tasks" }
            
            // Sync status
            div { class: "status",
                if sync.status.read().is_syncing {
                    "🔄 Syncing..."
                } else {
                    "✅ Synced"
                }
            }
            
            // Add task
            input {
                value: "{new_task_title.read()}",
                oninput: move |e| new_task_title.set(e.value()),
                placeholder: "New task..."
            }
            button {
                onclick: move |_| {
                    let task = Task {
                        id: uuid::Uuid::new_v4().to_string(),
                        title: new_task_title.read().clone(),
                        completed: false,
                        updated_at: js_sys::Date::now() as u64,
                    };
                    sync.add_local(task);
                    new_task_title.set(String::new());
                },
                "Add Task"
            }
            
            // Task list
            ul {
                for task in sync.data.read().iter() {
                    li {
                        input {
                            r#type: "checkbox",
                            checked: task.completed,
                            onchange: move |_| {
                                sync.update_local(&task.id, |t| {
                                    t.completed = !t.completed;
                                    t.updated_at = js_sys::Date::now() as u64;
                                });
                            }
                        }
                        span { "{task.title}" }
                    }
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

## API Reference

### SyncEngine

| Method | Description |
|--------|-------------|
| `pull() -> Result<SyncResult>` | Fetch changes from server |
| `push() -> Result<SyncResult>` | Push local changes to server |
| `sync() -> Result<SyncResult>` | Bidirectional sync |
| `hot_sync(query) -> Result<Vec<T>>` | Query with cache fallback |
| `start_background()` | Start periodic sync |
| `stop_background()` | Stop periodic sync |

### SyncResult

```rust
pub struct SyncResult {
    pub items_synced: usize,
    pub items_added: usize,
    pub items_updated: usize,
    pub items_deleted: usize,
    pub conflicts: Vec<Conflict>,
    pub duration_ms: u64,
}
```
