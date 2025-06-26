//! Unified fact storage system that consolidates FastFactLookup and CachedFactStore
//!
//! This module eliminates the duplication between FastFactLookup and CachedFactStore
//! by providing a single optimized implementation that supports both use cases.

use crate::bloom_filter::{FactBloomFilter, FactBloomStats};
use crate::cache::{CacheStats, LruCache};
use crate::fact_store::FactStore;
use crate::field_indexing::{FieldIndexStats, FieldIndexer};
use crate::types::{Fact, FactId, FactValue};
use crate::unified_memory_coordinator::MemoryConsumer;
use crate::unified_statistics::{UnifiedStats, UnifiedStatsBuilder};
use std::collections::HashMap;

/// Unified fact storage with configurable backends and intelligent caching
#[derive(Debug)]
pub struct OptimizedFactStore {
    /// Primary storage backend
    storage: FactStorageBackend,
    /// LRU cache for frequently accessed facts
    cache: Option<LruCache<FactId, Fact>>,
    /// Shared field indexing system for optimized queries
    field_indexer: FieldIndexer,
    /// Bloom filter for fast existence checks
    bloom_filter: Option<FactBloomFilter>,
    /// Performance statistics
    cache_hits: usize,
    cache_misses: usize,
    lookup_count: usize,
    bloom_filter_saves: usize, // Number of expensive lookups avoided by bloom filter
}

/// Backend storage strategies for different use cases
#[derive(Debug)]
enum FactStorageBackend {
    /// HashMap-based storage for O(1) lookups (like FastFactLookup)
    HashMap(HashMap<FactId, Fact>),
    /// Vector-based storage for memory efficiency (like VecFactStore)
    Vector(Vec<Fact>),
}

impl OptimizedFactStore {
    /// Create a new optimized fact store with HashMap backend (fastest lookups)
    pub fn new_fast(cache_capacity: usize) -> Self {
        Self::new_fast_with_bloom(cache_capacity, 10000) // Default 10k expected facts
    }

    /// Create a new optimized fact store with HashMap backend and bloom filter
    pub fn new_fast_with_bloom(cache_capacity: usize, expected_facts: usize) -> Self {
        Self {
            storage: FactStorageBackend::HashMap(HashMap::new()),
            cache: Some(LruCache::new(cache_capacity)),
            field_indexer: FieldIndexer::new(),
            bloom_filter: Some(FactBloomFilter::with_capacity(expected_facts)),
            cache_hits: 0,
            cache_misses: 0,
            lookup_count: 0,
            bloom_filter_saves: 0,
        }
    }

    /// Create a new optimized fact store with Vector backend (memory efficient)
    pub fn new_memory_efficient(cache_capacity: usize) -> Self {
        Self::new_memory_efficient_with_bloom(cache_capacity, 10000)
    }

    /// Create a new optimized fact store with Vector backend and bloom filter
    pub fn new_memory_efficient_with_bloom(cache_capacity: usize, expected_facts: usize) -> Self {
        Self {
            storage: FactStorageBackend::Vector(Vec::new()),
            cache: Some(LruCache::new(cache_capacity)),
            field_indexer: FieldIndexer::new(),
            bloom_filter: Some(FactBloomFilter::with_capacity(expected_facts)),
            cache_hits: 0,
            cache_misses: 0,
            lookup_count: 0,
            bloom_filter_saves: 0,
        }
    }

    /// Create with pre-allocated capacity for better performance
    pub fn with_capacity(
        initial_capacity: usize,
        cache_capacity: usize,
        use_hashmap: bool,
    ) -> Self {
        Self::with_full_capacity(
            initial_capacity,
            cache_capacity,
            use_hashmap,
            initial_capacity,
        )
    }

    /// Create with full capacity specification including bloom filter
    pub fn with_full_capacity(
        initial_capacity: usize,
        cache_capacity: usize,
        use_hashmap: bool,
        expected_facts: usize,
    ) -> Self {
        let storage = if use_hashmap {
            FactStorageBackend::HashMap(HashMap::with_capacity(initial_capacity))
        } else {
            FactStorageBackend::Vector(Vec::with_capacity(initial_capacity))
        };

        Self {
            storage,
            cache: Some(LruCache::new(cache_capacity)),
            field_indexer: FieldIndexer::new(),
            bloom_filter: Some(FactBloomFilter::with_capacity(expected_facts)),
            cache_hits: 0,
            cache_misses: 0,
            lookup_count: 0,
            bloom_filter_saves: 0,
        }
    }

    /// Create without caching for minimal memory usage
    pub fn without_cache(use_hashmap: bool) -> Self {
        Self::without_cache_and_bloom(use_hashmap, false)
    }

