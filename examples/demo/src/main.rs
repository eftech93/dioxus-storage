//! Dioxus Storage Demo
//!
//! This example demonstrates LocalStorage, SessionStorage, and IndexedDB.

use dioxus::prelude::*;
use dioxus_client_storage::prelude::*;
use dioxus_indexeddb::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn main() {
    dioxus::launch(app);
}

fn app() -> Element {
    rsx! {
        style { "{CSS}" }
        div { class: "app",
            h1 { "Dioxus Storage Demo" }
            p { class: "subtitle", "LocalStorage, SessionStorage, and IndexedDB" }

            div { class: "grid",
                LocalStorageDemo {}
                SessionStorageDemo {}
                IndexedDbDemo {}
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
// IndexedDB Demo
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Task {
    id: String,
    title: String,
    description: String,
    completed: bool,
    priority: String, // "low", "medium", "high"
}

impl Task {
    fn new(title: impl Into<String>, description: impl Into<String>, priority: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            description: description.into(),
            completed: false,
            priority: priority.into(),
        }
    }
}

#[component]
fn IndexedDbDemo() -> Element {
    let mut db_signal = use_signal(|| None::<Database>);
    let mut tasks = use_signal(Vec::<Task>::new);
    let mut loading = use_signal(|| false);
    let mut error_msg = use_signal(|| Option::<String>::None);

    // Initialize database with index on priority
    use_effect(move || {
        spawn(async move {
            loading.set(true);
            error_msg.set(None);

            let config = DatabaseConfig::new("demo_db_v2", 1)
                .with_store_and_indexes(
                    "tasks", 
                    "id",
                    vec![
                        IndexConfig::new("priority_idx", "priority", false),
                    ]
                );

            match Database::open(config).await {
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
    let mut priority_input = use_signal(|| "medium".to_string());

    let add_task = move |_| {
        let title = title_input.read().clone();
        if title.is_empty() {
            return;
        }

        let new_task = Task::new(title, desc_input.read().clone(), priority_input.read().clone());
        let db = db_signal.read().clone();
        let mut tasks = tasks.clone();

        spawn(async move {
            if let Some(db) = db {
                let collection: Collection<Task> = db.collection("tasks");
                if collection.put(&new_task.id, &new_task).await.is_ok() {
                    let mut current = tasks.read().clone();
                    current.push(new_task);
                    tasks.set(current);
                    title_input.set(String::new());
                    desc_input.set(String::new());
                    priority_input.set("medium".to_string());
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
                let collection: Collection<Task> = db.collection("tasks");
                if collection.put(&updated.id, &updated).await.is_ok() {
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
                let collection: Collection<Task> = db.collection("tasks");
                if collection.delete(&task_id).await.is_ok() {
                    let mut current = tasks.read().clone();
                    current.retain(|t| t.id != task_id);
                    tasks.set(current);
                }
            }
        });
    };

    let current_title = title_input.read().clone();
    let current_desc = desc_input.read().clone();
    let current_priority = priority_input.read().clone();
    let task_count = tasks.read().len();
    
    // Filter state
    let mut filter_priority = use_signal(|| Option::<String>::None);
    let current_filter = filter_priority.read().clone();
    
    // Load tasks when filter changes (demonstrates index usage)
    use_effect(move || {
        let db = db_signal.read().clone();
        let filter = filter_priority.read().clone();
        let mut tasks = tasks.clone();
        
        spawn(async move {
            if let Some(db) = db {
                let collection: Collection<Task> = db.collection("tasks");
                let result = match filter {
                    Some(priority) => {
                        // Use index to get tasks by priority
                        log::info!("Using index 'priority_idx' to filter by: {}", priority);
                        collection.get_by_index("priority_idx", &priority).await
                    }
                    None => collection.get_all().await,
                };
                
                match result {
                    Ok(data) => tasks.set(data),
                    Err(e) => log::error!("Filter error: {}", e),
                }
            }
        });
    });

    rsx! {
        div { class: "card indexeddb",
            h2 { "🗄️ IndexedDB" }

            if *loading.read() {
                div { class: "loading", "Initializing database..." }
            }

            if let Some(err) = error_msg.read().as_ref() {
                div { class: "error", "{err}" }
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
                    value: "{current_priority}",
                    onchange: move |e| priority_input.set(e.value()),
                    option { value: "low", "🟢 Low Priority" }
                    option { value: "medium", "🟡 Medium Priority" }
                    option { value: "high", "🔴 High Priority" }
                }
                button {
                    onclick: add_task,
                    disabled: current_title.is_empty(),
                    "➕ Add Task"
                }
            }

            div { class: "filter-section",
                h3 { "Filter by Priority (using Index)" }
                div { class: "filter-buttons",
                    button {
                        onclick: move |_| filter_priority.set(None),
                        class: if current_filter.is_none() { "active" } else { "" },
                        "All"
                    }
                    button {
                        onclick: move |_| filter_priority.set(Some("low".to_string())),
                        class: if current_filter == Some("low".to_string()) { "active" } else { "" },
                        "🟢 Low"
                    }
                    button {
                        onclick: move |_| filter_priority.set(Some("medium".to_string())),
                        class: if current_filter == Some("medium".to_string()) { "active" } else { "" },
                        "🟡 Medium"
                    }
                    button {
                        onclick: move |_| filter_priority.set(Some("high".to_string())),
                        class: if current_filter == Some("high".to_string()) { "active" } else { "" },
                        "🔴 High"
                    }
                }
            }

            div { class: "task-list",
                h3 { "Tasks ({task_count})" }

                if tasks.read().is_empty() {
                    p { class: "empty", "No tasks. Add one!" }
                } else {
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
fn TaskItem(task: Task, on_toggle: EventHandler<Task>, on_delete: EventHandler<String>) -> Element {
    let task_for_toggle = task.clone();
    let task_id = task.id.clone();
    let priority_icon = match task.priority.as_str() {
        "high" => "🔴",
        "medium" => "🟡",
        _ => "🟢",
    };

    rsx! {
        div {
            class: "task",
            class: if task.completed { "completed" } else { "" },
            class: "priority-{task.priority}",

            div { class: "task-content",
                input {
                    r#type: "checkbox",
                    checked: task.completed,
                    onchange: move |_| on_toggle.call(task_for_toggle.clone()),
                }
                div { class: "task-text",
                    span { class: "title", "{priority_icon} {task.title}" }
                    if !task.description.is_empty() {
                        span { class: "description", "{task.description}" }
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

.task {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 16px;
    background: #f9fafb;
    border-radius: 8px;
    margin-bottom: 12px;
    border-left: 4px solid #3b82f6;
}

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

.delete-btn {
    background: transparent;
    color: #ef4444;
    padding: 4px 8px;
    font-size: 1.2rem;
}

.delete-btn:hover {
    background: #fee2e2;
}

.empty {
    color: #999;
}

.filter-section {
    background: #f0f9ff;
    padding: 16px;
    border-radius: 8px;
    margin-bottom: 20px;
}

.filter-section h3 {
    margin: 0 0 12px 0;
    font-size: 0.95rem;
    color: #0369a1;
}

.filter-buttons {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
}

.filter-buttons button {
    background: white;
    color: #374151;
    border: 1px solid #d1d5db;
    padding: 6px 12px;
    font-size: 0.9rem;
}

.filter-buttons button:hover {
    background: #f3f4f6;
}

.filter-buttons button.active {
    background: #3b82f6;
    color: white;
    border-color: #3b82f6;
}

.task.priority-high {
    border-left-color: #ef4444;
}

.task.priority-medium {
    border-left-color: #f59e0b;
}

.task.priority-low {
    border-left-color: #22c55e;
}
"#;
