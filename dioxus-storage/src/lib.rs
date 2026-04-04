//! # Dioxus Storage
//!
//! Unified storage API for Dioxus applications.
//!
//! Supports:
//! - **IndexedDB** - Large structured data, async, multiple stores
//! - **LocalStorage** - Simple key-value, synchronous, persistent
//! - **SessionStorage** - Key-value, per-session
//!
//! ## Example
//!
//! ```rust,ignore
//! use dioxus_storage::prelude::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! struct AppState {
//!     theme: String,
//!     user_id: Option<String>,
//! }
//!
//! #[component]
//! fn App() -> Element {
//!     // Use local storage for simple settings
//!     let theme = use_local_storage::<String>("theme", "light".to_string());
//!     
//!     // Use IndexedDB for structured data
//!     let db = use_storage_db(
//!         DatabaseConfig::new("my_app", 1)
//!             .with_store("items", "id")
//!     );
//!     
//!     rsx! {
//!         button {
//!             onclick: move |_| {
//!                 theme.set("dark".to_string());
//!             },
//!             "Switch to Dark Mode"
//!         }
//!     }
//! }
//! ```

#![cfg(target_arch = "wasm32")]

mod error;
mod local_storage;
mod session_storage;
mod storage;

pub use error::{StorageError, Result};
pub use local_storage::{LocalStorage, use_local_storage};
pub use session_storage::{SessionStorage, use_session_storage};
pub use storage::{Storage, use_storage, StorageConfig};

#[cfg(feature = "indexeddb")]
pub use dioxus_indexeddb::{Database, DatabaseConfig as DbConfig, Collection};

/// Prelude module for convenient imports
pub mod prelude {
    pub use super::{Storage, StorageConfig, use_storage};
    pub use super::{LocalStorage, use_local_storage};
    pub use super::{SessionStorage, use_session_storage};
    pub use super::{StorageError, Result};
    
    #[cfg(feature = "indexeddb")]
    pub use super::{Database, DbConfig, Collection};
}

// Re-export indexeddb types when feature is enabled
#[cfg(feature = "indexeddb")]
pub mod indexeddb {
    pub use dioxus_indexeddb::*;
}
