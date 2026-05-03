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
pub fn use_collection<T: Serialize + DeserializeOwned + Clone + 'static>(
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
