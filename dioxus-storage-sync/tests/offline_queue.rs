#![cfg(target_arch = "wasm32")]

use dioxus_indexeddb::{Database, DatabaseConfig};
use dioxus_storage_sync::{OfflineQueue, QueueOp, QueuedOperation, Syncable};
use serde::{Deserialize, Serialize};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Task {
    id: String,
    title: String,
}

impl Syncable for Task {
    fn sync_id(&self) -> String {
        self.id.clone()
    }

    fn sync_timestamp(&self) -> i64 {
        0
    }

    fn mark_synced(&mut self) {}

    fn is_dirty(&self) -> bool {
        true
    }
}

async fn setup_queue_db() -> Database {
    let _ = Database::delete("queue_test_db_sync_queue").await;
    Database::open(
        DatabaseConfig::new("queue_test_db_sync_queue", 1).with_store("operations", "id"),
    )
    .await
    .expect("open queue db")
}

#[wasm_bindgen_test]
async fn test_offline_queue_enqueue_and_pending() {
    let db = setup_queue_db().await;
    let collection = db.collection::<QueuedOperation<Task>>("operations");
    let queue = OfflineQueue::with_collection(collection);

    let task = Task {
        id: "t1".into(),
        title: "Test".into(),
    };
    queue.enqueue("tasks", QueueOp::Insert(task)).await.unwrap();

    let pending = queue.pending_count().await;
    assert_eq!(pending, 1);
}

#[wasm_bindgen_test]
async fn test_offline_queue_all_and_dequeue() {
    let db = setup_queue_db().await;
    let collection = db.collection::<QueuedOperation<Task>>("operations");
    let queue = OfflineQueue::with_collection(collection);

    let task1 = Task {
        id: "t1".into(),
        title: "One".into(),
    };
    let task2 = Task {
        id: "t2".into(),
        title: "Two".into(),
    };

    queue
        .enqueue("tasks", QueueOp::Insert(task1.clone()))
        .await
        .unwrap();
    queue
        .enqueue("tasks", QueueOp::Update(task2.clone()))
        .await
        .unwrap();

    let all = queue.all().await.unwrap();
    assert_eq!(all.len(), 2);

    queue.dequeue(&all[0].id).await.unwrap();

    let pending = queue.pending_count().await;
    assert_eq!(pending, 1);

    let remaining = queue.all().await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert!(matches!(remaining[0].op, QueueOp::Update(_)));
}

#[wasm_bindgen_test]
async fn test_offline_queue_is_ready() {
    let db = setup_queue_db().await;
    let collection = db.collection::<QueuedOperation<Task>>("operations");
    let queue = OfflineQueue::with_collection(collection);

    assert!(queue.is_ready());

    let empty_queue = OfflineQueue::<Task>::new();
    assert!(!empty_queue.is_ready());
}

#[wasm_bindgen_test]
async fn test_offline_queue_delete_op() {
    let db = setup_queue_db().await;
    let collection = db.collection::<QueuedOperation<Task>>("operations");
    let queue = OfflineQueue::with_collection(collection);

    queue
        .enqueue("tasks", QueueOp::Delete("t3".into()))
        .await
        .unwrap();

    let all = queue.all().await.unwrap();
    assert_eq!(all.len(), 1);
    assert!(matches!(all[0].op, QueueOp::Delete(ref id) if id == "t3"));
}

#[wasm_bindgen_test]
async fn test_offline_queue_multiple_stores() {
    let db = setup_queue_db().await;
    let collection = db.collection::<QueuedOperation<Task>>("operations");
    let queue = OfflineQueue::with_collection(collection);

    queue
        .enqueue("store_a", QueueOp::Insert(Task { id: "a1".into(), title: "A".into() }))
        .await
        .unwrap();
    queue
        .enqueue("store_b", QueueOp::Insert(Task { id: "b1".into(), title: "B".into() }))
        .await
        .unwrap();

    let all = queue.all().await.unwrap();
    assert_eq!(all.len(), 2);
    let stores: Vec<String> = all.into_iter().map(|q| q.store_name).collect();
    assert!(stores.contains(&"store_a".to_string()));
    assert!(stores.contains(&"store_b".to_string()));
}
