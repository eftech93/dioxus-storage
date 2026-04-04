use crate::models::SyncEvent;
use crate::sync::SyncMode;
use dioxus::prelude::*;

#[component]
pub fn SyncLogViewer(events: Vec<SyncEvent>) -> Element {
    rsx! {
        div { class: "sync-log",
            h3 { "📋 Sync Log" }

            {if events.is_empty() {
                rsx! {
                    div { class: "log-empty",
                        "No sync events yet. Perform a sync to see logs."
                    }
                }
            } else {
                rsx! {
                    div { class: "log-entries",
                        for event in events.iter().cloned() {
                            SyncLogEntry { event: event.clone() }
                        }
                    }
                }
            }}
        }
    }
}

#[component]
fn SyncLogEntry(event: SyncEvent) -> Element {
    let class_name = if event.success {
        "log-entry success"
    } else {
        "log-entry error"
    };

    let mode_label = match event.mode {
        SyncMode::Hot => "🔥 HOT",
        SyncMode::Background => "🌙 BG",
    };

    let timestamp = event.timestamp.format("%H:%M:%S").to_string();

    rsx! {
        div { class: "{class_name}",
            div { class: "log-header",
                span { class: "log-timestamp", "{timestamp}" }
                span { class: "log-mode", "{mode_label}" }
            }
            div { class: "log-action", "{event.action}" }
            div { class: "log-details",
                span { "{event.items_count} items" }
                span { "•" }
                span { "{event.duration_ms}ms" }
            }
            {if !event.message.is_empty() {
                rsx! {
                    div { class: "log-message", "{event.message}" }
                }
            } else {
                rsx! {}
            }}
        }
    }
}
