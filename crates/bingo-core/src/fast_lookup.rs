//! Fast fact lookup optimization for RETE network performance
//!
//! This module provides optimized fact lookup structures that replace
//! linear searches with hash-based O(1) lookups for better performance.

use crate::cache::{CacheStats, LruCache};
use crate::types::{Fact, FactId};
use std::collections::HashMap;

/// Fast fact lookup structure for O(1) fact access by ID
#[derive(Debug)]
pub struct FastFactLookup {
    /// Primary hash map for O(1) fact lookup by ID
    fact_map: HashMap<FactId, Fact>,
    /// LRU cache for frequently accessed facts
    access_cache: LruCache<FactId, Fact>,
    /// Statistics for monitoring performance
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub lookup_count: usize,
}

impl FastFactLookup {
    /// Create a new fast fact lookup with specified cache capacity
    pub fn new(cache_capacity: usize) -> Self {
        Self {
            fact_map: HashMap::new(),
            access_cache: LruCache::new(cache_capacity),
            cache_hits: 0,
            cache_misses: 0,
            lookup_count: 0,
        }
    }

    /// Create with pre-allocated capacity for better performance
    pub fn with_capacity(initial_capacity: usize, cache_capacity: usize) -> Self {
        Self {
            fact_map: HashMap::with_capacity(initial_capacity),
            access_cache: LruCache::new(cache_capacity),
            cache_hits: 0,
            cache_misses: 0,
            lookup_count: 0,
        }
    }

    /// Insert or update a fact (O(1) operation)
    pub fn insert(&mut self, fact: Fact) {
        let fact_id = fact.id;

        // Update cache if the fact is already cached
        if self.access_cache.contains_key(&fact_id) {
            self.access_cache.put(fact_id, fact.clone());
        }

        self.fact_map.insert(fact_id, fact);
    }

    /// Get a fact by ID with LRU caching (O(1) average case)
    /// Returns a cloned fact to avoid borrowing issues
    pub fn get(&mut self, fact_id: FactId) -> Option<Fact> {
        self.lookup_count += 1;

        // Check cache first
        if let Some(cached_fact) = self.access_cache.get(&fact_id) {
            self.cache_hits += 1;
            return Some(cached_fact.clone());
        }

        // Cache miss - lookup in main map
        if let Some(fact) = self.fact_map.get(&fact_id) {
            self.cache_misses += 1;
            // Add to cache for future access
            self.access_cache.put(fact_id, fact.clone());
            Some(fact.clone())
        } else {
            self.cache_misses += 1;
            None
        }
    }

    /// Get multiple facts by their IDs efficiently (batch operation)
    pub fn get_many(&mut self, fact_ids: &[FactId]) -> Vec<Fact> {
        let mut results = Vec::with_capacity(fact_ids.len());

        for &fact_id in fact_ids {
            if let Some(fact) = self.get(fact_id) {
                results.push(fact);
            }
        }

        results
    }

    /// Remove a fact from both map and cache
    pub fn remove(&mut self, fact_id: FactId) -> Option<Fact> {
        self.access_cache.remove(&fact_id);
        self.fact_map.remove(&fact_id)
    }

    /// Clear all facts and reset statistics
    pub fn clear(&mut self) {
        self.fact_map.clear();
        self.access_cache.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
        self.lookup_count = 0;
    }

    /// Get the total number of facts
    pub fn len(&self) -> usize {
        self.fact_map.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.fact_map.is_empty()
    }

    /// Get cache hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total as f64) * 100.0
        }
    }

    /// Get detailed performance statistics
    pub fn stats(&self) -> FastLookupStats {
        FastLookupStats {
            cache_stats: self.access_cache.stats(),
            hit_rate: self.hit_rate(),
            total_lookups: self.lookup_count,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            facts_stored: self.fact_map.len(),
        }
    }

    /// Extend with facts from a vector
    pub fn extend_from_vec(&mut self, facts: Vec<Fact>) {
        // Pre-allocate capacity if needed
        let additional_capacity = facts.len();
        if self.fact_map.capacity() < self.fact_map.len() + additional_capacity {
            self.fact_map.reserve(additional_capacity);
        }

        for fact in facts {
            self.insert(fact);
        }
    }

    /// Get all fact IDs currently stored
    pub fn fact_ids(&self) -> Vec<FactId> {
        self.fact_map.keys().cloned().collect()
    }

    /// Check if a fact exists without affecting cache statistics
    pub fn contains(&self, fact_id: FactId) -> bool {
        self.fact_map.contains_key(&fact_id)
    }
}

