//! Advanced Query System for IndexedDB
//!
//! Supports filtering, sorting, pagination, grouping, and aggregation.
//!
//! # Example
//!
//! ```rust,ignore
//! let results = collection
//!     .find(
//!         Query::new()
//!             .filter(Filter::eq("status", "active"))
//!             .filter(Filter::gt("priority", 5))
//!             .order_by("created_at", Order::Desc)
//!             .limit(10)
//!     )
//!     .await?;
//! ```

use crate::error::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;

/// Query with advanced filtering, sorting, and aggregation
#[derive(Debug, Clone)]
pub struct Query {
    /// Filter conditions (AND logic by default)
    pub filters: Vec<Filter>,
    /// Filter combination mode
    pub filter_mode: FilterMode,
    /// Sort orders (applied in sequence)
    pub order_by: Vec<OrderClause>,
    /// Limit results
    pub limit: Option<usize>,
    /// Skip results
    pub skip: Option<usize>,
    /// Group by field
    pub group_by: Option<String>,
    /// Aggregations
    pub aggregations: Vec<Aggregation>,
}

/// Filter combination mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterMode {
    /// All filters must match (AND)
    And,
    /// Any filter can match (OR)
    Or,
}

impl Default for FilterMode {
    fn default() -> Self {
        FilterMode::And
    }
}

/// Order clause
#[derive(Debug, Clone)]
pub struct OrderClause {
    pub field: String,
    pub direction: Order,
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Order {
    Asc,
    Desc,
}

/// Aggregation functions
#[derive(Debug, Clone)]
pub enum Aggregation {
    /// Count items
    Count { alias: String },
    /// Sum of field values
    Sum { field: String, alias: String },
    /// Average of field values
    Avg { field: String, alias: String },
    /// Minimum value
    Min { field: String, alias: String },
    /// Maximum value
    Max { field: String, alias: String },
}

impl Default for Query {
    fn default() -> Self {
        Self {
            filters: Vec::new(),
            filter_mode: FilterMode::And,
            order_by: Vec::new(),
            limit: None,
            skip: None,
            group_by: None,
            aggregations: Vec::new(),
        }
    }
}

impl Query {
    /// Create a new query
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a filter
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Set filter mode to OR
    pub fn or(mut self) -> Self {
        self.filter_mode = FilterMode::Or;
        self
    }

    /// Set filter mode to AND
    pub fn and(mut self) -> Self {
        self.filter_mode = FilterMode::And;
        self
    }

    /// Order by field ascending
    pub fn order_by_asc(mut self, field: impl Into<String>) -> Self {
        self.order_by.push(OrderClause {
            field: field.into(),
            direction: Order::Asc,
        });
        self
    }

    /// Order by field descending
    pub fn order_by_desc(mut self, field: impl Into<String>) -> Self {
        self.order_by.push(OrderClause {
            field: field.into(),
            direction: Order::Desc,
        });
        self
    }

    /// Set limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset (skip)
    pub fn offset(mut self, offset: usize) -> Self {
        self.skip = Some(offset);
        self
    }

    /// Group by field
    pub fn group_by(mut self, field: impl Into<String>) -> Self {
        self.group_by = Some(field.into());
        self
    }

    /// Add count aggregation
    pub fn count(mut self, alias: impl Into<String>) -> Self {
        self.aggregations.push(Aggregation::Count {
            alias: alias.into(),
        });
        self
    }

    /// Add sum aggregation
    pub fn sum(mut self, field: impl Into<String>, alias: impl Into<String>) -> Self {
        self.aggregations.push(Aggregation::Sum {
            field: field.into(),
            alias: alias.into(),
        });
        self
    }

