# Dioxus Storage Backend

A complete backend example demonstrating:
- MongoDB database with 100 sample products
- REST API with pagination and search
- Sync endpoints for hot sync and background sync
- Docker Compose for easy setup

## Architecture

```
┌─────────────────┐     HTTP/JSON      ┌─────────────────┐     ┌─────────────┐
│  Dioxus App     │ ◄────────────────► │  Rust API       │ ◄──►│   MongoDB   │
│  (sync-demo)    │   Pagination       │  (Axum)         │     │  (products) │
│                 │   Search           │                 │     │  100 items  │
│  - Hot Sync     │   Sync             │  - /products    │     └─────────────┘
│  - Background   │                    │  - /sync        │
│  - Local cache  │                    │  - /search      │
└─────────────────┘                    └─────────────────┘
```

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Or: MongoDB installed locally + Rust toolchain

### Using Docker Compose

```bash
# Start MongoDB and API server
cd dioxus-storage/backend
docker-compose up -d

# Seed the database with 100 products
docker-compose exec mongodb mongosh -u admin -p secret --authenticationDatabase admin products_db /docker-entrypoint-initdb.d/init-mongo.js

# Or wait for auto-initialization (MongoDB runs the script on first start)
```

### Manual Setup

```bash
# 1. Start MongoDB locally
mongod --dbpath /path/to/data

# 2. Run initialization script
mongosh -u admin -p secret --authenticationDatabase admin products_db init-mongo.js

# 3. Run the API server
cd api
cargo run
```

## API Endpoints

### Health Check
```
GET /api/health
```

### Get Products (Paginated)
```
GET /api/products?page=1&per_page=5&category=Electronics&search=phone

Response:
{
  "data": [...],
  "total": 100,
  "page": 1,
  "per_page": 5,
  "total_pages": 20
}
```

### Search Products
```
GET /api/products/search?search=tech&page=1&per_page=5
```

### Get Categories
```
GET /api/products/categories
# Returns: ["Electronics", "Clothing", "Food", ...]
```

### Get Brands
```
GET /api/products/brands
```

### Sync (Pull)
```
GET /api/sync?last_sync_at=2024-01-01T00:00:00Z
```

### Sync (Bidirectional)
```
POST /api/sync
{
  "last_sync_at": "2024-01-01T00:00:00Z",
  "client_changes": [...]
}
```

## Data Model

### Product
```rust
struct Product {
    id: String,           // "prod_0001"
    name: String,         // "TechCorp Electronics Item 1"
    description: String,
    price: f64,
    category: String,     // "Electronics", "Clothing", etc.
    brand: String,
    color: String,
    stock: i32,
    rating: f64,
    in_stock: bool,
    created_at: DateTime,
    updated_at: DateTime,
}
```

## Sample Data

The database is initialized with 100 products:
- 8 categories: Electronics, Clothing, Food, Books, Home, Sports, Toys, Beauty
- 8 brands: TechCorp, StyleInc, FreshFoods, BookWorld, HomePlus, SportMax, ToyJoy, BeautyBar
- Various colors, prices ($10-$510), stock levels, and ratings

## Configuration

Environment variables:
- `MONGODB_URI` - MongoDB connection string
- `PORT` - API server port (default: 3001)

## Testing

```bash
# Health check
curl http://localhost:3001/api/health

# Get first page
curl http://localhost:3001/api/products?page=1&per_page=5

# Search
curl "http://localhost:3001/api/products/search?search=tech&page=1"

# Get categories
curl http://localhost:3001/api/products/categories
```