/// Performance statistics for fast fact lookup
#[derive(Debug, Clone)]
pub struct FastLookupStats {
    pub cache_stats: CacheStats,
    pub hit_rate: f64,
    pub total_lookups: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub facts_stored: usize,
}

impl FastLookupStats {
    /// Get the lookup efficiency as a percentage
    pub fn efficiency(&self) -> f64 {
        if self.total_lookups == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / self.total_lookups as f64) * 100.0
        }
    }

    /// Get average lookups per fact
    pub fn lookups_per_fact(&self) -> f64 {
        if self.facts_stored == 0 {
            0.0
        } else {
            self.total_lookups as f64 / self.facts_stored as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FactData, FactValue};
    use std::collections::HashMap;

    fn create_test_fact(id: FactId, field_name: &str, field_value: FactValue) -> Fact {
        let mut fields = HashMap::new();
        fields.insert(field_name.to_string(), field_value);

        Fact { id, external_id: None, timestamp: chrono::Utc::now(), data: FactData { fields } }
    }

    #[test]
    fn test_fast_lookup_basic_operations() {
        let mut lookup = FastFactLookup::new(10);

        // Test insertion and retrieval
        let fact1 = create_test_fact(1, "name", FactValue::String("Alice".to_string()));
        let fact2 = create_test_fact(2, "age", FactValue::Integer(30));

        lookup.insert(fact1.clone());
        lookup.insert(fact2.clone());

        assert_eq!(lookup.len(), 2);
        assert!(!lookup.is_empty());

        // Test retrieval
        let retrieved = lookup.get(1).unwrap();
        assert_eq!(retrieved.id, 1);

        let retrieved2 = lookup.get(2).unwrap();
        assert_eq!(retrieved2.id, 2);

        // Test non-existent fact
        assert!(lookup.get(999).is_none());
    }

    #[test]
    fn test_cache_performance() {
        let mut lookup = FastFactLookup::new(5);

        // Insert facts
        for i in 1..=10 {
            let fact = create_test_fact(i, "value", FactValue::Integer(i as i64));
            lookup.insert(fact);
        }

        // Access some facts multiple times
        for _ in 0..3 {
            lookup.get(1);
            lookup.get(2);
            lookup.get(3);
        }

        let stats = lookup.stats();

        // First access should be cache misses, subsequent should be hits
        assert!(stats.cache_hits > 0);
        assert!(stats.cache_misses > 0);
        assert!(stats.hit_rate > 0.0);
        assert_eq!(stats.total_lookups, 9); // 3 facts × 3 accesses each
    }

    #[test]
    fn test_batch_operations() {
        let mut lookup = FastFactLookup::new(20);

        // Create batch of facts
        let facts: Vec<Fact> = (1..=100)
            .map(|i| create_test_fact(i, "index", FactValue::Integer(i as i64)))
            .collect();

        // Test batch insertion
        lookup.extend_from_vec(facts);
        assert_eq!(lookup.len(), 100);

        // Test batch retrieval
        let fact_ids: Vec<FactId> = (1..=10).collect();
        let retrieved = lookup.get_many(&fact_ids);
        assert_eq!(retrieved.len(), 10);

        // Verify all retrieved facts have correct IDs
        for (i, fact) in retrieved.iter().enumerate() {
            assert_eq!(fact.id, (i + 1) as FactId);
        }
    }

    #[test]
    fn test_removal_and_clear() {
        let mut lookup = FastFactLookup::new(10);

        let fact = create_test_fact(42, "test", FactValue::Boolean(true));
        lookup.insert(fact.clone());

        assert!(lookup.contains(42));

        let removed = lookup.remove(42);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, 42);
        assert!(!lookup.contains(42));

        // Test clear
        lookup.insert(fact);
        assert!(!lookup.is_empty());
        lookup.clear();
        assert!(lookup.is_empty());
        assert_eq!(lookup.stats().total_lookups, 0);
    }

    #[test]
    fn test_statistics_accuracy() {
        let mut lookup = FastFactLookup::new(3);

        // Insert facts
        for i in 1..=5 {
            let fact = create_test_fact(i, "num", FactValue::Integer(i as i64));
            lookup.insert(fact);
        }

        // Perform lookups to generate statistics
        lookup.get(1); // Cache miss
        lookup.get(1); // Cache hit
        lookup.get(2); // Cache miss
        lookup.get(2); // Cache hit
        lookup.get(999); // Miss (not found)

        let stats = lookup.stats();
        assert_eq!(stats.total_lookups, 5);
        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.cache_misses, 3);
        assert_eq!(stats.facts_stored, 5);
        assert_eq!(stats.hit_rate, 40.0); // 2/5 = 40%
    }
}
