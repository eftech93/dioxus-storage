# dioxus-storage-sync

Two-way synchronization between local IndexedDB and backend API.

## Installation

```toml
[dependencies]
dioxus-storage-sync = "0.0.3"
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
use dioxus_indexeddb::{Collection, Database, DatabaseConfig};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
    id: String,
    name: String,
    price: f64,
}

impl Syncable for Product {
    fn sync_id(&self) -> String {
        self.id.clone()
    }

    fn sync_timestamp(&self) -> i64 {
        0
    }

    fn mark_synced(&mut self) {}

    fn is_dirty(&self) -> bool {
        true
    }
}

async fn example() {
    let db = Database::open(DatabaseConfig::new("my_app", 1)
        .with_store("products", "id"))
        .await
        .unwrap();

    let collection: Collection<Product> = db.collection("products");

    let config = SyncConfig::new("https://api.example.com")
        .with_mode(SyncMode::Bidirectional)
        .with_hot_sync(true)
        .with_background_sync(Duration::from_secs(30))
        .with_resource_path("products");

    let engine = SyncEngine::new(collection.clone(), config.clone());
    let manager = SyncManager::new(collection, config);
}
```

## Sync Modes

### Hot Sync

Checks local cache first, fetches from backend only if needed:

```rust
let config = SyncConfig::new("https://api.example.com")
    .with_hot_sync(true);

let engine = SyncEngine::new(collection, config);
let items = engine.hot_sync(&Query::new()).await.unwrap();
```

### Background Sync

Automatically syncs data periodically:

```rust
let config = SyncConfig::new("https://api.example.com")
    .with_background_sync(Duration::from_secs(30))
    .with_mode(SyncMode::Bidirectional);

let manager = SyncManager::new(collection, config);
manager.start(); // Starts background loop
```

### Manual Sync

Full control over when to sync:

```rust
let mut manager = SyncManager::new(collection, config);

// Pull + push + queue replay
let result = manager.sync_now().await.unwrap();
```

## Configuration

```rust
use dioxus_storage_sync::prelude::*;
use std::time::Duration;

let config = SyncConfig::new("https://api.example.com")
    // Resource path for REST endpoints (default: "items")
    .with_resource_path("products")

    // Sync mode
    .with_mode(SyncMode::Bidirectional)

    // Hot sync
    .with_hot_sync(true)

    // Background sync interval
    .with_background_sync(Duration::from_secs(30))

    // Batch size for push operations
    .with_batch_size(100)

    // Retry attempts for failed requests
    .with_retry_attempts(3)

    // Conflict resolution
    .with_conflict_resolution(ConflictResolution::LastWriteWins)

    // Auth header
    .with_auth_token("Bearer token123")

    // Custom headers
    .with_header("X-Custom", "value");
```

### Resource Path

The `resource_path` determines the API endpoint segment used by `SyncEngine` and `OfflineQueue`. For example, with `resource_path("products")`:

| Operation | Endpoint |
|-----------|----------|
| List / hot sync | `GET /products` |
| Sync changes | `GET /products/sync` |
| Batch push | `POST /products/batch` |
| Queue replay — Insert/Update | `PUT /products/{id}` |
| Queue replay — Delete | `DELETE /products/{id}` |

## Conflict Resolution Strategies

| Strategy | Description |
|----------|-------------|
| `ServerWins` | Always use server version |
| `LocalWins` | Always use local version |
| `LastWriteWins` | Use the most recent timestamp (default) |
| `Manual` | Keep in queue for manual resolution |

```rust
let config = SyncConfig::new("https://api.example.com")
    .with_conflict_resolution(ConflictResolution::LastWriteWins);
```

## Offline Queue

The sync manager automatically detects when the browser goes offline and queues mutations. When connectivity is restored, the queue is replayed automatically during the next background sync or manual sync.

### How It Works

- **Save/Delete** operations check the browser's online status.
- If **offline**, the operation is stored in a dedicated IndexedDB queue (`{collection_name}_sync_queue`).
- When the browser comes **back online**, the queue is replayed against the backend.
- **Conflicts** during replay are handled according to the configured `ConflictResolution`.

