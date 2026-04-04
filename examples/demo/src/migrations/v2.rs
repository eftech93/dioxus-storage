//! Migration v2 - Add Settings
//!
//! This migration adds a settings store for user preferences.
//!
//! Changes from v1:
//! - Added: settings store

use dioxus_indexeddb::prelude::*;

/// V2 Migration - Add settings store
///
/// Creates:
/// - settings store (key as key)
#[derive(Debug, Clone, Copy)]
pub struct V2Migration;

impl MigrationSet for V2Migration {
    fn version() -> u32 {
        2
    }

    fn operations() -> Vec<MigrationOp> {
        vec![
            // Create settings store for user preferences
            MigrationOp::CreateStore {
                name: "settings".to_string(),
                key_path: "key".to_string(),
                auto_increment: false,
            },
        ]
    }

    fn data_migration() -> Option<fn()> {
        Some(|| {
            log::info!("V2: Added settings store");
            // Could seed default settings here
        })
    }
}

/// Settings store metadata (V2+)
pub struct SettingsStore;

impl SettingsStore {
    pub fn name() -> &'static str {
        "settings"
    }
    pub fn key_path() -> &'static str {
        "key"
    }
}
