# dioxus-indexeddb

High-level IndexedDB bindings for Dioxus with reactive hooks.

## Installation

```toml
[dependencies]
dioxus-indexeddb = "0.0.1"
```

## Basic Usage

### Opening a Database

```rust
use dioxus_indexeddb::prelude::*;

#[component]
fn App() -> Element {
    let db = use_db(DatabaseConfig::new("my_app", 1)
        .with_store("users", "id")
        .with_store("products", "id"));
    
    rsx! {
        "Database ready!"
    }
}
```

### Using Collections

```rust
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
    let db = use_db(DatabaseConfig::new("my_app", 1)
        .with_store("users", "id"));
    
    let users = use_collection::<User>(db, "users");
    
    rsx! {
        div { class: "user-list",
            for user in users.read().iter() {
                div { class: "user-card",
                    h3 { "{user.name}" }
                    p { "{user.email}" }
                }
            }
        }
    }
}
```

### Querying Data

```rust
use dioxus_indexeddb::prelude::*;

#[component]
fn ActiveUsers() -> Element {
    let db = use_db(DatabaseConfig::new("my_app", 1)
        .with_store("users", "id"));
    
    let collection = use_collection::<User>(db, "users");
    
    // Query with filters
    let active_users = use_query(collection, |c| async move {
        c.query(
            Query::new()
                .filter(Filter::eq("status", "active"))
                .order_by("name", Order::Asc)
        ).await
    });
    
    rsx! {
        ul {
            for user in active_users.read().as_ref().unwrap_or(&vec![]).iter() {
                li { "{user.name}" }
            }
        }
    }
}
```

## API Reference

### Hooks

#### `use_db(config: DatabaseConfig) -> Signal<Option<Database>>`

Opens a database connection. The signal will be `Some(Database)` once connected.

#### `use_collection<T>(db: Signal<Option<Database>>, name: &str) -> Signal<Option<Collection<T>>>`

Gets a typed collection from the database.

#### `use_query<T, F, Fut>(collection: Signal<Option<Collection<T>>>, query_fn: F) -> Signal<Result<Vec<T>>>`

Executes a query and returns reactive results.

### Collection Methods

| Method | Description |
|--------|-------------|
| `get(key: &str) -> Result<Option<T>>` | Get a single item by key |
| `get_all() -> Result<Vec<T>>` | Get all items in the collection |
| `get_by_index(index: &str, value: &str) -> Result<Vec<T>>` | Query using an index |
| `get_one_by_index(index: &str, value: &str) -> Result<Option<T>>` | Get single item by unique index |
| `insert(key: &str, item: &T) -> Result<()>` | Insert a new item |
| `put(key: &str, item: &T) -> Result<()>` | Insert or update an item |
| `delete(key: &str) -> Result<()>` | Delete an item by key |
| `clear() -> Result<()>` | Clear all items |
| `query(query: Query) -> Result<Vec<T>>` | Execute a filtered query |
| `find(query: &Query) -> Result<QueryResult<T>>` | Execute query with index optimization |

### Query Builder

```rust
use dioxus_indexeddb::prelude::*;

let query = Query::new()
    .filter(Filter::eq("category", "electronics"))
    .filter(Filter::gt("price", 100.0))
    .filter(Filter::contains("name", "Pro"))
    .order_by("price", Order::Desc)
    .limit(10)
    .offset(0);
```

#### Filter Operators

- `Filter::eq(field, value)` - Equal
- `Filter::ne(field, value)` - Not equal
- `Filter::gt(field, value)` - Greater than
- `Filter::gte(field, value)` - Greater than or equal
- `Filter::lt(field, value)` - Less than
- `Filter::lte(field, value)` - Less than or equal
- `Filter::contains(field, value)` - String contains
- `Filter::starts_with(field, value)` - String starts with
- `Filter::ends_with(field, value)` - String ends with

## Database Migrations

```rust
use dioxus_indexeddb::prelude::*;

let migrations = MigrationManager::new()
    // Version 2: Add users store
    .add_migration(Migration::new(2).create_store("users", "id"))
    // Version 3: Add posts store and delete old store
    .add_migration(
        Migration::new(3)
            .create_store("posts", "id")
            .delete_store("legacy_data")
    );

let db = Database::open_with_migrations(
    DatabaseConfig::new("my_app", 3),
    migrations
).await.expect("Failed to open database");
```

## Indexes

Indexes enable fast queries on specific fields.

### Creating Indexes

```rust
use dioxus_indexeddb::prelude::*;

let config = DatabaseConfig::new("my_app", 1)
    .with_store_and_indexes(
        "users", 
        "id",
        vec![
            IndexConfig::new("email_idx", "email", true),   // unique index
            IndexConfig::new("age_idx", "age", false),      // non-unique index
        ]
    );

let db = Database::open(config).await?;
```

### Querying by Index

```rust
let collection = db.collection::<User>("users");

// Get user by unique email index
let user = collection.get_one_by_index("email_idx", "user@example.com").await?;

// Get all users with specific age
let users = collection.get_by_index("age_idx", "25").await?;

// Use index with Query builder
let results = collection.find(
    Query::new()
        .use_index("age_idx")
        .filter(Filter::gte("age", 18))
).await?;
```

### Adding Indexes to Existing Stores

Use migrations to add indexes to existing stores:

```rust
let migrations = MigrationManager::new()
    .add_migration(
        Migration::new(2)
            .create_store("users", "id")
            .create_index("users", "email_idx", "email", true)
    );

let db = Database::open_with_migrations(
    DatabaseConfig::new("my_app", 2),
    migrations
).await?;
```

## Transactions

```rust
use dioxus_indexeddb::prelude::*;

let db = Database::open(DatabaseConfig::new("my_app", 1)).await?;

let tx = db.transaction()
    .read_write()
    .with_store("users")
    .with_store("products")
    .build();

// Perform operations atomically
let users: Collection<User> = tx.collection("users");
let products: Collection<Product> = tx.collection("products");

users.put("user1", &user).await?;
products.put("product1", &product).await?;

tx.commit().await?;
```
