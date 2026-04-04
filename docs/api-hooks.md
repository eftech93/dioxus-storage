# API Reference - Hooks

## `use_db`

Opens a database connection.

```rust
use dioxus_indexeddb::prelude::*;

let db = use_db(DatabaseConfig::new("my_app", 1)
    .with_store("users", "id"));
```

**Parameters:**
- `config: DatabaseConfig` - Database configuration

**Returns:** `Signal<Option<Database>>`

The signal will be `None` initially and `Some(Database)` once connected.

## `use_collection`

Gets a typed collection from a database.

```rust
use dioxus_indexeddb::prelude::*;

let users = use_collection::<User>(db, "users");
```

**Parameters:**
- `db: Signal<Option<Database>>` - Database signal from `use_db`
- `name: &str` - Collection/store name

**Returns:** `Signal<Option<Collection<T>>>`

## `use_query`

Executes a query and returns reactive results.

```rust
use dioxus_indexeddb::prelude::*;

let active_users = use_query(collection, |c| async move {
    c.query(
        Query::new()
            .filter(Filter::eq("status", "active"))
    ).await
});
```

**Parameters:**
- `collection: Signal<Option<Collection<T>>>` - Collection signal
- `query_fn: F` - Async function that performs the query

**Returns:** `Signal<Result<Vec<T>>>`

## `use_local_storage`

Reactive LocalStorage hook.

```rust
use dioxus_storage::prelude::*;

let theme = use_local_storage::<String>("theme", "light".to_string());

// Read
let current = theme.read();

// Write
theme.set("dark".to_string());
```

**Parameters:**
- `key: &str` - Storage key
- `default: T` - Default value if key doesn't exist

**Returns:** `Signal<T>`

## `use_session_storage`

Reactive SessionStorage hook.

```rust
use dioxus_storage::prelude::*;

let form_data = use_session_storage::<String>("form_draft", String::new());
```

**Parameters:**
- `key: &str` - Storage key
- `default: T` - Default value if key doesn't exist

**Returns:** `Signal<T>`

## `use_storage`

Generic storage hook.

```rust
use dioxus_storage::prelude::*;

let value = use_storage::<bool>(
    StorageConfig::local(),
    "notifications",
    true
);
```

**Parameters:**
- `config: StorageConfig` - Storage configuration
- `key: &str` - Storage key
- `default: T` - Default value

**Returns:** `Signal<T>`

## `use_sync`

Backend synchronization hook.

```rust
use dioxus_storage_sync::prelude::*;

let sync = use_sync::<Product>(
    SyncConfig::new("https://api.example.com")
        .with_hot_sync(true)
);

// Access data
for product in sync.data.read().iter() {
    // Render product
}

// Trigger sync
sync.sync_now();
```

**Parameters:**
- `config: SyncConfig` - Sync configuration

**Returns:** `SyncHandle<T>`

### SyncHandle Methods

| Method | Description |
|--------|-------------|
| `data: Signal<Vec<T>>` | Access synced data |
| `status: Signal<SyncStatus>` | Check sync status |
| `sync_now()` | Trigger manual sync |
| `add_local(item)` | Add item locally |
| `update_local(id, f)` | Update item locally |
| `delete_local(id)` | Delete item locally |
