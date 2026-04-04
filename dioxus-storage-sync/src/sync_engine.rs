//! Core sync engine

use crate::{
    client::HttpClient,
    config::{ConflictResolution, SyncConfig, SyncMode},
    traits::{BackendAdapter, Syncable},
    Result, SyncError,
};
use dioxus_indexeddb::Collection;
use serde::{de::DeserializeOwned, Serialize};

/// Sync engine that handles hot and background sync
#[derive(Debug, Clone)]
pub struct SyncEngine<T: Syncable> {
    collection: Collection<T>,
    config: SyncConfig,
    client: HttpClient,
}

impl<T: Syncable + Serialize + DeserializeOwned> SyncEngine<T> {
    /// Create a new sync engine
    pub fn new(collection: Collection<T>, config: SyncConfig) -> Self {
        let client = HttpClient::new(config.clone());
        Self {
            collection,
            config,
            client,
        }
    }

    /// Get the underlying collection
    pub fn collection(&self) -> &Collection<T> {
        &self.collection
    }

    /// Hot sync: fetch from backend when local query returns empty
    pub async fn hot_sync(&self, _query: &dioxus_indexeddb::Query) -> Result<Vec<T>> {
        if !self.config.is_hot_sync_enabled() {
            return Ok(Vec::new());
        }
        
        log::info!("Performing hot sync");
        
        // Fetch from backend (simplified - no query params)
        let items: Vec<T> = self.client.fetch_with_retry("items", None).await?;
        
        // Store locally
        for item in &items {
            let mut item = item.clone();
            item.mark_synced();
            let _ = self.collection.put(&item.sync_id(), &item).await;
        }
        
        Ok(items)
    }

    /// Fetch single item with hot sync fallback
    pub async fn get_with_sync(&self, id: &str) -> Result<Option<T>> {
        // Try local first
        match self.collection.get(id).await {
            Ok(Some(item)) => return Ok(Some(item)),
            _ => {}
        }
        
        // If hot sync enabled, fetch from backend
        if self.config.is_hot_sync_enabled() {
            log::info!("Item {} not found locally, fetching from backend", id);
            
            let path = format!("items/{}", id);
            match self.client.get(&path).await {
                Ok(item) => {
                    // Store locally
                    let mut item: T = item;
                    item.mark_synced();
                    let _ = self.collection.put(id, &item).await;
                    return Ok(Some(item));
                }
                Err(SyncError::Http(ref e)) if e.contains("404") => return Ok(None),
                Err(e) => return Err(e),
            }
        }
        
        Ok(None)
    }

    /// Background sync: full synchronization
    pub async fn background_sync(&self) -> Result<SyncResult> {
        if !self.config.is_background_sync_enabled() {
            return Ok(SyncResult::default());
        }
        
        log::info!("Starting background sync");
        
        let mut result = SyncResult::default();
        
        match self.config.mode {
            SyncMode::PullOnly | SyncMode::Bidirectional => {
                result = self.pull_from_backend().await?;
            }
            _ => {}
        }
        
        match self.config.mode {
            SyncMode::PushOnly | SyncMode::Bidirectional => {
                let push_result = self.push_to_backend().await?;
                result.merge(push_result);
            }
            _ => {}
        }
        
        log::info!("Background sync complete: {:?}", result);
        Ok(result)
    }

    /// Pull changes from backend
    async fn pull_from_backend(&self) -> Result<SyncResult> {
        let mut result = SyncResult {
            pulled: 0,
            pushed: 0,
            conflicts: 0,
            errors: Vec::new(),
        };
        
        // Get last sync timestamp
        let since = self.get_last_sync_timestamp().await?;
        
        // Build params
        let params = since.map(|ts| serde_json::json!({ "since": ts }));
        
        // Fetch from backend
        let items: Vec<T> = if let Some(p) = params {
            self.client.fetch_with_retry("items/sync", Some(&p)).await?
        } else {
            self.client.fetch_with_retry::<Vec<T>>("items", None).await?
        };
        
        // Merge into local DB
        for item in items {
            let id = item.sync_id();
            
            match self.merge_item(item).await {
                Ok(MergeResult::Inserted) => result.pulled += 1,
                Ok(MergeResult::Updated) => result.pulled += 1,
                Ok(MergeResult::Conflict) => result.conflicts += 1,
                Ok(MergeResult::Skipped) => {}
                Err(e) => {
                    result.errors.push(format!("Failed to merge {}: {}", id, e));
                }
            }
        }
        
        // Update last sync timestamp
        self.update_last_sync_timestamp().await?;
        
        Ok(result)
    }

