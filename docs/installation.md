# Installation

## Prerequisites

- Rust 1.75 or later
- Dioxus 0.6 or later
- For web builds: `wasm32-unknown-unknown` target

## Install Rust target

```bash
rustup target add wasm32-unknown-unknown
```

## Install Dioxus CLI (optional)

```bash
cargo install dioxus-cli
```

## Adding dependencies

### dioxus-indexeddb

For applications using IndexedDB directly:

```toml
[dependencies]
dioxus-indexeddb = "0.0.1"
serde = { version = "1.0", features = ["derive"] }
```

### dioxus-storage

For unified storage API:

```toml
[dependencies]
dioxus-storage = "0.0.1"
```

With specific features:

```toml
[dependencies]
# Only IndexedDB
dioxus-storage = { version = "0.0.1", default-features = false, features = ["indexeddb"] }

# Only LocalStorage
dioxus-storage = { version = "0.0.1", default-features = false, features = ["localstorage"] }

# All storage types
dioxus-storage = { version = "0.0.1", features = ["indexeddb", "localstorage", "sessionstorage"] }
```

### dioxus-storage-sync

For backend synchronization:

```toml
[dependencies]
dioxus-storage-sync = "0.0.1"
```

## Git dependencies

To use the latest git version:

```toml
[dependencies]
dioxus-indexeddb = { git = "https://github.com/eftech93/dioxus-storage" }
dioxus-storage = { git = "https://github.com/eftech93/dioxus-storage" }
dioxus-storage-sync = { git = "https://github.com/eftech93/dioxus-storage" }
```

## Workspace setup

For a multi-crate workspace:

```toml
# Cargo.toml (workspace root)
[workspace]
members = ["frontend", "backend"]

[workspace.dependencies]
dioxus = "0.6"
dioxus-storage = "0.0.1"
serde = { version = "1.0", features = ["derive"] }
```

```toml
# frontend/Cargo.toml
[package]
name = "frontend"
version = "0.1.0"
edition = "2021"

[dependencies]
dioxus = { workspace = true }
dioxus-storage = { workspace = true }
serde = { workspace = true }
```

## Verify installation

Create a test file:

```rust
// src/main.rs
use dioxus::prelude::*;
use dioxus_storage::prelude::*;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let value = use_local_storage::<String>("test", "hello".to_string());
    
    rsx! {
        "Value: {value.read()}"
    }
}
```

Build for web:

```bash
dx build --platform web
```

Or with cargo:

```bash
cargo build --target wasm32-unknown-unknown
```
