# API Reference - Database

## Database

Represents an IndexedDB database connection.

```rust
use dioxus_indexeddb::prelude::*;

let db = Database::open(
    DatabaseConfig::new("my_app", 1)
        .with_store("users", "id")
        .with_store("products", "id")
).await?;
```

## Static Methods

### `open`

Open a database connection.

```rust
let db = Database::open(config).await?;
```

**Parameters:**
- `config: DatabaseConfig` - Database configuration

**Returns:** `Result<Database>`

### `open_with_migrations`

Open database with migrations.

```rust
let migrations = MigrationManager::new()
    .add_migration(Migration::new(2).create_store("new_store", "id"));

let db = Database::open_with_migrations(config, migrations).await?;
```

**Parameters:**
- `config: DatabaseConfig` - Database configuration
- `migrations: MigrationManager` - Migrations to apply

**Returns:** `Result<Database>`

### `is_available`

Check if IndexedDB is available in the current environment.

```rust
if Database::is_available() {
    // Use IndexedDB
} else {
    // Fallback to LocalStorage
}
```

**Returns:** `bool`

## Instance Methods

### `collection`

Get a typed collection.

```rust
let users: Collection<User> = db.collection("users");
```

**Parameters:**
- `name: &str` - Store name

**Returns:** `Collection<T>`

### `transaction`

Start a transaction builder.

```rust
let tx = db.transaction()
    .read_write()
    .with_store("users")
    .with_store("orders")
    .build();
```

**Returns:** `TransactionBuilder`

### `close`

Close the database connection.

```rust
db.close();
```

## DatabaseConfig

Configuration for opening a database.

```rust
let config = DatabaseConfig::new("my_app", 1)  // name, version
    .with_store("users", "id")                  // store, key path
    .with_store("products", "sku")
    .with_auto_increment_store("logs", "id");   // auto-increment key
```

### Methods

| Method | Description |
|--------|-------------|
| `new(name, version)` | Create new config |
| `with_store(name, key_path)` | Add object store |
| `with_store_and_indexes(name, key_path, indexes)` | Add store with indexes |
| `with_index(store, name, key_path, unique)` | Add index to existing store |
| `with_auto_increment_store(name, key_path)` | Add auto-increment store |

## IndexConfig

Configuration for an IndexedDB index.

```rust
let index = IndexConfig::new("email_idx", "email", true);  // unique index
let index = IndexConfig::new("age_idx", "age", false);     // non-unique
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | `impl Into<String>` | Index name |
| `key_path` | `impl Into<String>` | Field to index |
| `unique` | `bool` | Enforce unique values |

## Transaction

Multi-store transaction for atomic operations.

```rust
let tx = db.transaction()
    .read_write()
    .with_store("users")
    .with_store("accounts")
    .build();

let users: Collection<User> = tx.collection("users");
let accounts: Collection<Account> = tx.collection("accounts");

// All operations are atomic
users.put("user-1", &user).await?;
accounts.put("acc-1", &account).await?;

tx.commit().await?;
```

### TransactionBuilder Methods

| Method | Description |
|--------|-------------|
| `read_only()` | Read-only transaction |
| `read_write()` | Read-write transaction |
| `with_store(name)` | Add store to transaction |
| `build()` | Create transaction |

### Transaction Methods

| Method | Description |
|--------|-------------|
| `collection::<T>(name)` | Get typed collection |
| `commit().await` | Commit transaction |
| `abort()` | Abort transaction |

## Complete Example

```rust
use dioxus_indexeddb::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transfer {
    from: String,
    to: String,
    amount: f64,
}

async fn transfer_funds(
    db: &Database,
    transfer: Transfer
) -> Result<()> {
    // Start atomic transaction
    let tx = db.transaction()
        .read_write()
        .with_store("users")
        .build();
    
    let users: Collection<User> = tx.collection("users");
    
    // Get sender
    let mut from_user = users.get(&transfer.from).await?
        .ok_or("Sender not found")?;
    
    // Get recipient
    let mut to_user = users.get(&transfer.to).await?
        .ok_or("Recipient not found")?;
    
    // Check balance
    if from_user.balance < transfer.amount {
        return Err("Insufficient funds".into());
    }
    
    // Perform transfer
    from_user.balance -= transfer.amount;
    to_user.balance += transfer.amount;
    
    // Save changes (atomic)
    users.put(&from_user.id, &from_user).await?;
    users.put(&to_user.id, &to_user).await?;
    
    // Commit transaction
    tx.commit().await?;
    
    Ok(())
}
```
