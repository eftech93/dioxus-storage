//! Typed collection for CRUD operations

use crate::error::{IndexedDbError, Result};
use crate::{from_js_value, to_js_value};
use idb::{Database as IdbDatabase, TransactionMode};
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

impl<T: Serialize + DeserializeOwned> Collection<T> {
    /// Create a new collection
    pub(crate) fn new(db: Rc<RefCell<IdbDatabase>>, name: String) -> Self {
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
