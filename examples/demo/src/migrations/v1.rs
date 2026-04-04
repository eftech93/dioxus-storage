//! Migration v1 - Initial Schema
//!
//! This is the first version of the database schema.
//! It creates the core `tasks` store.

use dioxus_indexeddb::prelude::*;

/// V1 Migration - Initial schema
///
/// Creates:
/// - tasks store (id as key)
#[derive(Debug, Clone, Copy)]
pub struct V1Migration;

impl MigrationSet for V1Migration {
    fn version() -> u32 {
        1
    }

    fn operations() -> Vec<MigrationOp> {
        vec![
            // Create tasks store with 'id' as key
            MigrationOp::CreateStore {
                name: "tasks".to_string(),
                key_path: "id".to_string(),
                auto_increment: false,
            },
        ]
    }

    fn data_migration() -> Option<fn()> {
        // Optional: Seed initial data
        Some(|| {
            log::info!("V1: Initial schema created");
        })
    }
}

/// Task store metadata (V1)
pub struct TaskStore;

impl TaskStore {
    pub fn name() -> &'static str {
        "tasks"
    }
    pub fn key_path() -> &'static str {
        "id"
    }
}