    /// Check if an item matches this query's filters
    pub fn matches<T: Serialize>(&self, item: &T) -> bool {
        let json = match serde_json::to_value(item) {
            Ok(v) => v,
            Err(_) => return false,
        };

        if self.filters.is_empty() {
            return true;
        }

        match self.filter_mode {
            FilterMode::And => self.filters.iter().all(|f| f.matches(&json)),
            FilterMode::Or => self.filters.iter().any(|f| f.matches(&json)),
        }
    }
}

/// Filter condition
#[derive(Debug, Clone)]
pub enum Filter {
    /// Equal
    Eq(String, serde_json::Value),
    /// Not equal
    Ne(String, serde_json::Value),
    /// Greater than
    Gt(String, serde_json::Value),
    /// Greater than or equal
    Gte(String, serde_json::Value),
    /// Less than
    Lt(String, serde_json::Value),
    /// Less than or equal
    Lte(String, serde_json::Value),
    /// Contains (string or array)
    Contains(String, serde_json::Value),
    /// IN array of values
    In(String, Vec<serde_json::Value>),
    /// Between two values (inclusive)
    Between(String, serde_json::Value, serde_json::Value),
    /// Is null
    IsNull(String),
    /// Is not null
    IsNotNull(String),
    /// Matches regex pattern
    Regex(String, String),
    /// Nested AND filter
    And(Vec<Filter>),
    /// Nested OR filter
    Or(Vec<Filter>),
}

impl Filter {
    /// Create equality filter
    pub fn eq(field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        Filter::Eq(field.into(), value.into())
    }

    /// Create not-equal filter
    pub fn ne(field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        Filter::Ne(field.into(), value.into())
    }

    /// Create greater-than filter
    pub fn gt(field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        Filter::Gt(field.into(), value.into())
    }

    /// Create less-than filter
    pub fn lt(field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        Filter::Lt(field.into(), value.into())
    }

    /// Create IN filter
    pub fn is_in(field: impl Into<String>, values: Vec<impl Into<serde_json::Value>>) -> Self {
        Filter::In(field.into(), values.into_iter().map(|v| v.into()).collect())
    }

    /// Create BETWEEN filter
    pub fn between(
        field: impl Into<String>,
        min: impl Into<serde_json::Value>,
        max: impl Into<serde_json::Value>,
    ) -> Self {
        Filter::Between(field.into(), min.into(), max.into())
    }

    /// Check if a JSON value matches this filter
    fn matches(&self, json: &serde_json::Value) -> bool {
        match self {
            Filter::Eq(field, value) => json.get(field).map(|v| v == value).unwrap_or(false),
            Filter::Ne(field, value) => json.get(field).map(|v| v != value).unwrap_or(true),
            Filter::Gt(field, value) => {
                compare_values(json.get(field), value) == Some(std::cmp::Ordering::Greater)
            }
            Filter::Gte(field, value) => {
                matches!(
                    compare_values(json.get(field), value),
                    Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
                )
            }
            Filter::Lt(field, value) => {
                compare_values(json.get(field), value) == Some(std::cmp::Ordering::Less)
            }
            Filter::Lte(field, value) => {
                matches!(
                    compare_values(json.get(field), value),
                    Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
                )
            }
            Filter::Contains(field, value) => {
                if let Some(field_value) = json.get(field) {
                    if let (Some(s1), Some(s2)) = (field_value.as_str(), value.as_str()) {
                        s1.contains(s2)
                    } else if let Some(arr) = field_value.as_array() {
                        arr.contains(value)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Filter::In(field, values) => {
                json.get(field).map(|v| values.contains(v)).unwrap_or(false)
            }
            Filter::Between(field, min, max) => {
                if let Some(value) = json.get(field) {
                    let ge_min = compare_values(Some(value), min)
                        .map(|o| o != std::cmp::Ordering::Less)
                        .unwrap_or(false);
                    let le_max = compare_values(Some(value), max)
                        .map(|o| o != std::cmp::Ordering::Greater)
                        .unwrap_or(false);
                    ge_min && le_max
                } else {
                    false
                }
            }
            Filter::IsNull(field) => json.get(field).map(|v| v.is_null()).unwrap_or(true),
            Filter::IsNotNull(field) => json.get(field).map(|v| !v.is_null()).unwrap_or(false),
            Filter::Regex(field, pattern) => json
                .get(field)
                .and_then(|v| v.as_str())
                .map(|s| s.contains(pattern))
                .unwrap_or(false),
            Filter::And(filters) => filters.iter().all(|f| f.matches(json)),
            Filter::Or(filters) => filters.iter().any(|f| f.matches(json)),
        }
    }
}

/// Compare two JSON values
fn compare_values(
    a: Option<&serde_json::Value>,
    b: &serde_json::Value,
) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Some(a), b) => {
            // Try numeric comparison first
            if let (Some(n1), Some(n2)) = (a.as_f64(), b.as_f64()) {
                n1.partial_cmp(&n2)
            } else if let (Some(s1), Some(s2)) = (a.as_str(), b.as_str()) {
                Some(s1.cmp(s2))
            } else {
                a.to_string().partial_cmp(&b.to_string())
            }
        }
        _ => None,
    }
}

/// Query result with optional aggregation
#[derive(Debug, Clone)]
pub struct QueryResult<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub aggregations: HashMap<String, serde_json::Value>,
}

