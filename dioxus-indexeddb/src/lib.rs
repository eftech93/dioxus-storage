//! # Dioxus IndexedDB
//!
//! Type-safe IndexedDB bindings for Dioxus with reactive hooks.
//!
//! ## Example
//!
//! ```rust,ignore
//! use dioxus_indexeddb::prelude::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! struct User {
//!     id: String,
//!     name: String,
//! }
//!
//! #[component]
//! fn UserList() -> Element {
//!     let users = use_collection::<User>("users");
//!     
//!     rsx! {
//!         div {
//!             for user in users.read().iter() {
//!                 p { "{user.name}" }
//!             }
//!         }
//!     }
//! }
//! ```

#![cfg(target_arch = "wasm32")]

mod collection;
mod cursor;
mod database;
mod error;
mod hooks;
mod migration;
mod query;
mod transaction;

pub mod schema;

pub use collection::Collection;
pub use cursor::{Cursor, CursorBound};
pub use database::{Database, DatabaseConfig, IndexConfig, StoreConfig};
pub use error::{IndexedDbError, Result};
pub use hooks::{use_collection, use_db, use_query};
pub use migration::{DatabaseBuilder, Migration, MigrationManager, MigrationOp};
pub use query::{
    execute_query, Aggregation, Filter, FilterMode, Order, OrderClause, Pagination, Query,
    QueryResult,
};
pub use transaction::Transaction;

// Re-export idb types for advanced usage
pub use idb::{CursorDirection, KeyRange, Query as IdbQuery};

pub use schema::{Schema, SchemaDatabase, Store, StoreDefinition};

/// Prelude module for convenient imports
pub mod prelude {
    pub use super::schema::migration_set::{MigrationRegistry, MigrationSet, SchemaMigration};
    pub use super::schema::{Schema, SchemaDatabase, Store, StoreDefinition};
    pub use super::{execute_query, Filter, Order, Query, QueryResult};
    pub use super::{use_collection, use_db, use_query};
    pub use super::{Collection, Cursor, CursorBound, Database, DatabaseConfig, IndexedDbError, Result};
    pub use super::{DatabaseBuilder, Migration, MigrationManager, MigrationOp};
    pub use super::{IndexConfig, StoreConfig};
    pub use idb::{CursorDirection, KeyRange};
}

use wasm_bindgen::JsValue;

/// Convert a Rust value to a JS value for storage
pub fn to_js_value<T: serde::Serialize>(value: &T) -> Result<JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    value
        .serialize(&serializer)
        .map_err(|e| IndexedDbError::Serialization(e.to_string()))
}

/// Convert a JS value to a Rust value
pub fn from_js_value<T: serde::de::DeserializeOwned>(js_value: &JsValue) -> Result<T> {
    serde_wasm_bindgen::from_value(js_value.clone())
        .map_err(|e| IndexedDbError::Serialization(e.to_string()))
}
