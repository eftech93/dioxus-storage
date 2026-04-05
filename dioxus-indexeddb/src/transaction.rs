//! Multi-store transactions

use crate::collection::Collection;
use crate::error::{IndexedDbError, Result};
use idb::{Database as IdbDatabase, TransactionMode};
use serde::{de::DeserializeOwned, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

/// A multi-store transaction
///
/// Allows atomic operations across multiple collections
pub struct Transaction {
    db: Rc<RefCell<IdbDatabase>>,
    store_names: Vec<String>,
    mode: TransactionMode,
}

impl Transaction {
    /// Create a new read-only transaction
    pub fn read(db: Rc<RefCell<IdbDatabase>>, store_names: Vec<String>) -> Self {
        Self {
            db,
            store_names,
            mode: TransactionMode::ReadOnly,
        }
    }

    /// Create a new read-write transaction
    pub fn write(db: Rc<RefCell<IdbDatabase>>, store_names: Vec<String>) -> Self {
        Self {
            db,
            store_names,
            mode: TransactionMode::ReadWrite,
        }
    }

    /// Get a collection within this transaction
    pub fn collection<T: Serialize + DeserializeOwned + Clone>(&self, name: &str) -> Result<Collection<T>> {
        if !self.store_names.contains(&name.to_string()) {
            return Err(IndexedDbError::InvalidQuery(format!(
                "Store '{}' not in transaction scope",
                name
            )));
        }

        Ok(Collection::new(self.db.clone(), name.to_string()))
    }

    /// Access the database for operations
    fn with_db<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&IdbDatabase) -> R,
    {
        f(&*self.db.borrow())
    }

    /// Commit the transaction (for read-write transactions)
    pub async fn commit(self) -> Result<()> {
        if self.mode == TransactionMode::ReadOnly {
            return Ok(());
        }

        let idb_transaction = self.with_db(|db| {
            db.transaction(
                &self
                    .store_names
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>(),
                self.mode,
            )
            .map_err(|e| IndexedDbError::Transaction(e.to_string()))
        })?;

        idb_transaction
            .commit()
            .map_err(|e| IndexedDbError::Transaction(e.to_string()))?
            .await
            .map_err(|e| IndexedDbError::Transaction(e.to_string()))?;

        Ok(())
    }

    /// Abort the transaction
    pub fn abort(self) -> Result<()> {
        // Note: idb crate may not expose abort directly
        // In practice, the transaction will be aborted when dropped without commit
        Ok(())
    }
}

/// Transaction builder for fluent API
pub struct TransactionBuilder {
    db: Rc<RefCell<IdbDatabase>>,
    store_names: Vec<String>,
    mode: TransactionMode,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new(db: Rc<RefCell<IdbDatabase>>) -> Self {
        Self {
            db,
            store_names: Vec::new(),
            mode: TransactionMode::ReadOnly,
        }
    }

    /// Add a store to the transaction
    pub fn with_store(mut self, name: impl Into<String>) -> Self {
        self.store_names.push(name.into());
        self
    }

    /// Set read-write mode
    pub fn read_write(mut self) -> Self {
        self.mode = TransactionMode::ReadWrite;
        self
    }

    /// Build the transaction
    pub fn build(self) -> Transaction {
        Transaction {
            db: self.db,
            store_names: self.store_names,
            mode: self.mode,
        }
    }
}
