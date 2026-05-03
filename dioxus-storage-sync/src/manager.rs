//! Sync manager for background and hot sync

use crate::{
    config::SyncConfig,
    offline_queue::{OfflineQueue, QueueOp, QueueReplayResult},
    sync_engine::{SyncEngine, SyncResult},
    traits::Syncable,
    Result, SyncError,
};
use dioxus::prelude::*;
use dioxus_indexeddb::{Collection, Database, DatabaseConfig};
use dioxus_signals::{Readable, Signal, Writable};
use serde::{de::DeserializeOwned, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

/// Sync manager that handles background sync scheduling
#[derive(Debug, Clone)]
pub struct SyncManager<T: Syncable> {
    engine: SyncEngine<T>,
    config: SyncConfig,
    status: Signal<SyncStatus>,
    is_running: Rc<RefCell<bool>>,
    queue: OfflineQueue<T>,
    queue_collection: Signal<Option<Collection<crate::offline_queue::QueuedOperation<T>>>>,
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
    /// Is the browser currently online
    pub is_online: bool,
    /// Number of pending offline operations
    pub queue_pending: usize,
    /// Is the queue currently being replayed
    pub queue_replaying: bool,
    /// Last queue replay result
    pub queue_result: Option<QueueReplayResult>,
}

impl<T: Syncable + Serialize + DeserializeOwned + Clone> SyncManager<T> {
    /// Create a new sync manager
    pub fn new(collection: Collection<T>, config: SyncConfig) -> Self {
        let engine = SyncEngine::new(collection.clone(), config.clone());
        let status = use_signal(|| SyncStatus {
            is_online: is_online(),
            ..SyncStatus::default()
        });
        let queue_collection = use_signal(|| None);

        // Initialize offline queue database
        {
            let collection_name = collection.name().to_string();
            let mut qcs = queue_collection;
            spawn(async move {
                let db_name = format!("{}_sync_queue", collection_name);
                match Database::open(
                    DatabaseConfig::new(&db_name, 1).with_store("operations", "id"),
                )
                .await
                {
                    Ok(db) => {
                        let qc =
                            db.collection::<crate::offline_queue::QueuedOperation<T>>("operations");
                        qcs.set(Some(qc));
                        log::info!("Offline queue DB initialized: {}", db_name);
                    }
                    Err(e) => {
                        log::error!("Failed to open offline queue DB: {}", e);
                    }
                }
            });
        }

        // Setup online/offline listeners
        {
            let mut status_signal = status;
            let window = match web_sys::window() {
                Some(w) => w,
                None => {
                    log::warn!("No window available for online/offline detection");
                    return Self {
                        engine,
                        config,
                        status,
                        is_running: Rc::new(RefCell::new(false)),
                        queue: OfflineQueue::new(),
                        queue_collection,
                    };
                }
            };

            // Online listener
            let online_closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let current = status_signal.read().clone();
                status_signal.set(SyncStatus {
                    is_online: true,
                    ..current
                });
                log::info!("Browser is back online");
            }) as Box<dyn FnMut(_)>);
            let _ = window.add_event_listener_with_callback(
                "online",
                online_closure.as_ref().unchecked_ref(),
            );
            online_closure.forget();

            // Offline listener
            let mut status_signal = status;
            let offline_closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let current = status_signal.read().clone();
                status_signal.set(SyncStatus {
                    is_online: false,
                    ..current
                });
                log::warn!("Browser went offline");
            }) as Box<dyn FnMut(_)>);
            let _ = window.add_event_listener_with_callback(
                "offline",
                offline_closure.as_ref().unchecked_ref(),
            );
            offline_closure.forget();
        }

        Self {
            engine,
            config,
            status,
            is_running: Rc::new(RefCell::new(false)),
            queue: OfflineQueue::new(),
            queue_collection,
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
        let queue_collection = self.queue_collection;
        let config = self.config.clone();

        spawn(async move {
            log::info!("Starting background sync loop");

            loop {
                if !*is_running.borrow() {
                    log::info!("Sync loop stopped");
                    break;
                }

                // Update queue pending count before sync
                if let Some(ref qc) = *queue_collection.read() {
                    let count = qc.get_all().await.map(|v| v.len()).unwrap_or(0);
                    let current = status.read().clone();
                    status.set(SyncStatus {
                        queue_pending: count,
                        ..current
                    });
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

                        let current = status.read().clone();
                        status.set(SyncStatus {
                            is_syncing: false,
                            last_result: Some(result),
                            last_sync_time: Some(time_str),
                            error: None,
                            ..current
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

                // Replay offline queue if online
                if is_online() {
                    if let Some(ref qc) = *queue_collection.read() {
                        let queue = OfflineQueue::with_collection(qc.clone());
                        let current = status.read().clone();
                        status.set(SyncStatus {
                            queue_replaying: true,
                            ..current
                        });

                        match queue
                            .replay(engine.client(), engine.collection(), &config)
                            .await
                        {
                            Ok(result) => {
                                let pending = queue.pending_count().await;
                                let current = status.read().clone();
                                status.set(SyncStatus {
                                    queue_replaying: false,
                                    queue_result: Some(result),
                                    queue_pending: pending,
                                    ..current
                                });
                            }
                            Err(e) => {
                                log::error!("Queue replay failed: {}", e);
                                let pending = queue.pending_count().await;
                                let current = status.read().clone();
                                status.set(SyncStatus {
                                    queue_replaying: false,
                                    queue_pending: pending,
                                    ..current
                                });
                            }
                        }
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
                let current = self.status.read().clone();
                self.status.set(SyncStatus {
                    is_syncing: false,
                    last_result: Some(r.clone()),
                    last_sync_time: Some(time_str),
                    error: None,
                    ..current
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

        // Also replay queue after manual sync
        if is_online() {
            let _ = self.replay_queue().await;
        }

        result
    }

    /// Query with hot sync fallback
    pub async fn query_with_hot_sync(&self, query: &dioxus_indexeddb::Query) -> Result<Vec<T>> {
        // Query local first
        let local_results = self
            .engine
            .collection()
            .get_all()
            .await
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

    /// Save item locally (or queue if offline)
    pub async fn save(&self, item: &T) -> Result<()> {
        if is_online() {
            self.engine
                .collection()
                .put(&item.sync_id(), item)
                .await
                .map_err(|e| SyncError::IndexedDb(e.to_string()))
        } else {
            log::info!("Offline: queuing save for {}", item.sync_id());
            if let Some(ref qc) = *self.queue_collection.read() {
                let queue = OfflineQueue::with_collection(qc.clone());
                queue
                    .enqueue(
                        self.engine.collection().name(),
                        QueueOp::Update(item.clone()),
                    )
                    .await?;
                let pending = queue.pending_count().await;
                let mut s = self.status;
                let current = s.read().clone();
                s.set(SyncStatus {
                    queue_pending: pending,
                    ..current
                });
            }
            // Still save locally so the user sees the change
            self.engine
                .collection()
                .put(&item.sync_id(), item)
                .await
                .map_err(|e| SyncError::IndexedDb(e.to_string()))
        }
    }

    /// Delete item (or queue if offline)
    pub async fn delete(&self, id: &str) -> Result<()> {
        if is_online() {
            self.engine
                .collection()
                .delete(id)
                .await
                .map_err(|e| SyncError::IndexedDb(e.to_string()))
        } else {
            log::info!("Offline: queuing delete for {}", id);
            if let Some(ref qc) = *self.queue_collection.read() {
                let queue = OfflineQueue::with_collection(qc.clone());
                queue
                    .enqueue(
                        self.engine.collection().name(),
                        QueueOp::Delete(id.to_string()),
                    )
                    .await?;
                let pending = queue.pending_count().await;
                let mut s = self.status;
                let current = s.read().clone();
                s.set(SyncStatus {
                    queue_pending: pending,
                    ..current
                });
            }
            // Still delete locally
            self.engine
                .collection()
                .delete(id)
                .await
                .map_err(|e| SyncError::IndexedDb(e.to_string()))
        }
    }

    /// Get all items locally
    pub async fn get_all(&self) -> Result<Vec<T>> {
        self.engine
            .collection()
            .get_all()
            .await
            .map_err(|e| SyncError::IndexedDb(e.to_string()))
    }

    /// Replay the offline queue manually
    pub async fn replay_queue(&self) -> Result<QueueReplayResult> {
        if !is_online() {
            log::warn!("Cannot replay queue while offline");
            return Ok(QueueReplayResult::default());
        }

        let qc = match self.queue_collection.read().as_ref() {
            Some(qc) => qc.clone(),
            None => {
                log::warn!("Queue not initialized yet");
                return Ok(QueueReplayResult::default());
            }
        };

        let queue = OfflineQueue::with_collection(qc);
        let mut s = self.status;
        let current = s.read().clone();
        s.set(SyncStatus {
            queue_replaying: true,
            ..current
        });

        let result = queue
            .replay(self.engine.client(), self.engine.collection(), &self.config)
            .await;

        let pending = queue.pending_count().await;
        match &result {
            Ok(r) => {
                let current = s.read().clone();
                s.set(SyncStatus {
                    queue_replaying: false,
                    queue_result: Some(r.clone()),
                    queue_pending: pending,
                    ..current
                });
            }
            Err(_) => {
                let current = s.read().clone();
                s.set(SyncStatus {
                    queue_replaying: false,
                    queue_pending: pending,
                    ..current
                });
            }
        }

        result
    }
}

/// Hook for using sync manager
pub fn use_sync_manager<T: Syncable + Serialize + DeserializeOwned + Clone>(
    collection: Collection<T>,
    config: SyncConfig,
) -> SyncManager<T> {
    SyncManager::new(collection, config)
}

/// Check if the browser is currently online
fn is_online() -> bool {
    web_sys::window()
        .map(|w| w.navigator().on_line())
        .unwrap_or(true)
}
