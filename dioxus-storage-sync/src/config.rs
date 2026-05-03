//! Sync configuration

use std::time::Duration;

/// Sync mode configuration
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Backend API base URL
    pub api_url: String,
    /// Resource path segment for API endpoints (e.g. "products", "tasks")
    pub resource_path: String,
    /// Enable hot sync (on-demand fetching)
    pub hot_sync: bool,
    /// Enable background sync with interval
    pub background_sync: Option<Duration>,
    /// Authentication token
    pub auth_token: Option<String>,
    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,
    /// Batch size for syncing
    pub batch_size: usize,
    /// Retry attempts
    pub retry_attempts: u32,
    /// Headers to include in requests
    pub headers: Vec<(String, String)>,
    /// Sync mode (bidirectional, pull-only, push-only)
    pub mode: SyncMode,
}

/// Sync mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    /// Pull from backend and push local changes
    Bidirectional,
    /// Only pull from backend
    PullOnly,
    /// Only push local changes
    PushOnly,
}

impl Default for SyncMode {
    fn default() -> Self {
        SyncMode::Bidirectional
    }
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Prefer server version
    ServerWins,
    /// Prefer local version
    LocalWins,
    /// Use timestamp (last write wins)
    LastWriteWins,
    /// Custom resolution (manual merge required)
    Manual,
}

impl Default for ConflictResolution {
    fn default() -> Self {
        ConflictResolution::LastWriteWins
    }
}

impl SyncConfig {
    /// Create a new sync config with API URL
    pub fn new(api_url: impl Into<String>) -> Self {
        Self {
            api_url: api_url.into(),
            resource_path: "items".to_string(),
            hot_sync: false,
            background_sync: None,
            auth_token: None,
            conflict_resolution: ConflictResolution::default(),
            batch_size: 100,
            retry_attempts: 3,
            headers: Vec::new(),
            mode: SyncMode::default(),
        }
    }

    /// Set the resource path for API endpoints
    pub fn with_resource_path(mut self, path: impl Into<String>) -> Self {
        self.resource_path = path.into();
        self
    }

    /// Enable hot sync
    pub fn with_hot_sync(mut self, enabled: bool) -> Self {
        self.hot_sync = enabled;
        self
    }

    /// Enable background sync with interval
    pub fn with_background_sync(mut self, interval: Duration) -> Self {
        self.background_sync = Some(interval);
        self
    }

    /// Set authentication token
    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Set conflict resolution strategy
    pub fn with_conflict_resolution(mut self, strategy: ConflictResolution) -> Self {
        self.conflict_resolution = strategy;
        self
    }

    /// Set batch size
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Set retry attempts
    pub fn with_retry_attempts(mut self, attempts: u32) -> Self {
        self.retry_attempts = attempts;
        self
    }

    /// Add custom header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Set sync mode
    pub fn with_mode(mut self, mode: SyncMode) -> Self {
        self.mode = mode;
        self
    }

    /// Get full endpoint URL
    pub fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.api_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    /// Build headers for requests
    pub fn build_headers(&self) -> Vec<(String, String)> {
        let mut headers = self.headers.clone();

        if let Some(token) = &self.auth_token {
            headers.push(("Authorization".to_string(), format!("Bearer {}", token)));
        }

        headers.push(("Content-Type".to_string(), "application/json".to_string()));

        headers
    }

    /// Check if hot sync is enabled
    pub fn is_hot_sync_enabled(&self) -> bool {
        self.hot_sync
    }

    /// Check if background sync is enabled
    pub fn is_background_sync_enabled(&self) -> bool {
        self.background_sync.is_some()
    }
}
