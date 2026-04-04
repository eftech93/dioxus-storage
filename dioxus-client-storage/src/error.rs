//! Error types for dioxus-client-storage

use thiserror::Error;

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Error, Debug, Clone)]
pub enum StorageError {
    #[error("Storage not available")]
    NotAvailable,

    #[error("Quota exceeded")]
    QuotaExceeded,

    #[error("Key not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IndexedDB error: {0}")]
    IndexedDb(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::Serialization(err.to_string())
    }
}

#[cfg(feature = "indexeddb")]
impl From<dioxus_indexeddb::IndexedDbError> for StorageError {
    fn from(err: dioxus_indexeddb::IndexedDbError) -> Self {
        StorageError::IndexedDb(err.to_string())
    }
}
