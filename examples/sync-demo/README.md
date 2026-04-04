# Sync Demo

A complete example demonstrating **Dioxus Storage Sync** with a real backend.

## Features

### 🔥 Hot Sync
- Checks local IndexedDB first
- Fetches from backend only when needed
- Immediate UI feedback

### 🌙 Background Sync  
- Periodically syncs all data (30s interval)
- Keeps local cache up to date
- Visual sync logging

### 📄 Paginated Sync
- Syncs 10 pages × 5 items = 50 products
- Shows progress for each page
- Stores all data locally in IndexedDB

### 🔍 Search & Filter
- Search by name, description, brand, category
- Category dropdown filter
- Works with both local and remote data

### 📋 Visual Sync Logging
- Real-time sync event log
- Shows timestamp, mode, duration
- Success/error indicators

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Sync Demo App                               │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  Hot Sync    │  │ Background   │  │   Sync Log Panel     │  │
│  │  (on-demand) │  │ Sync (30s)   │  │   (visual feedback)  │  │
│  └──────┬───────┘  └──────┬───────┘  └──────────────────────┘  │
│         │                 │                                      │
│  ┌──────▼─────────────────▼──────┐                               │
│  │      SyncService               │                               │
│  │  - IndexedDB (local cache)    │                               │
│  │  - HTTP Client (backend)      │                               │
│  └──────────────┬────────────────┘                               │
│                 │                                                │
│                 ▼                                                │
│  ┌─────────────────────────────────────┐                        │
│  │        Products Grid                 │                        │
│  │   (5 per page, 10 pages total)       │                        │
│  └─────────────────────────────────────┘                        │
└─────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼ HTTP/JSON
┌─────────────────────────────────────────────────────────────────┐
│                    Rust API (Axum)                               │
│  - GET /api/products (paginated)                                │
│  - GET /api/products/search                                     │
│  - GET /api/sync                                                │
└─────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                    MongoDB                                       │
│         100 sample products (8 categories)                      │
└─────────────────────────────────────────────────────────────────┘
```

## Quick Start

### 1. Start the Backend

```bash
cd backend
docker-compose up -d
```

### 2. Run the Demo

```bash
# In this directory
dx serve --platform web

# Or
cargo run --features web
```

### 3. Open Browser

Navigate to `http://localhost:8080` (or the URL shown by dx)

## How to Use

### Hot Sync Mode (Default)
1. Select "🔥 Hot Sync" mode
2. Click "📥 Load Current Page" to fetch page 1 (5 items)
3. Navigate through pages with pagination
4. Search filters local data first, then fetches if needed

### Background Sync Mode
1. Select "🌙 Background Sync" mode  
2. Click "▶️ Start Background Sync"
3. Watch the sync log - updates every 30 seconds
4. All 100 products are periodically synced to IndexedDB

### Sync All Pages
1. Click "🔄 Sync All Pages (50 items)"
2. Watch the progress in the sync log
3. Each of the 10 pages is fetched and stored
4. Total time and item count shown when complete

## Sync Event Log

The right panel shows all sync operations:
- **Timestamp** - When the sync occurred
- **Mode** - 🔥 Hot or 🌙 Background
- **Action** - What was performed
- **Items** - How many items affected
- **Duration** - How long it took (ms)
- **Message** - Additional details

Green = Success, Red = Error

## Data Flow Examples

### Hot Sync (Search)
```
User types "tech" in search
         │
         ▼
Check IndexedDB for "tech" products
         │
         ├── Found? → Display immediately
         │
         └── Not found? 
              │
              ▼
         GET /api/products?search=tech
              │
              ▼
         Store in IndexedDB
              │
              ▼
         Display results
```

### Background Sync
```
Every 30 seconds:
         │
         ▼
for page in 1..=10:
    GET /api/products?page=N&per_page=5
         │
         ▼
Store all 50 products in IndexedDB
         │
         ▼
Log: "Background Sync: 50 items"
```

## File Structure

```
sync-demo/
├── src/
│   ├── main.rs              # Main app component
│   ├── models.rs            # Product, SyncEvent types
│   ├── sync.rs              # SyncService (Hot/Background)
│   └── components/
│       ├── mod.rs
│       ├── product_card.rs  # Product display
│       ├── sync_log.rs      # Sync event viewer
│       ├── filter_panel.rs  # Search & filters
│       └── pagination.rs    # Page navigation
├── public/
│   └── style.css            # Styling
├── Cargo.toml
└── README.md
```

## API Integration

The demo connects to `http://localhost:3001/api` by default.

To change the backend URL:
```bash
export API_URL=http://your-backend.com/api
dx serve --platform web
```

## Learn More

- [dioxus-storage-sync](../../dioxus-storage-sync/README.md) - Sync system docs
- [dioxus-indexeddb](../../dioxus-indexeddb/README.md) - IndexedDB docs
- [Backend README](./backend/README.md) - API docs