    /// Push local changes to backend
    async fn push_to_backend(&self) -> Result<SyncResult> {
        let mut result = SyncResult {
            pulled: 0,
            pushed: 0,
            conflicts: 0,
            errors: Vec::new(),
        };
        
        // Get dirty items
        let local_items = self.collection.get_all().await
            .map_err(|e| SyncError::IndexedDb(e.to_string()))?;
        
        let dirty_items: Vec<T> = local_items
            .into_iter()
            .filter(|item| item.is_dirty())
            .collect();
        
        if dirty_items.is_empty() {
            log::info!("No local changes to push");
            return Ok(result);
        }
        
        log::info!("Pushing {} items to backend", dirty_items.len());
        
        // Push in batches
        for chunk in dirty_items.chunks(self.config.batch_size) {
            let path = "items/batch";
            
            match self.client.post::<serde_json::Value, _>(path, &chunk).await {
                Ok(_) => {
                    result.pushed += chunk.len();
                    
                    // Mark as synced locally
                    for item in chunk {
                        let mut item = item.clone();
                        item.mark_synced();
                        let _ = self.collection.put(&item.sync_id(), &item).await;
                    }
                }
                Err(e) => {
                    result.errors.push(format!("Batch push failed: {}", e));
                }
            }
        }
        
        Ok(result)
    }

    /// Merge a single item from backend
    async fn merge_item(&self, backend_item: T) -> Result<MergeResult> {
        let id = backend_item.sync_id();
        
        match self.collection.get(&id).await {
            Ok(Some(local_item)) => {
                // Check for conflicts
                if local_item.is_dirty() {
                    // Conflict: both sides have changes
                    match self.config.conflict_resolution {
                        ConflictResolution::ServerWins => {
                            // Replace with backend version
                            let mut item = backend_item;
                            item.mark_synced();
                            self.collection.put(&id, &item).await
                                .map_err(|e| SyncError::IndexedDb(e.to_string()))?;
                            Ok(MergeResult::Updated)
                        }
                        ConflictResolution::LocalWins => {
                            // Keep local, will be pushed
                            Ok(MergeResult::Skipped)
                        }
                        ConflictResolution::LastWriteWins => {
                            let backend_time = backend_item.sync_timestamp();
                            let local_time = local_item.sync_timestamp();
                            
                            if backend_time > local_time {
                                let mut item = backend_item;
                                item.mark_synced();
                                self.collection.put(&id, &item).await
                                    .map_err(|e| SyncError::IndexedDb(e.to_string()))?;
                                Ok(MergeResult::Updated)
                            } else {
                                Ok(MergeResult::Skipped)
                            }
                        }
                        ConflictResolution::Manual => {
                            // Mark as conflict for manual resolution
                            Ok(MergeResult::Conflict)
                        }
                    }
                } else {
                    // No local changes, accept backend version
                    let mut item = backend_item;
                    item.mark_synced();
                    self.collection.put(&id, &item).await
                        .map_err(|e| SyncError::IndexedDb(e.to_string()))?;
                    Ok(MergeResult::Updated)
                }
            }
            Ok(None) => {
                // New item from backend
                let mut item = backend_item;
                item.mark_synced();
                self.collection.put(&id, &item).await
                    .map_err(|e| SyncError::IndexedDb(e.to_string()))?;
                Ok(MergeResult::Inserted)
            }
            Err(e) => Err(SyncError::IndexedDb(e.to_string())),
        }
    }

    /// Get last sync timestamp
    async fn get_last_sync_timestamp(&self) -> Result<Option<i64>> {
        // For now, return None - in production this would query a metadata store
        Ok(None)
    }

    /// Update last sync timestamp
    async fn update_last_sync_timestamp(&self) -> Result<()> {
        // For now, just log - in production this would update a metadata store
        log::info!("Sync timestamp updated");
        Ok(())
    }
}

/// Sync metadata stored in IndexedDB
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SyncMetadata {
    id: String,
    timestamp: i64,
}

impl Syncable for SyncMetadata {
    fn sync_id(&self) -> String {
        self.id.clone()
    }
    
    fn sync_timestamp(&self) -> i64 {
        self.timestamp
    }
    
    fn mark_synced(&mut self) {}
    
    fn is_dirty(&self) -> bool {
        false
    }
}

/// Result of a sync operation
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    pub pulled: usize,
    pub pushed: usize,
    pub conflicts: usize,
    pub errors: Vec<String>,
}

impl SyncResult {
    /// Merge another result into this one
    pub fn merge(&mut self, other: SyncResult) {
        self.pulled += other.pulled;
        self.pushed += other.pushed;
        self.conflicts += other.conflicts;
        self.errors.extend(other.errors);
    }
    
    /// Check if sync was successful
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Internal merge result
enum MergeResult {
    Inserted,
    Updated,
    Conflict,
    Skipped,
}
