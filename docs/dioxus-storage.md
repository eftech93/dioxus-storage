# dioxus-storage

Unified storage API supporting LocalStorage, SessionStorage, and IndexedDB.

## Installation

```toml
[dependencies]
dioxus-storage = "0.0.1"
```

Or with specific features:

```toml
[dependencies]
dioxus-storage = { version = "0.0.1", default-features = false, features = ["indexeddb"] }
```

## Features

- `indexeddb` - Include IndexedDB support (enabled by default)
- `localstorage` - Include LocalStorage support (enabled by default)
- `sessionstorage` - Include SessionStorage support

## LocalStorage

Simple key-value storage that persists across browser sessions.

```rust
use dioxus_storage::prelude::*;

#[component]
fn App() -> Element {
    // Read/write a value from LocalStorage
    let theme = use_local_storage::<String>("theme", "light".to_string());
    
    rsx! {
        div {
            p { "Current theme: {theme.read()}" }
            button {
                onclick: move |_| {
                    theme.set("dark".to_string());
                },
                "Switch to Dark"
            }
        }
    }
}
```

### LocalStorage API

```rust
use dioxus_storage::LocalStorage;

// Direct usage (not reactive)
let storage = LocalStorage::new();

// Set a value
storage.set("key", &value)?;

// Get a value
let value: Option<String> = storage.get("key")?;

// Remove a value
storage.remove("key")?;

// Clear all values
storage.clear()?;
```

## SessionStorage

Key-value storage that persists only for the current session.

```rust
use dioxus_storage::prelude::*;

#[component]
fn App() -> Element {
    // Session-scoped storage
    let session_data = use_session_storage::<String>("temp_data", String::new());
    
    rsx! {
        input {
            value: "{session_data.read()}",
            oninput: move |e| session_data.set(e.value())
        }
    }
}
```

## IndexedDB Integration

When the `indexeddb` feature is enabled, you get access to the full IndexedDB API:

```rust
use dioxus_storage::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: String,
    text: String,
    completed: bool,
}

#[component]
fn TodoApp() -> Element {
    let db = use_db(DbConfig::new("todos_app", 1)
        .with_store("todos", "id"));
    
    let todos = use_collection::<Todo>(db, "todos");
    
    rsx! {
        ul {
            for todo in todos.read().iter() {
                li { 
                    input {
                        r#type: "checkbox",
                        checked: todo.completed,
                    }
                    "{todo.text}"
                }
            }
        }
    }
}
```

## Storage Hook

Generic storage hook that works with any storage backend:

```rust
use dioxus_storage::prelude::*;

#[component]
fn Settings() -> Element {
    // Uses LocalStorage by default
    let setting = use_storage::<bool>(
        StorageConfig::local(),
        "notifications_enabled",
        true
    );
    
    rsx! {
        label {
            input {
                r#type: "checkbox",
                checked: *setting.read(),
                onchange: move |e| setting.set(e.checked())
            }
            "Enable Notifications"
        }
    }
}
```

## Choosing the Right Storage

| Storage | Capacity | Persistence | Use Case |
|---------|----------|-------------|----------|
| LocalStorage | ~5-10 MB | Permanent | User preferences, settings |
| SessionStorage | ~5-10 MB | Session only | Temporary form data, wizard state |
| IndexedDB | Hundreds of MB | Permanent | Large datasets, offline apps |

## Complete Example

```rust
use dioxus::prelude::*;
use dioxus_storage::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppSettings {
    theme: String,
    font_size: i32,
    sidebar_collapsed: bool,
}

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    // Simple settings in LocalStorage
    let theme = use_local_storage::<String>("theme", "light".to_string());
    
    // Complex data in IndexedDB
    let db = use_db(DbConfig::new("my_app", 1)
        .with_store("settings", "id"));
    
    rsx! {
        div { class: "app",
            Navbar { theme: theme.clone() }
            MainContent { db }
        }
    }
}

#[component]
fn Navbar(theme: Signal<String>) -> Element {
    rsx! {
        nav {
            button {
                onclick: move |_| {
                    let new_theme = if *theme.read() == "light" { "dark" } else { "light" };
                    theme.set(new_theme.to_string());
                },
                "Toggle Theme"
            }
        }
    }
}
```