    /// Create without caching and optionally without bloom filter for minimal memory usage
    pub fn without_cache_and_bloom(use_hashmap: bool, enable_bloom: bool) -> Self {
        let storage = if use_hashmap {
            FactStorageBackend::HashMap(HashMap::new())
        } else {
            FactStorageBackend::Vector(Vec::new())
        };

        Self {
            storage,
            cache: None,
            field_indexer: FieldIndexer::new(),
            bloom_filter: if enable_bloom {
                Some(FactBloomFilter::with_capacity(1000))
            } else {
                None
            },
            cache_hits: 0,
            cache_misses: 0,
            lookup_count: 0,
            bloom_filter_saves: 0,
        }
    }

    /// Get a fact by ID with optional caching (mutable for cache updates)
    pub fn get_mut(&mut self, fact_id: FactId) -> Option<Fact> {
        self.lookup_count += 1;

        // Check bloom filter first for fast negative lookup
        if let Some(bloom_filter) = &mut self.bloom_filter {
            if !bloom_filter.might_contain_fact(fact_id) {
                // Definitely not in the set - avoid expensive lookup
                self.bloom_filter_saves += 1;
                self.cache_misses += 1;
                return None;
            }
        }

        // Check cache first if enabled
        if let Some(cache) = &mut self.cache {
            if let Some(cached_fact) = cache.get(&fact_id) {
                self.cache_hits += 1;
                return Some(cached_fact.clone());
            }
        }

        // Cache miss or no cache - lookup in backend storage
        let fact = match &self.storage {
            FactStorageBackend::HashMap(map) => map.get(&fact_id).cloned(),
            FactStorageBackend::Vector(vec) => {
                // Try fast path first (ID matches index)
                if let Some(fact) = vec.get(fact_id as usize) {
                    if fact.id == fact_id {
                        Some(fact.clone())
                    } else {
                        // Fallback to linear search
                        vec.iter().find(|f| f.id == fact_id).cloned()
                    }
                } else {
                    // Linear search for all cases
                    vec.iter().find(|f| f.id == fact_id).cloned()
                }
            }
        };

        match fact {
            Some(found_fact) => {
                self.cache_misses += 1;
                // Add to cache for future access
                if let Some(cache) = &mut self.cache {
                    cache.put(fact_id, found_fact.clone());
                }
                Some(found_fact)
            }
            None => {
                self.cache_misses += 1;
                None
            }
        }
    }

    /// Get multiple facts by their IDs efficiently (batch operation)
    pub fn get_many(&mut self, fact_ids: &[FactId]) -> Vec<Fact> {
        let mut results = Vec::with_capacity(fact_ids.len());

        for &fact_id in fact_ids {
            if let Some(fact) = self.get_mut(fact_id) {
                results.push(fact);
            }
        }

        results
    }

    /// Insert or update a fact
    pub fn insert(&mut self, fact: Fact) -> FactId {
        let fact_id = match &mut self.storage {
            FactStorageBackend::HashMap(map) => {
                let id = fact.id;
                map.insert(id, fact.clone());
                id
            }
            FactStorageBackend::Vector(vec) => {
                // For vector storage, assign sequential IDs
                let id = vec.len() as FactId;
                let mut indexed_fact = fact.clone();
                indexed_fact.id = id;
                vec.push(indexed_fact.clone());
                id
            }
        };

        // Update cache if enabled
        if let Some(cache) = &mut self.cache {
            if cache.contains_key(&fact_id) {
                cache.put(fact_id, fact.clone());
            }
        }

        // Update field indexes using shared indexer
        self.field_indexer.index_fact(&fact);

        // Add to bloom filter if enabled
        if let Some(bloom_filter) = &mut self.bloom_filter {
            bloom_filter.add_fact(&fact);
        }

        fact_id
    }

    /// Remove a fact from storage and cache
    pub fn remove(&mut self, fact_id: FactId) -> Option<Fact> {
        // Get the fact before removing it (needed for indexer)
        let fact = match &self.storage {
            FactStorageBackend::HashMap(map) => map.get(&fact_id).cloned(),
            FactStorageBackend::Vector(vec) => vec.iter().find(|f| f.id == fact_id).cloned(),
        };

        if let Some(fact_to_remove) = &fact {
            // Remove from field indexes
            self.field_indexer.remove_fact(fact_to_remove);
        }

        // Remove from cache
        if let Some(cache) = &mut self.cache {
            cache.remove(&fact_id);
        }

        // Remove from backend storage
        match &mut self.storage {
            FactStorageBackend::HashMap(map) => map.remove(&fact_id),
            FactStorageBackend::Vector(vec) => {
                vec.iter().position(|f| f.id == fact_id).map(|pos| vec.remove(pos))
            }
        }
    }

