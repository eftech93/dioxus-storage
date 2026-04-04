//! Dioxus Storage Sync
//!
//! Backend synchronization for dioxus-storage with two modes:
//!
//! # Hot Sync (On-Demand)
//!
//! Fetches data from backend when a query doesn't find results locally.
//!
//! ```rust,ignore
//! let synced_collection = SyncedCollection::new(
//!     local_collection,
//!     SyncConfig::new("https://api.example.com/tasks")
//!         .with_hot_sync(true)
//! );
//!
//! // This will query local DB first, then fetch from backend if empty
//! let tasks = synced_collection.query(
//!     Query::new().filter(Filter::eq("status", "active"))
//! ).await?;
//! ```
//!
//! # Background Sync (Periodic)
//!
//! Automatically syncs data every N seconds.
//!
//! ```rust,ignore
//! let sync = SyncManager::new(
//!     SyncConfig::new("https://api.example.com/tasks")
//!         .with_background_sync(Duration::from_secs(30))
//! );
//!
//! sync.start().await?; // Runs forever, syncing every 30s
//! ```

#![cfg(target_arch = "wasm32")]

use thiserror::Error;

mod client;
mod config;
mod manager;
mod sync_engine;
mod traits;

pub use client::{HttpClient, SyncClient};
pub use config::{ConflictResolution, SyncConfig, SyncMode};
pub use manager::{SyncManager, SyncStatus};
pub use sync_engine::{SyncEngine, SyncResult};
pub use traits::{Syncable, BackendAdapter};

/// Result type for sync operations
pub type Result<T> = std::result::Result<T, SyncError>;

/// Sync error types
#[derive(Error, Debug, Clone)]
pub enum SyncError {
    #[error("HTTP error: {0}")]
    Http(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("IndexedDB error: {0}")]
    IndexedDb(String),
    
    #[error("Conflict detected: {0}")]
    Conflict(String),
    
    #[error("Sync not configured")]
    NotConfigured,
    
    #[error("Backend unavailable")]
    BackendUnavailable,
    
    #[error("Authentication required")]
    Unauthorized,
    
    #[error("Rate limited")]
    RateLimited,
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<reqwest::Error> for SyncError {
    fn from(e: reqwest::Error) -> Self {
        SyncError::Http(e.to_string())
    }
}

impl From<serde_json::Error> for SyncError {
    fn from(e: serde_json::Error) -> Self {
        SyncError::Serialization(e.to_string())
    }
}

impl From<dioxus_indexeddb::IndexedDbError> for SyncError {
    fn from(e: dioxus_indexeddb::IndexedDbError) -> Self {
        SyncError::IndexedDb(e.to_string())
    }
}

/// Prelude for convenient imports
pub mod prelude {
    pub use super::{SyncConfig, SyncMode, ConflictResolution};
    pub use super::{SyncManager, SyncStatus};
    pub use super::{SyncEngine, SyncResult};
    pub use super::{BackendAdapter, Syncable};
    pub use super::{HttpClient, SyncClient};
    pub use super::{SyncError, Result};
    pub use dioxus_signals::Signal;
}
