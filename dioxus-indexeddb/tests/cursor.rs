#![cfg(target_arch = "wasm32")]

use dioxus_indexeddb::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Item {
    id: String,
    name: String,
}

async fn setup_db() -> Database {
    // Clean up any previous test database
    let _ = Database::delete("cursor_test_db").await;

    let config = DatabaseConfig::new("cursor_test_db", 1).with_store("items", "id");
    Database::open(config).await.expect("open db")
}

async fn clear_store(db: &Database) {
    let collection: Collection<Item> = db.collection("items");
    let all = collection.get_all().await.unwrap();
    for item in all {
        collection.delete(&item.id).await.unwrap();
    }
}

#[wasm_bindgen_test]
async fn test_cursor_basic_iteration() {
    let db = setup_db().await;
    clear_store(&db).await;

    let collection: Collection<Item> = db.collection("items");
    collection
        .put(
            "a",
            &Item {
                id: "a".into(),
                name: "Alice".into(),
            },
        )
        .await
        .unwrap();
    collection
        .put(
            "b",
            &Item {
                id: "b".into(),
                name: "Bob".into(),
            },
        )
        .await
        .unwrap();
    collection
        .put(
            "c",
            &Item {
                id: "c".into(),
                name: "Carol".into(),
            },
        )
        .await
        .unwrap();

    let mut cursor = collection
        .open_cursor(None, Some(CursorDirection::Next))
        .await
        .unwrap();
    let mut items = Vec::new();
    while let Some(item) = cursor.next().await.unwrap() {
        items.push(item);
    }

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].name, "Alice");
    assert_eq!(items[1].name, "Bob");
    assert_eq!(items[2].name, "Carol");
}

#[wasm_bindgen_test]
async fn test_cursor_reverse_iteration() {
    let db = setup_db().await;
    clear_store(&db).await;

    let collection: Collection<Item> = db.collection("items");
    collection
        .put(
            "a",
            &Item {
                id: "a".into(),
                name: "Alice".into(),
            },
        )
        .await
        .unwrap();
    collection
        .put(
            "b",
            &Item {
                id: "b".into(),
                name: "Bob".into(),
            },
        )
        .await
        .unwrap();
    collection
        .put(
            "c",
            &Item {
                id: "c".into(),
                name: "Carol".into(),
            },
        )
        .await
        .unwrap();

    let mut cursor = collection
        .open_cursor(None, Some(CursorDirection::Prev))
        .await
        .unwrap();
    let mut items = Vec::new();
    while let Some(item) = cursor.next().await.unwrap() {
        items.push(item);
    }

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].name, "Carol");
    assert_eq!(items[1].name, "Bob");
    assert_eq!(items[2].name, "Alice");
}

#[wasm_bindgen_test]
async fn test_cursor_advance() {
    let db = setup_db().await;
    clear_store(&db).await;

    let collection: Collection<Item> = db.collection("items");
    collection
        .put(
            "a",
            &Item {
                id: "a".into(),
                name: "Alice".into(),
            },
        )
        .await
        .unwrap();
    collection
        .put(
            "b",
            &Item {
                id: "b".into(),
                name: "Bob".into(),
            },
        )
        .await
        .unwrap();
    collection
        .put(
            "c",
            &Item {
                id: "c".into(),
                name: "Carol".into(),
            },
        )
        .await
        .unwrap();

    let mut cursor = collection
        .open_cursor(None, Some(CursorDirection::Next))
        .await
        .unwrap();
    let first = cursor.next().await.unwrap();
    assert!(first.is_some());

    let advanced = cursor.advance(1).await.unwrap();
    assert!(advanced.is_some());
    assert_eq!(advanced.unwrap().name, "Carol");

    let last = cursor.next().await.unwrap();
    assert!(last.is_none());
}

#[wasm_bindgen_test]
async fn test_cursor_empty_store() {
    let db = setup_db().await;
    clear_store(&db).await;

    let collection: Collection<Item> = db.collection("items");
    let mut cursor = collection
        .open_cursor(None, Some(CursorDirection::Next))
        .await
        .unwrap();
    let item = cursor.next().await.unwrap();
    assert!(item.is_none());
}

#[wasm_bindgen_test]
async fn test_cursor_stream() {
    let db = setup_db().await;
    clear_store(&db).await;

    let collection: Collection<Item> = db.collection("items");
    collection
        .put(
            "a",
            &Item {
                id: "a".into(),
                name: "Alice".into(),
            },
        )
        .await
        .unwrap();
    collection
        .put(
            "b",
            &Item {
                id: "b".into(),
                name: "Bob".into(),
            },
        )
        .await
        .unwrap();

    use futures::StreamExt;
    let stream = collection
        .open_cursor(None, Some(CursorDirection::Next))
        .await
        .unwrap()
        .into_stream();
    let items: Vec<Item> = stream.filter_map(|r| async move { r.ok() }).collect().await;

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].name, "Alice");
    assert_eq!(items[1].name, "Bob");
}

#[wasm_bindgen_test]
async fn test_cursor_bound_range() {
    let db = setup_db().await;
    clear_store(&db).await;

    let collection: Collection<Item> = db.collection("items");
    collection
        .put(
            "a",
            &Item {
                id: "a".into(),
                name: "Alice".into(),
            },
        )
        .await
        .unwrap();
    collection
        .put(
            "b",
            &Item {
                id: "b".into(),
                name: "Bob".into(),
            },
        )
        .await
        .unwrap();
    collection
        .put(
            "c",
            &Item {
                id: "c".into(),
                name: "Carol".into(),
            },
        )
        .await
        .unwrap();

    let bound = CursorBound::LowerBound("b".to_string(), false);
    let query = Some(bound.to_query().unwrap());
    let mut cursor = collection
        .open_cursor(query, Some(CursorDirection::Next))
        .await
        .unwrap();
    let mut items = Vec::new();
    while let Some(item) = cursor.next().await.unwrap() {
        items.push(item);
    }

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].name, "Bob");
    assert_eq!(items[1].name, "Carol");
}
