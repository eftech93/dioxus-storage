//! Database migrations
//!
//! This module follows a folder-based migration strategy similar to Prisma or Entity Framework.
//! Each version has its own file with schema changes and optional data migrations.
//!
//! # Migration Structure
//!
//! - `v1.rs` - Initial schema (version 1)
//! - `v2.rs` - Migration from v1 to v2 (adds settings store)
//! - `v3.rs` - Migration from v2 to v3 (adds archived_tasks, removes old_temp)
//!
//! To create a new migration:
//! 1. Create a new file `v{N}.rs`
//! 2. Implement `MigrationSet` trait
//! 3. Register in `registry()` function below

use dioxus_indexeddb::prelude::*;

pub mod v1;
pub mod v2;
pub mod v3;

/// Build the migration registry
///
/// This is called when opening the database. All migrations are applied
/// in order based on their version number.
pub fn registry() -> MigrationRegistry {
    MigrationRegistry::new()
        .register::<v1::V1Migration>()
        .register::<v2::V2Migration>()
        .register::<v3::V3Migration>()
}

/// Current database version
pub const CURRENT_VERSION: u32 = 3;

/// Helper to validate migration chain
pub fn validate_migrations() {
    let registry = registry();
    let migrations = registry.migrations();

    log::info!("Registered {} migrations:", migrations.len());
    for (i, m) in migrations.iter().enumerate() {
        log::info!("  Migration {}: version {}", i + 1, m.version);
    }
}
