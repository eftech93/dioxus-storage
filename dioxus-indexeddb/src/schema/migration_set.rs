//! Folder-based migration system
//!
//! Organize migrations in modules like:
//!
//! ```text
//! migrations/
//!   mod.rs
//!   v1.rs    # Initial schema
//!   v2.rs    # Migration from v1 to v2
//!   v3.rs    # Migration from v2 to v3
//! ```

use crate::migration::{Migration, MigrationManager, MigrationOp};

/// A set of migrations for a specific version
pub trait MigrationSet: Send + Sync {
    /// Target version (e.g., 2 means migrating TO version 2)
    fn version() -> u32;

    /// Migration operations to perform
    fn operations() -> Vec<MigrationOp>;

    /// Optional data migration callback
    fn data_migration() -> Option<fn()> {
        None
    }

    /// Build the migration
    fn build_migration() -> Migration {
        let mut m = Migration::new(Self::version());
        for op in Self::operations() {
            m = m.with_op(op);
        }
        // Note: data_migration is stored separately and handled by the migration system
        m
    }
}

/// A schema migration that adds/removes stores
pub trait SchemaMigration: MigrationSet {
    /// Previous version (for validation)
    fn from_version() -> u32;

    /// Store definitions to add in this version
    fn add_stores() -> Vec<crate::schema::StoreDefinition>;

    /// Store names to remove in this version
    fn remove_stores() -> Vec<String>;

    /// Build operations from schema changes
    fn build_schema_operations() -> Vec<MigrationOp> {
        let mut ops = Vec::new();

        // Add stores
        for store in Self::add_stores() {
            ops.push(MigrationOp::CreateStore {
                name: store.name,
                key_path: store.key_path,
                auto_increment: store.auto_increment,
            });
        }

        // Remove stores
        for name in Self::remove_stores() {
            ops.push(MigrationOp::DeleteStore { name });
        }

        ops
    }
}

/// Registry for all migrations
#[derive(Debug, Default)]
pub struct MigrationRegistry {
    migrations: Vec<Migration>,
}

impl MigrationRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    /// Register a migration set
    pub fn register<T: MigrationSet>(mut self) -> Self {
        self.migrations.push(T::build_migration());
        self
    }

    /// Register a raw migration
    pub fn register_migration(mut self, migration: Migration) -> Self {
        self.migrations.push(migration);
        self
    }

    /// Convert to migration manager
    pub fn into_manager(self) -> MigrationManager {
        let mut manager = MigrationManager::new();
        for migration in self.migrations {
            manager = manager.add_migration(migration);
        }
        manager
    }

    /// Get all migrations
    pub fn migrations(&self) -> &[Migration] {
        &self.migrations
    }
}

/// Helper macro to define a migration module
///
/// ```rust,ignore
/// define_migration! {
///     pub struct V2Migration {
///         version: 2,
///         operations: vec![
///             MigrationOp::CreateStore { ... },
///         ],
///         data_migration: Some(|| { ... }),
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_migration {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            version: $version:expr,
            operations: $ops:expr,
            $(data_migration: $data_fn:expr,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy)]
        $vis struct $name;

        impl $crate::schema::migration_set::MigrationSet for $name {
            fn version() -> u32 {
                $version
            }

            fn operations() -> Vec<$crate::migration::MigrationOp> {
                $ops
            }

            $(
                fn data_migration() -> Option<fn()> {
                    Some($data_fn)
                }
            )?
        }
    };
}

/// Re-export for macro
pub use define_migration;

/// Helper to create a simple schema migration
#[macro_export]
macro_rules! schema_migration {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            version: $version:expr,
            from: $from:expr,
            add: [$($store:expr),* $(,)?],
            remove: [$($remove:expr),* $(,)?],
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy)]
        $vis struct $name;

        impl $crate::schema::migration_set::MigrationSet for $name {
            fn version() -> u32 {
                $version
            }

            fn operations() -> Vec<$crate::migration::MigrationOp> {
                let mut ops = Vec::new();
                $(
                    ops.push($crate::migration::MigrationOp::CreateStore {
                        name: $store.name.to_string(),
                        key_path: $store.key_path.to_string(),
                        auto_increment: $store.auto_increment,
                    });
                )*
                $(
                    ops.push($crate::migration::MigrationOp::DeleteStore {
                        name: $remove.to_string(),
                    });
                )*
                ops
            }
        }

        impl $crate::schema::migration_set::SchemaMigration for $name {
            fn from_version() -> u32 {
                $from
            }

            fn add_stores() -> Vec<$crate::schema::StoreDefinition> {
                vec![$($store),*]
            }

            fn remove_stores() -> Vec<String> {
                vec![$($remove.to_string()),*]
            }
        }
    };
}

pub use schema_migration;
