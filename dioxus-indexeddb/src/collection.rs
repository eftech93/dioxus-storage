//! Typed collection for CRUD operations

use crate::cursor::Cursor;
use crate::error::{IndexedDbError, Result};
use crate::{from_js_value, to_js_value};
use idb::{CursorDirection, Database as IdbDatabase, Query as IdbQuery, TransactionMode};
use serde::{de::DeserializeOwned, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

/// A typed collection for storing and querying data
#[derive(Debug)]
pub struct Collection<T> {
    db: Rc<RefCell<IdbDatabase>>,
    name: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned + Clone> Collection<T> {
    /// Create a new collection
    pub fn new(db: Rc<RefCell<IdbDatabase>>, name: String) -> Self {
        Self {
            db,
            name,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Access the database for operations
    fn with_db<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&IdbDatabase) -> R,
    {
        f(&*self.db.borrow())
    }

    /// Get the database reference (for sync operations)
    pub fn db(&self) -> Rc<RefCell<IdbDatabase>> {
        self.db.clone()
    }

    /// Get the store name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the name of the underlying database
    pub fn database_name(&self) -> String {
        self.db.borrow().name()
    }

    /// Create a sibling collection on the same database for a different type
    pub fn sibling_collection<U: Serialize + DeserializeOwned + Clone>(
        &self,
        name: &str,
    ) -> Collection<U> {
        Collection::new(self.db.clone(), name.to_string())
    }

    /// Open a cursor for iterating over the collection
    ///
    /// # Example
    /// ```rust,ignore
    /// let mut cursor = collection.open_cursor(None, Some(CursorDirection::Next)).await?;
    /// while let Some(item) = cursor.next().await? {
    ///     println!("{:?}", item);
    /// }
    /// ```
    pub async fn open_cursor(
        &self,
        query: Option<IdbQuery>,
        direction: Option<CursorDirection>,
    ) -> Result<Cursor<T>> {
        let transaction = self.with_db(|db| {
            db.transaction(&[&self.name], TransactionMode::ReadOnly)
                .map_err(|e| IndexedDbError::Transaction(e.to_string()))
        })?;

        let store = transaction
            .object_store(&self.name)
            .map_err(|_| IndexedDbError::StoreNotFound(self.name.clone()))?;

        let request = store
            .open_cursor(query, direction)
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        let maybe_cursor = request
            .await
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        match maybe_cursor {
            Some(cursor) => Ok(Cursor::new(cursor, transaction)),
            None => Ok(Cursor::empty(transaction)),
        }
    }

    /// Open a cursor on an index
    ///
    /// # Example
    /// ```rust,ignore
    /// let mut cursor = collection
    ///     .open_cursor_on_index("email_idx", None, Some(CursorDirection::Next))
    ///     .await?;
    /// while let Some(item) = cursor.next().await? {
    ///     println!("{:?}", item);
    /// }
    /// ```
    pub async fn open_cursor_on_index(
        &self,
        index_name: &str,
        query: Option<IdbQuery>,
        direction: Option<CursorDirection>,
    ) -> Result<Cursor<T>> {
        let transaction = self.with_db(|db| {
            db.transaction(&[&self.name], TransactionMode::ReadOnly)
                .map_err(|e| IndexedDbError::Transaction(e.to_string()))
        })?;

        let store = transaction
            .object_store(&self.name)
            .map_err(|_| IndexedDbError::StoreNotFound(self.name.clone()))?;

        let index = store.index(index_name).map_err(|e| {
            IndexedDbError::Database(format!("Index '{}' not found: {:?}", index_name, e))
        })?;

        let request = index
            .open_cursor(query, direction)
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        let maybe_cursor = request
            .await
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        match maybe_cursor {
            Some(cursor) => Ok(Cursor::new(cursor, transaction)),
            None => Ok(Cursor::empty(transaction)),
        }
    }

    /// Get all items in the collection
    pub async fn get_all(&self) -> Result<Vec<T>> {
        let transaction = self.with_db(|db| {
            db.transaction(&[&self.name], TransactionMode::ReadOnly)
                .map_err(|e| IndexedDbError::Transaction(e.to_string()))
        })?;

        let store = transaction
            .object_store(&self.name)
            .map_err(|_| IndexedDbError::StoreNotFound(self.name.clone()))?;

        let request = store
            .get_all(None, None)
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        let result = request
            .await
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        let mut items = Vec::new();
        for i in 0..result.len() {
            if let Some(js_value) = result.get(i) {
                match from_js_value(&js_value) {
                    Ok(item) => items.push(item),
                    Err(e) => {
                        log::warn!("Failed to deserialize item at index {}: {}", i, e);
                    }
                }
            }
        }

        Ok(items)
    }

    /// Get an item by key
    pub async fn get(&self, key: &str) -> Result<Option<T>> {
        let transaction = self.with_db(|db| {
            db.transaction(&[&self.name], TransactionMode::ReadOnly)
                .map_err(|e| IndexedDbError::Transaction(e.to_string()))
        })?;

        let store = transaction
            .object_store(&self.name)
            .map_err(|_| IndexedDbError::StoreNotFound(self.name.clone()))?;

        let js_key = wasm_bindgen::JsValue::from_str(key);
        let request = store
            .get(js_key)
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        let result = request
            .await
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        match result {
            Some(js_value) => {
                let item = from_js_value(&js_value)?;
                Ok(Some(item))
            }
            None => Ok(None),
        }
    }

    /// Insert a new item (fails if key exists)
    pub async fn insert(&self, key: &str, item: &T) -> Result<()> {
        let transaction = self.with_db(|db| {
            db.transaction(&[&self.name], TransactionMode::ReadWrite)
                .map_err(|e| IndexedDbError::Transaction(e.to_string()))
        })?;

        let store = transaction
            .object_store(&self.name)
            .map_err(|_| IndexedDbError::StoreNotFound(self.name.clone()))?;

        // Check if key already exists
        let js_key = wasm_bindgen::JsValue::from_str(key);
        let check_request = store
            .get(js_key.clone())
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        if check_request
            .await
            .map_err(|e| IndexedDbError::Database(e.to_string()))?
            .is_some()
        {
            return Err(IndexedDbError::Constraint(format!(
                "Key '{}' already exists",
                key
            )));
        }

        let js_value = to_js_value(item)?;

        let request = store
            .add(&js_value, None)
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        request
            .await
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        transaction
            .commit()
            .map_err(|e| IndexedDbError::Transaction(e.to_string()))?
            .await
            .map_err(|e| IndexedDbError::Transaction(e.to_string()))?;

        Ok(())
    }

    /// Insert or update an item
    pub async fn put(&self, key: &str, item: &T) -> Result<()> {
        let transaction = self.with_db(|db| {
            db.transaction(&[&self.name], TransactionMode::ReadWrite)
                .map_err(|e| IndexedDbError::Transaction(e.to_string()))
        })?;

        let store = transaction
            .object_store(&self.name)
            .map_err(|_| IndexedDbError::StoreNotFound(self.name.clone()))?;

        let js_value = to_js_value(item)?;

        let request = store
            .put(&js_value, None)
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        request
            .await
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        transaction
            .commit()
            .map_err(|e| IndexedDbError::Transaction(e.to_string()))?
            .await
            .map_err(|e| IndexedDbError::Transaction(e.to_string()))?;

        Ok(())
    }

    /// Delete an item by key
    pub async fn delete(&self, key: &str) -> Result<()> {
        let transaction = self.with_db(|db| {
            db.transaction(&[&self.name], TransactionMode::ReadWrite)
                .map_err(|e| IndexedDbError::Transaction(e.to_string()))
        })?;

        let store = transaction
            .object_store(&self.name)
            .map_err(|_| IndexedDbError::StoreNotFound(self.name.clone()))?;

        let js_key = wasm_bindgen::JsValue::from_str(key);
        let request = store
            .delete(js_key)
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        request
            .await
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        transaction
            .commit()
            .map_err(|e| IndexedDbError::Transaction(e.to_string()))?
            .await
            .map_err(|e| IndexedDbError::Transaction(e.to_string()))?;

        Ok(())
    }

    /// Clear all items in the collection
    pub async fn clear(&self) -> Result<()> {
        let transaction = self.with_db(|db| {
            db.transaction(&[&self.name], TransactionMode::ReadWrite)
                .map_err(|e| IndexedDbError::Transaction(e.to_string()))
        })?;

        let store = transaction
            .object_store(&self.name)
            .map_err(|_| IndexedDbError::StoreNotFound(self.name.clone()))?;

        let request = store
            .clear()
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        request
            .await
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        transaction
            .commit()
            .map_err(|e| IndexedDbError::Transaction(e.to_string()))?
            .await
            .map_err(|e| IndexedDbError::Transaction(e.to_string()))?;

        Ok(())
    }

    /// Count items in the collection
    pub async fn count(&self) -> Result<u32> {
        let transaction = self.with_db(|db| {
            db.transaction(&[&self.name], TransactionMode::ReadOnly)
                .map_err(|e| IndexedDbError::Transaction(e.to_string()))
        })?;

        let store = transaction
            .object_store(&self.name)
            .map_err(|_| IndexedDbError::StoreNotFound(self.name.clone()))?;

        let request = store
            .count(None)
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        let count = request
            .await
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        Ok(count as u32)
    }

    /// Get items by index value
    ///
    /// # Example
    /// ```rust,ignore
    /// let users = collection.get_by_index("email_idx", "user@example.com").await?;
    /// ```
    pub async fn get_by_index(&self, index_name: &str, value: &str) -> Result<Vec<T>> {
        let transaction = self.with_db(|db| {
            db.transaction(&[&self.name], TransactionMode::ReadOnly)
                .map_err(|e| IndexedDbError::Transaction(e.to_string()))
        })?;

        let store = transaction
            .object_store(&self.name)
            .map_err(|_| IndexedDbError::StoreNotFound(self.name.clone()))?;

        let index = store.index(index_name).map_err(|e| {
            IndexedDbError::Database(format!("Index '{}' not found: {:?}", index_name, e))
        })?;

        let js_value = wasm_bindgen::JsValue::from_str(value);
        let query = IdbQuery::Key(js_value);
        let request = index
            .get_all(Some(query), None)
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        let result = request
            .await
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;

        let mut items = Vec::new();
        for i in 0..result.len() {
            if let Some(js_value) = result.get(i) {
                match from_js_value(&js_value) {
                    Ok(item) => items.push(item),
                    Err(e) => {
                        log::warn!("Failed to deserialize item at index {}: {}", i, e);
                    }
                }
            }
        }

        Ok(items)
    }

    /// Get a single item by unique index value
    ///
    /// Returns `None` if no item matches or if multiple items match
    /// (only use with unique indexes).
    ///
    /// # Example
    /// ```rust,ignore
    /// let user = collection.get_one_by_index("email_idx", "user@example.com").await?;
    /// ```
    pub async fn get_one_by_index(&self, index_name: &str, value: &str) -> Result<Option<T>> {
        let items = self.get_by_index(index_name, value).await?;
        if items.len() == 1 {
            Ok(Some(items.into_iter().next().unwrap()))
        } else if items.is_empty() {
            Ok(None)
        } else {
            Err(IndexedDbError::Constraint(format!(
                "Multiple items found for index '{}' with value '{}'",
                index_name, value
            )))
        }
    }

    /// Query the collection using a Query object
    ///
    /// This method respects the `use_index` setting in the Query.
    ///
    /// # Example
    /// ```rust,ignore
    /// let results = collection
    ///     .find(Query::new()
    ///         .use_index("age_idx")
    ///         .filter(Filter::gte("age", 18))
    ///         .order_by_desc("age")
    ///         .limit(10)
    ///     )
    ///     .await?;
    /// ```
    pub async fn find(&self, query: &crate::query::Query) -> Result<crate::query::QueryResult<T>> {
        // If an index is specified and we have a single equality filter on that field,
        // we can use the index directly
        if let Some(ref index_name) = query.index_name {
            // Check if we have a single equality filter that matches the index
            if query.filters.len() == 1 {
                if let Some(crate::query::Filter::Eq(field, value)) = query.filters.first() {
                    // Get the index key path to verify it matches
                    let items = self
                        .get_by_index(index_name, value.as_str().unwrap_or(""))
                        .await?;
                    let filtered = crate::query::execute_query(items, query);
                    return Ok(filtered);
                }
            }
        }

        // Fall back to scanning all items
        let items = self.get_all().await?;
        let result = crate::query::execute_query(items, query);
        Ok(result)
    }
}

impl<T> Clone for Collection<T> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            name: self.name.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}
