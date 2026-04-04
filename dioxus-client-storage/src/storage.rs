//! Unified storage API

use crate::error::{StorageError, Result};
use crate::local_storage::LocalStorage;
use crate::session_storage::SessionStorage;
use dioxus::hooks::*;
use dioxus_signals::*;
use serde::{de::DeserializeOwned, Serialize};

#[cfg(feature = "indexeddb")]
use dioxus_indexeddb::{Database, DatabaseConfig};

/// Storage backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageBackend {
    /// Browser's LocalStorage (persistent, ~5-10MB)
    Local,
    /// Browser's SessionStorage (per-session, ~5-10MB)
    Session,
    /// IndexedDB (large data, structured, async)
    #[cfg(feature = "indexeddb")]
    IndexedDb,
}

/// Configuration for storage
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Which backend to use
    pub backend: StorageBackend,
    /// Key prefix (for namespacing)
    pub key_prefix: Option<String>,
}

impl StorageConfig {
    /// Use LocalStorage
    pub fn local() -> Self {
        Self {
            backend: StorageBackend::Local,
            key_prefix: None,
        }
    }

    /// Use SessionStorage
    pub fn session() -> Self {
        Self {
            backend: StorageBackend::Session,
            key_prefix: None,
        }
    }

    /// Use IndexedDB
    #[cfg(feature = "indexeddb")]
    pub fn indexed_db() -> Self {
        Self {
            backend: StorageBackend::IndexedDb,
            key_prefix: None,
        }
    }

    /// Set key prefix
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }
}

/// Unified storage interface
#[derive(Debug, Clone)]
pub struct Storage {
    config: StorageConfig,
}

impl Storage {
    /// Create a new storage instance
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }

    /// Get the full key (with prefix if set)
    fn full_key(&self, key: &str) -> String {
        match &self.config.key_prefix {
            Some(prefix) => format!("{}:{}", prefix, key),
            None => key.to_string(),
        }
    }

    /// Get an item (sync for Local/Session, async for IndexedDB)
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let key = self.full_key(key);
        
        match self.config.backend {
            StorageBackend::Local => LocalStorage::get(&key),
            StorageBackend::Session => SessionStorage::get(&key),
            #[cfg(feature = "indexeddb")]
            StorageBackend::IndexedDb => {
                panic!("Use get_async for IndexedDB backend")
            }
        }
    }

    /// Set an item (sync for Local/Session)
    pub fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let key = self.full_key(key);
        
        match self.config.backend {
            StorageBackend::Local => LocalStorage::set(&key, value),
            StorageBackend::Session => SessionStorage::set(&key, value),
            #[cfg(feature = "indexeddb")]
            StorageBackend::IndexedDb => {
                panic!("Use set_async for IndexedDB backend")
            }
        }
    }

    /// Remove an item
    pub fn remove(&self, key: &str) -> Result<()> {
        let key = self.full_key(key);
        
        match self.config.backend {
            StorageBackend::Local => LocalStorage::remove(&key),
            StorageBackend::Session => SessionStorage::remove(&key),
            #[cfg(feature = "indexeddb")]
            StorageBackend::IndexedDb => {
                panic!("Use remove_async for IndexedDB backend")
            }
        }
    }

    /// Check if storage is available
    pub fn is_available(&self) -> bool {
        match self.config.backend {
            StorageBackend::Local => LocalStorage::is_available(),
            StorageBackend::Session => SessionStorage::is_available(),
            #[cfg(feature = "indexeddb")]
            StorageBackend::IndexedDb => Database::is_available(),
        }
    }
}

/// Hook for unified storage
///
/// Example:
/// ```rust,ignore
/// let storage = use_storage(StorageConfig::local().with_prefix("my_app"));
/// let theme = use_storage_value(&storage, "theme", "light".to_string());
/// ```
pub fn use_storage(config: StorageConfig) -> Signal<Storage> {
    use_signal(|| Storage::new(config))
}

/// Hook for a specific storage value
pub fn use_storage_value<T: Serialize + DeserializeOwned + Clone>(
    storage: &Storage,
    key: impl Into<String>,
    default: T,
) -> Signal<T> {
    let key = key.into();
    let storage = storage.clone();
    
    let initial = storage.get::<T>(&key)
        .ok()
        .flatten()
        .unwrap_or(default);
    
    let signal = use_signal(|| initial);
    
    {
        let key = key.clone();
        let storage = storage.clone();
        use_effect(move || {
            let value = signal.read().clone();
            if let Err(e) = storage.set(&key, &value) {
                log::warn!("Failed to save to storage: {}", e);
            }
        });
    }
    
    signal
}

/// Hook for IndexedDB database (when using IndexedDB backend)
#[cfg(feature = "indexeddb")]
pub fn _use_storage_db(_config: DatabaseConfig) -> Signal<Option<Database>> {
    use_signal(|| None::<Database>)
}
