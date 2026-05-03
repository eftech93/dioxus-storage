//! SessionStorage API

use crate::error::{Result, StorageError};
use dioxus::hooks::*;
use dioxus_signals::*;
use serde::{de::DeserializeOwned, Serialize};

/// SessionStorage wrapper with type-safe API
#[derive(Debug, Clone)]
pub struct SessionStorage;

impl SessionStorage {
    /// Check if SessionStorage is available
    pub fn is_available() -> bool {
        web_sys::window()
            .and_then(|w| w.session_storage().ok())
            .flatten()
            .is_some()
    }

    /// Get the raw web_sys::Storage
    fn storage() -> Result<web_sys::Storage> {
        web_sys::window()
            .and_then(|w| w.session_storage().ok())
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

        storage.clear().map_err(|_| StorageError::NotAvailable)?;

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

/// Hook to use SessionStorage with reactive updates
pub fn use_session_storage<T: Serialize + DeserializeOwned + Clone>(
    key: impl Into<String>,
    default: T,
) -> Signal<T> {
    let key = key.into();

    let initial = SessionStorage::get::<T>(&key)
        .ok()
        .flatten()
        .unwrap_or(default);

    let signal = use_signal(|| initial);

    {
        let key = key.clone();
        use_effect(move || {
            let value = signal.read().clone();
            if let Err(e) = SessionStorage::set(&key, &value) {
                log::warn!("Failed to save to SessionStorage: {}", e);
            }
        });
    }

    signal
}
