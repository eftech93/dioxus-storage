use dioxus::prelude::*;
use dioxus_logger::tracing::Level;

mod components;
mod models;
mod sync;

use components::{FilterPanel, Pagination, ProductCard, SyncLogViewer};
use models::Product;
use models::SyncEvent;
use sync::{SyncMode, SyncService};

const API_URL: &str = "http://localhost:3001/api";

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

#[component]
fn App() -> Element {
    // IndexedDB state
    let mut db = use_signal(|| None::<dioxus_indexeddb::database::Database>);
    let mut local_product_count = use_signal(|| 0usize);
    let mut last_sync_time = use_signal(|| None::<String>);

    // UI State
    let mut products = use_signal(Vec::<Product>::new);
    let mut filtered_products = use_signal(Vec::<Product>::new);
    let mut current_page = use_signal(|| 1u32);
    let total_pages = use_signal(|| 20u32);
    let mut is_loading = use_signal(|| false);
    let mut search_query = use_signal(String::new);
    let mut selected_category = use_signal(|| None::<String>);
    let mut sync_mode = use_signal(|| SyncMode::Hot);
    let mut sync_events = use_signal(Vec::<SyncEvent>::new);
    let _background_sync_active = use_signal(|| false);

    // Initialize IndexedDB on mount
    use_effect(move || {
        spawn(async move {
            match SyncService::init_database().await {
                Ok(database) => {
                    log::info!("IndexedDB initialized successfully");
                    db.set(Some(database));
                }
                Err(e) => {
                    log::error!("Failed to initialize IndexedDB: {}", e);
                }
            }
        });
    });

    // Update local stats when DB changes
    use_effect(move || {
        if let Some(ref database) = *db.read() {
            let db = database.clone();
            spawn(async move {
                if let Ok(products) = SyncService::get_local_products(&db).await {
                    local_product_count.set(products.len());
                }
                if let Ok(Some(meta)) = SyncService::get_sync_meta(&db).await {
                    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(meta.timestamp));
                    last_sync_time.set(Some(format!(
                        "{}:{:02}:{:02}",
                        date.get_hours(),
                        date.get_minutes(),
                        date.get_seconds()
                    )));
                }
            });
        }
    });

    // Load products for current page
    // hard_sync: true = force fetch from backend, false = allow cache
    let load_page = use_callback(move |(hard_sync,): (bool,)| {
        let mut is_loading = is_loading.clone();
        let mut products = products.clone();
        let mut filtered_products = filtered_products.clone();
        let mut sync_events = sync_events.clone();
        let mode = *sync_mode.read();
        let db_signal = db.clone();
        let page = *current_page.read();

        spawn(async move {
            is_loading.set(true);
            let service = SyncService::new(API_URL);

            let start_time = web_time::Instant::now();

            let result = if mode == SyncMode::Hot && !hard_sync {
                if let Some(ref database) = *db_signal.read() {
                    service
                        .hot_sync_products(
                            database,
                            page,
                            5,
                            String::new(),
                            None,
                            false, // allow cache
                        )
                        .await
                } else {
                    // Fallback to backend if IndexedDB not available
                    service
                        .fetch_from_backend(page, 5, String::new(), None)
                        .await
                        .map(|(p, t)| (p, t, "backend"))
                }
            } else {
                // Hard sync or Background mode - fetch from backend
                if let Some(ref database) = *db_signal.read() {
                    service
                        .hot_sync_products(
                            database,
                            page,
                            5,
                            String::new(),
                            None,
                            true, // hard sync
                        )
                        .await
                } else {
                    service
                        .fetch_from_backend(page, 5, String::new(), None)
                        .await
                        .map(|(p, t)| (p, t, "backend"))
                }
            };

            let duration = start_time.elapsed();

            match result {
                Ok((prods, total, source)) => {
                    let pages = ((total as f32) / 5.0).ceil() as u32;
                    products.set(prods.clone());
                    filtered_products.set(prods);

                    let action = if hard_sync { "Hard Sync" } else { "Fetch Page" };

                    sync_events.write().insert(
                        0,
                        SyncEvent {
                            timestamp: chrono::Local::now(),
                            mode,
                            action: action.to_string(),
                            items_count: 5,
                            duration_ms: duration.as_millis() as u64,
                            success: true,
                            message: format!("Page {} of {} (from {})", page, pages, source),
                        },
                    );

                    // Update local stats
                    if let Some(ref database) = *db_signal.read() {
                        if let Ok(local_prods) = SyncService::get_local_products(database).await {
                            local_product_count.set(local_prods.len());
                        }
                    }
                }
                Err(e) => {
                    sync_events.write().insert(
                        0,
                        SyncEvent {
                            timestamp: chrono::Local::now(),
                            mode,
                            action: "Fetch Failed".to_string(),
                            items_count: 0,
                            duration_ms: duration.as_millis() as u64,
                            success: false,
                            message: e.to_string(),
                        },
                    );
                }
            }

            is_loading.set(false);
        });
    });

    // Sync all pages
    let sync_all = use_callback(move |()| {
        let mut products = products.clone();
        let mut sync_events = sync_events.clone();
        let db_signal = db.clone();

        spawn(async move {
            let service = SyncService::new(API_URL);
            let start_time = web_time::Instant::now();
            let mut all_products = Vec::new();

            for page in 1..=20 {
                match service
                    .fetch_from_backend(page, 5, String::new(), None)
                    .await
                {
                    Ok((mut prods, _)) => {
                        all_products.append(&mut prods);
                        sync_events.write().insert(
                            0,
                            SyncEvent {
                                timestamp: chrono::Local::now(),
                                mode: SyncMode::Hot,
                                action: format!("Sync Page {}", page),
                                items_count: prods.len(),
                                duration_ms: start_time.elapsed().as_millis() as u64,
                                success: true,
                                message: format!("Page {}/20 fetched", page),
                            },
                        );
                    }
                    Err(e) => {
                        sync_events.write().insert(
                            0,
                            SyncEvent {
                                timestamp: chrono::Local::now(),
                                mode: SyncMode::Hot,
                                action: format!("Page {} Failed", page),
                                items_count: 0,
                                duration_ms: start_time.elapsed().as_millis() as u64,
                                success: false,
                                message: e.to_string(),
                            },
                        );
                        break;
                    }
                }
                gloo_timers::future::TimeoutFuture::new(50).await;
            }

            let total_duration = start_time.elapsed();

            // Store in IndexedDB
            if let Some(ref database) = *db_signal.read() {
                match service.store_products_in_db(database, &all_products).await {
                    Ok(_) => {
                        sync_events.write().insert(
                            0,
                            SyncEvent {
                                timestamp: chrono::Local::now(),
                                mode: SyncMode::Hot,
                                action: "Full Sync Complete".to_string(),
                                items_count: all_products.len(),
                                duration_ms: total_duration.as_millis() as u64,
                                success: true,
                                message: format!(
                                    "All {} products stored locally",
                                    all_products.len()
                                ),
                            },
                        );

                        // Update stats
                        local_product_count.set(all_products.len());
                        let date = js_sys::Date::new_0();
                        last_sync_time.set(Some(format!(
                            "{}:{:02}:{:02}",
                            date.get_hours(),
                            date.get_minutes(),
                            date.get_seconds()
                        )));
                    }
                    Err(e) => {
                        sync_events.write().insert(
                            0,
                            SyncEvent {
                                timestamp: chrono::Local::now(),
                                mode: SyncMode::Hot,
                                action: "Store Failed".to_string(),
                                items_count: all_products.len(),
                                duration_ms: total_duration.as_millis() as u64,
                                success: false,
                                message: e.to_string(),
                            },
                        );
                    }
                }
            }

            products.set(all_products);
        });
    });

    // Clear local storage
    let clear_storage = use_callback(move |()| {
        let mut sync_events = sync_events.clone();
        let db_signal = db.clone();
        let mut local_count = local_product_count.clone();
        let mut last_sync = last_sync_time.clone();

        spawn(async move {
            if let Some(ref database) = *db_signal.read() {
                match SyncService::clear_local_storage(database).await {
                    Ok(_) => {
                        sync_events.write().insert(
                            0,
                            SyncEvent {
                                timestamp: chrono::Local::now(),
                                mode: SyncMode::Hot,
                                action: "Clear Storage".to_string(),
                                items_count: 0,
                                duration_ms: 0,
                                success: true,
                                message: "Local storage cleared".to_string(),
                            },
                        );
                        local_count.set(0);
                        last_sync.set(None);
                    }
                    Err(e) => {
                        sync_events.write().insert(
                            0,
                            SyncEvent {
                                timestamp: chrono::Local::now(),
                                mode: SyncMode::Hot,
                                action: "Clear Failed".to_string(),
                                items_count: 0,
                                duration_ms: 0,
                                success: false,
                                message: e.to_string(),
                            },
                        );
                    }
                }
            }
        });
    });

    // Search handler
    let on_search = move |query: String| {
        search_query.set(query.clone());

        let all = products.read().clone();
        if query.is_empty() {
            filtered_products.set(all);
        } else {
            let filtered: Vec<_> = all
                .into_iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&query.to_lowercase())
                        || p.description.to_lowercase().contains(&query.to_lowercase())
                        || p.category.to_lowercase().contains(&query.to_lowercase())
                        || p.brand.to_lowercase().contains(&query.to_lowercase())
                })
                .collect();
            filtered_products.set(filtered);
        }
    };

    // Category filter
    let on_category = move |cat: Option<String>| {
        selected_category.set(cat);
    };

    // Page change
    let on_page = move |page: u32| {
        current_page.set(page);
        load_page.call((false,)); // normal load (cache allowed)
    };

    rsx! {
        div { class: "sync-demo",
            header { class: "header",
                h1 { "🔄 Dioxus Storage Sync Demo" }
                p { "Demonstrating hot sync, background sync, and local IndexedDB storage" }
            }

            div { class: "control-panel",
                div { class: "mode-selector",
                    h3 { "Sync Mode" }
                    label { class: "radio-label",
                        input {
                            r#type: "radio",
                            name: "sync_mode",
                            checked: *sync_mode.read() == SyncMode::Hot,
                            onchange: move |_| sync_mode.set(SyncMode::Hot),
                        }
                        "🔥 Hot Sync (query cache + local first)"
                    }
                    label { class: "radio-label",
                        input {
                            r#type: "radio",
                            name: "sync_mode",
                            checked: *sync_mode.read() == SyncMode::Background,
                            onchange: move |_| sync_mode.set(SyncMode::Background),
                        }
                        "🌙 Backend Only (always fetch)"
                    }
                }

                div { class: "action-buttons",
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| load_page.call((false,)), // normal load (cache allowed)
                        disabled: *is_loading.read(),
                        if *is_loading.read() {
                            "⏳ Loading..."
                        } else {
                            "📥 Load Page (Cached)"
                        }
                    }
                    button {
                        class: "btn btn-warning",
                        onclick: move |_| load_page.call((true,)), // hard sync
                        disabled: *is_loading.read(),
                        "🔄 Hard Sync"
                    }
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| sync_all.call(()),
                        "📦 Sync All (100 items)"
                    }
                    button {
                        class: "btn btn-danger",
                        onclick: move |_| clear_storage.call(()),
                        "🗑️ Clear Local"
                    }
                }

                div { class: "storage-stats",
                    p { "📦 Local products: {local_product_count}" }
                    if let Some(time) = last_sync_time.read().as_ref() {
                        p { "🕐 Last sync: {time}" }
                    }
                }
            }

            div { class: "main-content",
                div { class: "products-panel",
                    FilterPanel {
                        search_query: search_query.read().clone(),
                        selected_category: selected_category.read().clone(),
                        on_search: on_search,
                        on_category_change: on_category,
                    }

                    div { class: "products-grid",
                        if *is_loading.read() {
                            div { class: "loading-spinner", "⏳ Loading products..." }
                        } else if filtered_products.read().is_empty() {
                            div { class: "empty-state", "📭 No products found. Click 'Sync All' to fetch data." }
                        } else {
                            for product in filtered_products.read().iter().cloned() {
                                ProductCard { product: product }
                            }
                        }
                    }

                    Pagination {
                        current_page: *current_page.read(),
                        total_pages: *total_pages.read(),
                        on_page_change: on_page,
                    }
                }

                div { class: "sync-panel",
                    SyncLogViewer { events: sync_events.read().clone() }
                }
            }
        }
    }
}
