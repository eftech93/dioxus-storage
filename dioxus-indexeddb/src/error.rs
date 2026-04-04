//! Error types for dioxus-indexeddb

use thiserror::Error;

pub type Result<T> = std::result::Result<T, IndexedDbError>;

#[derive(Error, Debug, Clone)]
pub enum IndexedDbError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Store not found: {0}")]
    StoreNotFound(String),

    #[error("Key not found: {0}")]
    NotFound(String),

    #[error("Constraint violation: {0}")]
    Constraint(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("IndexedDB not available in this environment")]
    NotAvailable,
}

impl From<idb::Error> for IndexedDbError {
    fn from(err: idb::Error) -> Self {
        IndexedDbError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for IndexedDbError {
    fn from(err: serde_json::Error) -> Self {
        IndexedDbError::Serialization(err.to_string())
    }
}
