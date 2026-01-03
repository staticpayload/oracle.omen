//! Deterministic query operations for memory store.

use crate::{document::DocumentValue, store::MemoryStore};
use std::collections::BTreeMap;

/// Query result
pub type QueryResult<T> = Result<T, QueryError>;

/// Query errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryError {
    /// Invalid query
    InvalidQuery(String),

    /// Key not found
    NotFound(String),

    /// Type mismatch
    TypeMismatch { expected: String, found: String },
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::InvalidQuery(msg) => write!(f, "Invalid query: {}", msg),
            QueryError::NotFound(key) => write!(f, "Key not found: {}", key),
            QueryError::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected {}, found {}", expected, found)
            }
        }
    }
}

impl std::error::Error for QueryError {}

/// Query builder for deterministic memory queries
pub struct QueryBuilder<'a> {
    store: &'a MemoryStore,
    filters: Vec<Filter>,
    order: QueryOrder,
    limit: Option<usize>,
}

impl<'a> QueryBuilder<'a> {
    /// Create a new query builder
    pub fn new(store: &'a MemoryStore) -> Self {
        Self {
            store,
            filters: Vec::new(),
            order: QueryOrder::Key,
            limit: None,
        }
    }

    /// Add a filter
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Set ordering
    pub fn order_by(mut self, order: QueryOrder) -> Self {
        self.order = order;
        self
    }

    /// Set limit
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Execute the query
    pub fn execute(self) -> QueryResult<Vec<QueryResultItem>> {
        let mut results: Vec<QueryResultItem> = self
            .store
            .keys()
            .into_iter()
            .filter_map(|key| {
                self.store
                    .read(&key)
                    .map(|doc| QueryResultItem::from_document(key, doc))
            })
            .filter(|item| self.matches_filters(item))
            .collect();

        // Apply ordering
        match &self.order {
            QueryOrder::Key => {
                results.sort_by(|a, b| a.key.cmp(&b.key));
            }
            QueryOrder::KeyDesc => {
                results.sort_by(|a, b| b.key.cmp(&a.key));
            }
            QueryOrder::Event => {
                results.sort_by(|a, b| a.event.cmp(&b.event));
            }
        }

        // Apply limit
        if let Some(n) = self.limit {
            results.truncate(n);
        }

        Ok(results)
    }

    /// Check if item matches all filters
    fn matches_filters(&self, item: &QueryResultItem) -> bool {
        self.filters.iter().all(|f| f.matches(item))
    }
}

/// Query result item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryResultItem {
    /// Document key
    pub key: String,

    /// Document value
    pub value: DocumentValue,

    /// Causal event
    pub event: u64,
}

impl QueryResultItem {
    /// Create from a document
    fn from_document(key: String, doc: &crate::document::Document) -> Self {
        Self {
            key,
            value: doc.value.clone(),
            event: doc.causal_event,
        }
    }
}

/// Query ordering
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryOrder {
    /// Order by key (ascending)
    Key,

    /// Order by key (descending)
    KeyDesc,

    /// Order by causal event
    Event,
}

/// Query filter
pub enum Filter {
    /// Key equals
    KeyEquals(String),

    /// Key prefix
    KeyPrefix(String),

    /// Value type
    TypeEquals(String),

    /// Custom predicate
    Custom(Box<dyn Fn(&QueryResultItem) -> bool>),
}

impl Clone for Filter {
    fn clone(&self) -> Self {
        match self {
            Filter::KeyEquals(s) => Filter::KeyEquals(s.clone()),
            Filter::KeyPrefix(s) => Filter::KeyPrefix(s.clone()),
            Filter::TypeEquals(s) => Filter::TypeEquals(s.clone()),
            // Custom filter can't be cloned, create a new one that always returns false
            Filter::Custom(_) => Filter::KeyEquals("__uncloneable_filter__".to_string()),
        }
    }
}

impl std::fmt::Debug for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::KeyEquals(s) => f.debug_tuple("KeyEquals").field(s).finish(),
            Filter::KeyPrefix(s) => f.debug_tuple("KeyPrefix").field(s).finish(),
            Filter::TypeEquals(s) => f.debug_tuple("TypeEquals").field(s).finish(),
            Filter::Custom(_) => f.debug_tuple("Custom").field(&"<function>").finish(),
        }
    }
}

impl Filter {
    /// Check if item matches filter
    fn matches(&self, item: &QueryResultItem) -> bool {
        match self {
            Filter::KeyEquals(key) => &item.key == key,
            Filter::KeyPrefix(prefix) => item.key.starts_with(prefix),
            Filter::TypeEquals(t) => item.value.type_name() == t,
            Filter::Custom(f) => f(item),
        }
    }
}

/// Deterministic retrieval operations
pub trait DeterministicQuery {
    /// Get all keys in deterministic order
    fn keys_sorted(&self) -> Vec<String>;

    /// Get all documents in deterministic order
    fn all_sorted(&self) -> Vec<(String, DocumentValue)>;
}

impl DeterministicQuery for MemoryStore {
    fn keys_sorted(&self) -> Vec<String> {
        self.keys()
    }

    fn all_sorted(&self) -> Vec<(String, DocumentValue)> {
        let mut items: Vec<_> = self
            .keys()
            .into_iter()
            .filter_map(|k| self.read(&k).map(|d| (k.clone(), d.value.clone())))
            .collect();
        items.sort_by(|a, b| a.0.cmp(&b.0));
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;

    #[test]
    fn test_deterministic_query() {
        let mut store = MemoryStore::new();
        store.write(Document::new("b", DocumentValue::Integer(2), 1));
        store.write(Document::new("a", DocumentValue::Integer(1), 1));
        store.write(Document::new("c", DocumentValue::Integer(3), 1));

        let keys = store.keys_sorted();
        assert_eq!(keys, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_query_builder() {
        let mut store = MemoryStore::new();
        store.write(Document::new("key1", DocumentValue::Integer(1), 1));
        store.write(Document::new("key2", DocumentValue::String("test".to_string()), 1));
        store.write(Document::new("prefix_a", DocumentValue::Integer(2), 1));
        store.write(Document::new("prefix_b", DocumentValue::Integer(3), 1));

        // Query for keys with prefix "prefix_"
        let results = QueryBuilder::new(&store)
            .filter(Filter::KeyPrefix("prefix_".to_string()))
            .execute()
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].key, "prefix_a");
        assert_eq!(results[1].key, "prefix_b");
    }

    #[test]
    fn test_query_filter_type() {
        let mut store = MemoryStore::new();
        store.write(Document::new("a", DocumentValue::Integer(1), 1));
        store.write(Document::new("b", DocumentValue::String("test".to_string()), 1));
        store.write(Document::new("c", DocumentValue::Integer(2), 1));

        let results = QueryBuilder::new(&store)
            .filter(Filter::TypeEquals("integer".to_string()))
            .execute()
            .unwrap();

        assert_eq!(results.len(), 2);
    }
}
