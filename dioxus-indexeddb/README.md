# dioxus-indexeddb

[![Crates.io](https://img.shields.io/crates/v/dioxus-indexeddb.svg)](https://crates.io/crates/dioxus-indexeddb)
[![Docs.rs](https://docs.rs/dioxus-indexeddb/badge.svg)](https://docs.rs/dioxus-indexeddb)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

High-level IndexedDB bindings for Dioxus with reactive hooks and type-safe collections.

## Features

- 🦀 **Type-safe collections** with serde serialization
- 🪝 **Reactive hooks** - `use_db`, `use_collection`, `use_query`
- 🔍 **Query builder** with filtering, sorting, and pagination
- 📝 **Multi-store transactions** for atomic operations
- 🔄 **Database migrations** for schema versioning
- ⚡ **Async/await API** throughout

## Installation

```toml
[dependencies]
dioxus-indexeddb = "0.0.1"
serde = { version = "1.0", features = ["derive"] }
```

## Quick Start

```rust
use dioxus::prelude::*;
use dioxus_indexeddb::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    email: String,
}

#[component]
fn UserList() -> Element {
    // Open database
    let db = use_db(DatabaseConfig::new("my_app", 1)
        .with_store("users", "id"));
    
    // Get typed collection
    let users = use_collection::<User>(db, "users");
    
    rsx! {
        div {
            for user in users.read().iter() {
                p { "{user.name} - {user.email}" }
            }
        }
    }
}
```

## Querying Data

```rust
use dioxus_indexeddb::prelude::*;

#[component]
fn ActiveUsers() -> Element {
    let db = use_db(DatabaseConfig::new("my_app", 1)
        .with_store("users", "id"));
    
    let users = use_collection::<User>(db, "users");
    
    // Query with filters
    let active = use_query(users, |c| async move {
        c.query(
            Query::new()
                .filter(Filter::eq("status", "active"))
                .order_by("name", Order::Asc)
                .limit(10)
        ).await
    });
    
    rsx! {
        ul {
            for user in active.read().as_ref().unwrap_or(&vec![]).iter() {
                li { "{user.name}" }
            }
        }
    }
}
```

## Database Migrations

```rust
use dioxus_indexeddb::prelude::*;

let migrations = MigrationManager::new()
    .add_migration(Migration::new(2).create_store("users", "id"))
    .add_migration(Migration::new(3).create_store("posts", "id"));

let db = Database::open_with_migrations(
    DatabaseConfig::new("my_app", 3),
    migrations
).await?;
```

## API Overview

### Hooks

- `use_db(config)` - Open database connection
- `use_collection::<T>(db, name)` - Get typed collection
- `use_query(collection, query_fn)` - Reactive query results

### Collection Methods

- `get(key)` / `get_all()` - Read data
- `insert(key, item)` / `put(key, item)` - Write data
- `delete(key)` / `clear()` - Remove data
- `query(query)` - Filtered, sorted results

### Query Filters

- `Filter::eq(field, value)` - Equal
- `Filter::gt(field, value)` - Greater than
- `Filter::contains(field, value)` - String contains
- `Filter::starts_with(field, value)` - String starts with

## License

MIT OR Apache-2.0
