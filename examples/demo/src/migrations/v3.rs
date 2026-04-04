//! Migration v3 - Archive and Cleanup
//!
//! This migration adds archived_tasks support and cleans up old stores.
//!
//! Changes from v2:
//! - Added: archived_tasks store
//! - Removed: old_temp store (if exists)

use dioxus_indexeddb::prelude::*;

/// V3 Migration - Archive functionality
///
/// Creates:
/// - archived_tasks store
/// Removes:
/// - old_temp store (legacy)
#[derive(Debug, Clone, Copy)]
pub struct V3Migration;

impl MigrationSet for V3Migration {
    fn version() -> u32 {
        3
    }

    fn operations() -> Vec<MigrationOp> {
        vec![
            // Create archived_tasks store for soft-deleted tasks
            MigrationOp::CreateStore {
                name: "archived_tasks".to_string(),
                key_path: "id".to_string(),
                auto_increment: false,
            },
            // Remove legacy temp store if it exists
            MigrationOp::DeleteStore {
                name: "old_temp".to_string(),
            },
        ]
    }

    fn data_migration() -> Option<fn()> {
        Some(|| {
            log::info!("V3: Setting up archive functionality");
            
            // Example: Migrate existing tasks with "deleted" flag to archived_tasks
            // This is where you'd put data transformation logic
            
            log::info!("V3: Data migration complete");
        })
    }
}

/// Archived tasks store metadata (V3+)
pub struct ArchivedTaskStore;

impl ArchivedTaskStore {
    pub fn name() -> &'static str { "archived_tasks" }
    pub fn key_path() -> &'static str { "id" }
}

/// Helper function to archive a task (move from tasks to archived_tasks)
pub async fn archive_task(
    _db: &Database,
    task_id: &str,
) -> std::result::Result<(), dioxus_indexeddb::IndexedDbError> {
    // In a real implementation, you would:
    // 1. Get the task from tasks store
    // 2. Add archived_at timestamp
    // 3. Save to archived_tasks store
    // 4. Delete from tasks store
    
    log::info!("Archiving task: {}", task_id);
    Ok(())
}
