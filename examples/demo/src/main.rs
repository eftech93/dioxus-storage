//! Dioxus Storage Demo
//!
//! This example demonstrates the Prisma-like migration system with folder-based migrations.
//!
//! # Migration Structure
//!
//! ```
//! src/migrations/
//!   mod.rs      # Migration registry
//!   v1.rs       # Initial schema (tasks store)
//!   v2.rs       # Add settings store
//!   v3.rs       # Add archived_tasks, remove old_temp
//! ```

use dioxus::prelude::*;
use dioxus_indexeddb::{
    Database, DatabaseConfig, Collection, Migration, MigrationManager, MigrationOp,
    IndexedDbError, Store, Schema, StoreDefinition, SchemaDatabase, define_store,
};
use dioxus_storage::{use_local_storage, use_session_storage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Include our migrations module
mod migrations;

fn main() {
    // Validate migrations on startup
    migrations::validate_migrations();
    dioxus::launch(app);
}

fn app() -> Element {
    rsx! {
        style { "{CSS}" }
        div { class: "app",
            h1 { "Dioxus Storage Demo" }
            p { class: "subtitle", "Prisma-like migrations with folder-based versioning" }
            
            div { class: "grid",
                // LocalStorage Demo
                LocalStorageDemo {}
                
                // SessionStorage Demo
                SessionStorageDemo {}
                
                // IndexedDB Demo with Schema Migrations
                IndexedDbDemo {}
                
                // Migration Info
                MigrationInfo {}
            }
        }
    }
}

// =============================================================================
// Migration Info Card
// =============================================================================

#[component]
fn MigrationInfo() -> Element {
    rsx! {
        div { class: "card migration-card",
            h2 { "📋 Migration Structure" }
            p { class: "description", "Folder-based migrations like Prisma/EF Core" }
            
            div { class: "file-tree",
                div { class: "folder", "📁 src/migrations/" }
                div { class: "file", "  📄 mod.rs - Registry" }
                div { class: "file version", "  📄 v1.rs - Initial (tasks)" }
                div { class: "file version", "  📄 v2.rs - Add settings" }
                div { class: "file version", "  📄 v3.rs - Add archived_tasks" }
            }
            
            h3 { "Type-Safe Stores" }
            pre { class: "code",
                "pub struct TaskStore;

impl Store for TaskStore {{
    fn store_name() -> &'static str {{ "tasks" }}
    fn key_path() -> &'static str {{ "id" }}
}}"
            }
            
            h3 { "Migration Definition" }
            pre { class: "code",
                "impl MigrationSet for V2Migration {{
    fn version() -> u32 {{ 2 }}
    
    fn operations() -> Vec<MigrationOp> {{
        vec![MigrationOp::CreateStore {{
            name: "settings".into(),
            key_path: "key".into(),
            auto_increment: false,
        }}]
    }}
}}"
            }
        }
    }
}

// =============================================================================
// LocalStorage Demo
// =============================================================================

#[component]
fn LocalStorageDemo() -> Element {
    let mut theme = use_local_storage::<String>("demo_theme", "light".to_string());
    let mut username = use_local_storage::<String>("demo_username", "".to_string());
    let mut counter = use_local_storage::<i32>("demo_counter", 0);

    let current_theme = theme.read().clone();
    let current_username = username.read().clone();
    let current_counter = *counter.read();

    rsx! {
        div { class: "card",
            h2 { "📦 LocalStorage" }
            p { class: "description", "Persistent key-value storage" }
            
            div { class: "section",
                label { "Theme:" }
                select {
                    value: "{current_theme}",
                    onchange: move |e| theme.set(e.value()),
                    option { value: "light", "☀️ Light" }
                    option { value: "dark", "🌙 Dark" }
                    option { value: "auto", "⚡ Auto" }
                }
                p { class: "value", "Current: {current_theme}" }
            }
            
            div { class: "section",
                label { "Username:" }
                input {
                    value: "{current_username}",
                    placeholder: "Enter username...",
                    oninput: move |e| username.set(e.value()),
                }
            }
            
            div { class: "section",
                label { "Counter:" }
                div { class: "row",
                    button { onclick: move |_| counter.set(current_counter - 1), "-" }
                    span { class: "counter", "{current_counter}" }
                    button { onclick: move |_| counter.set(current_counter + 1), "+" }
                }
            }
        }
    }
}

// =============================================================================
// SessionStorage Demo
// =============================================================================