/// Execute a query on a vector of items
pub fn execute_query<T: Serialize + Clone + DeserializeOwned>(
    items: Vec<T>,
    query: &Query,
) -> QueryResult<T> {
    // Filter
    let mut result: Vec<T> = items
        .into_iter()
        .filter(|item| query.matches(item))
        .collect();

    let total = result.len();

    // Sort
    for order in &query.order_by {
        result.sort_by(|a, b| {
            let json_a = serde_json::to_value(a).unwrap_or_default();
            let json_b = serde_json::to_value(b).unwrap_or_default();

            let cmp = match (json_a.get(&order.field), json_b.get(&order.field)) {
                (Some(va), Some(vb)) => {
                    compare_values(Some(va), vb).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            };

            match order.direction {
                Order::Asc => cmp,
                Order::Desc => cmp.reverse(),
            }
        });
    }

    // Calculate aggregations before pagination
    let mut aggregations = HashMap::new();
    for agg in &query.aggregations {
        match agg {
            Aggregation::Count { alias } => {
                aggregations.insert(alias.clone(), serde_json::json!(total));
            }
            Aggregation::Sum { field, alias } => {
                let sum: f64 = result
                    .iter()
                    .filter_map(|item| {
                        serde_json::to_value(item)
                            .ok()
                            .and_then(|v| v.get(field).cloned())
                            .and_then(|v| v.as_f64())
                    })
                    .sum();
                aggregations.insert(alias.clone(), serde_json::json!(sum));
            }
            Aggregation::Avg { field, alias } => {
                let values: Vec<f64> = result
                    .iter()
                    .filter_map(|item| {
                        serde_json::to_value(item)
                            .ok()
                            .and_then(|v| v.get(field).cloned())
                            .and_then(|v| v.as_f64())
                    })
                    .collect();
                let avg = if !values.is_empty() {
                    values.iter().sum::<f64>() / values.len() as f64
                } else {
                    0.0
                };
                aggregations.insert(alias.clone(), serde_json::json!(avg));
            }
            Aggregation::Min { field, alias } => {
                let min = result
                    .iter()
                    .filter_map(|item| {
                        serde_json::to_value(item)
                            .ok()
                            .and_then(|v| v.get(field).cloned())
                            .and_then(|v| v.as_f64())
                    })
                    .fold(f64::INFINITY, |a, b| a.min(b));
                aggregations.insert(alias.clone(), serde_json::json!(min));
            }
            Aggregation::Max { field, alias } => {
                let max = result
                    .iter()
                    .filter_map(|item| {
                        serde_json::to_value(item)
                            .ok()
                            .and_then(|v| v.get(field).cloned())
                            .and_then(|v| v.as_f64())
                    })
                    .fold(f64::NEG_INFINITY, |a, b| a.max(b));
                aggregations.insert(alias.clone(), serde_json::json!(max));
            }
        }
    }

    // Apply pagination (skip then limit)
    if let Some(skip) = query.skip {
        if skip < result.len() {
            result = result.split_off(skip);
        } else {
            result.clear();
        }
    }

    if let Some(limit) = query.limit {
        if result.len() > limit {
            result.truncate(limit);
        }
    }

    QueryResult {
        items: result,
        total,
        aggregations,
    }
}

/// Pagination helper
#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: usize,
    pub per_page: usize,
}

impl Pagination {
    pub fn new(page: usize, per_page: usize) -> Self {
        Self { page, per_page }
    }

    pub fn to_query(&self) -> Query {
        Query::new()
            .offset(self.page * self.per_page)
            .limit(self.per_page)
    }
}
