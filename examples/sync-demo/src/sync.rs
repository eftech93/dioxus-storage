use dioxus_indexeddb::{Collection, Database, DatabaseConfig};
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::models::{PaginatedResponse, Product};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncMode {
    Hot,        // Check local cache first, return immediately
    Background, // Always fetch from backend
}

pub struct SyncService {
    api_url: String,
}

impl Clone for SyncService {
    fn clone(&self) -> Self {
        Self {
            api_url: self.api_url.clone(),
        }
    }
}

impl SyncService {
    pub fn new(api_url: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
        }
    }

    /// Initialize the IndexedDB database for products
    pub async fn init_database() -> std::result::Result<Database, String> {
        let config = DatabaseConfig::new("sync_demo", 2)
            .with_store("products", "id")
            .with_store("sync_meta", "key")
            .with_store("query_cache", "query_key");

        Database::open(config)
            .await
            .map_err(|e| format!("Failed to open IndexedDB: {:?}", e))
    }

    /// Generate a unique key for a query
    fn query_key(page: u32, per_page: u32, search: &str, category: &Option<String>) -> String {
        let mut hasher = DefaultHasher::new();
        page.hash(&mut hasher);
        per_page.hash(&mut hasher);
        search.hash(&mut hasher);
        category.hash(&mut hasher);
        format!("query_{:x}", hasher.finish())
    }

    /// Hot sync: Check local cache first, only fetch if not cached or hard_sync is true
    ///
    /// # Arguments
    /// * `db` - The IndexedDB database
    /// * `page` - Page number
    /// * `per_page` - Items per page  
    /// * `search` - Search query string
    /// * `category` - Category filter
    /// * `hard_sync` - If true, force fetch from backend even if cached
    ///
    /// # Returns
    /// (products, total_count, source) where source is "cache" or "backend"
    pub async fn hot_sync_products(
        &self,
        db: &Database,
        page: u32,
        per_page: u32,
        search: String,
        category: Option<String>,
        hard_sync: bool,
    ) -> std::result::Result<(Vec<Product>, u64, &'static str), String> {
        let query_key = Self::query_key(page, per_page, &search, &category);
        let cache_collection: Collection<QueryCacheEntry> = db.collection("query_cache");

        // Check if we have a cached result for this exact query
        if !hard_sync {
            if let Ok(Some(cached)) = cache_collection.get(&query_key).await {
                // Check if cache is still valid (e.g., less than 5 minutes old)
                let now = js_sys::Date::now();
                let cache_age_ms = now - cached.timestamp;
                const MAX_CACHE_AGE_MS: f64 = 5.0 * 60.0 * 1000.0; // 5 minutes

                if cache_age_ms < MAX_CACHE_AGE_MS {
                    log::info!("Cache HIT for query {}", query_key);

                    // Background refresh if cache is older than 1 minute
                    if cache_age_ms > 60.0 * 1000.0 {
                        let service = self.clone();
                        let db = db.clone();
                        let search_bg = search.clone();
                        let category_bg = category.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            let _ = service
                                .fetch_and_cache(&db, page, per_page, search_bg, category_bg)
                                .await;
                        });
                    }

                    return Ok((cached.products, cached.total, "cache"));
                } else {
                    log::info!("Cache EXPIRED for query {}", query_key);
                }
            } else {
                log::info!("Cache MISS for query {}", query_key);
            }
        } else {
            log::info!("Hard sync requested for query {}", query_key);
        }

        // Fetch from backend and cache the result
        self.fetch_and_cache(db, page, per_page, search, category)
            .await
    }

    /// Fetch from backend and store in query cache
    async fn fetch_and_cache(
        &self,
        db: &Database,
        page: u32,
        per_page: u32,
        search: String,
        category: Option<String>,
    ) -> std::result::Result<(Vec<Product>, u64, &'static str), String> {
        let query_key = Self::query_key(page, per_page, &search, &category);

        // Fetch from backend
        let (products, total) = self
            .fetch_from_backend(page, per_page, search, category)
            .await?;

        // Cache the query result
        let cache_collection: Collection<QueryCacheEntry> = db.collection("query_cache");
        let cache_entry = QueryCacheEntry {
            query_key: query_key.clone(),
            page,
            per_page,
            products: products.clone(),
            total,
            timestamp: js_sys::Date::now(),
        };

        cache_collection
            .put(&query_key, &cache_entry)
            .await
            .map_err(|e| format!("Failed to cache query: {:?}", e))?;

        // Also store individual products for offline access
        self.store_products_in_db(db, &products).await?;

        log::info!(
            "Cached query {} with {} products",
            query_key,
            products.len()
        );
        Ok((products, total, "backend"))
    }

    /// Fetch products from backend API
    pub async fn fetch_from_backend(
        &self,
        page: u32,
        per_page: u32,
        search: String,
        category: Option<String>,
    ) -> std::result::Result<(Vec<Product>, u64), String> {
        let mut url = format!(
            "{}/products?page={}&per_page={}",
            self.api_url, page, per_page
        );

        if !search.is_empty() {
            url.push_str(&format!("&search={}", urlencoding::encode(&search)));
        }

        if let Some(cat) = category {
            url.push_str(&format!("&category={}", urlencoding::encode(&cat)));
        }

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {:?}", e))?;

        if !response.ok() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let paginated: PaginatedResponse<Product> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {:?}", e))?;

        Ok((paginated.data, paginated.total))
    }

    /// Store products in IndexedDB (for offline access)
    pub async fn store_products_in_db(
        &self,
        db: &Database,
        products: &[Product],
    ) -> std::result::Result<(), String> {
        let products_collection: Collection<Product> = db.collection("products");

        for product in products {
            products_collection
                .put(&product.id, product)
                .await
                .map_err(|e| format!("Failed to store product {}: {:?}", product.id, e))?;
        }

        // Update sync metadata
        let meta_collection: Collection<SyncMeta> = db.collection("sync_meta");
        let meta = SyncMeta {
            key: "last_sync".to_string(),
            timestamp: js_sys::Date::now(),
            count: products.len() as u64,
        };
        meta_collection
            .put("last_sync", &meta)
            .await
            .map_err(|e| format!("Failed to store sync meta: {:?}", e))?;

        log::info!("Stored {} products in IndexedDB", products.len());
        Ok(())
    }

    /// Get all locally stored products
    pub async fn get_local_products(db: &Database) -> std::result::Result<Vec<Product>, String> {
        let products_collection: Collection<Product> = db.collection("products");
        products_collection
            .get_all()
            .await
            .map_err(|e| format!("Failed to read from IndexedDB: {:?}", e))
    }

    /// Get sync metadata
    pub async fn get_sync_meta(db: &Database) -> std::result::Result<Option<SyncMeta>, String> {
        let meta_collection: Collection<SyncMeta> = db.collection("sync_meta");
        meta_collection
            .get("last_sync")
            .await
            .map_err(|e| format!("Failed to read sync meta: {:?}", e))
    }

    /// Clear all query caches (but keep products for offline access)
    pub async fn clear_query_cache(db: &Database) -> std::result::Result<(), String> {
        let cache_collection: Collection<QueryCacheEntry> = db.collection("query_cache");
        cache_collection
            .clear()
            .await
            .map_err(|e| format!("Failed to clear query cache: {:?}", e))?;

        log::info!("Cleared query cache");
        Ok(())
    }

    /// Clear local storage
    pub async fn clear_local_storage(db: &Database) -> std::result::Result<(), String> {
        let products_collection: Collection<Product> = db.collection("products");
        products_collection
            .clear()
            .await
            .map_err(|e| format!("Failed to clear IndexedDB: {:?}", e))?;

        let meta_collection: Collection<SyncMeta> = db.collection("sync_meta");
        meta_collection
            .clear()
            .await
            .map_err(|e| format!("Failed to clear sync meta: {:?}", e))?;

        let cache_collection: Collection<QueryCacheEntry> = db.collection("query_cache");
        cache_collection
            .clear()
            .await
            .map_err(|e| format!("Failed to clear query cache: {:?}", e))?;

        log::info!("Cleared local storage");
        Ok(())
    }

    /// Background sync: Pull all changes from server
    pub async fn background_sync(&self, db: &Database) -> std::result::Result<usize, String> {
        let mut all_products = Vec::new();

        for page in 1..=20 {
            let url = format!("{}/products?page={}&per_page=5", self.api_url, page);

            let response = Request::get(&url)
                .send()
                .await
                .map_err(|e| format!("Request failed: {:?}", e))?;

            if !response.ok() {
                return Err(format!("HTTP error: {}", response.status()));
            }

            let paginated: PaginatedResponse<Product> = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {:?}", e))?;

            if paginated.data.is_empty() {
                break;
            }

            all_products.extend(paginated.data);
        }

        let count = all_products.len();
        self.store_products_in_db(db, &all_products).await?;

        // Also cache individual page queries for the first few pages
        let cache_collection: Collection<QueryCacheEntry> = db.collection("query_cache");
        for page in 1..=5 {
            let start = ((page - 1) * 5) as usize;
            let end = (start + 5).min(all_products.len());
            if start >= all_products.len() {
                break;
            }
            let page_products = all_products[start..end].to_vec();
            let query_key = Self::query_key(page, 5, "", &None);
            let cache_entry = QueryCacheEntry {
                query_key: query_key.clone(),
                page,
                per_page: 5,
                products: page_products,
                total: all_products.len() as u64,
                timestamp: js_sys::Date::now(),
            };
            let _ = cache_collection.put(&query_key, &cache_entry).await;
        }

        Ok(count)
    }

    /// Get available categories from backend
    pub async fn get_categories(&self) -> std::result::Result<Vec<String>, String> {
        let url = format!("{}/products/categories", self.api_url);

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {:?}", e))?;

        if !response.ok() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {:?}", e))
    }
}

/// Sync metadata stored in IndexedDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMeta {
    pub key: String,
    pub timestamp: f64,
    pub count: u64,
}

/// Query cache entry - stores the exact result of a query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCacheEntry {
    pub query_key: String,
    pub page: u32,
    pub per_page: u32,
    pub products: Vec<Product>,
    pub total: u64,
    pub timestamp: f64,
}