#[component]
fn SessionStorageDemo() -> Element {
    let mut session_token = use_session_storage::<String>("demo_session_token", "".to_string());
    let mut temp_data = use_session_storage::<String>("demo_temp", "".to_string());

    let current_token = session_token.read().clone();
    let current_temp = temp_data.read().clone();

    let generate_token = move |_| {
        let token = format!("tok_{}", &Uuid::new_v4().to_string()[..8]);
        session_token.set(token);
    };

    rsx! {
        div { class: "card",
            h2 { "⏱️ SessionStorage" }
            p { class: "description", "Per-session storage" }
            
            div { class: "section",
                label { "Session Token:" }
                div { class: "row",
                    input {
                        readonly: true,
                        value: "{current_token}",
                        class: if current_token.is_empty() { "empty" } else { "" },
                    }
                    button { onclick: generate_token, "Generate" }
                }
                p { class: "hint", "Lost when tab closes" }
            }
            
            div { class: "section",
                label { "Temp Notes:" }
                textarea {
                    value: "{current_temp}",
                    oninput: move |e| temp_data.set(e.value()),
                }
            }
        }
    }
}

// =============================================================================
// IndexedDB Demo with Schema & Migrations
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Task {
    id: String,
    title: String,
    description: String,
    completed: bool,
    priority: Priority,
    created_at: i64,
}

impl Store for Task {
    fn store_name() -> &'static str {
        "tasks"
    }

    fn key_path() -> &'static str {
        "id"
    }

    fn key(&self) -> String {
        self.id.clone()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
enum Priority {
    Low,
    Medium,
    High,
}

impl Task {
    fn new(title: impl Into<String>, description: impl Into<String>, priority: Priority) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            description: description.into(),
            completed: false,
            priority,
            created_at: js_sys::Date::now() as i64,
        }
    }
}

