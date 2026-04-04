# API Reference - Collections

## Collection

A type-safe wrapper around an IndexedDB object store.

```rust
use dioxus_indexeddb::prelude::*;

let users: Collection<User> = db.collection("users");
```

## Methods

### `get`

Get a single item by key.

```rust
let user: Option<User> = users.get("user-123").await?;
```

**Parameters:**
- `key: &str` - Item key

**Returns:** `Result<Option<T>>`

### `get_all`

Get all items in the collection.

```rust
let all_users: Vec<User> = users.get_all().await?;
```

**Returns:** `Result<Vec<T>>`

### `insert`

Insert a new item. Fails if key already exists.

```rust
users.insert("user-123", &user).await?;
```

**Parameters:**
- `key: &str` - Item key
- `item: &T` - Item to insert

**Returns:** `Result<()>`

### `put`

Insert or update an item.

```rust
users.put("user-123", &user).await?;
```

**Parameters:**
- `key: &str` - Item key
- `item: &T` - Item to store

**Returns:** `Result<()>`

### `delete`

Delete an item by key.

```rust
users.delete("user-123").await?;
```

**Parameters:**
- `key: &str` - Item key to delete

**Returns:** `Result<()>`

### `clear`

Delete all items in the collection.

```rust
users.clear().await?;
```

**Returns:** `Result<()>`

### `query`

Execute a filtered query.

```rust
let active_users = users.query(
    Query::new()
        .filter(Filter::eq("status", "active"))
        .order_by("name", Order::Asc)
).await?;
```

**Parameters:**
- `query: Query` - Query configuration

**Returns:** `Result<Vec<T>>`

### `count`

Count items matching a query.

```rust
let count = users.count(
    Query::new()
        .filter(Filter::eq("status", "active"))
).await?;
```

**Parameters:**
- `query: Query` - Query filters (optional)

**Returns:** `Result<usize>`

### `exists`

Check if an item exists.

```rust
let exists = users.exists("user-123").await?;
```

**Parameters:**
- `key: &str` - Item key

**Returns:** `Result<bool>`

## Example Usage

```rust
use dioxus::prelude::*;
use dioxus_indexeddb::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
    id: String,
    name: String,
    price: f64,
    category: String,
}

async fn manage_products(collection: Collection<Product>) -> Result<()> {
    // Add products
    collection.put("prod-1", &Product {
        id: "prod-1".to_string(),
        name: "Laptop".to_string(),
        price: 999.99,
        category: "electronics".to_string(),
    }).await?;
    
    collection.put("prod-2", &Product {
        id: "prod-2".to_string(),
        name: "Mouse".to_string(),
        price: 29.99,
        category: "electronics".to_string(),
    }).await?;
    
    // Query electronics
    let electronics = collection.query(
        Query::new()
            .filter(Filter::eq("category", "electronics"))
            .order_by("price", Order::Desc)
    ).await?;
    
    println!("Found {} electronics", electronics.len());
    
    // Get specific product
    if let Some(laptop) = collection.get("prod-1").await? {
        println!("Laptop price: ${}", laptop.price);
    }
    
    // Check if exists
    let has_mouse = collection.exists("prod-2").await?;
    println!("Has mouse: {}", has_mouse);
    
    // Count all products
    let total = collection.count(Query::new()).await?;
    println!("Total products: {}", total);
    
    // Delete a product
    collection.delete("prod-2").await?;
    
    Ok(())
}
```
