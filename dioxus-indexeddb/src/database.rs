//! Database management

use crate::collection::Collection;
use crate::error::{IndexedDbError, Result};
use crate::migration::{Migration, MigrationManager};
use idb::{Database as IdbDatabase, DatabaseEvent, Factory, KeyPath, ObjectStoreParams};
use std::cell::RefCell;
use std::rc::Rc;

/// Configuration for opening a database
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database name
    pub name: String,
    /// Database version
    pub version: u32,
    /// Object store configurations
    pub stores: Vec<StoreConfig>,
}

impl DatabaseConfig {
    /// Create a new database configuration
    pub fn new(name: impl Into<String>, version: u32) -> Self {
        Self {
            name: name.into(),
            version,
            stores: Vec::new(),
        }
    }

    /// Add an object store
    pub fn with_store(mut self, name: impl Into<String>, key_path: impl Into<String>) -> Self {
        self.stores.push(StoreConfig {
            name: name.into(),
            key_path: key_path.into(),
            auto_increment: false,
            indexes: Vec::new(),
        });
        self
    }

    /// Add an object store with auto-increment
    pub fn with_auto_increment_store(
        mut self,
        name: impl Into<String>,
        key_path: impl Into<String>,
    ) -> Self {
        self.stores.push(StoreConfig {
            name: name.into(),
            key_path: key_path.into(),
            auto_increment: true,
            indexes: Vec::new(),
        });
        self
    }
}

/// Object store configuration
#[derive(Debug, Clone)]
pub struct StoreConfig {
    pub name: String,
    pub key_path: String,
    pub auto_increment: bool,
    pub indexes: Vec<IndexConfig>,
}

/// Index configuration
#[derive(Debug, Clone)]
pub struct IndexConfig {
    pub name: String,
    pub key_path: String,
    pub unique: bool,
}

/// A connection to an IndexedDB database
#[derive(Debug, Clone)]
pub struct Database {
    inner: Rc<RefCell<IdbDatabase>>,
}

impl Database {
    /// Open a database with the given configuration
    pub async fn open(config: DatabaseConfig) -> Result<Self> {
        Self::open_with_migrations(config, MigrationManager::new()).await
    }

    /// Open a database with migrations
    pub async fn open_with_migrations(
        config: DatabaseConfig,
        migrations: MigrationManager,
    ) -> Result<Self> {
        let factory = Factory::new().map_err(|_| IndexedDbError::NotAvailable)?;

        let mut open_request = factory
            .open(&config.name, Some(config.version))
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        let stores = config.stores.clone();

        open_request.on_upgrade_needed(move |event| {
            let database = match event.database() {
                Ok(db) => db,
                Err(e) => {
                    log::error!("Failed to get database from upgrade event: {:?}", e);
                    return;
                }
            };

            // Get old version from event (0 if new database)
            let old_version = event.old_version().unwrap_or(0);
            let new_version = event
                .new_version()
                .ok()
                .flatten()
                .unwrap_or(old_version + 1);

            // Run migrations first
            if let Err(e) = migrations.run_migrations(&database, old_version, new_version) {
                log::error!("Migration failed: {}", e);
            }

            // Then create any stores from config that don't exist yet
            let store_names = database.store_names();

            for store_config in &stores {
                // Check if store already exists
                if store_names.contains(&store_config.name) {
                    continue;
                }

                let mut params = ObjectStoreParams::new();
                params.key_path(Some(KeyPath::new_single(&store_config.key_path)));
                params.auto_increment(store_config.auto_increment);

                if let Err(e) = database.create_object_store(&store_config.name, params) {
                    log::error!(
                        "Failed to create object store '{}': {:?}",
                        store_config.name,
                        e
                    );
                }
            }
        });

        let database = open_request
            .await
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        Ok(Self {
            inner: Rc::new(RefCell::new(database)),
        })
    }

    /// Get a collection for the given store name
    pub fn collection<T: serde::Serialize + serde::de::DeserializeOwned>(
        &self,
        name: &str,
    ) -> Collection<T> {
        Collection::new(self.inner.clone(), name.to_string())
    }

    /// Get access to the inner database
    pub(crate) fn with_db<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&IdbDatabase) -> R,
    {
        f(&*self.inner.borrow())
    }

    /// Check if a store exists
    pub fn has_store(&self, name: &str) -> bool {
        self.with_db(|db| db.store_names().contains(&name.to_string()))
    }

    /// Delete the database
    pub async fn delete(name: &str) -> Result<()> {
        let factory = Factory::new().map_err(|_| IndexedDbError::NotAvailable)?;

        let req = factory
            .delete(name)
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        req.await
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        Ok(())
    }

    /// Check if IndexedDB is available
    pub fn is_available() -> bool {
        web_sys::window()
            .and_then(|w| w.indexed_db().ok())
            .is_some()
    }
}

// Note: idb::Database doesn't implement Clone, so we use reference counting internally
