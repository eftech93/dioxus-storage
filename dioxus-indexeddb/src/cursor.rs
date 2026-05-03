//! Cursor-based iteration for large datasets
//!
//! Provides efficient traversal of IndexedDB records without loading everything
//! into memory.
//!
//! # Example
//! ```rust,ignore
//! use dioxus_indexeddb::prelude::*;
//!
//! // Iterate all records
//! let mut cursor = collection.open_cursor(None, Some(CursorDirection::Next)).await?;
//! while let Some(item) = cursor.next().await? {
//!     println!("{:?}", item);
//! }
//!
//! // Iterate as a Stream
//! let stream = collection.open_cursor(None, Some(CursorDirection::Next)).await?.into_stream();
//! while let Some(result) = stream.next().await {
//!     match result {
//!         Ok(item) => println!("{:?}", item),
//!         Err(e) => log::error!("Cursor error: {}", e),
//!     }
//! }
//! ```

use crate::error::{IndexedDbError, Result};
use crate::from_js_value;
use idb::{ManagedCursor, Query as IdbQuery, Transaction};
use serde::de::DeserializeOwned;

/// A typed cursor for iterating over IndexedDB records efficiently.
///
/// Cursors are useful for traversing large datasets without loading everything
/// into memory at once. The cursor holds the underlying transaction alive
/// until it is dropped.
pub struct Cursor<T> {
    inner: Option<ManagedCursor>,
    _transaction: Transaction,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: DeserializeOwned> Cursor<T> {
    pub(crate) fn new(cursor: idb::Cursor, transaction: Transaction) -> Self {
        Self {
            inner: Some(cursor.into_managed()),
            _transaction: transaction,
            _phantom: std::marker::PhantomData,
        }
    }

    pub(crate) fn empty(transaction: Transaction) -> Self {
        Self {
            inner: None,
            _transaction: transaction,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Advance the cursor and return the next item.
    ///
    /// Returns `Ok(None)` when the cursor has reached the end.
    pub async fn next(&mut self) -> Result<Option<T>> {
        let inner = match self.inner.as_mut() {
            Some(inner) => inner,
            None => return Ok(None),
        };

        let js_value = inner
            .value()
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;
        let item = match js_value {
            Some(js) => Some(from_js_value(&js)?),
            None => {
                self.inner = None;
                return Ok(None);
            }
        };

        // Advance for the next call.
        match inner.next(None).await {
            Ok(()) => {}
            Err(idb::Error::CursorFinished) => {
                self.inner = None;
            }
            Err(e) => return Err(IndexedDbError::Database(e.to_string())),
        }

        Ok(item)
    }

    /// Advance the cursor by `count` records and return the item at the new position.
    ///
    /// Returns `Ok(None)` if the cursor is finished.
    pub async fn advance(&mut self, count: u32) -> Result<Option<T>> {
        let inner = match self.inner.as_mut() {
            Some(inner) => inner,
            None => return Ok(None),
        };

        match inner.advance(count).await {
            Ok(()) => {}
            Err(idb::Error::CursorFinished) => {
                self.inner = None;
                return Ok(None);
            }
            Err(e) => return Err(IndexedDbError::Database(e.to_string())),
        }

        let js_value = inner
            .value()
            .map_err(|e| IndexedDbError::Database(e.to_string()))?;
        js_value.map(|js| from_js_value(&js)).transpose()
    }

    /// Convert this cursor into a [`futures::Stream`].
    ///
    /// Each item yielded by the stream is a `Result<T>`.
    pub fn into_stream(self) -> impl futures::Stream<Item = Result<T>> {
        futures::stream::unfold(self, |mut cursor| async move {
            match cursor.next().await {
                Ok(Some(item)) => Some((Ok(item), cursor)),
                Ok(None) => None,
                Err(e) => Some((Err(e), cursor)),
            }
        })
    }
}

/// Range bound for cursor queries.
///
/// Provides a convenient way to specify key ranges for cursor iteration.
#[derive(Debug, Clone)]
pub enum CursorBound {
    /// Only keys equal to the given value.
    Only(String),
    /// Keys greater than or equal to the lower bound.
    ///
    /// If `open` is `true`, the lower bound itself is excluded.
    LowerBound(String, bool),
    /// Keys less than or equal to the upper bound.
    ///
    /// If `open` is `true`, the upper bound itself is excluded.
    UpperBound(String, bool),
    /// Keys within a range.
    Range {
        /// Lower bound value.
        lower: String,
        /// Upper bound value.
        upper: String,
        /// Is lower bound exclusive?
        lower_open: bool,
        /// Is upper bound exclusive?
        upper_open: bool,
    },
}

impl CursorBound {
    /// Convert this bound into an `idb::Query`.
    pub fn to_query(&self) -> Result<IdbQuery> {
        match self {
            CursorBound::Only(key) => {
                let js = wasm_bindgen::JsValue::from_str(key);
                Ok(IdbQuery::Key(js))
            }
            CursorBound::LowerBound(key, open) => {
                let js = wasm_bindgen::JsValue::from_str(key);
                let range = idb::KeyRange::lower_bound(&js, Some(*open))
                    .map_err(|e| IndexedDbError::Database(e.to_string()))?;
                Ok(IdbQuery::KeyRange(range))
            }
            CursorBound::UpperBound(key, open) => {
                let js = wasm_bindgen::JsValue::from_str(key);
                let range = idb::KeyRange::upper_bound(&js, Some(*open))
                    .map_err(|e| IndexedDbError::Database(e.to_string()))?;
                Ok(IdbQuery::KeyRange(range))
            }
            CursorBound::Range {
                lower,
                upper,
                lower_open,
                upper_open,
            } => {
                let lower_js = wasm_bindgen::JsValue::from_str(lower);
                let upper_js = wasm_bindgen::JsValue::from_str(upper);
                let range = idb::KeyRange::bound(
                    &lower_js,
                    &upper_js,
                    Some(*lower_open),
                    Some(*upper_open),
                )
                .map_err(|e| IndexedDbError::Database(e.to_string()))?;
                Ok(IdbQuery::KeyRange(range))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_cursor_bound_only() {
        let bound = CursorBound::Only("test-key".to_string());
        assert!(matches!(bound, CursorBound::Only(ref s) if s == "test-key"));
    }

    #[wasm_bindgen_test]
    fn test_cursor_bound_range() {
        let bound = CursorBound::Range {
            lower: "a".to_string(),
            upper: "z".to_string(),
            lower_open: false,
            upper_open: true,
        };
        assert!(matches!(
            bound,
            CursorBound::Range {
                lower: ref l,
                upper: ref u,
                lower_open: false,
                upper_open: true,
            } if l == "a" && u == "z"
        ));
    }

    #[wasm_bindgen_test]
    fn test_cursor_bound_lower() {
        let bound = CursorBound::LowerBound("start".to_string(), true);
        assert!(matches!(bound, CursorBound::LowerBound(ref s, true) if s == "start"));
    }

    #[wasm_bindgen_test]
    fn test_cursor_bound_upper() {
        let bound = CursorBound::UpperBound("end".to_string(), false);
        assert!(matches!(bound, CursorBound::UpperBound(ref s, false) if s == "end"));
    }
}
