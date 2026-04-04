# Query Builder

The query builder provides a fluent API for filtering, sorting, and paginating data.

## Basic Query

```rust
use dioxus_indexeddb::prelude::*;

let query = Query::new();
```

## Filters

### Comparison Filters

```rust
use dioxus_indexeddb::prelude::*;

let query = Query::new()
    .filter(Filter::eq("status", "active"))           // Equal
    .filter(Filter::ne("status", "deleted"))          // Not equal
    .filter(Filter::gt("price", 100.0))               // Greater than
    .filter(Filter::gte("quantity", 10))              // Greater than or equal
    .filter(Filter::lt("price", 1000.0))              // Less than
    .filter(Filter::lte("age", 65));                  // Less than or equal
```

### String Filters

```rust
let query = Query::new()
    .filter(Filter::contains("name", "Pro"))          // Contains substring
    .filter(Filter::starts_with("email", "admin@"))   // Starts with
    .filter(Filter::ends_with("email", "@company.com")); // Ends with
```

### Combining Filters

```rust
let query = Query::new()
    .filter(Filter::eq("category", "electronics"))
    .filter(Filter::gt("price", 100.0))
    .filter(Filter::lt("price", 1000.0));
```

Filters are combined with AND logic by default.

## Sorting

```rust
use dioxus_indexeddb::prelude::*;

let query = Query::new()
    .order_by("created_at", Order::Desc)  // Newest first
    .order_by("name", Order::Asc);         // Then by name A-Z
```

## Pagination

```rust
use dioxus_indexeddb::prelude::*;

let query = Query::new()
    .limit(10)    // Items per page
    .offset(20);  // Skip first 20 (page 3)
```

## Complete Example

```rust
use dioxus::prelude::*;
use dioxus_indexeddb::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
    id: String,
    name: String,
    category: String,
    price: f64,
    in_stock: bool,
    rating: f64,
}

#[component]
fn ProductList() -> Element {
    let db = use_db(DatabaseConfig::new("shop", 1)
        .with_store("products", "id"));
    
    let products = use_collection::<Product>(db, "products");
    let mut page = use_signal(|| 1u32);
    const PER_PAGE: u32 = 10;
    
    // Build query based on current state
    let filtered = use_query(products, move |c| async move {
        c.query(
            Query::new()
                .filter(Filter::eq("in_stock", true))
                .filter(Filter::gt("rating", 4.0))
                .order_by("price", Order::Asc)
                .limit(PER_PAGE)
                .offset((page - 1) * PER_PAGE)
        ).await
    });
    
    rsx! {
        div { class: "product-list",
            // Pagination controls
            div { class: "pagination",
                button { onclick: move |_| page.set(*page - 1), "Previous" }
                span { "Page {page}" }
                button { onclick: move |_| page.set(*page + 1), "Next" }
            }
            
            // Products
            div { class: "products",
                for product in filtered.read().as_ref().unwrap_or(&vec![]).iter() {
                    div { class: "product-card",
                        h3 { "{product.name}" }
                        p { "${product.price}" }
                        p { "★ {product.rating}" }
                    }
                }
            }
        }
    }
}
```

## Query Performance Tips

1. **Use specific filters first** - More restrictive filters reduce data scanned
2. **Index commonly filtered fields** - Create indexes for query optimization
3. **Limit results** - Always use `.limit()` for large datasets
4. **Avoid full scans** - Structure queries to use indexes

## Programmatic Query Building

```rust
fn build_search_query(search: &str, category: Option<&str>, min_price: f64) -> Query {
    let mut query = Query::new();
    
    if !search.is_empty() {
        query = query.filter(Filter::contains("name", search));
    }
    
    if let Some(cat) = category {
        query = query.filter(Filter::eq("category", cat));
    }
    
    if min_price > 0.0 {
        query = query.filter(Filter::gte("price", min_price));
    }
    
    query.order_by("created_at", Order::Desc)
}
```
