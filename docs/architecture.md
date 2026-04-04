# Architecture

## Overview

Dioxus Storage is organized into three layers:

```
┌─────────────────────────────────────────────────────────────────┐
│                      Application Layer                           │
│                      (Your Dioxus App)                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Hooks Layer                                 │
│  use_db, use_collection, use_query, use_local_storage, etc.     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Storage Layer                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  IndexedDB   │  │ LocalStorage │  │   SessionStorage     │  │
│  │              │  │              │  │                      │  │
│  │ - Large data │  │ - Simple KV  │  │ - Session KV         │  │
│  │ - Structured │  │ - Persistent │  │ - Temporary          │  │
│  │ - Async      │  │ - Sync       │  │ - Sync               │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Sync Layer (optional)                       │
│              dioxus-storage-sync                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Hot Sync  │  Background Sync  │  Conflict Resolution   │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Crate Relationships

### dioxus-indexeddb

The foundation crate providing IndexedDB access.

```
dioxus-indexeddb
├── database.rs      # Database connection
├── collection.rs    # Typed collections
├── query.rs         # Query builder
├── transaction.rs   # Multi-store transactions
├── migration.rs     # Schema migrations
├── hooks.rs         # Dioxus hooks
└── schema/          # Schema definitions
```

### dioxus-storage

Unified API that wraps IndexedDB, LocalStorage, and SessionStorage.

```
dioxus-storage
├── storage.rs           # Generic storage trait
├── local_storage.rs     # LocalStorage implementation
├── session_storage.rs   # SessionStorage implementation
└── re-exports           # From dioxus-indexeddb (optional)
```

### dioxus-storage-sync

Backend synchronization built on top of IndexedDB.

```
dioxus-storage-sync
├── sync_engine.rs    # Core sync logic
├── manager.rs        # Sync manager with hooks
├── client.rs         # HTTP client
├── config.rs         # Sync configuration
└── traits.rs         # Syncable trait
```

## Data Flow

### Reading Data (Hot Sync)

```
Component renders
      │
      ▼
use_query hook
      │
      ├──► Check local cache
      │         │
      │         ├──► Cache hit ──► Return data
      │         │
      │         └──► Cache miss
      │                   │
      ▼                   ▼
Fetch from backend ◄──────┘
      │
      ├──► Store in IndexedDB
      │
      └──► Return data
```

### Writing Data

```
User action
      │
      ▼
Update local state
      │
      ├──► Update IndexedDB (immediate)
      │
      └──► Queue for sync (if sync enabled)
                │
                ▼
          Background sync
                │
                ├──► Push to backend
                │
                └──► Mark as synced
```

## Reactive Architecture

All hooks return `Signal<T>` which integrates with Dioxus reactivity:

```rust
// Signal reads trigger re-renders
let users = use_collection::<User>(db, "users");

// When data changes...
users.write().push(new_user);

// ...components reading it automatically re-render
for user in users.read().iter() {
    // This re-runs when users changes
}
```

## Storage Comparison

| Feature | LocalStorage | SessionStorage | IndexedDB |
|---------|--------------|----------------|-----------|
| Capacity | ~5-10 MB | ~5-10 MB | Hundreds of MB |
| Data Type | Strings only | Strings only | Structured |
| Async | No | No | Yes |
| Transactions | No | No | Yes |
| Indexing | No | No | Yes |
| Persistence | Permanent | Session | Permanent |
| Use Case | Settings | Temp data | Large datasets |

## Sync Modes

### Hot Sync

Optimistic reading with background refresh:

1. Return cached data immediately
2. Fetch fresh data in background
3. Update UI when fresh data arrives

Best for: Read-heavy UIs where stale data is acceptable

### Background Sync

Periodic full synchronization:

1. Sync all data every N seconds
2. Queue local changes
3. Push changes periodically

Best for: Offline-first apps, keeping data fresh

### Manual Sync

Explicit control over synchronization:

1. User triggers sync
2. Show loading state
3. Update when complete

Best for: Forms, explicit save actions

## Security Considerations

1. **XSS Protection** - All data is sanitized by serde
2. **Origin Isolation** - Storage is per-origin (domain)
3. **HTTPS Required** - Some storage requires secure context
4. **Quota Limits** - Respect browser storage limits

## Performance Tips

1. **Use indexes** for frequently queried fields
2. **Batch operations** in transactions
3. **Limit query results** with pagination
4. **Cache aggressively** for read-heavy UIs
5. **Lazy load** large datasets