#[component]
fn IndexedDbDemo() -> Element {
    let mut db_signal = use_signal(|| None::<Database>);
    let mut tasks = use_signal(Vec::<Task>::new);
    let mut loading = use_signal(|| false);
    let mut error_msg = use_signal(|| Option::<String>::None);

    // Initialize database with migrations
    use_effect(move || {
        spawn(async move {
            loading.set(true);
            error_msg.set(None);

            match init_database_with_migrations().await {
                Ok(db) => {
                    let collection: Collection<Task> = db.collection("tasks");
                    match collection.get_all().await {
                        Ok(data) => tasks.set(data),
                        Err(e) => error_msg.set(Some(format!("Load error: {}", e))),
                    }
                    db_signal.set(Some(db));
                    loading.set(false);
                }
                Err(e) => {
                    error_msg.set(Some(format!("DB error: {}", e)));
                    loading.set(false);
                }
            }
        });
    });

    let mut title_input = use_signal(|| "".to_string());
    let mut desc_input = use_signal(|| "".to_string());
    let mut priority_input = use_signal(|| Priority::Medium);

    let add_task = move |_| {
        let title = title_input.read().clone();
        if title.is_empty() { return; }

        let new_task = Task::new(title, desc_input.read().clone(), *priority_input.read());
        let db = db_signal.read().clone();
        let mut tasks = tasks.clone();

        spawn(async move {
            if let Some(db) = db {
                let collection: Collection<Task> = db.collection(Task::store_name());
                if collection.put(&new_task.key(), &new_task).await.is_ok() {
                    let mut current = tasks.read().clone();
                    current.push(new_task);
                    tasks.set(current);
                    title_input.set(String::new());
                    desc_input.set(String::new());
                }
            }
        });
    };

    let toggle_task = move |task: Task| {
        let db = db_signal.read().clone();
        let mut tasks = tasks.clone();
        let mut updated = task.clone();
        updated.completed = !updated.completed;

        spawn(async move {
            if let Some(db) = db {
                let collection: Collection<Task> = db.collection(Task::store_name());
                if collection.put(&updated.key(), &updated).await.is_ok() {
                    let mut current = tasks.read().clone();
                    if let Some(idx) = current.iter().position(|t| t.id == updated.id) {
                        current[idx] = updated;
                        tasks.set(current);
                    }
                }
            }
        });
    };

    let delete_task = move |task_id: String| {
        let db = db_signal.read().clone();
        let mut tasks = tasks.clone();

        spawn(async move {
            if let Some(db) = db {
                let collection: Collection<Task> = db.collection(Task::store_name());
                if collection.delete(&task_id).await.is_ok() {
                    let mut current = tasks.read().clone();
                    current.retain(|t| t.id != task_id);
                    tasks.set(current);
                }
            }
        });
    };

    let clear_all = move |_| {
        let db = db_signal.read().clone();
        let mut tasks = tasks.clone();

        spawn(async move {
            if let Some(db) = db {
                let collection: Collection<Task> = db.collection(Task::store_name());
                if collection.clear().await.is_ok() {
                    tasks.set(Vec::new());
                }
            }
        });
    };

    let current_title = title_input.read().clone();
    let current_desc = desc_input.read().clone();
    let current_priority = *priority_input.read();
    let task_count = tasks.read().len();

    rsx! {
        div { class: "card indexeddb",
            h2 { "🗄️ IndexedDB with Schema" }
            
            if *loading.read() {
                div { class: "loading", "Initializing database with migrations..." }
            }

            if let Some(err) = error_msg.read().as_ref() {
                div { class: "error", "{err}" }
            }

            div { class: "schema-info",
                span { class: "badge", "DB: dioxus_storage_demo" }
                span { class: "badge", "Version: {migrations::CURRENT_VERSION}" }
                span { class: "badge", "Store: tasks" }
            }

            div { class: "form",
                input {
                    placeholder: "Task title...",
                    value: "{current_title}",
                    oninput: move |e| title_input.set(e.value()),
                }
                textarea {
                    placeholder: "Description...",
                    value: "{current_desc}",
                    oninput: move |e| desc_input.set(e.value()),
                    rows: 2,
                }
                select {
                    value: match current_priority {
                        Priority::Low => "low",
                        Priority::Medium => "medium",
                        Priority::High => "high",
                    },
                    onchange: move |e| {
                        priority_input.set(match e.value().as_str() {
                            "low" => Priority::Low,
                            "high" => Priority::High,
                            _ => Priority::Medium,
                        });
                    },
                    option { value: "low", "🟢 Low" }
                    option { value: "medium", "🟡 Medium" }
                    option { value: "high", "🔴 High" }
                }
                button { 
                    onclick: add_task,
                    disabled: current_title.is_empty(),
                    "➕ Add Task"
                }
            }

            div { class: "task-list",
                h3 { "Tasks ({task_count})" }
                
                if tasks.read().is_empty() {
                    p { class: "empty", "No tasks. Add one!" }
                } else {
                    div { class: "actions",
                        button { class: "danger", onclick: clear_all, "🗑️ Clear All" }
                    }
                    
                    for task in tasks.read().iter().rev().cloned().collect::<Vec<_>>() {
                        TaskItem {
                            task: task.clone(),
                            on_toggle: toggle_task,
                            on_delete: delete_task,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TaskItem(
    task: Task,
    on_toggle: EventHandler<Task>,
    on_delete: EventHandler<String>,
) -> Element {
    let priority_class = match task.priority {
        Priority::Low => "priority-low",
        Priority::Medium => "priority-medium",
        Priority::High => "priority-high",
    };

    let task_for_toggle = task.clone();
    let task_id = task.id.clone();

    rsx! {
        div { 
            class: "task {priority_class}",
            class: if task.completed { "completed" } else { "" },
            
            div { class: "task-content",
                input {
                    type: "checkbox",
                    checked: task.completed,
                    onchange: move |_| on_toggle.call(task_for_toggle.clone()),
                }
                div { class: "task-text",
                    span { class: "title", "{task.title}" }
                    if !task.description.is_empty() {
                        span { class: "description", "{task.description}" }
                    }
                    span { class: "meta", 
                        "ID: {&task.id[..8]} | {format_time(task.created_at)}"
                    }
                }
            }
            
            button { 
                class: "delete-btn",
                onclick: move |_| on_delete.call(task_id.clone()),
                "🗑️"
            }
        }
    }
}

async fn init_database_with_migrations() -> Result<Database, IndexedDbError> {
    // Build config from schema (all stores defined in migrations)
    let mut config = DatabaseConfig::new("dioxus_storage_demo", migrations::CURRENT_VERSION);
    
    // Add stores defined in migrations
    config = config.with_store("tasks", "id");
    config = config.with_store("settings", "key");
    config = config.with_store("archived_tasks", "id");

    // Open with migrations
    Database::open_with_migrations(
        config,
        migrations::registry().into_manager(),
    ).await
}

fn format_time(timestamp: i64) -> String {
    let date = js_sys::Date::new(&js_sys::Number::from(timestamp as f64));
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date(),
        date.get_hours(),
        date.get_minutes()
    )
}

const CSS: &str = r#"
* { box-sizing: border-box; }

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #f5f5f5;
    margin: 0;
    padding: 20px;
    line-height: 1.6;
}

.app {
    max-width: 1200px;
    margin: 0 auto;
}

h1 {
    text-align: center;
    color: #333;
    margin-bottom: 5px;
}

.subtitle {
    text-align: center;
    color: #666;
    margin-bottom: 30px;
}

.grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
    gap: 20px;
}

.card {
    background: white;
    border-radius: 12px;
    padding: 24px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.card.indexeddb {
    grid-column: 1 / -1;
}

.card.migration-card {
    background: #f8fafc;
}

h2 {
    margin-top: 0;
    color: #222;
}

.description {
    color: #666;
    font-size: 0.9rem;
    margin-top: -10px;
    margin-bottom: 20px;
}

.section {
    margin-bottom: 20px;
}

label {
    display: block;
    font-weight: 600;
    margin-bottom: 8px;
    color: #444;
}

input, select, textarea {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 14px;
}

input:focus, select:focus, textarea:focus {
    outline: none;
    border-color: #3b82f6;
}

button {
    padding: 10px 16px;
    background: #3b82f6;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
}

button:hover:not(:disabled) {
    background: #2563eb;
}

button:disabled {
    background: #9ca3af;
    cursor: not-allowed;
}

button.danger {
    background: #ef4444;
}

.row {
    display: flex;
    gap: 10px;
    align-items: center;
}

.counter {
    font-size: 1.5rem;
    font-weight: 600;
    min-width: 40px;
    text-align: center;
}

.hint {
    font-size: 0.8rem;
    color: #999;
}

.loading, .error {
    padding: 12px;
    border-radius: 6px;
    margin-bottom: 16px;
}

.loading {
    background: #dbeafe;
    color: #1e40af;
}

.error {
    background: #fee2e2;
    color: #991b1b;
}

.schema-info {
    display: flex;
    gap: 10px;
    margin-bottom: 16px;
}

.badge {
    padding: 4px 12px;
    background: #e0e7ff;
    color: #4338ca;
    border-radius: 20px;
    font-size: 0.85rem;
    font-weight: 500;
}

.form {
    background: #f9fafb;
    padding: 20px;
    border-radius: 8px;
    margin-bottom: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.task-list h3 {
    margin-bottom: 16px;
}

.actions {
    margin-bottom: 16px;
}

.task {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 16px;
    background: #f9fafb;
    border-radius: 8px;
    margin-bottom: 12px;
    border-left: 4px solid #ccc;
}

.task.priority-low { border-left-color: #22c55e; }
.task.priority-medium { border-left-color: #f59e0b; }
.task.priority-high { border-left-color: #ef4444; }

.task.completed { opacity: 0.6; }
.task.completed .title { text-decoration: line-through; }

.task-content {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    flex: 1;
}

.task-content input[type="checkbox"] {
    width: 20px;
    height: 20px;
    margin-top: 2px;
}

.task-text {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
}

.task .title {
    font-weight: 600;
    color: #111;
}

.task .description {
    font-size: 0.9rem;
    color: #666;
}

.task .meta {
    font-size: 0.75rem;
    color: #999;
}

.delete-btn {
    background: transparent;
    color: #ef4444;
    padding: 4px 8px;
    font-size: 1.2rem;
}

.delete-btn:hover {
    background: #fee2e2;
}

/* Migration card styles */
.file-tree {
    background: #1e293b;
    color: #e2e8f0;
    padding: 16px;
    border-radius: 8px;
    font-family: 'Monaco', 'Consolas', monospace;
    font-size: 0.9rem;
    margin-bottom: 20px;
}

.file-tree .folder {
    color: #fbbf24;
}

.file-tree .file {
    padding-left: 16px;
}

.file-tree .file.version {
    color: #4ade80;
}

.code {
    background: #1e293b;
    color: #e2e8f0;
    padding: 16px;
    border-radius: 8px;
    overflow-x: auto;
    font-family: 'Monaco', 'Consolas', monospace;
    font-size: 0.85rem;
    line-height: 1.5;
}

.empty {
    color: #999;
}
"#;
