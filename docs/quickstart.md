# Quick Start

## 1. LocalStorage (5 minutes)

The simplest way to persist data:

```rust
use dioxus::prelude::*;
use dioxus_storage::prelude::*;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    // This value persists across page reloads!
    let counter = use_local_storage::<i32>("counter", 0);
    
    rsx! {
        div {
            h1 { "Counter: {counter.read()}" }
            button {
                onclick: move |_| counter.set(*counter.read() + 1),
                "Increment"
            }
        }
    }
}
```

## 2. IndexedDB (10 minutes)

For structured data:

```rust
use dioxus::prelude::*;
use dioxus_indexeddb::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Note {
    id: String,
    content: String,
    created_at: u64,
}

#[component]
fn NotesApp() -> Element {
    // Open database
    let db = use_db(DatabaseConfig::new("notes_db", 1)
        .with_store("notes", "id"));
    
    // Get collection
    let notes = use_collection::<Note>(db, "notes");
    let mut new_content = use_signal(String::new);
    
    rsx! {
        div {
            h1 { "My Notes" }
            
            // Add note form
            input {
                value: "{new_content.read()}",
                oninput: move |e| new_content.set(e.value()),
                placeholder: "New note..."
            }
            
            // Note list
            ul {
                for note in notes.read().iter() {
                    li { "{note.content}" }
                }
            }
        }
    }
}
```

## 3. Backend Sync (15 minutes)

Sync with a REST API:

```rust
use dioxus::prelude::*;
use dioxus_storage_sync::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    id: String,
    title: String,
    done: bool,
}

impl Syncable for Task {
    fn sync_id(&self) -> String { self.id.clone() }
    fn sync_timestamp(&self) -> i64 { 0 }
    fn mark_synced(&mut self) {}
    fn is_dirty(&self) -> bool { true }
}

#[component]
fn TaskApp() -> Element {
    let collection: Collection<Task> = /* initialized elsewhere */;

    let config = SyncConfig::new("http://localhost:3001/api")
        .with_hot_sync(true)
        .with_resource_path("tasks");

    let manager = SyncManager::new(collection, config);
    let mut tasks = use_signal(Vec::<Task>::new);

    rsx! {
        div {
            h1 { "Tasks" }

            // Load button
            button {
                onclick: move |_| {
                    let mgr = manager.clone();
                    spawn(async move {
                        if let Ok(all) = mgr.get_all().await {
                            tasks.set(all);
                        }
                    });
                },
                "🔄 Load Tasks"
            }

            // Task list
            ul {
                for task in tasks.read().iter() {
                    li {
                        if task.done { "✅" } else { "⬜" }
                        " {task.title}"
                    }
                }
            }
        }
    }
}
```

## Next Steps

- Learn about [Database Migrations](migrations.md)
- Explore the [Query Builder](queries.md)
- Check out the [Examples](examples.md)
