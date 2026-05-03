//! Traits for syncable types and backend adapters

use crate::Result;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

/// Trait for types that can be synced
///
/// This provides metadata for synchronization
pub trait Syncable: Serialize + DeserializeOwned + Clone + 'static {
    /// Get the unique ID for this item
    fn sync_id(&self) -> String;

    /// Get the last modified timestamp
    fn sync_timestamp(&self) -> i64;

    /// Check if this item is marked for deletion (soft delete)
    fn is_deleted(&self) -> bool {
        false
    }

    /// Mark as synced with server
    fn mark_synced(&mut self);

    /// Check if item has unsaved changes
    fn is_dirty(&self) -> bool;
}

/// Backend adapter trait
///
/// Implement this for your specific backend API
#[async_trait(?Send)]
pub trait BackendAdapter {
    /// The item type
    type Item: Syncable;

    /// Fetch all items modified since timestamp
    async fn fetch_since(&self, since: Option<i64>) -> Result<Vec<Self::Item>>;

    /// Fetch a single item by ID
    async fn fetch_one(&self, id: &str) -> Result<Option<Self::Item>>;

    /// Create or update an item
    async fn upsert(&self, item: &Self::Item) -> Result<Self::Item>;

    /// Delete an item
    async fn delete(&self, id: &str) -> Result<()>;

    /// Get last sync timestamp from backend
    async fn last_sync_timestamp(&self) -> Result<Option<i64>>;
}
