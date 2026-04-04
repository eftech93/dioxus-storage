# Database Migrations

When your data model changes, you need migrations to transform the database schema.

## Basic Migration

```rust
use dioxus_indexeddb::prelude::*;

// Define migrations
let migrations = MigrationManager::new()
    .add_migration(Migration::new(2)
        .create_store("users", "id")
        .create_store("products", "id")
    );

// Open database with migrations
let db = Database::open_with_migrations(
    DatabaseConfig::new("my_app", 2),  // Current version is 2
    migrations
).await.expect("Failed to open database");
```

## Migration Operations

### Create Store

```rust
Migration::new(2)
    .create_store("users", "id")  // store name, key path
```

### Create Store with Auto-increment

```rust
Migration::new(2)
    .create_auto_increment_store("logs", "id")
```

### Delete Store

```rust
Migration::new(3)
    .delete_store("old_data")
```

### Data Migration

```rust
Migration::new(4)
    .create_store("new_users", "id")
    .with_data_migration(|db| {
        // Custom migration code
        log::info!("Running data migration for version 4");
        
        // Example: Copy data from old store to new store
        // This runs inside a migration transaction
    })
```

## Complete Example

```rust
use dioxus_indexeddb::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserV1 {
    id: String,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserV2 {
    id: String,
    name: String,
    email: String,  // New field in v2
}

async fn init_db() -> Database {
    let migrations = MigrationManager::new()
        // Version 1: Initial schema
        .add_migration(Migration::new(1)
            .create_store("users", "id")
        )
        // Version 2: Add email field
        .add_migration(Migration::new(2)
            .with_data_migration(|db| {
                // Migrate users to have email field
                log::info!("Migrating users to v2");
            })
        )
        // Version 3: Add posts store
        .add_migration(Migration::new(3)
            .create_store("posts", "id")
            .create_index("posts", "author_id")  // Index for querying
        );

    Database::open_with_migrations(
        DatabaseConfig::new("my_app", 3),
        migrations
    ).await.expect("Failed to open database")
}
```

## Migration Best Practices

1. **Always increase version** - Never decrease or reuse version numbers
2. **Make migrations idempotent** - Running twice should have same effect
3. **Keep migrations simple** - Complex logic can fail and block the database
4. **Test migrations** - Test upgrade from each previous version
5. **Backup data** - Consider exporting data before major migrations

## Handling Migration Errors

```rust
match Database::open_with_migrations(config, migrations).await {
    Ok(db) => db,
    Err(e) => {
        log::error!("Migration failed: {}", e);
        // Option 1: Clear and start fresh
        let _ = idb::Factory::new()
            .unwrap()
            .delete("my_app")
            .await;
        // Option 2: Show error to user
        panic!("Database upgrade failed. Please contact support.");
    }
}
```
