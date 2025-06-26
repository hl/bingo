//! LRU cache implementation for frequently accessed facts and tokens
//!
//! This module provides caching capabilities to improve performance when
//! the same facts or tokens are repeatedly accessed during rule evaluation.

use std::collections::HashMap;
use std::hash::Hash;

/// A simple LRU (Least Recently Used) cache implementation
///
/// This cache maintains items in order of access, evicting the least recently used
/// items when the cache reaches its capacity limit.
#[derive(Debug)]
pub struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, (V, usize)>, // Key -> (Value, Access order)
    access_counter: usize,
    min_access: usize,
}

impl<K, V> LruCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Create a new LRU cache with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::with_capacity(capacity),
            access_counter: 0,
            min_access: 0,
        }
    }

    /// Get a value from the cache, updating its access time
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some((value, access_time)) = self.map.get_mut(key) {
            self.access_counter += 1;
            *access_time = self.access_counter;
            Some(value)
        } else {
            None
        }
    }

    /// Insert a key-value pair into the cache
    ///
    /// If the cache is at capacity, the least recently used item will be evicted.
    pub fn put(&mut self, key: K, value: V) {
        // Don't insert anything if capacity is 0
        if self.capacity == 0 {
            return;
        }

        self.access_counter += 1;

        if self.map.len() >= self.capacity && !self.map.contains_key(&key) {
            self.evict_lru();
        }

        self.map.insert(key, (value, self.access_counter));
    }

    /// Remove a key from the cache and return the value if it exists
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key).map(|(value, _)| value)
    }

    /// Check if the cache contains a key
    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Get the current number of items in the cache
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clear all items from the cache
    pub fn clear(&mut self) {
        self.map.clear();
        self.access_counter = 0;
        self.min_access = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            capacity: self.capacity,
            size: self.map.len(),
            access_counter: self.access_counter,
        }
    }

    /// Evict the least recently used item from the cache
    fn evict_lru(&mut self) {
        if self.map.is_empty() {
            return;
        }

        // Find the key with the minimum access time
        let mut lru_key = None;
        let mut min_access_time = usize::MAX;

        for (key, (_, access_time)) in &self.map {
            if *access_time < min_access_time {
                min_access_time = *access_time;
                lru_key = Some(key.clone());
            }
        }

        if let Some(key) = lru_key {
            self.map.remove(&key);
            self.min_access = min_access_time;
        }
    }
}

/// Cache statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub capacity: usize,
    pub size: usize,
    pub access_counter: usize,
}

impl CacheStats {
    /// Calculate the cache utilization as a percentage
    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            (self.size as f64 / self.capacity as f64) * 100.0
        }
    }

    /// Estimate memory usage in bytes
    pub fn memory_usage_bytes(&self) -> usize {
        // Rough estimate: 64 bytes per cache entry
        self.size * 64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_basic_operations() {
        let mut cache = LruCache::new(3);

        // Test insertion and retrieval
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        assert_eq!(cache.len(), 3);
        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.get(&"b"), Some(&2));
        assert_eq!(cache.get(&"c"), Some(&3));
        assert_eq!(cache.get(&"d"), None);
    }

    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = LruCache::new(2);

        // Fill cache to capacity
        cache.put("a", 1);
        cache.put("b", 2);
        assert_eq!(cache.len(), 2);

        // Access "a" to make it more recently used
        cache.get(&"a");

        // Add new item - should evict "b" (least recently used)
        cache.put("c", 3);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&"a"), Some(&1)); // Still present
        assert_eq!(cache.get(&"b"), None); // Evicted
        assert_eq!(cache.get(&"c"), Some(&3)); // Newly added
    }

    #[test]
    fn test_lru_cache_update_existing() {
        let mut cache = LruCache::new(2);

        cache.put("a", 1);
        cache.put("b", 2);

        // Update existing key
        cache.put("a", 10);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&"a"), Some(&10));
        assert_eq!(cache.get(&"b"), Some(&2));
    }

    #[test]
    fn test_lru_cache_contains_key() {
        let mut cache = LruCache::new(2);

        cache.put("a", 1);
        assert!(cache.contains_key(&"a"));
        assert!(!cache.contains_key(&"b"));

        cache.put("b", 2);
        cache.put("c", 3); // Should evict "a"

        assert!(!cache.contains_key(&"a"));
        assert!(cache.contains_key(&"b"));
        assert!(cache.contains_key(&"c"));
    }

    #[test]
    fn test_lru_cache_clear() {
        let mut cache = LruCache::new(3);

        cache.put("a", 1);
        cache.put("b", 2);
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        assert_eq!(cache.get(&"a"), None);
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = LruCache::new(4);

        let stats = cache.stats();
        assert_eq!(stats.capacity, 4);
        assert_eq!(stats.size, 0);
        assert_eq!(stats.utilization(), 0.0);

        cache.put("a", 1);
        cache.put("b", 2);

        let stats = cache.stats();
        assert_eq!(stats.capacity, 4);
        assert_eq!(stats.size, 2);
        assert_eq!(stats.utilization(), 50.0);

        cache.put("c", 3);
        cache.put("d", 4);

        let stats = cache.stats();
        assert_eq!(stats.capacity, 4);
        assert_eq!(stats.size, 4);
        assert_eq!(stats.utilization(), 100.0);
    }

    #[test]
    fn test_lru_access_pattern() {
        let mut cache = LruCache::new(3);

        // Add items in order
        cache.put(1, "one");
        cache.put(2, "two");
        cache.put(3, "three");

        // Access item 1 to make it most recent
        cache.get(&1);

        // Add new item - should evict item 2 (oldest unaccessed)
        cache.put(4, "four");

        assert_eq!(cache.get(&1), Some(&"one")); // Still present (recently accessed)
        assert_eq!(cache.get(&2), None); // Evicted
        assert_eq!(cache.get(&3), Some(&"three")); // Still present
        assert_eq!(cache.get(&4), Some(&"four")); // Newly added
    }

    #[test]
    fn test_zero_capacity_cache() {
        let mut cache = LruCache::new(0);

        cache.put("a", 1);
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.get(&"a"), None);

        let stats = cache.stats();
        assert_eq!(stats.capacity, 0);
        assert_eq!(stats.utilization(), 0.0);
    }
}