### Manual Replay

```rust
let manager = SyncManager::new(collection, config);

// Manually trigger queue replay
let result = manager.replay_queue().await.unwrap();
println!(
    "Replayed: {}, Failed: {}, Conflicts: {}",
    result.success, result.failed, result.conflicts
);
```

### Queue Status in UI

```rust
#[component]
fn QueueStatus() -> Element {
    let collection: Collection<Task> = /* ... */;
    let config = SyncConfig::new("https://api.example.com")
        .with_resource_path("tasks");
    let manager = use_sync_manager(collection, config);
    let status = manager.status().read().clone();

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

## Complete Example

```rust
use dioxus::prelude::*;
use dioxus_indexeddb::{Collection, Database, DatabaseConfig};
use dioxus_storage_sync::prelude::*;
use serde::{Serialize, Deserialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    id: String,
    title: String,
    completed: bool,
}

impl Syncable for Task {
    fn sync_id(&self) -> String {
        self.id.clone()
    }

    fn sync_timestamp(&self) -> i64 {
        0
    }

    fn mark_synced(&mut self) {}

    fn is_dirty(&self) -> bool {
        true
    }
}

#[component]
fn TaskApp() -> Element {
    let collection: Collection<Task> = /* initialized elsewhere */;

    let config = SyncConfig::new("http://localhost:3001/api")
        .with_mode(SyncMode::Bidirectional)
        .with_hot_sync(true)
        .with_background_sync(Duration::from_secs(60))
        .with_resource_path("tasks")
        .with_conflict_resolution(ConflictResolution::LastWriteWins);

    let manager = SyncManager::new(collection, config);
    let mut new_task_title = use_signal(String::new);

    rsx! {
        div { class: "task-app",
            h1 { "Tasks" }

            // Sync status
            div { class: "status",
                let status = manager.status().read().clone();
                if status.is_syncing {
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
                    };
                    spawn(async move {
                        let _ = manager.save(&task).await;
                    });
                    new_task_title.set(String::new());
                },
                "Add Task"
            }

            // Manual sync button
            button {
                onclick: move |_| {
                    let mgr = manager.clone();
                    spawn(async move {
                        let _ = mgr.sync_now().await;
                    });
                },
                "🔄 Sync Now"
            }
        }
    }
}
```

## API Reference

### SyncConfig

| Method | Description |
|--------|-------------|
| `new(api_url)` | Create config with base API URL |
| `with_resource_path(path)` | Set REST resource path (default: `"items"`) |
| `with_mode(mode)` | Set sync mode |
| `with_hot_sync(bool)` | Enable/disable hot sync |
| `with_background_sync(duration)` | Enable background sync with interval |
| `with_batch_size(size)` | Set push batch size |
| `with_retry_attempts(n)` | Set HTTP retry count |
| `with_conflict_resolution(strategy)` | Set conflict resolution strategy |
| `with_auth_token(token)` | Set Bearer token |
| `with_header(key, value)` | Add custom HTTP header |

### SyncManager

| Method | Description |
|--------|-------------|
| `new(collection, config)` | Create a new sync manager |
| `start()` | Start background sync loop |
| `stop()` | Stop background sync loop |
| `sync_now()` | Perform manual sync + queue replay |
| `save(item)` | Save locally (queue if offline) |
| `delete(id)` | Delete locally (queue if offline) |
| `get(id)` | Get item with hot sync fallback |
| `get_all()` | Get all local items |
| `replay_queue()` | Manually replay offline queue |
| `status()` | Access `Signal<SyncStatus>` |

### SyncStatus

```rust
pub struct SyncStatus {
    pub is_syncing: bool,
    pub last_result: Option<SyncResult>,
    pub last_sync_time: Option<String>,
    pub error: Option<String>,
    pub is_online: bool,
    pub queue_pending: usize,
    pub queue_replaying: bool,
    pub queue_result: Option<QueueReplayResult>,
}
```

### QueueReplayResult

```rust
pub struct QueueReplayResult {
    pub success: usize,
    pub failed: usize,
    pub conflicts: usize,
    pub errors: Vec<String>,
}
```
