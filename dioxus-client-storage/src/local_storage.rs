//! LocalStorage API

use crate::error::{StorageError, Result};
use dioxus::hooks::*;
use dioxus_signals::*;
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::JsCast;

/// LocalStorage wrapper with type-safe API
#[derive(Debug, Clone)]
pub struct LocalStorage;

impl LocalStorage {
    /// Check if LocalStorage is available
    pub fn is_available() -> bool {
        web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            .is_some()
    }

    /// Get the raw web_sys::Storage
    fn storage() -> Result<web_sys::Storage> {
        web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            .ok_or(StorageError::NotAvailable)
    }

    /// Get an item
    pub fn get<T: DeserializeOwned>(key: &str) -> Result<Option<T>> {
        let storage = Self::storage()?;
        
        let value = storage
            .get_item(key)
            .map_err(|_| StorageError::NotAvailable)?;

        match value {
            Some(json_str) => {
                let item = serde_json::from_str(&json_str)?;
                Ok(Some(item))
            }
            None => Ok(None),
        }
    }

    /// Get a raw string item
    pub fn get_string(key: &str) -> Result<Option<String>> {
        let storage = Self::storage()?;
        
        storage
            .get_item(key)
            .map_err(|_| StorageError::NotAvailable)
    }

    /// Set an item
    pub fn set<T: Serialize>(key: &str, value: &T) -> Result<()> {
        let storage = Self::storage()?;
        let json = serde_json::to_string(value)?;
        
        storage
            .set_item(key, &json)
            .map_err(|_| StorageError::QuotaExceeded)?;
        
        Ok(())
    }

    /// Set a raw string item
    pub fn set_string(key: &str, value: &str) -> Result<()> {
        let storage = Self::storage()?;
        
        storage
            .set_item(key, value)
            .map_err(|_| StorageError::QuotaExceeded)?;
        
        Ok(())
    }

    /// Remove an item
    pub fn remove(key: &str) -> Result<()> {
        let storage = Self::storage()?;
        
        storage
            .remove_item(key)
            .map_err(|_| StorageError::NotAvailable)?;
        
        Ok(())
    }

    /// Clear all items
    pub fn clear() -> Result<()> {
        let storage = Self::storage()?;
        
        storage
            .clear()
            .map_err(|_| StorageError::NotAvailable)?;
        
        Ok(())
    }

    /// Get all keys
    pub fn keys() -> Result<Vec<String>> {
        let storage = Self::storage()?;
        let length = storage.length().map_err(|_| StorageError::NotAvailable)?;
        
        let mut keys = Vec::new();
        for i in 0..length {
            if let Ok(Some(key)) = storage.key(i) {
                keys.push(key);
            }
        }
        
        Ok(keys)
    }

    /// Check if a key exists
    pub fn has(key: &str) -> Result<bool> {
        let storage = Self::storage()?;
        
        storage
            .get_item(key)
            .map(|v| v.is_some())
            .map_err(|_| StorageError::NotAvailable)
    }
}

/// Hook to use LocalStorage with reactive updates
///
/// Example:
/// ```rust,ignore
/// let theme = use_local_storage::<String>("theme", "light".to_string());
/// 
/// // Read
/// let current_theme = theme.read();
/// 
/// // Write
/// theme.set("dark".to_string());
/// ```
pub fn use_local_storage<T: Serialize + DeserializeOwned + Clone>(
    key: impl Into<String>,
    default: T,
) -> Signal<T> {
    let key = key.into();
    
    // Try to load from storage, fall back to default
    let initial = LocalStorage::get::<T>(&key)
        .ok()
        .flatten()
        .unwrap_or(default);
    
    let signal = use_signal(|| initial);
    
    // Sync with storage when signal changes
    {
        let key = key.clone();
        use_effect(move || {
            let value = signal.read().clone();
            if let Err(e) = LocalStorage::set(&key, &value) {
                log::warn!("Failed to save to LocalStorage: {}", e);
            }
        });
    }
    
    signal
}

/// Hook for optional LocalStorage value
pub fn use_local_storage_opt<T: Serialize + DeserializeOwned + Clone>(
    key: impl Into<String>,
) -> Signal<Option<T>> {
    let key = key.into();
    
    let initial = LocalStorage::get::<T>(&key).ok().flatten();
    let signal = use_signal(|| initial);
    
    {
        let key = key.clone();
        use_effect(move || {
            let value = signal.read().clone();
            match value {
                Some(v) => {
                    if let Err(e) = LocalStorage::set(&key, &v) {
                        log::warn!("Failed to save to LocalStorage: {}", e);
                    }
                }
                None => {
                    if let Err(e) = LocalStorage::remove(&key) {
                        log::warn!("Failed to remove from LocalStorage: {}", e);
                    }
                }
            }
        });
    }
    
    signal
}
