# Backend Setup Guide

This guide walks you through setting up the complete backend with MongoDB and the sync demo.

## Architecture

```
┌─────────────────┐      HTTP/JSON      ┌─────────────────┐      ┌─────────────┐
│  Sync Demo      │ ◄─────────────────► │  Rust API       │ ◄───►│   MongoDB   │
│  (Dioxus)       │   Paginated sync    │  (Axum)         │      │  100 items  │
│                 │   Hot/Background    │                 │      └─────────────┘
│  - Hot Sync     │   Search/Filter     │  - /products    │
│  - Visual logs  │                     │  - /sync        │
└─────────────────┘                     └─────────────────┘
```

## Quick Start

### 1. Start MongoDB & API

```bash
cd examples/sync-demo/backend

# Start services
docker-compose up -d

# Verify MongoDB is running
docker-compose ps

# Verify API is accessible
curl http://localhost:3001/api/health
```

### 2. Initialize Database (First Time Only)

The MongoDB container automatically runs `init-mongo.js` on first startup:
- Creates 100 sample products
- 8 categories (Electronics, Clothing, Food, Books, Home, Sports, Toys, Beauty)
- 8 brands with various colors, prices, stock levels

### 3. Run Sync Demo

```bash
cd examples/sync-demo

# Install dioxus CLI if needed:
# cargo install dioxus-cli

# Run the app
dx serve --platform web
```

## Features Demo

### 🔥 Hot Sync Mode
1. Select "Hot Sync" radio button
2. Click "Load Page" - fetches 5 products from backend
3. Navigate through pages (20 pages total)
4. Search filters work on local data first

### 🌙 Background Sync Mode  
1. Select "Background Sync"
2. Sync automatically happens in background
3. Watch sync log for periodic updates

### 🔄 Sync All Pages
1. Click "Sync All (50 items)"
2. Watch progress in sync log (10 pages × 5 items)
3. All data stored locally in IndexedDB

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/health` | Health check |
| `GET /api/products?page=1&per_page=5` | Paginated products |
| `GET /api/products/search?search=tech` | Search products |
| `GET /api/products/categories` | List categories |
| `GET /api/sync` | Pull changes |
| `POST /api/sync` | Bidirectional sync |

## Configuration

### Environment Variables

Create `.env` in `examples/sync-demo/backend/api/`:
```
MONGODB_URI=mongodb://admin:secret@localhost:27017/products_db?authSource=admin
PORT=3001
```

### Docker Compose Services

| Service | Port | Description |
|---------|------|-------------|
| mongodb | 27017 | MongoDB database |
| api | 3001 | Rust Axum API |

## Data Model

### Product
```rust
struct Product {
    id: String,           // "prod_0001"
    name: String,
    description: String,
    price: f64,
    category: String,     // "Electronics", etc.
    brand: String,
    color: String,
    stock: i32,
    rating: f64,
    in_stock: bool,
    created_at: DateTime,
    updated_at: DateTime,
}
```

## Troubleshooting

### MongoDB Connection Failed
```bash
# Check MongoDB logs
docker-compose logs mongodb

# Reset MongoDB (deletes all data!)
docker-compose down -v
docker-compose up -d
```

### API Not Responding
```bash
# Restart API
docker-compose restart api

# Check API logs
docker-compose logs api
```

### Port Already in Use
```bash
# Kill processes on port 3001
lsof -ti:3001 | xargs kill -9
```

## Testing the API

```bash
# Health check
curl http://localhost:3001/api/health

# Get products
curl "http://localhost:3001/api/products?page=1&per_page=5"

# Search
curl "http://localhost:3001/api/products/search?search=tech&page=1"

# Get categories
curl http://localhost:3001/api/products/categories
```

## Project Structure

```
examples/sync-demo/backend/
├── docker-compose.yml      # MongoDB + API services
├── init-mongo.js           # 100 sample products
├── api/
│   ├── Cargo.toml          # Rust dependencies
│   ├── Dockerfile          # Container build
│   └── src/
│       └── main.rs         # Axum server
└── README.md               # Backend docs

examples/sync-demo/
├── src/
│   ├── main.rs             # Main app
│   ├── models.rs           # Product, SyncEvent types
│   ├── sync.rs             # SyncService
│   └── components/         # UI components
├── public/
│   └── style.css           # Styling
└── README.md               # Demo docs
```
