use crate::models::Product;
use dioxus::prelude::*;

#[component]
pub fn ProductCard(product: Product) -> Element {
    rsx! {
        div { class: "product-card",
            div { class: "product-header",
                h4 { "{product.name}" }
                span { class: "product-id", "#{product.id}" }
            }

            div { class: "product-meta",
                span { class: "category-badge", "{product.category}" }
                span { class: "brand-badge", "{product.brand}" }
            }

            p { class: "product-description", "{product.description}" }

            div { class: "product-details",
                div { class: "detail-item",
                    label { "Price:" }
                    span { class: "price", "${product.price:.2}" }
                }
                div { class: "detail-item",
                    label { "Stock:" }
                    span { class: if product.stock > 10 { "stock-high" } else { "stock-low" },
                        "{product.stock} units"
                    }
                }
                div { class: "detail-item",
                    label { "Rating:" }
                    span { class: "rating",
                        for _ in 0..product.rating as i32 {
                            "⭐"
                        }
                        " ({product.rating})"
                    }
                }
            }

            div { class: "product-footer",
                span { class: "color-badge", style: "background-color: {product.color.to_lowercase()}",
                    "{product.color}"
                }
                if product.in_stock {
                    span { class: "in-stock", "✓ In Stock" }
                } else {
                    span { class: "out-of-stock", "✗ Out of Stock" }
                }
            }
        }
    }
}
