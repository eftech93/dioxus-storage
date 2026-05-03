//! Database migrations for IndexedDB
//!
//! Migrations are run during the `onupgradeneeded` event when opening a database.
//! Each migration specifies a target version and operations to perform.

use crate::error::{IndexedDbError, Result};
use idb::{Database as IdbDatabase, KeyPath, ObjectStoreParams};

/// A database migration
#[derive(Debug, Clone)]
pub struct Migration {
    /// Version this migration upgrades TO
    pub version: u32,
    /// Operations to perform
    pub operations: Vec<MigrationOp>,
    /// Optional data migration callback (runs after schema changes)
    pub data_migration: Option<fn(&IdbDatabase)>,
}

impl Migration {
    /// Create a new migration for a specific version
    pub fn new(version: u32) -> Self {
        Self {
            version,
            operations: Vec::new(),
            data_migration: None,
        }
    }

    /// Add an operation
    pub fn with_op(mut self, op: MigrationOp) -> Self {
        self.operations.push(op);
        self
    }

    /// Create a store
    pub fn create_store(mut self, name: impl Into<String>, key_path: impl Into<String>) -> Self {
        self.operations.push(MigrationOp::CreateStore {
            name: name.into(),
            key_path: key_path.into(),
            auto_increment: false,
        });
        self
    }

    /// Create a store with auto-increment
    pub fn create_auto_increment_store(
        mut self,
        name: impl Into<String>,
        key_path: impl Into<String>,
    ) -> Self {
        self.operations.push(MigrationOp::CreateStore {
            name: name.into(),
            key_path: key_path.into(),
            auto_increment: true,
        });
        self
    }

    /// Delete a store
    pub fn delete_store(mut self, name: impl Into<String>) -> Self {
        self.operations
            .push(MigrationOp::DeleteStore { name: name.into() });
        self
    }

    /// Create an index on a store
    pub fn create_index(
        mut self,
        store_name: impl Into<String>,
        index_name: impl Into<String>,
        key_path: impl Into<String>,
        unique: bool,
    ) -> Self {
        self.operations.push(MigrationOp::CreateIndex {
            store_name: store_name.into(),
            index_name: index_name.into(),
            key_path: key_path.into(),
            unique,
        });
        self
    }

    /// Delete an index from a store
    pub fn delete_index(
        mut self,
        store_name: impl Into<String>,
        index_name: impl Into<String>,
    ) -> Self {
        self.operations.push(MigrationOp::DeleteIndex {
            store_name: store_name.into(),
            index_name: index_name.into(),
        });
        self
    }

    /// Set a data migration function
    pub fn with_data_migration(mut self, f: fn(&IdbDatabase)) -> Self {
        self.data_migration = Some(f);
        self
    }

    /// Execute this migration
    pub(crate) fn execute(
        &self,
        db: &IdbDatabase,
        old_version: u32,
        new_version: u32,
    ) -> Result<()> {
        // Only run if this migration applies to the upgrade path
        if self.version > old_version && self.version <= new_version {
            log::info!(
                "Running migration to version {} (from {} to {})",
                self.version,
                old_version,
                new_version
            );

            for op in &self.operations {
                op.execute(db)?;
            }

            // Run data migration if provided
            if let Some(data_migration) = self.data_migration {
                log::info!("Running data migration for version {}", self.version);
                data_migration(db);
            }
        }

        Ok(())
    }
}

/// A migration operation
#[derive(Debug, Clone)]
pub enum MigrationOp {
    /// Create an object store
    CreateStore {
        name: String,
        key_path: String,
        auto_increment: bool,
    },
    /// Delete an object store
    DeleteStore { name: String },
    /// Create an index
    CreateIndex {
        store_name: String,
        index_name: String,
        key_path: String,
        unique: bool,
    },
    /// Delete an index
    DeleteIndex {
        store_name: String,
        index_name: String,
    },
}

impl MigrationOp {
    /// Execute this operation
    fn execute(&self, db: &IdbDatabase) -> Result<()> {
        match self {
            MigrationOp::CreateStore {
                name,
                key_path,
                auto_increment,
            } => {
                // Check if store already exists
                if db.store_names().contains(name) {
                    log::warn!("Store '{}' already exists, skipping creation", name);
                    return Ok(());
                }

                let mut params = ObjectStoreParams::new();
                params.key_path(Some(KeyPath::new_single(key_path)));
                params.auto_increment(*auto_increment);

                db.create_object_store(name, params).map_err(|e| {
                    IndexedDbError::Database(format!("Failed to create store '{}': {:?}", name, e))
                })?;

                log::info!("Created store '{}'", name);
                Ok(())
            }

            MigrationOp::DeleteStore { name } => {
                if !db.store_names().contains(name) {
                    log::warn!("Store '{}' doesn't exist, skipping deletion", name);
                    return Ok(());
                }

                db.delete_object_store(name).map_err(|e| {
                    IndexedDbError::Database(format!("Failed to delete store '{}': {:?}", name, e))
                })?;

                log::info!("Deleted store '{}'", name);
                Ok(())
            }

            MigrationOp::CreateIndex {
                store_name,
                index_name,
                key_path: _,
                unique: _,
            } => {
                // Note: This would need access to the transaction to create an index
                // For now, log that this needs to be handled differently
                log::warn!(
                    "Creating indexes requires transaction access. \
                     Index '{}' on store '{}' should be created during store creation or in a separate migration.",
                    index_name,
                    store_name
                );
                Ok(())
            }

            MigrationOp::DeleteIndex {
                store_name,
                index_name,
            } => {
                log::warn!(
                    "Deleting index '{}' on store '{}' requires transaction access",
                    index_name,
                    store_name
                );
                Ok(())
            }
        }
    }
}

/// Migration manager that handles running migrations
#[derive(Debug, Clone, Default)]
pub struct MigrationManager {
    migrations: Vec<Migration>,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    /// Add a migration
    pub fn add_migration(mut self, migration: Migration) -> Self {
        self.migrations.push(migration);
        self
    }

    /// Run all applicable migrations
    pub(crate) fn run_migrations(
        &self,
        db: &IdbDatabase,
        old_version: u32,
        new_version: u32,
    ) -> Result<()> {
        // Sort migrations by version
        let mut migrations = self.migrations.clone();
        migrations.sort_by_key(|m| m.version);

        for migration in &migrations {
            migration.execute(db, old_version, new_version)?;
        }

        Ok(())
    }
}

/// Builder for database configuration with migrations
#[derive(Debug, Clone)]
pub struct DatabaseBuilder {
    name: String,
    version: u32,
    migrations: Vec<Migration>,
}

impl DatabaseBuilder {
    /// Create a new database builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: 1,
            migrations: Vec::new(),
        }
    }

    /// Set the database version
    pub fn version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    /// Add a migration
    pub fn with_migration(mut self, migration: Migration) -> Self {
        self.migrations.push(migration);
        self
    }

    /// Get the database name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the database version
    pub fn get_version(&self) -> u32 {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_builder() {
        let migration = Migration::new(2)
            .create_store("users", "id")
            .create_store("posts", "id")
            .delete_store("old_store");

        assert_eq!(migration.version, 2);
        assert_eq!(migration.operations.len(), 3);
    }

    #[test]
    fn test_migration_manager() {
        let manager = MigrationManager::new()
            .add_migration(Migration::new(2).create_store("users", "id"))
            .add_migration(Migration::new(3).create_store("posts", "id"));

        assert_eq!(manager.migrations.len(), 2);
    }
}
