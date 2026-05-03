# Examples

## Basic Demo

A simple demonstration of LocalStorage, SessionStorage, and IndexedDB with migrations.

```bash
cd examples/demo
dx serve --platform web
```

**Features:**
- Theme switching with LocalStorage
- Form data with SessionStorage
- Todo list with IndexedDB
- Database migrations demo
- **Cursor demo** - Iterate large datasets without loading everything into memory

## Sync Demo

Complete example with backend API, MongoDB, and real-time sync.

```bash
# 1. Start the backend
cd examples/sync-demo/backend
docker-compose up -d

# 2. Run the frontend
cd examples/sync-demo
dx serve --platform web
```

**Features:**
- 100 sample products in MongoDB
- Hot sync vs Background sync modes
- Query caching
- Visual sync logging
- Pagination
- **Offline queue demo** - Queue mutations when offline, replay on reconnect

### Backend API

The demo backend provides:

| Endpoint | Description |
|----------|-------------|
| `GET /api/health` | Health check |
| `GET /api/products` | Paginated products |
| `GET /api/products/search` | Search products |
| `GET /api/products/categories` | List categories |
| `GET /api/products/brands` | List brands |
| `GET /api/tasks/:id` | Fetch a single task |
| `PUT /api/tasks/:id` | Upsert a task (offline queue demo) |
| `DELETE /api/tasks/:id` | Delete a task (offline queue demo) |

### Demo Flow

1. **Start the demo** - Open `http://localhost:8080`
2. **Click "Sync All"** - Fetches all 100 products to IndexedDB
3. **Switch to "Hot Sync" mode** - Subsequent loads are instant from cache
4. **Try "Hard Sync"** - Forces fresh data from backend
5. **Navigate pages** - Each page is cached separately
6. **Test offline queue** - Use DevTools Network → Offline, add tasks, then replay

## Code Examples

### Counter with Persistence

```rust
use dioxus::prelude::*;
use dioxus_client_storage::prelude::*;

#[component]
fn Counter() -> Element {
    let count = use_local_storage::<i32>("counter", 0);

    rsx! {
        div { class: "counter",
            h2 { "Count: {count.read()}" }
            button { onclick: move |_| count.set(*count.read() - 1), "-" }
            button { onclick: move |_| count.set(*count.read() + 1), "+" }
            button { onclick: move |_| count.set(0), "Reset" }
        }
    }
}
```

### Todo List with IndexedDB

```rust
use dioxus::prelude::*;
use dioxus_indexeddb::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: String,
    text: String,
    done: bool,
}

#[component]
fn TodoList() -> Element {
    let db = use_db(DatabaseConfig::new("todos", 1)
        .with_store("items", "id"));

    let todos = use_collection::<Todo>(db, "items");
    let mut new_text = use_signal(String::new);

    let add_todo = move |_| {
        let todo = Todo {
            id: uuid::Uuid::new_v4().to_string(),
            text: new_text.read().clone(),
            done: false,
        };
        // Add to collection...
        new_text.set(String::new());
    };

    rsx! {
        div { class: "todo-list",
            input {
                value: "{new_text.read()}",
                oninput: move |e| new_text.set(e.value()),
                onkeypress: move |e| if e.key() == "Enter" { add_todo(()) }
            }
            ul {
                for todo in todos.read().iter() {
                    li { class: if todo.done { "done" } else { "" },
                        "{todo.text}"
                    }
                }
            }
        }
    }
}
```

### Cursor Iteration

```rust
use dioxus_indexeddb::prelude::*;
use futures::StreamExt;

let collection = db.collection::<Item>("items");

// Manual iteration
let mut cursor = collection
    .open_cursor(None, Some(CursorDirection::Next))
    .await?;
while let Some(item) = cursor.next().await? {
    println!("{}", item.name);
}

// Stream API
let cursor = collection
    .open_cursor(None, Some(CursorDirection::Next))
    .await?;
let names: Vec<String> = cursor
    .into_stream()
    .filter_map(|r| async move { r.ok().map(|i| i.name) })
    .collect()
    .await;
```

### User Preferences

```rust
use dioxus::prelude::*;
use dioxus_client_storage::prelude::*;

#[derive(Debug, Clone, Default)]
struct Preferences {
    theme: String,
    font_size: i32,
    notifications: bool,
}

#[component]
fn Settings() -> Element {
    let prefs = use_local_storage::<Preferences>("prefs", Preferences::default());

    rsx! {
        div { class: "settings",
            h2 { "Preferences" }

            select {
                value: "{prefs.read().theme}",
                onchange: move |e| {
                    let mut p = prefs.read().clone();
                    p.theme = e.value();
                    prefs.set(p);
                },
                option { value: "light", "Light Theme" }
                option { value: "dark", "Dark Theme" }
            }

            label {
                input {
                    r#type: "checkbox",
                    checked: prefs.read().notifications,
                    onchange: move |e| {
                        let mut p = prefs.read().clone();
                        p.notifications = e.checked();
                        prefs.set(p);
                    }
                }
                "Enable Notifications"
            }
        }
    }
}
```

## More Examples

Check the GitHub repository for more examples:
- [github.com/eftech93/dioxus-storage/tree/main/examples](https://github.com/eftech93/dioxus-storage/tree/main/examples)
