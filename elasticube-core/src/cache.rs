//! Query result caching for improved performance
//!
//! Implements an LRU (Least Recently Used) cache for query results to avoid
//! re-executing identical queries.

use crate::query::QueryResult;
use lru::LruCache;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

/// A query cache key based on the SQL query string
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct QueryCacheKey {
    /// The SQL query string (normalized)
    query: String,
}

impl QueryCacheKey {
    /// Create a new cache key from a query string
    pub fn new(query: impl Into<String>) -> Self {
        let query = query.into();
        // Normalize the query (trim whitespace, convert to lowercase)
        let normalized = query.trim().to_lowercase();
        Self {
            query: normalized,
        }
    }
}

/// Query result cache with LRU eviction policy
pub struct QueryCache {
    /// LRU cache storing query results
    cache: Arc<Mutex<LruCache<QueryCacheKey, QueryResult>>>,

    /// Number of cache hits
    hits: Arc<Mutex<usize>>,

    /// Number of cache misses
    misses: Arc<Mutex<usize>>,
}

impl QueryCache {
    /// Create a new query cache with the specified capacity
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of cached query results
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(100).unwrap());

        Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        }
    }

    /// Get a cached query result if it exists
    ///
    /// # Arguments
    /// * `key` - The query cache key
    ///
    /// # Returns
    /// Some(QueryResult) if the query is cached, None otherwise
    pub fn get(&self, key: &QueryCacheKey) -> Option<QueryResult> {
        let mut cache = self.cache.lock().unwrap();
        if let Some(result) = cache.get(key) {
            // Cache hit
            *self.hits.lock().unwrap() += 1;
            Some(result.clone())
        } else {
            // Cache miss
            *self.misses.lock().unwrap() += 1;
            None
        }
    }

    /// Insert a query result into the cache
    ///
    /// # Arguments
    /// * `key` - The query cache key
    /// * `result` - The query result to cache
    pub fn put(&self, key: QueryCacheKey, result: QueryResult) {
        let mut cache = self.cache.lock().unwrap();
        cache.put(key, result);
    }

    /// Clear all cached results
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        *self.hits.lock().unwrap() = 0;
        *self.misses.lock().unwrap() = 0;
    }

    /// Get the current cache size (number of entries)
    pub fn len(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = *self.hits.lock().unwrap();
        let misses = *self.misses.lock().unwrap();
        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        CacheStats {
            hits,
            misses,
            total_requests: total,
            hit_rate,
            entries: self.len(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: usize,

    /// Number of cache misses
    pub misses: usize,

    /// Total number of requests
    pub total_requests: usize,

    /// Cache hit rate (percentage)
    pub hit_rate: f64,

    /// Current number of cached entries
    pub entries: usize,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache Stats: {} hits, {} misses, {:.2}% hit rate, {} entries",
            self.hits, self.misses, self.hit_rate, self.entries
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_dummy_result() -> QueryResult {
        QueryResult::new_for_testing(Vec::new(), 0)
    }

    #[test]
    fn test_cache_key_normalization() {
        let key1 = QueryCacheKey::new("SELECT * FROM cube");
        let key2 = QueryCacheKey::new("  select * from cube  ");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_put_get() {
        let cache = QueryCache::new(10);
        let key = QueryCacheKey::new("SELECT * FROM cube");
        let result = create_dummy_result();

        cache.put(key.clone(), result.clone());

        let cached = cache.get(&key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().row_count(), result.row_count());
    }

    #[test]
    fn test_cache_miss() {
        let cache = QueryCache::new(10);
        let key = QueryCacheKey::new("SELECT * FROM cube");

        let cached = cache.get(&key);
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_eviction() {
        let cache = QueryCache::new(2);

        cache.put(QueryCacheKey::new("query1"), create_dummy_result());
        cache.put(QueryCacheKey::new("query2"), create_dummy_result());
        cache.put(QueryCacheKey::new("query3"), create_dummy_result());

        // query1 should have been evicted
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_cache_clear() {
        let cache = QueryCache::new(10);
        cache.put(QueryCacheKey::new("query1"), create_dummy_result());
        cache.put(QueryCacheKey::new("query2"), create_dummy_result());

        assert_eq!(cache.len(), 2);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_stats() {
        let cache = QueryCache::new(10);
        let key = QueryCacheKey::new("SELECT * FROM cube");

        cache.put(key.clone(), create_dummy_result());

        cache.get(&key); // Hit
        cache.get(&QueryCacheKey::new("nonexistent")); // Miss

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.hit_rate, 50.0);
    }
}
