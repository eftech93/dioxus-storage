use dioxus::prelude::*;

#[component]
pub fn Pagination(
    current_page: u32,
    total_pages: u32,
    on_page_change: EventHandler<u32>,
) -> Element {
    if total_pages <= 1 {
        return rsx! { div {} };
    }

    let mut pages_to_show = vec![];

    // Always show first page
    pages_to_show.push(1);

    // Show pages around current
    let start = (current_page.saturating_sub(2)).max(2);
    let end = (current_page + 2).min(total_pages - 1);

    if start > 2 {
        pages_to_show.push(0); // Ellipsis
    }

    for p in start..=end {
        pages_to_show.push(p);
    }

    if end < total_pages - 1 {
        pages_to_show.push(0); // Ellipsis
    }

    // Always show last page
    if total_pages > 1 {
        pages_to_show.push(total_pages);
    }

    rsx! {
        div { class: "pagination",
            // Previous button
            button {
                class: "page-btn prev",
                disabled: current_page == 1,
                onclick: move |_| on_page_change.call(current_page - 1),
                "← Previous"
            }

            // Page numbers
            div { class: "page-numbers",
                for page in pages_to_show {
                    if page == 0 {
                        span { class: "ellipsis", "..." }
                    } else {
                        button {
                            class: if page == current_page { "page-btn active" } else { "page-btn" },
                            onclick: move |_| on_page_change.call(page),
                            "{page}"
                        }
                    }
                }
            }

            // Next button
            button {
                class: "page-btn next",
                disabled: current_page >= total_pages,
                onclick: move |_| on_page_change.call(current_page + 1),
                "Next →"
            }
        }
    }
}
