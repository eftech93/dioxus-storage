use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    options::FindOptions,
    Client, Collection,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

// Product model - use i64 timestamps for MongoDB compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// Paginated response
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

// Query parameters for fetching products
#[derive(Debug, Deserialize)]
pub struct GetProductsQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub search: Option<String>,
    pub in_stock: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

// Task model for offline queue demo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

// App state
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Client>,
    pub products_collection: Collection<Product>,
    pub tasks_collection: Collection<Task>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Get configuration from environment
    let mongodb_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://admin:secret@localhost:27017/products_db?authSource=admin".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()?;

    info!("Connecting to MongoDB...");
    
    // Connect to MongoDB
    let client = Client::with_uri_str(&mongodb_uri).await?;
    let db = client.database("products_db");
    let products_collection = db.collection::<Product>("products");

    info!("Connected to MongoDB successfully");

    let tasks_collection = db.collection::<Task>("tasks");

    info!("Connected to MongoDB successfully");

    let app_state = AppState {
        db: Arc::new(client),
        products_collection,
        tasks_collection,
    };

    // Build router
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/products", get(get_products))
        .route("/api/products/search", get(search_products))
        .route("/api/products/categories", get(get_categories))
        .route("/api/products/brands", get(get_brands))
        .route("/api/tasks/:id", put(upsert_task))
        .route("/api/tasks/:id", get(get_task))
        .route("/api/tasks/:id", axum::routing::delete(delete_task))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("API server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Health check endpoint
async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.list_database_names().await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "status": "healthy" }))),
        Err(e) => {
            warn!("Health check failed: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "status": "unhealthy", "error": e.to_string() })),
            )
        }
    }
}

// Get products with pagination and filtering
async fn get_products(
    State(state): State<AppState>,
    Query(query): Query<GetProductsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(10).clamp(1, 100);
    let skip = ((page - 1) * per_page) as u64;

    // Build filter
    let mut filter = Document::new();
    
    if let Some(category) = &query.category {
        filter.insert("category", category);
    }
    
    if let Some(brand) = &query.brand {
        filter.insert("brand", brand);
    }
    
    if let Some(in_stock) = query.in_stock {
        filter.insert("in_stock", in_stock);
    }
    
    if let (Some(min), Some(max)) = (query.min_price, query.max_price) {
        filter.insert("price", doc! { "$gte": min, "$lte": max });
    } else if let Some(min) = query.min_price {
        filter.insert("price", doc! { "$gte": min });
    } else if let Some(max) = query.max_price {
        filter.insert("price", doc! { "$lte": max });
    }

    // Build sort options
    let sort_by = query.sort_by.unwrap_or_else(|| "created_at".to_string());
    let sort_order = if query.sort_order.as_deref() == Some("asc") { 1 } else { -1 };
    
    let find_options = FindOptions::builder()
        .skip(skip)
        .limit(per_page as i64)
        .sort(doc! { sort_by: sort_order })
        .build();

    // Execute query
    let cursor = state.products_collection.find(filter.clone()).with_options(find_options).await?;
    let products: Vec<Product> = cursor.try_collect().await?;
    
    // Get total count
    let total = state.products_collection.count_documents(filter).await?;
    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    info!(
        "Fetched {} products (page {} of {}, total: {})",
        products.len(),
        page,
        total_pages,
        total
    );

    let response = PaginatedResponse {
        data: products,
        total,
        page,
        per_page,
        total_pages,
    };

    Ok((StatusCode::OK, Json(response)))
}

// Search products by text
async fn search_products(
    State(state): State<AppState>,
    Query(query): Query<GetProductsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let search = query.search.as_deref().unwrap_or("");
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(10).clamp(1, 100);
    let skip = ((page - 1) * per_page) as u64;

    let filter = if search.is_empty() {
        Document::new()
    } else {
        doc! {
            "$or": [
                { "name": { "$regex": search, "$options": "i" } },
                { "description": { "$regex": search, "$options": "i" } },
                { "category": { "$regex": search, "$options": "i" } },
                { "brand": { "$regex": search, "$options": "i" } },
            ]
        }
    };

    let find_options = FindOptions::builder()
        .skip(skip)
        .limit(per_page as i64)
        .sort(doc! { "name": 1 })
        .build();

    let cursor = state.products_collection.find(filter.clone()).with_options(find_options).await?;
    let products: Vec<Product> = cursor.try_collect().await?;
    let total = state.products_collection.count_documents(filter).await?;
    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    info!("Search '{}' returned {} products", search, products.len());

    let response = PaginatedResponse {
        data: products,
        total,
        page,
        per_page,
        total_pages,
    };

    Ok((StatusCode::OK, Json(response)))
}

// Get all unique categories
async fn get_categories(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let categories: Vec<String> = state
        .products_collection
        .distinct("category", Document::new())
        .await?
        .into_iter()
        .filter_map(|b| b.as_str().map(|s| s.to_string()))
        .collect();

    Ok((StatusCode::OK, Json(categories)))
}

// Get all unique brands
async fn get_brands(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let brands: Vec<String> = state
        .products_collection
        .distinct("brand", Document::new())
        .await?
        .into_iter()
        .filter_map(|b| b.as_str().map(|s| s.to_string()))
        .collect();

    Ok((StatusCode::OK, Json(brands)))
}

// Task endpoints for offline queue demo
async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    match state.tasks_collection.find_one(doc! { "id": &id }).await? {
        Some(task) => Ok((StatusCode::OK, Json(serde_json::to_value(task).unwrap()))),
        None => Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "not found" })))),
    }
}

async fn upsert_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(task): Json<Task>,
) -> Result<impl IntoResponse, AppError> {
    let _ = state
        .tasks_collection
        .replace_one(doc! { "id": &id }, &task)
        .upsert(true)
        .await?;
    info!("Upserted task {}", id);
    Ok((StatusCode::OK, Json(task)))
}

async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let _ = state.tasks_collection.delete_one(doc! { "id": &id }).await?;
    info!("Deleted task {}", id);
    Ok((StatusCode::OK, Json(serde_json::json!({ "deleted": true }))))
}

// Error handling
#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self.0.downcast_ref::<mongodb::error::Error>() {
            Some(_) => (StatusCode::SERVICE_UNAVAILABLE, "Database error".to_string()),
            None => (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
        };

        let body = Json(serde_json::json!({
            "error": true,
            "message": message,
        }));

        (status, body).into_response()
    }
}
