# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.0.1]: https://github.com/eftech93/dioxus-client-storage/releases/tag/v0.0.1
