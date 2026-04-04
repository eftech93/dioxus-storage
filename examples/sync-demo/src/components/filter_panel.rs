use dioxus::prelude::*;

#[component]
pub fn FilterPanel(
    search_query: String,
    selected_category: Option<String>,
    on_search: EventHandler<String>,
    on_category_change: EventHandler<Option<String>>,
) -> Element {
    let categories = vec![
        "All",
        "Electronics",
        "Clothing",
        "Food",
        "Books",
        "Home",
        "Sports",
        "Toys",
        "Beauty",
    ];

    rsx! {
        div { class: "filter-panel",
            // Search input
            div { class: "filter-group",
                label { "🔍 Search" }
                input {
                    class: "search-input",
                    r#type: "text",
                    placeholder: "Search products...",
                    value: "{search_query}",
                    oninput: move |e| on_search.call(e.value()),
                }
            }

            // Category filter
            div { class: "filter-group",
                label { "📂 Category" }
                select {
                    class: "category-select",
                    onchange: move |e| {
                        let val = e.value();
                        on_category_change.call(if val == "All" { None } else { Some(val) });
                    },
                    for cat in categories {
                        option {
                            value: "{cat}",
                            selected: selected_category.as_deref() == Some(cat) || (cat == "All" && selected_category.is_none()),
                            "{cat}"
                        }
                    }
                }
            }
        }
    }
}
