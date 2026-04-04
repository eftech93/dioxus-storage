//! Dioxus hooks for IndexedDB

use crate::collection::Collection;
use crate::database::{Database, DatabaseConfig};
use crate::error::Result;
use dioxus::hooks::*;
use dioxus::prelude::spawn;
use dioxus_signals::*;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;

/// Hook to get a database connection
///
/// Example:
/// ```rust,ignore
/// let db = use_db(DatabaseConfig::new("my_app", 1)
///     .with_store("users", "id")
///     .with_store("products", "id"));
/// ```
pub fn use_db(config: DatabaseConfig) -> Signal<Option<Database>> {
    let mut db_signal = use_signal(|| None::<Database>);

    use_effect(move || {
        let config = config.clone();
        spawn(async move {
            match Database::open(config).await {
                Ok(db) => {
                    db_signal.set(Some(db));
                }
                Err(e) => {
                    log::error!("Failed to open database: {}", e);
                }
            }
        });
    });

    db_signal
}

/// Hook to get a typed collection
///
/// Example:
/// ```rust,ignore
/// let users = use_collection::<User>(db, "users");
/// ```
pub fn use_collection<T: Serialize + DeserializeOwned + 'static>(
    db: Signal<Option<Database>>,
    name: &str,
) -> Signal<Option<Collection<T>>> {
    let name = name.to_string();
    let mut collection_signal = use_signal(|| None::<Collection<T>>);
    
    use_effect(move || {
        if let Some(ref db) = *db.read() {
            collection_signal.set(Some(db.collection::<T>(&name)));
        }
    });
    
    collection_signal
}

/// Hook to query a collection with automatic updates
///
/// Returns a signal that automatically re-fetches when dependencies change
///
/// Example:
/// ```rust,ignore
/// let users = use_query(collection_signal, |c| async move {
///     c.get_all().await
/// });
/// ```
pub fn use_query<T, F, Fut>(
    collection: Signal<Option<Collection<T>>>,
    query_fn: F,
) -> Signal<Result<Vec<T>>>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
    F: Fn(Collection<T>) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<Vec<T>>> + 'static,
{
    let mut result = use_signal(|| Ok(Vec::new()));
    
    use_effect(move || {
        if let Some(ref collection) = *collection.read() {
            let query_fn = query_fn.clone();
            let collection = collection.clone();
            spawn(async move {
                match query_fn(collection).await {
                    Ok(data) => result.set(Ok(data)),
                    Err(e) => result.set(Err(e)),
                }
            });
        }
    });

    result
}

/// Hook for a mutable collection that can be read and modified
///
/// This provides a higher-level API with automatic re-rendering
///
/// Example:
/// ```rust,ignore
/// let (users, users_writer) = use_collection_state::<User>(&db, "users");
///
/// // Read
/// for user in users.read().iter() {
///     // render user
/// }
///
/// // Write
/// users_writer.insert("user1", &new_user).await;
/// ```
pub fn use_collection_state<T>(
    db: &Database,
    name: &str,
) -> (Signal<Vec<T>>, CollectionWriter<T>)
where
    T: Serialize + DeserializeOwned + Clone + 'static,
{
    let collection: Collection<T> = db.collection(name);
    let mut items = use_signal(Vec::new);
    
    // Initial load
    {
        let collection = collection.clone();
        let mut items = items.clone();
        use_effect(move || {
            let collection = collection.clone();
            spawn(async move {
                match collection.get_all().await {
                    Ok(data) => items.set(data),
                    Err(e) => log::error!("Failed to load collection: {}", e),
                }
            });
        });
    }

    let writer = CollectionWriter { collection };
    (items, writer)
}

/// Writer for collection mutations
pub struct CollectionWriter<T: Serialize + DeserializeOwned> {
    collection: Collection<T>,
}

impl<T: Serialize + DeserializeOwned + Clone> CollectionWriter<T> {
    /// Insert a new item
    pub async fn insert(&self, key: &str, item: &T) -> crate::error::Result<()> {
        self.collection.insert(key, item).await
    }

    /// Update or insert an item
    pub async fn put(&self, key: &str, item: &T) -> crate::error::Result<()> {
        self.collection.put(key, item).await
    }

    /// Delete an item
    pub async fn delete(&self, key: &str) -> crate::error::Result<()> {
        self.collection.delete(key).await
    }

    /// Get the underlying collection
    pub fn collection(&self) -> &Collection<T> {
        &self.collection
    }
}

impl<T: Serialize + DeserializeOwned> Clone for CollectionWriter<T> {
    fn clone(&self) -> Self {
        Self {
            collection: self.collection.clone(),
        }
    }
}

/// Hook to check if IndexedDB is available
pub fn use_indexeddb_available() -> bool {
    Database::is_available()
}