    /// Clear all facts and reset statistics
    pub fn clear(&mut self) {
        match &mut self.storage {
            FactStorageBackend::HashMap(map) => map.clear(),
            FactStorageBackend::Vector(vec) => vec.clear(),
        }

        if let Some(cache) = &mut self.cache {
            cache.clear();
        }

        self.field_indexer.clear();

        if let Some(bloom_filter) = &mut self.bloom_filter {
            bloom_filter.clear();
        }

        self.cache_hits = 0;
        self.cache_misses = 0;
        self.lookup_count = 0;
        self.bloom_filter_saves = 0;
    }

    /// Get the total number of facts
    pub fn len(&self) -> usize {
        match &self.storage {
            FactStorageBackend::HashMap(map) => map.len(),
            FactStorageBackend::Vector(vec) => vec.len(),
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if a fact exists without affecting cache statistics
    pub fn contains(&self, fact_id: FactId) -> bool {
        match &self.storage {
            FactStorageBackend::HashMap(map) => map.contains_key(&fact_id),
            FactStorageBackend::Vector(vec) => vec.iter().any(|f| f.id == fact_id),
        }
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
    pub fn stats(&self) -> OptimizedStoreStats {
        OptimizedStoreStats {
            cache_stats: self.cache.as_ref().map(|c| c.stats()),
            field_index_stats: self.field_indexer.stats(),
            bloom_filter_stats: self.bloom_filter.as_ref().map(|bf| bf.stats()),
            hit_rate: self.hit_rate(),
            total_lookups: self.lookup_count,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            bloom_filter_saves: self.bloom_filter_saves,
            facts_stored: self.len(),
            backend_type: match &self.storage {
                FactStorageBackend::HashMap(_) => "HashMap",
                FactStorageBackend::Vector(_) => "Vector",
            },
        }
    }

    /// Extend with facts from a vector
    pub fn extend_from_vec(&mut self, facts: Vec<Fact>) {
        // Pre-allocate capacity if using HashMap
        if let FactStorageBackend::HashMap(map) = &mut self.storage {
            let additional_capacity = facts.len();
            if map.capacity() < map.len() + additional_capacity {
                map.reserve(additional_capacity);
            }
        }

        for fact in facts {
            self.insert(fact);
        }
    }

    /// Get all fact IDs currently stored
    pub fn fact_ids(&self) -> Vec<FactId> {
        match &self.storage {
            FactStorageBackend::HashMap(map) => map.keys().cloned().collect(),
            FactStorageBackend::Vector(vec) => vec.iter().map(|f| f.id).collect(),
        }
    }

    /// Find facts by field value using indexes
    pub fn find_by_field(&self, field: &str, value: &FactValue) -> Vec<FactId> {
        self.field_indexer.find_by_field(field, value)
    }

    /// Find facts by multiple criteria
    pub fn find_by_criteria(&self, criteria: &[(String, FactValue)]) -> Vec<FactId> {
        self.field_indexer.find_by_criteria(criteria)
    }

    /// Get field indexing statistics
    pub fn get_field_index_stats(&self) -> FieldIndexStats {
        self.field_indexer.stats()
    }

    /// Generate unified statistics for this fact store
    pub fn generate_unified_stats(&self) -> UnifiedStats {
        let backend_type = match &self.storage {
            FactStorageBackend::HashMap(_) => "HashMap",
            FactStorageBackend::Vector(_) => "Vector",
        };

        let mut builder = UnifiedStatsBuilder::new().with_fact_storage(
            backend_type,
            self.len(),
            self.lookup_count,
        );

        // Add cache statistics if cache is enabled
        if let Some(cache) = &self.cache {
            builder = builder.with_cache(
                "OptimizedFactStore",
                cache.stats(),
                self.cache_hits,
                self.cache_misses,
            );
        }

        let mut unified_stats = builder.build();

        // Add field indexing statistics
        unified_stats.register_indexing(self.field_indexer.stats());

        unified_stats
    }

    /// Get bloom filter effectiveness percentage
    pub fn bloom_filter_effectiveness(&self) -> f64 {
        if self.lookup_count == 0 {
            0.0
        } else {
            (self.bloom_filter_saves as f64 / self.lookup_count as f64) * 100.0
        }
    }

    /// Check if the bloom filter should be resized and do so if needed
    pub fn check_and_resize_bloom_filter(&mut self, new_expected_facts: usize) -> bool {
        if let Some(bloom_filter) = &mut self.bloom_filter {
            bloom_filter.check_and_resize(new_expected_facts)
        } else {
            false
        }
    }

    /// Get bloom filter statistics if enabled
    pub fn get_bloom_filter_stats(&self) -> Option<FactBloomStats> {
        self.bloom_filter.as_ref().map(|bf| bf.stats())
    }
}

/// Performance statistics for optimized fact store
#[derive(Debug, Clone)]
pub struct OptimizedStoreStats {
    pub cache_stats: Option<CacheStats>,
    pub field_index_stats: FieldIndexStats,
    pub bloom_filter_stats: Option<FactBloomStats>,
    pub hit_rate: f64,
    pub total_lookups: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub bloom_filter_saves: usize,
    pub facts_stored: usize,
    pub backend_type: &'static str,
}

impl OptimizedStoreStats {
    /// Get the lookup efficiency as a percentage (cache hits + bloom filter saves)
    pub fn efficiency(&self) -> f64 {
        if self.total_lookups == 0 {
            0.0
        } else {
            let efficient_lookups = self.cache_hits + self.bloom_filter_saves;
            (efficient_lookups as f64 / self.total_lookups as f64) * 100.0
        }
    }

    /// Get cache-only efficiency percentage
    pub fn cache_efficiency(&self) -> f64 {
        if self.total_lookups == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / self.total_lookups as f64) * 100.0
        }
    }

    /// Get bloom filter effectiveness percentage
    pub fn bloom_filter_effectiveness(&self) -> f64 {
        if self.total_lookups == 0 {
            0.0
        } else {
            (self.bloom_filter_saves as f64 / self.total_lookups as f64) * 100.0
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

    /// Estimate memory usage in bytes
    pub fn memory_usage_bytes(&self) -> usize {
        // Rough estimate: 128 bytes per fact (Fact struct size + HashMap overhead)
        self.facts_stored * 128
            + self.cache_stats.as_ref().map_or(0, |s| s.memory_usage_bytes())
            + self.field_index_stats.memory_usage_bytes
            + self.bloom_filter_stats.as_ref().map_or(0, |s| s.total_memory_usage)
    }
}

// Implement FactStore trait for backward compatibility
impl FactStore for OptimizedFactStore {
    fn insert(&mut self, fact: Fact) -> FactId {
        self.insert(fact)
    }

    fn get(&self, id: FactId) -> Option<&Fact> {
        // Note: This is a limitation of the trait design - we can't use the cache here
        // because it requires &mut self. For now, we'll access storage directly.
        match &self.storage {
            FactStorageBackend::HashMap(map) => map.get(&id),
            FactStorageBackend::Vector(vec) => {
                if let Some(fact) = vec.get(id as usize) {
                    if fact.id == id {
                        Some(fact)
                    } else {
                        vec.iter().find(|f| f.id == id)
                    }
                } else {
                    vec.iter().find(|f| f.id == id)
                }
            }
        }
    }

    fn extend_from_vec(&mut self, facts: Vec<Fact>) {
        self.extend_from_vec(facts)
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn find_by_field(&self, field: &str, value: &FactValue) -> Vec<&Fact> {
        let fact_ids = self.find_by_field(field, value);
        fact_ids.iter().filter_map(|&id| self.get(id)).collect()
    }

    fn find_by_criteria(&self, criteria: &[(String, FactValue)]) -> Vec<&Fact> {
        if criteria.is_empty() {
            return Vec::new();
        }

        // Start with fact IDs matching the first criterion
        let (first_field, first_value) = &criteria[0];
        let candidate_ids = self.find_by_field(first_field, first_value);

        // Convert to fact references and filter by remaining criteria
        let mut candidates: Vec<&Fact> =
            candidate_ids.iter().filter_map(|&id| self.get(id)).collect();

        // Filter by remaining criteria
        for (field, value) in &criteria[1..] {
            candidates.retain(|fact| {
                fact.data
                    .fields
                    .get(field)
                    .map(|fact_value| fact_value == value)
                    .unwrap_or(false)
            });
        }

        candidates
    }

    fn cache_stats(&self) -> Option<CacheStats> {
        self.cache.as_ref().map(|c| c.stats())
    }

    fn clear_cache(&mut self) {
        if let Some(cache) = &mut self.cache {
            cache.clear();
        }
        self.cache_hits = 0;
        self.cache_misses = 0;
    }
}

impl MemoryConsumer for OptimizedFactStore {
    fn memory_usage_bytes(&self) -> usize {
        // Rough estimate: 128 bytes per fact (Fact struct size + HashMap overhead)
        self.len() * 128
            + self.cache.as_ref().map_or(0, |c| c.stats().memory_usage_bytes())
            + self.field_indexer.estimate_memory_usage()
            + self.bloom_filter.as_ref().map_or(0, |bf| bf.stats().total_memory_usage)
    }

    fn reduce_memory_usage(&mut self, reduction_factor: f64) -> usize {
        let initial_size = self.memory_usage_bytes();

        // Reduce cache size
        if let Some(cache) = &mut self.cache {
            let current_size = cache.len();
            let target_size = (current_size as f64 * reduction_factor) as usize;
            let items_to_remove = current_size.saturating_sub(target_size);

            // Clear cache entries (LruCache doesn't have direct size reduction)
            // For now, we'll clear a portion of the cache
            if items_to_remove > 0 && current_size > 0 {
                let clear_ratio = items_to_remove as f64 / current_size as f64;
                if clear_ratio > 0.5 {
                    cache.clear();
                }
            }
        }

        // Clear bloom filter if reduction is significant
        if reduction_factor < 0.5 {
            if let Some(bloom_filter) = &mut self.bloom_filter {
                bloom_filter.clear();
            }
        }

        // For the main storage, we don't aggressively reduce memory as it holds core data.
        // More sophisticated strategies would involve data tiering or archiving.
        initial_size - self.memory_usage_bytes() // Return bytes freed
    }

    fn get_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        stats.insert("facts_stored".to_string(), self.len() as f64);
        stats.insert(
            "memory_usage_bytes".to_string(),
            self.memory_usage_bytes() as f64,
        );
        stats.insert("cache_hit_rate".to_string(), self.hit_rate());
        stats.insert("total_lookups".to_string(), self.lookup_count as f64);
        stats.insert(
            "bloom_filter_saves".to_string(),
            self.bloom_filter_saves as f64,
        );
        stats
    }

    fn name(&self) -> &str {
        "OptimizedFactStore"
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

        Fact { id, data: FactData { fields } }
    }

    #[test]
    fn test_unified_store_basic_operations() {
        let mut store = OptimizedFactStore::new_fast(10);

        // Test insertion and retrieval
        let fact1 = create_test_fact(1, "name", FactValue::String("Alice".to_string()));
        let fact2 = create_test_fact(2, "age", FactValue::Integer(30));

        store.insert(fact1.clone());
        store.insert(fact2.clone());

        assert_eq!(store.len(), 2);
        assert!(!store.is_empty());

        // Test mutable retrieval (with cache)
        let retrieved = store.get_mut(1).unwrap();
        assert_eq!(retrieved.id, 1);

        let retrieved2 = store.get_mut(2).unwrap();
        assert_eq!(retrieved2.id, 2);

        // Test non-existent fact
        assert!(store.get_mut(999).is_none());
    }

    #[test]
    fn test_cache_performance() {
        let mut store = OptimizedFactStore::new_fast(5);

        // Insert facts
        for i in 1..=10 {
            let fact = create_test_fact(i, "value", FactValue::Integer(i as i64));
            store.insert(fact);
        }

        // Access some facts multiple times to test caching
        for _ in 0..3 {
            store.get_mut(1);
            store.get_mut(2);
            store.get_mut(3);
        }

        let stats = store.stats();

        // First access should be cache misses, subsequent should be hits
        assert!(stats.cache_hits > 0);
        assert!(stats.cache_misses > 0);
        assert!(stats.hit_rate > 0.0);
        assert_eq!(stats.total_lookups, 9); // 3 facts × 3 accesses each
    }

    #[test]
    fn test_backend_switching() {
        // Test HashMap backend
        let mut hash_store = OptimizedFactStore::new_fast(10);
        let fact = create_test_fact(42, "test", FactValue::Boolean(true));
        hash_store.insert(fact.clone());
        assert_eq!(hash_store.stats().backend_type, "HashMap");

        // Test Vector backend
        let mut vec_store = OptimizedFactStore::new_memory_efficient(10);
        vec_store.insert(fact.clone());
        assert_eq!(vec_store.stats().backend_type, "Vector");
    }

    #[test]
    fn test_no_cache_mode() {
        let mut store = OptimizedFactStore::without_cache(true);

        let fact = create_test_fact(1, "test", FactValue::String("no_cache".to_string()));
        store.insert(fact);

        // Multiple accesses should all be cache misses
        store.get_mut(1);
        store.get_mut(1);
        store.get_mut(1);

        let stats = store.stats();
        assert_eq!(stats.cache_hits, 0);
        assert!(stats.cache_misses > 0);
        assert!(stats.cache_stats.is_none());
    }
}
