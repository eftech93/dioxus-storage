# Dioxus Storage

[![Documentation](https://img.shields.io/badge/docs-eftech93.github.io/dioxus--storage-9D6B4C.svg)](https://eftech93.github.io/dioxus-storage)

Type-safe storage solutions for Dioxus web applications.

## Crates

### `dioxus-indexeddb`

High-level IndexedDB bindings with Dioxus hooks.

```rust
use dioxus_indexeddb::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
}

#[component]
fn UserList() -> Element {
    let db = use_db(DatabaseConfig::new("my_app", 1)
        .with_store("users", "id"));
    
    let users = use_collection::<User>(db, "users");
    
    rsx! {
        div {
            for user in users.read().iter() {
                p { "{user.name}" }
            }
        }
    }
}
```

**Features:**
- Type-safe collections with serde serialization
- Dioxus hooks: `use_db`, `use_collection`, `use_query`
- Query builder with filtering and sorting
- **Index support** for fast queries
- Multi-store transactions
- Async/await API

### `dioxus-client-storage`

Unified storage API supporting LocalStorage, SessionStorage, and IndexedDB.

```rust
use dioxus_client_storage::prelude::*;

#[component]
fn App() -> Element {
    // Simple key-value storage (sync)
    let theme = use_local_storage::<String>("theme", "light".to_string());
    
    rsx! {
        button {
            onclick: move |_| {
                theme.set("dark".to_string());
            },
            "Switch Theme"
        }
    }
}
```

**Features:**
- `LocalStorage` - Persistent key-value storage
- `SessionStorage` - Per-session key-value storage
- `IndexedDB` integration via `dioxus-indexeddb`
- Reactive hooks that sync with storage

### `dioxus-client-storage-sync`

Two-way synchronization between local IndexedDB and backend API.

```rust
use dioxus_client_storage_sync::prelude::*;

#[component]
fn ProductList() -> Element {
    // Configure sync
    let config = SyncConfig::new("http://api.example.com")
        .with_collection("products");
    
    let sync = use_sync::<Product>(config);
    
    // Hot sync: checks local first, fetches if empty
    let products = sync.query_with_hot_sync(Query::new()).await?;
    
    // Background sync: periodic updates
    sync.start_background_sync(Duration::from_secs(30));
}
```

**Features:**
- 🔥 **Hot Sync** - On-demand fetching with local cache
- 🌙 **Background Sync** - Periodic synchronization
- 📊 **Conflict Resolution** - Handle simultaneous updates
- 🔄 **Bidirectional** - Push local changes to server

## Examples

### Basic Demo (`examples/demo`)
Demonstrates LocalStorage, SessionStorage, and IndexedDB with migrations.

```bash
cd examples/demo
dx serve --platform web
```

### Sync Demo (`examples/sync-demo`)
Complete example with backend API:
- 100 sample products in MongoDB
- Paginated sync (10 pages × 5 items)
- Hot sync vs Background sync
- Visual sync logging

```bash
# 1. Start backend
cd examples/sync-demo/backend
docker-compose up -d

# 2. Run sync demo
cd examples/sync-demo
dx serve --platform web
```

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
dioxus-client-storage = { git = "https://github.com/eftech93/dioxus-client-storage" }
```

### Using Indexes

```rust
use dioxus_indexeddb::prelude::*;

// Create database with indexes
let db = Database::open(
    DatabaseConfig::new("my_app", 1)
        .with_store_and_indexes(
            "users", 
            "id",
            vec![
                IndexConfig::new("email_idx", "email", true),   // unique
                IndexConfig::new("age_idx", "age", false),      // non-unique
            ]
        )
).await?;

let collection = db.collection::<User>("users");

// Query using index
let user = collection.get_one_by_index("email_idx", "user@example.com").await?;
let adults = collection.get_by_index("age_idx", "25").await?;
```

Or use individual crates:

```toml
[dependencies]
dioxus-indexeddb = { git = "https://github.com/eftech93/dioxus-client-storage", package = "dioxus-indexeddb" }
dioxus-client-storage-sync = { git = "https://github.com/eftech93/dioxus-client-storage", package = "dioxus-client-storage-sync" }
```

## Migrations

`dioxus-indexeddb` supports database migrations:

```rust
use dioxus_indexeddb::prelude::*;

async fn init_db() -> Database {
    let migrations = MigrationManager::new()
        // Version 2: Add users store
        .add_migration(Migration::new(2).create_store("users", "id"))
        // Version 3: Add posts store and delete old store
        .add_migration(
            Migration::new(3)
                .create_store("posts", "id")
                .delete_store("legacy_data")
        )
        // Version 4: Data migration
        .add_migration(
            Migration::new(4)
                .create_store("comments", "id")
                .with_data_migration(|db| {
                    // Migrate data from old format to new format
                    log::info!("Running data migration for version 4");
                })
        );

    Database::open_with_migrations(
        DatabaseConfig::new("my_app", 4)
            .with_store("settings", "key"),
        migrations
    ).await.expect("Failed to open database")
}
```

Migration operations:
- `create_store(name, key_path)` - Create a new object store
- `create_auto_increment_store(name, key_path)` - Create with auto-increment
- `delete_store(name)` - Delete an existing store
- `with_data_migration(fn)` - Run custom data migration code

## Workspace Structure

```
dioxus-client-storage/
├── Cargo.toml              # Workspace root
├── README.md
├── dioxus-indexeddb/       # IndexedDB with hooks
│   ├── Cargo.toml
│   └── src/
├── dioxus-client-storage/         # Unified storage API
│   ├── Cargo.toml
│   └── src/
├── dioxus-client-storage-sync/    # Sync with backend
│   ├── Cargo.toml
│   └── src/
└── examples/
    ├── demo/               # Basic storage demo
    └── sync-demo/          # Full sync demo
        ├── backend/        # Rust API + MongoDB
        │   ├── docker-compose.yml
        │   ├── api/        # Axum server
        │   └── init-mongo.js
        └── src/            # Dioxus app
```

## Author

Esteban Puello <eftech93@gmail.com>

## License

MIT OR Apache-2.0
