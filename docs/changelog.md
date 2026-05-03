# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.3] - 2025-05-02

### Added

#### dioxus-indexeddb
- **Cursor-based Iteration** - Efficient traversal of large datasets
  - `Cursor<T>` - Typed cursor with `next()`, `advance()`, and `into_stream()`
  - `Collection::open_cursor()` - Open a cursor on a collection
  - `Collection::open_cursor_on_index()` - Open a cursor on an index
  - `CursorDirection` support (Next, NextUnique, Prev, PrevUnique)
  - `CursorBound` - Convenient range bounds (Only, LowerBound, UpperBound, Range)
  - `into_stream()` converts a cursor into a `futures::Stream`

#### dioxus-storage-sync
- **Offline Queue** - Queue mutations when offline and replay when restored
  - Detects online/offline status via browser events
  - Queues `Insert`, `Update`, and `Delete` operations when offline
  - Persists queue to a dedicated IndexedDB database
  - Automatically replays queue during background sync when back online
  - Conflict resolution during replay (ServerWins, Manual, etc.)
  - Queue status exposed in `SyncStatus` (`queue_pending`, `queue_replaying`, `is_online`)
  - `SyncManager::replay_queue()` for manual replay

## [0.0.2] - 2025-04-05

### Added

#### dioxus-indexeddb
- **Index Support** - Fast queries on specific fields
  - `IndexConfig` - Configuration for defining indexes
  - `DatabaseConfig::with_store_and_indexes()` - Create stores with indexes
  - `DatabaseConfig::with_index()` - Add indexes to existing stores
  - `Collection::get_by_index()` - Query items by index value
  - `Collection::get_one_by_index()` - Get single item by unique index
  - `Collection::find()` - Execute Query with index optimization
  - `Query::use_index()` - Hint to use specific index

## [0.0.1] - 2024-04-03

### Added

- Initial release of all three crates

#### dioxus-indexeddb
- Type-safe IndexedDB collections with serde serialization
- `use_db`, `use_collection`, `use_query` hooks
- Query builder with filtering and sorting
- Multi-store transactions
- Database migrations
- Async/await API

#### dioxus-client-storage
- Unified storage API
- `LocalStorage` - Persistent key-value storage
- `SessionStorage` - Per-session key-value storage
- `use_local_storage`, `use_session_storage` hooks
- IndexedDB integration via `dioxus-indexeddb`

#### dioxus-client-storage-sync
- Hot sync mode - On-demand fetching with local cache
- Background sync mode - Periodic synchronization
- Conflict resolution strategies
- Bidirectional sync (push local changes to server)
- `use_sync` hook for reactive sync state

### Examples
- Basic demo showing all storage types
- Sync demo with MongoDB backend
- Complete documentation with docsify

[0.0.2]: https://github.com/eftech93/dioxus-client-storage/releases/tag/v0.0.2
[0.0.1]: https://github.com/eftech93/dioxus-client-storage/releases/tag/v0.0.1
