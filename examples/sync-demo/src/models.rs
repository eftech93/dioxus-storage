use serde::{Deserialize, Serialize};

/// Product model matching the backend schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub category: String,
    pub brand: String,
    pub color: String,
    pub stock: i32,
    pub rating: f64,
    pub in_stock: bool,
    pub created_at: f64,
    pub updated_at: f64,
}

impl Product {
    /// Get the key for IndexedDB storage
    pub fn key(&self) -> String {
        self.id.clone()
    }
}

/// Product sync status for tracking local changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Synced,
    Modified,
    New,
    Deleted,
}

impl Default for SyncStatus {
    fn default() -> Self {
        SyncStatus::New
    }
}

/// Paginated response from backend
#[derive(Debug, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

/// Sync event for logging
#[derive(Debug, Clone, PartialEq)]
pub struct SyncEvent {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub mode: super::sync::SyncMode,
    pub action: String,
    pub items_count: usize,
    pub duration_ms: u64,
    pub success: bool,
    pub message: String,
}
