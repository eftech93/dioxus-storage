# Dioxus Storage

[![Documentation](https://img.shields.io/badge/docs-eftech93.github.io/dioxus--storage-9D6B4C.svg)](https://eftech93.github.io/dioxus-storage)

> Type-safe storage solutions for Dioxus web applications

Dioxus Storage is a collection of crates providing unified, type-safe storage APIs for Dioxus web applications. It supports IndexedDB, LocalStorage, SessionStorage, and backend synchronization.

## Crates

### `dioxus-indexeddb` [![crates.io](https://img.shields.io/crates/v/dioxus-indexeddb.svg)](https://crates.io/crates/dioxus-indexeddb)

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
- Multi-store transactions
- Async/await API

### `dioxus-client-storage` [![crates.io](https://img.shields.io/crates/v/dioxus-client-storage.svg)](https://crates.io/crates/dioxus-client-storage)

Unified storage API supporting LocalStorage, SessionStorage, and IndexedDB.

```rust
use dioxus_storage::prelude::*;

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

### `dioxus-client-storage-sync` [![crates.io](https://img.shields.io/crates/v/dioxus-client-storage-sync.svg)](https://crates.io/crates/dioxus-client-storage-sync)

Two-way synchronization between local IndexedDB and backend API.

```rust
use dioxus_storage_sync::prelude::*;
use dioxus_indexeddb::{Collection, DatabaseConfig};

#[component]
fn ProductList() -> Element {
    let collection: Collection<Product> = /* initialized elsewhere */;

    let config = SyncConfig::new("http://api.example.com")
        .with_resource_path("products")
        .with_hot_sync(true)
        .with_background_sync(Duration::from_secs(30));

    let manager = SyncManager::new(collection, config);

    // Start background sync loop
    manager.start();

    rsx! {
        div {
            for product in manager.get_all().await.unwrap_or_default() {
                p { "{product.name}" }
            }
        }
    }
}
```

**Features:**
- 🔥 **Hot Sync** - On-demand fetching with local cache
- 🌙 **Background Sync** - Periodic synchronization
- 📊 **Conflict Resolution** - Handle simultaneous updates
- 🔄 **Bidirectional** - Push local changes to server

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
dioxus-client-storage = "0.0.1"
```

Or use individual crates:

```toml
[dependencies]
dioxus-indexeddb = "0.0.1"
dioxus-client-storage-sync = "0.0.1"
```

## Examples

### Basic Demo

```bash
cd examples/demo
dx serve --platform web
```

### Sync Demo (with backend)

```bash
cd examples/sync-demo/backend
docker-compose up -d
cd ..
dx serve --platform web
```

## Workspace Structure

```
dioxus-client-storage/
├── Cargo.toml              # Workspace root
├── README.md
├── dioxus-indexeddb/       # IndexedDB with hooks
├── dioxus-client-storage/         # Unified storage API
├── dioxus-client-storage-sync/    # Sync with backend
└── examples/
    ├── demo/               # Basic storage demo
    └── sync-demo/          # Full sync demo
```

## License

MIT OR Apache-2.0
