//! Sync manager for background and hot sync

use crate::{
    config::SyncConfig,
    sync_engine::{SyncEngine, SyncResult},
    traits::Syncable,
    Result, SyncError,
};
use dioxus::prelude::*;
use dioxus_indexeddb::Collection;
use dioxus_signals::{Signal, Writable, Readable};
use serde::{de::DeserializeOwned, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

/// Sync manager that handles background sync scheduling
#[derive(Debug, Clone)]
pub struct SyncManager<T: Syncable> {
    engine: SyncEngine<T>,
    config: SyncConfig,
    status: Signal<SyncStatus>,
    is_running: Rc<RefCell<bool>>,
}

/// Sync status for UI display
#[derive(Debug, Clone, Default)]
pub struct SyncStatus {
    /// Is currently syncing
    pub is_syncing: bool,
    /// Last sync result
    pub last_result: Option<SyncResult>,
    /// Last sync time
    pub last_sync_time: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

impl<T: Syncable + Serialize + DeserializeOwned> SyncManager<T> {
    /// Create a new sync manager
    pub fn new(collection: Collection<T>, config: SyncConfig) -> Self {
        let engine = SyncEngine::new(collection, config.clone());
        let status = use_signal(SyncStatus::default);
        
        Self {
            engine,
            config,
            status,
            is_running: Rc::new(RefCell::new(false)),
        }
    }

    /// Get the current status
    pub fn status(&self) -> Signal<SyncStatus> {
        self.status
    }

    /// Start background sync loop
    pub fn start(&self) {
        if *self.is_running.borrow() {
            log::warn!("Sync manager already running");
            return;
        }
        
        let interval = match self.config.background_sync {
            Some(d) => d,
            None => {
                log::info!("Background sync not enabled");
                return;
            }
        };
        
        *self.is_running.borrow_mut() = true;
        
        let engine = self.engine.clone();
        let mut status = self.status;
        let is_running = self.is_running.clone();
        
        spawn(async move {
            log::info!("Starting background sync loop");
            
            loop {
                if !*is_running.borrow() {
                    log::info!("Sync loop stopped");
                    break;
                }
                
                // Perform sync
                match engine.background_sync().await {
                    Ok(result) => {
                        let now = js_sys::Date::new_0();
                        let time_str = format!(
                            "{:02}:{:02}:{:02}",
                            now.get_hours(),
                            now.get_minutes(),
                            now.get_seconds()
                        );
                        
                        status.set(SyncStatus {
                            is_syncing: false,
                            last_result: Some(result),
                            last_sync_time: Some(time_str),
                            error: None,
                        });
                        
                        log::info!("Background sync completed");
                    }
                    Err(e) => {
                        log::error!("Background sync failed: {}", e);
                        let current = status.read().clone();
                        status.set(SyncStatus {
                            is_syncing: false,
                            error: Some(e.to_string()),
                            ..current
                        });
                    }
                }
                
                gloo_timers::future::sleep(interval).await;
            }
        });
    }

    /// Stop background sync
    pub fn stop(&self) {
        *self.is_running.borrow_mut() = false;
        log::info!("Sync manager stopped");
    }

    /// Perform manual sync
    pub async fn sync_now(&mut self) -> Result<SyncResult> {
        let result = self.engine.background_sync().await;
        
        match &result {
            Ok(r) => {
                let now = js_sys::Date::new_0();
                let time_str = format!(
                    "{:02}:{:02}:{:02}",
                    now.get_hours(),
                    now.get_minutes(),
                    now.get_seconds()
                );
                self.status.set(SyncStatus {
                    is_syncing: false,
                    last_result: Some(r.clone()),
                    last_sync_time: Some(time_str),
                    error: None,
                });
            }
            Err(e) => {
                let current = self.status.read().clone();
                self.status.set(SyncStatus {
                    is_syncing: false,
                    error: Some(e.to_string()),
                    ..current
                });
            }
        }
        
        result
    }

    /// Query with hot sync fallback
    pub async fn query_with_hot_sync(
        &self,
        query: &dioxus_indexeddb::Query,
    ) -> Result<Vec<T>> {
        // Query local first
        let local_results = self.engine.collection().get_all().await
            .map_err(|e| SyncError::IndexedDb(e.to_string()))?;
        
        let filtered = dioxus_indexeddb::execute_query(local_results, query);
        
        // If hot sync enabled and no results, fetch from backend
        if self.config.is_hot_sync_enabled() && filtered.items.is_empty() {
            log::info!("No local results, performing hot sync");
            
            match self.engine.hot_sync(query).await {
                Ok(backend_items) => return Ok(backend_items),
                Err(e) => {
                    log::warn!("Hot sync failed: {}", e);
                    return Ok(Vec::new());
                }
            }
        }
        
        Ok(filtered.items)
    }

    /// Get item by ID with hot sync
    pub async fn get(&self, id: &str) -> Result<Option<T>> {
        self.engine.get_with_sync(id).await
    }

    /// Save item locally
    pub async fn save(&self, item: &T) -> Result<()> {
        self.engine.collection().put(&item.sync_id(), item).await
            .map_err(|e| SyncError::IndexedDb(e.to_string()))
    }

    /// Delete item
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.engine.collection().delete(id).await
            .map_err(|e| SyncError::IndexedDb(e.to_string()))
    }

    /// Get all items locally
    pub async fn get_all(&self) -> Result<Vec<T>> {
        self.engine.collection().get_all().await
            .map_err(|e| SyncError::IndexedDb(e.to_string()))
    }
}

/// Hook for using sync manager
pub fn use_sync_manager<T: Syncable + Serialize + DeserializeOwned>(
    collection: Collection<T>,
    config: SyncConfig,
) -> SyncManager<T> {
    SyncManager::new(collection, config)
}
