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

/// Default backend adapter using HTTP
#[derive(Debug, Clone)]
pub struct HttpBackendAdapter<T: Syncable> {
    client: crate::client::HttpClient,
    endpoint: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Syncable> HttpBackendAdapter<T> {
    /// Create a new HTTP backend adapter
    pub fn new(client: crate::client::HttpClient, endpoint: impl Into<String>) -> Self {
        Self {
            client,
            endpoint: endpoint.into(),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait(?Send)]
impl<T: Syncable> BackendAdapter for HttpBackendAdapter<T> {
    type Item = T;
    
    async fn fetch_since(&self, since: Option<i64>) -> Result<Vec<Self::Item>> {
        let params = since.map(|ts| serde_json::json!({ "since": ts }));
        
        if let Some(p) = params {
            self.client.get_with_params(&self.endpoint, &p).await
        } else {
            self.client.get(&self.endpoint).await
        }
    }
    
    async fn fetch_one(&self, id: &str) -> Result<Option<Self::Item>> {
        let path = format!("{}/{}", self.endpoint, id);
        match self.client.get(&path).await {
            Ok(item) => Ok(Some(item)),
            Err(crate::SyncError::Http(ref e)) if e.contains("404") => Ok(None),
            Err(e) => Err(e),
        }
    }
    
    async fn upsert(&self, item: &Self::Item) -> Result<Self::Item> {
        let path = format!("{}/{}", self.endpoint, item.sync_id());
        self.client.put(&path, item).await
    }
    
    async fn delete(&self, id: &str) -> Result<()> {
        let path = format!("{}/{}", self.endpoint, id);
        let _: serde_json::Value = self.client.delete(&path).await?;
        Ok(())
    }
    
    async fn last_sync_timestamp(&self) -> Result<Option<i64>> {
        let path = format!("{}/sync-status", self.endpoint);
        let result: serde_json::Value = self.client.get(&path).await?;
        Ok(result.get("last_sync").and_then(|v| v.as_i64()))
    }
}
