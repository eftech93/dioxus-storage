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

pub use error::{Result, StorageError};
pub use local_storage::{use_local_storage, LocalStorage};
pub use session_storage::{use_session_storage, SessionStorage};
pub use storage::{use_storage, Storage, StorageConfig};

#[cfg(feature = "indexeddb")]
pub use dioxus_indexeddb::{Collection, Database, DatabaseConfig as DbConfig};

/// Prelude module for convenient imports
pub mod prelude {
    pub use super::{use_local_storage, LocalStorage};
    pub use super::{use_session_storage, SessionStorage};
    pub use super::{use_storage, Storage, StorageConfig};
    pub use super::{Result, StorageError};

    #[cfg(feature = "indexeddb")]
    pub use super::{Collection, Database, DbConfig};
}

// Re-export indexeddb types when feature is enabled
#[cfg(feature = "indexeddb")]
pub mod indexeddb {
    pub use dioxus_indexeddb::*;
}
