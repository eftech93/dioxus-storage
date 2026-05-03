//! HTTP client for backend communication

use crate::{config::SyncConfig, Result, SyncError};
use reqwest::{Method, RequestBuilder};
use serde::{de::DeserializeOwned, Serialize};

/// HTTP client for sync operations
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: reqwest::Client,
    config: SyncConfig,
}

impl HttpClient {
    /// Create a new HTTP client
    pub fn new(config: SyncConfig) -> Self {
        let client = reqwest::Client::new();
        Self { client, config }
    }

    /// Build a request with config headers
    fn build_request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = self.config.endpoint(path);
        let mut req = self.client.request(method, &url);

        for (key, value) in self.config.build_headers() {
            req = req.header(&key, value);
        }

        req
    }

    /// GET request
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let response = self.build_request(Method::GET, path).send().await?;

        if !response.status().is_success() {
            return Err(handle_error_status(response.status()));
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// GET with query parameters
    pub async fn get_with_params<T: DeserializeOwned, P: Serialize>(
        &self,
        path: &str,
        params: &P,
    ) -> Result<T> {
        let response = self
            .build_request(Method::GET, path)
            .query(params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(handle_error_status(response.status()));
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// POST request
    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let response = self
            .build_request(Method::POST, path)
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(handle_error_status(response.status()));
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// PUT request
    pub async fn put<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let response = self
            .build_request(Method::PUT, path)
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(handle_error_status(response.status()));
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// DELETE request
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let response = self.build_request(Method::DELETE, path).send().await?;

        if !response.status().is_success() {
            return Err(handle_error_status(response.status()));
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// Fetch with retry logic
    pub async fn fetch_with_retry<T: DeserializeOwned>(
        &self,
        path: &str,
        params: Option<&serde_json::Value>,
    ) -> Result<T> {
        let mut last_error = None;

        for attempt in 0..self.config.retry_attempts {
            let result = if let Some(p) = params {
                self.get_with_params(path, p).await
            } else {
                self.get(path).await
            };

            match result {
                Ok(data) => return Ok(data),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.retry_attempts - 1 {
                        // Exponential backoff
                        let delay = std::time::Duration::from_millis(100 * 2_u64.pow(attempt));
                        gloo_timers::future::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| SyncError::Unknown("All retry attempts failed".to_string())))
    }
}

/// Handle HTTP error status codes
fn handle_error_status(status: reqwest::StatusCode) -> SyncError {
    match status.as_u16() {
        401 => SyncError::Unauthorized,
        429 => SyncError::RateLimited,
        502..=504 => SyncError::BackendUnavailable,
        _ => SyncError::Http(format!("HTTP {}", status)),
    }
}

/// Trait for sync clients
#[async_trait::async_trait(?Send)]
pub trait SyncClient {
    /// Fetch items from backend
    async fn fetch_items<T: DeserializeOwned>(
        &self,
        store_name: &str,
        since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<T>>;

    /// Push items to backend
    async fn push_items<T: Serialize>(&self, store_name: &str, items: Vec<T>)
        -> Result<PushResult>;

    /// Delete item on backend
    async fn delete_item(&self, store_name: &str, id: &str) -> Result<()>;

    /// Check backend health
    async fn health_check(&self) -> Result<bool>;
}

/// Result of a push operation
#[derive(Debug, Clone)]
pub struct PushResult {
    pub synced: usize,
    pub failed: usize,
    pub conflicts: Vec<ConflictInfo>,
}

/// Conflict information
#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub id: String,
    pub local_version: serde_json::Value,
    pub server_version: serde_json::Value,
}
