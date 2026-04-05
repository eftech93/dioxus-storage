//! Schema definition and type-safe store management
//!
//! Inspired by Prisma and Entity Framework, this module provides:
//! - Type-safe store definitions
//! - Migration-based schema evolution
//! - Code-first database generation
//!
//! # Example
//!
//! ```rust,ignore
//! // Define your schema
//! pub struct AppSchema;
//!
//! impl Schema for AppSchema {
//!     fn stores() -> Vec<StoreDefinition> {
//!         vec![
//!             Task::store_def(),
//!             User::store_def(),
//!         ]
//!     }
//! }
//!
//! // Use with migrations
//! let db = Database::open_schema(
//!     "my_app",
//!     3,
//!     AppSchema,
//!     migration_registry(),
//! ).await?;
//! ```

use crate::collection::Collection;
use crate::database::Database;
use crate::error::{IndexedDbError, Result};
use serde::{de::DeserializeOwned, Serialize};

pub mod migration_set;

pub use migration_set::{MigrationRegistry, MigrationSet, SchemaMigration};

/// A store (object store) definition
#[derive(Debug, Clone)]
pub struct StoreDefinition {
    /// Store name
    pub name: String,
    /// Key path
    pub key_path: String,
    /// Auto-increment keys
    pub auto_increment: bool,
    /// Indexes
    pub indexes: Vec<IndexDefinition>,
}

impl StoreDefinition {
    /// Create a new store definition
    pub fn new(name: impl Into<String>, key_path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            key_path: key_path.into(),
            auto_increment: false,
            indexes: Vec::new(),
        }
    }

    /// Enable auto-increment
    pub fn auto_increment(mut self) -> Self {
        self.auto_increment = true;
        self
    }

    /// Add an index
    pub fn with_index(
        mut self,
        name: impl Into<String>,
        key_path: impl Into<String>,
        unique: bool,
    ) -> Self {
        self.indexes.push(IndexDefinition {
            name: name.into(),
            key_path: key_path.into(),
            unique,
        });
        self
    }
}

/// An index definition
#[derive(Debug, Clone)]
pub struct IndexDefinition {
    pub name: String,
    pub key_path: String,
    pub unique: bool,
}

/// A trait for types that can be stored in IndexedDB
///
/// This provides type-safe access to stores
pub trait Store: Serialize + DeserializeOwned + Clone + 'static {
    /// Get the store name
    fn store_name() -> &'static str;

    /// Get the key path
    fn key_path() -> &'static str;

    /// Get the store definition
    fn store_def() -> StoreDefinition {
        StoreDefinition::new(Self::store_name(), Self::key_path())
    }

    /// Get the entity key value
    fn key(&self) -> String;
}

/// Schema definition trait
///
/// Implement this for your application's schema
pub trait Schema {
    /// Get all store definitions
    fn stores() -> Vec<StoreDefinition>;

    /// Get store names
    fn store_names() -> Vec<String> {
        Self::stores().into_iter().map(|s| s.name).collect()
    }
}

/// Schema-aware database
#[derive(Debug, Clone)]
pub struct SchemaDatabase {
    db: Database,
}

impl SchemaDatabase {
    /// Open a database with schema and migrations
    pub async fn open<S: Schema>(
        name: &str,
        version: u32,
        _schema: S,
        registry: MigrationRegistry,
    ) -> Result<Self> {
        // Build config from schema
        let mut config = crate::database::DatabaseConfig::new(name, version);
        for store in S::stores() {
            if store.auto_increment {
                config = config.with_auto_increment_store(&store.name, &store.key_path);
            } else {
                config = config.with_store(&store.name, &store.key_path);
            }
        }

        let db = crate::database::Database::open_with_migrations(config, registry.into_manager())
            .await?;

        Ok(Self { db })
    }

    /// Get a typed collection
    pub fn collection<T: Store + Clone>(&self) -> Collection<T> {
        self.db.collection(T::store_name())
    }

    /// Get the underlying database
    pub fn inner(&self) -> &Database {
        &self.db
    }

    /// Check if store exists
    pub fn has_store<T: Store>(&self) -> bool {
        self.db.has_store(T::store_name())
    }
}

/// Helper macro to define a store
///
/// ```rust,ignore
/// define_store! {
///     pub struct TaskStore "tasks" "id";
/// }
/// ```
#[macro_export]
macro_rules! define_store {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident $store_name:literal $key_path:literal;
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        $vis struct $name;

        impl $crate::schema::Store for $name {
            fn store_name() -> &'static str {
                $store_name
            }

            fn key_path() -> &'static str {
                $key_path
            }
        }
    };
}

/// Re-export for macro
pub use define_store;
