use dioxus::prelude::*;
use dioxus_indexeddb::{Collection, Database, DatabaseConfig};
use dioxus_storage_sync::{ConflictResolution, SyncConfig, SyncManager, SyncMode, Syncable};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Task {
    id: String,
    title: String,
    completed: bool,
}

impl Syncable for Task {
    fn sync_id(&self) -> String {
        self.id.clone()
    }

    fn sync_timestamp(&self) -> i64 {
        0
    }

    fn mark_synced(&mut self) {}

    fn is_dirty(&self) -> bool {
        true
    }
}

#[component]
fn QueueTaskRow(task: Task, on_toggle: EventHandler<Task>, on_delete: EventHandler<String>) -> Element {
    let task_for_toggle = task.clone();
    let task_id = task.id.clone();
    rsx! {
        div { class: "queue-task-row",
            input {
                r#type: "checkbox",
                checked: task.completed,
                onchange: move |_| on_toggle.call(task_for_toggle.clone()),
            }
            span { class: if task.completed { "completed" } else { "" }, "{task.title}" }
            button {
                class: "delete-btn",
                onclick: move |_| on_delete.call(task_id.clone()),
                "🗑️"
            }
        }
    }
}

#[component]
pub fn OfflineQueueDemo() -> Element {
    let mut db = use_signal(|| None::<Database>);

    use_effect(move || {
        spawn(async move {
            let _ = Database::delete("queue_demo_db").await;
            let config = DatabaseConfig::new("queue_demo_db", 1).with_store("tasks", "id");
            match Database::open(config).await {
                Ok(database) => {
                    db.set(Some(database));
                }
                Err(e) => log::error!("Queue demo DB error: {}", e),
            }
        });
    });

    let db_ref = db.read();
    match db_ref.as_ref() {
        Some(database) => rsx! { OfflineQueueDemoInner { db: database.clone() } },
        None => rsx! { div { class: "loading", "Loading queue demo..." } },
    }
}

#[component]
fn OfflineQueueDemoInner(db: Database) -> Element {
    let collection: Collection<Task> = db.collection("tasks");
    let config = SyncConfig::new("http://localhost:3001/api")
        .with_mode(SyncMode::Bidirectional)
        .with_hot_sync(true)
        .with_conflict_resolution(ConflictResolution::LastWriteWins);
    let manager = Rc::new(SyncManager::new(collection, config));

    let tasks = use_signal(Vec::<Task>::new);
    let mut title_input = use_signal(|| "".to_string());
    let mut initialized = use_signal(|| false);

    let mgr_effect = manager.clone();
    use_effect(move || {
        if *initialized.read() {
            return;
        }
        initialized.set(true);

        let mgr = mgr_effect.clone();
        let mut tasks = tasks.clone();
        spawn(async move {
            loop {
                if let Ok(all) = mgr.get_all().await {
                    tasks.set(all);
                }
                gloo_timers::future::sleep(std::time::Duration::from_secs(2)).await;
            }
        });
    });

    let manager_sig = use_signal(|| manager.clone());

    let add_task = move |_| {
        let title = title_input.read().clone();
        if title.is_empty() {
            return;
        }
        let task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            completed: false,
        };
        let mgr = manager_sig.read().clone();
        let mut input = title_input.clone();
        spawn(async move {
            let _ = mgr.save(&task).await;
            input.set(String::new());
        });
    };

    let replay_queue = move |_| {
        let mgr = manager_sig.read().clone();
        spawn(async move {
            let _ = mgr.replay_queue().await;
        });
    };

    let mgr = manager_sig.read().clone();
    let status = mgr.status().read().clone();
    let is_online = status.is_online;
    let queue_pending = status.queue_pending;
    let queue_replaying = status.queue_replaying;

    rsx! {
        div { class: "queue-demo-card",
            h2 { "📴 Offline Queue Demo" }
            p { class: "subtitle", "Operations queue when offline and replay when restored" }

            div { class: "queue-status-bar",
                span { class: if is_online { "online" } else { "offline" },
                    if is_online { "🟢 Online" } else { "🔴 Offline" }
                }
                span { " | Pending: {queue_pending}" }
                if queue_replaying {
                    span { " | 🔄 Replaying..." }
                }
            }

            div { class: "queue-form",
                input {
                    placeholder: "New task...",
                    value: "{title_input.read()}",
                    oninput: move |e| title_input.set(e.value()),
                }
                button { class: "btn btn-primary", onclick: add_task, "➕ Add Task" }
                button {
                    class: "btn btn-secondary",
                    onclick: replay_queue,
                    disabled: queue_pending == 0,
                    "🔄 Replay Queue ({queue_pending})"
                }
            }

            div { class: "queue-task-list",
                if tasks.read().is_empty() {
                    p { class: "empty", "No tasks yet. Add one!" }
                } else {
                    for task in tasks.read().iter().cloned() {
                        QueueTaskRow {
                            task: task.clone(),
                            on_toggle: move |t: Task| {
                                let mut updated = t.clone();
                                updated.completed = !updated.completed;
                                let mgr = manager_sig.read().clone();
                                spawn(async move {
                                    let _ = mgr.save(&updated).await;
                                });
                            },
                            on_delete: move |id: String| {
                                let mgr = manager_sig.read().clone();
                                spawn(async move {
                                    let _ = mgr.delete(&id).await;
                                });
                            },
                        }
                    }
                }
            }

            p { class: "hint",
                "Tip: Open DevTools → Network → Offline, then add a task to see it queued."
            }
        }
    }
}
