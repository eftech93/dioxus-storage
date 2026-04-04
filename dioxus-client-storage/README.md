# dioxus-client-storage

[![Crates.io](https://img.shields.io/crates/v/dioxus-client-storage.svg)](https://crates.io/crates/dioxus-client-storage)
[![Docs.rs](https://docs.rs/dioxus-client-storage/badge.svg)](https://docs.rs/dioxus-client-storage)
[![Documentation](https://img.shields.io/badge/docs-eftech93.github.io/dioxus--storage-9D6B4C.svg)](https://eftech93.github.io/dioxus-storage)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

Unified storage API for Dioxus supporting LocalStorage, SessionStorage, and IndexedDB.

## Features

- 💾 **LocalStorage** - Persistent key-value storage
- 📂 **SessionStorage** - Per-session key-value storage  
- 🗄️ **IndexedDB** - Large structured data (via `dioxus-indexeddb`)
- 🪝 **Reactive hooks** - `use_local_storage`, `use_session_storage`
- ⚡ **Sync & async** APIs for different use cases

## Installation

```toml
[dependencies]
dioxus-client-storage = "0.0.1"
```

With specific features:

```toml
# Only LocalStorage
dioxus-client-storage = { version = "0.0.1", default-features = false, features = ["localstorage"] }

# Only IndexedDB
dioxus-client-storage = { version = "0.0.1", default-features = false, features = ["indexeddb"] }

# All storage types
dioxus-client-storage = { version = "0.0.1", features = ["indexeddb", "localstorage", "sessionstorage"] }
```

## LocalStorage

Persistent storage that survives page reloads and browser restarts.

```rust
use dioxus::prelude::*;
use dioxus_storage::prelude::*;

#[component]
fn App() -> Element {
    // This value persists across reloads!
    let theme = use_local_storage::<String>("theme", "light".to_string());
    
    rsx! {
        div {
            p { "Current theme: {theme.read()}" }
            button {
                onclick: move |_| theme.set("dark".to_string()),
                "Switch to Dark"
            }
        }
    }
}
```

### Direct API

```rust
use dioxus_storage::LocalStorage;

let storage = LocalStorage::new();

// Store
storage.set("key", &value)?;

// Retrieve
let value: Option<String> = storage.get("key")?;

// Remove
storage.remove("key")?;

// Clear all
storage.clear()?;
```

## SessionStorage

Storage that persists only for the current session (tab).

```rust
use dioxus::prelude::*;
use dioxus_storage::prelude::*;

#[component]
fn FormWizard() -> Element {
    // Data is lost when tab closes
    let draft = use_session_storage::<String>("form_draft", String::new());
    
    rsx! {
        input {
            value: "{draft.read()}",
            oninput: move |e| draft.set(e.value())
        }
    }
}
```

## IndexedDB Integration

When the `indexeddb` feature is enabled:

```rust
use dioxus::prelude::*;
use dioxus_storage::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: String,
    text: String,
    done: bool,
}

#[component]
fn TodoApp() -> Element {
    let db = use_db(DbConfig::new("todos", 1)
        .with_store("items", "id"));
    
    let todos = use_collection::<Todo>(db, "items");
    
    rsx! {
        ul {
            for todo in todos.read().iter() {
                li { 
                    input { r#type: "checkbox", checked: todo.done }
                    "{todo.text}"
                }
            }
        }
    }
}
```

## Choosing the Right Storage

| Storage | Capacity | Persistence | Use Case |
|---------|----------|-------------|----------|
| LocalStorage | ~5-10 MB | Permanent | User preferences, settings |
| SessionStorage | ~5-10 MB | Session only | Form drafts, wizard state |
| IndexedDB | Hundreds of MB | Permanent | Large datasets, offline apps |

## License

MIT OR Apache-2.0
