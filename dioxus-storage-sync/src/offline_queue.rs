//! Offline queue for sync operations
//!
//! Queues mutations when the device is offline and replays them when
//! connectivity is restored.

use crate::{
    client::HttpClient,
    config::{ConflictResolution, SyncConfig},
    traits::Syncable,
    Result, SyncError,
};
use dioxus_indexeddb::Collection;
use serde::{de::DeserializeOwned, Serialize};

/// A queued operation waiting to be replayed
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueuedOperation<T> {
    /// Unique ID for this queue entry
    pub id: String,
    /// The target store name
    pub store_name: String,
    /// Unix timestamp when the operation was queued
    pub timestamp: i64,
    /// The actual operation
    pub op: QueueOp<T>,
}

/// Type of queued operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum QueueOp<T> {
    /// Insert a new item
    Insert(T),
    /// Update an existing item
    Update(T),
    /// Delete an item by ID
    Delete(String),
}

/// Result of replaying the offline queue
#[derive(Debug, Clone, Default)]
pub struct QueueReplayResult {
    /// Number of operations successfully replayed
    pub success: usize,
    /// Number of operations that failed
    pub failed: usize,
    /// Number of operations with conflicts
    pub conflicts: usize,
    /// Error messages for failed operations
    pub errors: Vec<String>,
}

/// Offline queue manager
#[derive(Debug, Clone)]
pub struct OfflineQueue<T: Syncable> {
    collection: Option<Collection<QueuedOperation<T>>>,
}

impl<T: Syncable + Serialize + DeserializeOwned + Clone> OfflineQueue<T> {
    /// Create a new offline queue (not initialized)
    pub fn new() -> Self {
        Self { collection: None }
    }

    /// Initialize the queue with a collection
    pub fn with_collection(collection: Collection<QueuedOperation<T>>) -> Self {
        Self {
            collection: Some(collection),
        }
    }

    /// Check if the queue is ready
    pub fn is_ready(&self) -> bool {
        self.collection.is_some()
    }

    /// Get the number of pending operations
    pub async fn pending_count(&self) -> usize {
        match self.collection.as_ref() {
            Some(c) => c.get_all().await.map(|v| v.len()).unwrap_or(0),
            None => 0,
        }
    }

    /// Enqueue an operation
    pub async fn enqueue(&self, store_name: &str, op: QueueOp<T>) -> Result<()> {
        let collection = self.collection.as_ref().ok_or(SyncError::NotConfigured)?;
        let id = uuid::Uuid::new_v4().to_string();
        let queued = QueuedOperation {
            id,
            store_name: store_name.to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            op,
        };
        collection
            .put(&queued.id, &queued)
            .await
            .map_err(|e| SyncError::IndexedDb(e.to_string()))?;
        Ok(())
    }

    /// Remove a specific operation from the queue
    pub async fn dequeue(&self, id: &str) -> Result<()> {
        let collection = self.collection.as_ref().ok_or(SyncError::NotConfigured)?;
        collection
            .delete(id)
            .await
            .map_err(|e| SyncError::IndexedDb(e.to_string()))?;
        Ok(())
    }

    /// Get all queued operations
    pub async fn all(&self) -> Result<Vec<QueuedOperation<T>>> {
        let collection = self.collection.as_ref().ok_or(SyncError::NotConfigured)?;
        collection
            .get_all()
            .await
            .map_err(|e| SyncError::IndexedDb(e.to_string()))
    }

    /// Replay all queued operations against the backend
    pub async fn replay(
        &self,
        client: &HttpClient,
        local_collection: &Collection<T>,
        config: &SyncConfig,
    ) -> Result<QueueReplayResult> {
        let ops = self.all().await?;
        let mut result = QueueReplayResult::default();

        for op in ops {
            let res = match &op.op {
                QueueOp::Insert(item) | QueueOp::Update(item) => {
                    let path = format!("items/{}", item.sync_id());
                    client.put(&path, item).await
                }
                QueueOp::Delete(id) => {
                    let path = format!("items/{}", id);
                    let _: serde_json::Value = client.delete(&path).await?;
                    Ok(())
                }
            };

            match res {
                Ok(()) => {
                    // Apply locally so the state is consistent
                    match &op.op {
                        QueueOp::Insert(item) | QueueOp::Update(item) => {
                            let mut item = item.clone();
                            item.mark_synced();
                            let _ = local_collection.put(&item.sync_id(), &item).await;
                        }
                        QueueOp::Delete(id) => {
                            let _ = local_collection.delete(id).await;
                        }
                    }
                    let _ = self.dequeue(&op.id).await;
                    result.success += 1;
                }
                Err(SyncError::Http(ref e))
                    if e.contains("409") || e.contains("Conflict") =>
                {
                    result.conflicts += 1;
                    match config.conflict_resolution {
                        ConflictResolution::ServerWins => {
                            // Drop the local change; remove from queue.
                            let _ = self.dequeue(&op.id).await;
                        }
                        ConflictResolution::Manual => {
                            // Keep in queue for manual resolution
                        }
                        _ => {
                            // LocalWins / LastWriteWins: retry later
                        }
                    }
                }
                Err(e) => {
                    result.failed += 1;
                    result.errors.push(format!("Queue op {} failed: {}", op.id, e));
                }
            }
        }

        Ok(result)
    }
}

impl<T: Syncable> Default for OfflineQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_queue_op_variants() {
        let op: QueueOp<String> = QueueOp::Delete("id-1".to_string());
        assert!(matches!(op, QueueOp::Delete(ref id) if id == "id-1"));
    }

    #[wasm_bindgen_test]
    fn test_queued_operation_fields() {
        let op = QueuedOperation::<String> {
            id: "q-1".to_string(),
            store_name: "tasks".to_string(),
            timestamp: 1234567890,
            op: QueueOp::Delete("id-1".to_string()),
        };
        assert_eq!(op.id, "q-1");
        assert_eq!(op.store_name, "tasks");
        assert_eq!(op.timestamp, 1234567890);
    }

    #[wasm_bindgen_test]
    fn test_queue_replay_result_default() {
        let result = QueueReplayResult::default();
        assert_eq!(result.success, 0);
        assert_eq!(result.failed, 0);
        assert_eq!(result.conflicts, 0);
        assert!(result.errors.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_queue_replay_result_clone() {
        let mut result = QueueReplayResult::default();
        result.success = 5;
        result.failed = 1;
        let cloned = result.clone();
        assert_eq!(cloned.success, 5);
        assert_eq!(cloned.failed, 1);
    }
}
