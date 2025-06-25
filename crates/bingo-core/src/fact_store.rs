use crate::cache::{CacheStats, LruCache};
use crate::types::{Fact, FactId, FactValue};
use std::collections::HashMap;

/// Abstraction for fact storage that allows swapping allocation strategies
pub trait FactStore: Send + Sync {
    /// Insert a fact and return its ID
    fn insert(&mut self, fact: Fact) -> FactId;

    /// Get a fact by ID
    fn get(&self, id: FactId) -> Option<&Fact>;

    /// Extend with facts from a Vec (object-safe alternative to generic extend)
    fn extend_from_vec(&mut self, facts: Vec<Fact>);

    /// Get the number of stored facts
    fn len(&self) -> usize;

    /// Check if empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all facts
    fn clear(&mut self);

    /// Find facts by field value (optimized lookup)
    fn find_by_field(&self, field: &str, value: &FactValue) -> Vec<&Fact>;

    /// Find facts by multiple field criteria
    fn find_by_criteria(&self, criteria: &[(String, FactValue)]) -> Vec<&Fact>;

    /// Get cache statistics (if caching is enabled)
    fn cache_stats(&self) -> Option<CacheStats> {
        None
    }

    /// Clear cache (if caching is enabled)
    fn clear_cache(&mut self) {
        // Default implementation does nothing
    }
}

/// Standard Vec-based fact store for baseline implementation
#[derive(Debug, Default)]
pub struct VecFactStore {
    facts: Vec<Fact>,
    // Hash indexes for common lookup patterns
    field_indexes: HashMap<String, HashMap<String, Vec<FactId>>>,
}

impl VecFactStore {
    /// Create a new Vec-based fact store
    pub fn new() -> Self {
        Self { facts: Vec::new(), field_indexes: HashMap::new() }
    }

    /// Create with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self { facts: Vec::with_capacity(capacity), field_indexes: HashMap::new() }
    }

    /// Update indexes when a fact is added (only index commonly used fields for performance)
    fn update_indexes(&mut self, fact: &Fact) {
        // Only index fields that are commonly used for lookups
        let indexed_fields = ["entity_id", "id", "user_id", "customer_id", "status", "category"];

        for (field_name, field_value) in &fact.data.fields {
            if indexed_fields.contains(&field_name.as_str()) {
                let value_key = self.fact_value_to_index_key(field_value);

                self.field_indexes
                    .entry(field_name.clone())
                    .or_default()
                    .entry(value_key)
                    .or_default()
                    .push(fact.id);
            }
        }
    }

    /// Convert FactValue to string key for indexing
    fn fact_value_to_index_key(&self, value: &FactValue) -> String {
        value.as_string()
    }

    /// Get all facts as a slice
    pub fn facts(&self) -> &[Fact] {
        &self.facts
    }

    /// Insert a fact with a predetermined ID (for partitioned stores)
    pub fn insert_with_id(&mut self, mut fact: Fact, predetermined_id: FactId) -> FactId {
        fact.id = predetermined_id;
        self.update_indexes(&fact);
        self.facts.push(fact);
        predetermined_id
    }
}

impl FactStore for VecFactStore {
    fn insert(&mut self, fact: Fact) -> FactId {
        let id = self.facts.len() as FactId;

        // Set the ID based on position, then update indexes
        let mut indexed_fact = fact;
        indexed_fact.id = id;

        // Update indexes with the fact that has proper ID
        self.update_indexes(&indexed_fact);

        self.facts.push(indexed_fact);
        id
    }

    fn get(&self, id: FactId) -> Option<&Fact> {
        // First try the fast path: if ID matches array index, use direct access
        if let Some(fact) = self.facts.get(id as usize) {
            if fact.id == id {
                return Some(fact);
            }
        }

        // Fallback: search linearly for the correct fact ID
        // This is necessary when facts have non-sequential IDs (e.g., in partitioned stores)
        self.facts.iter().find(|fact| fact.id == id)
    }

    fn extend_from_vec(&mut self, facts: Vec<Fact>) {
        for fact in facts {
            self.insert(fact);
        }
    }

    fn len(&self) -> usize {
        self.facts.len()
    }

    fn clear(&mut self) {
        self.facts.clear();
        self.field_indexes.clear();
    }

    fn find_by_field(&self, field: &str, value: &FactValue) -> Vec<&Fact> {
        let value_key = self.fact_value_to_index_key(value);

        if let Some(field_index) = self.field_indexes.get(field) {
            if let Some(fact_ids) = field_index.get(&value_key) {
                return fact_ids.iter().filter_map(|&id| self.get(id)).collect();
            }
        }

        Vec::new()
    }

    fn find_by_criteria(&self, criteria: &[(String, FactValue)]) -> Vec<&Fact> {
        if criteria.is_empty() {
            return Vec::new();
        }

        // Start with facts matching the first criterion
        let (first_field, first_value) = &criteria[0];
        let mut candidates = self.find_by_field(first_field, first_value);

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
}

impl VecFactStore {
    /// Extend with any iterator (convenience method for VecFactStore)
    pub fn extend<I: IntoIterator<Item = Fact>>(&mut self, facts: I) {
        self.facts.extend(facts);
    }
}

#[cfg(all(feature = "arena-alloc", not(target_arch = "wasm32")))]
mod arena_store {
    use super::*;
    use std::collections::HashMap;

    /// Arena-based fact store for high-performance allocation (Phase 2+)
    /// Note: Using Vec-based storage for thread safety with Rust 1.87.0+
    #[derive(Default)]
    pub struct ArenaFactStore {
        facts: Vec<Fact>,
        fact_map: HashMap<FactId, usize>,
        field_indexes: HashMap<String, HashMap<String, Vec<FactId>>>,
        next_id: FactId,
    }

    impl ArenaFactStore {
        pub fn new() -> Self {
            Self {
                facts: Vec::new(),
                fact_map: HashMap::new(),
                field_indexes: HashMap::new(),
                next_id: 0,
            }
        }

        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                facts: Vec::with_capacity(capacity),
                fact_map: HashMap::with_capacity(capacity),
                field_indexes: HashMap::new(),
                next_id: 0,
            }
        }

        /// Update indexes when a fact is added (only index commonly used fields for performance)
        fn update_indexes(&mut self, fact: &Fact) {
            // Only index fields that are commonly used for lookups
            let indexed_fields =
                ["entity_id", "id", "user_id", "customer_id", "status", "category"];

            for (field_name, field_value) in &fact.data.fields {
                if indexed_fields.contains(&field_name.as_str()) {
                    let value_key = self.fact_value_to_index_key(field_value);

                    self.field_indexes
                        .entry(field_name.clone())
                        .or_default()
                        .entry(value_key)
                        .or_default()
                        .push(fact.id);
                }
            }
        }

        /// Convert FactValue to string key for indexing
        fn fact_value_to_index_key(&self, value: &FactValue) -> String {
            match value {
                FactValue::String(s) => s.clone(),
                FactValue::Integer(i) => i.to_string(),
                FactValue::Float(f) => f.to_string(),
                FactValue::Boolean(b) => b.to_string(),
                FactValue::Array(_) => "[array]".to_string(),
                FactValue::Object(_) => "[object]".to_string(),
                FactValue::Date(date) => date.to_rfc3339(),
                FactValue::Null => "[null]".to_string(),
            }
        }
    }

    impl FactStore for ArenaFactStore {
        fn insert(&mut self, fact: Fact) -> FactId {
            let id = self.next_id;
            self.next_id += 1;

            // Set the fact ID and update indexes
            let mut indexed_fact = fact;
            indexed_fact.id = id;
            self.update_indexes(&indexed_fact);

            // Store fact in Vec and map ID to index
            let index = self.facts.len();
            self.facts.push(indexed_fact);
            self.fact_map.insert(id, index);
            id
        }

        fn get(&self, id: FactId) -> Option<&Fact> {
            self.fact_map.get(&id).and_then(|&index| self.facts.get(index))
        }

        fn extend_from_vec(&mut self, facts: Vec<Fact>) {
            for fact in facts {
                self.insert(fact);
            }
        }

        fn len(&self) -> usize {
            self.facts.len()
        }

        fn is_empty(&self) -> bool {
            self.facts.is_empty()
        }

        fn clear(&mut self) {
            self.facts.clear();
            self.fact_map.clear();
            self.field_indexes.clear();
            self.next_id = 0;
        }

        fn find_by_field(&self, field: &str, value: &FactValue) -> Vec<&Fact> {
            let value_key = self.fact_value_to_index_key(value);

            if let Some(field_index) = self.field_indexes.get(field) {
                if let Some(fact_ids) = field_index.get(&value_key) {
                    return fact_ids.iter().filter_map(|&id| self.get(id)).collect();
                }
            }

            Vec::new()
        }

        fn find_by_criteria(&self, criteria: &[(String, FactValue)]) -> Vec<&Fact> {
            if criteria.is_empty() {
                return Vec::new();
            }

            // Start with facts matching the first criterion
            let (first_field, first_value) = &criteria[0];
            let mut candidates = self.find_by_field(first_field, first_value);

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
    }
}

#[cfg(all(feature = "arena-alloc", not(target_arch = "wasm32")))]
pub use arena_store::ArenaFactStore;

/// Cached fact store that wraps another store with LRU caching
///
/// This provides caching for frequently accessed facts, which can significantly
/// improve performance in scenarios with repeated fact lookups.
#[derive(Debug)]
pub struct CachedFactStore {
    inner: VecFactStore,
    cache: LruCache<FactId, Fact>,
    cache_hits: usize,
    cache_misses: usize,
}

impl CachedFactStore {
    /// Create a new cached fact store with the specified cache capacity
    pub fn new(cache_capacity: usize) -> Self {
        Self {
            inner: VecFactStore::new(),
            cache: LruCache::new(cache_capacity),
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// Create a new cached fact store with specified inner capacity and cache capacity
    pub fn with_capacity(inner_capacity: usize, cache_capacity: usize) -> Self {
        Self {
            inner: VecFactStore::with_capacity(inner_capacity),
            cache: LruCache::new(cache_capacity),
            cache_hits: 0,
            cache_misses: 0,
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

    /// Get detailed cache statistics
    pub fn detailed_stats(&self) -> CachedStoreStats {
        CachedStoreStats {
            cache_stats: self.cache.stats(),
            hit_rate: self.hit_rate(),
            total_hits: self.cache_hits,
            total_misses: self.cache_misses,
        }
    }
}

impl FactStore for CachedFactStore {
    fn insert(&mut self, fact: Fact) -> FactId {
        let id = self.inner.insert(fact);
        // Pre-populate cache with the fact as stored (with correct ID)
        if let Some(stored_fact) = self.inner.get(id) {
            self.cache.put(id, stored_fact.clone());
        }
        id
    }

    fn get(&self, id: FactId) -> Option<&Fact> {
        // Note: We can't update cache hit/miss counters here because of &self
        // This is a limitation of the current trait design
        self.inner.get(id)
    }

    fn extend_from_vec(&mut self, facts: Vec<Fact>) {
        // Clear cache when bulk loading to maintain consistency
        self.cache.clear();
        self.inner.extend_from_vec(facts);
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn clear(&mut self) {
        self.inner.clear();
        self.cache.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }

    fn find_by_field(&self, field: &str, value: &FactValue) -> Vec<&Fact> {
        self.inner.find_by_field(field, value)
    }

    fn find_by_criteria(&self, criteria: &[(String, FactValue)]) -> Vec<&Fact> {
        self.inner.find_by_criteria(criteria)
    }

    fn cache_stats(&self) -> Option<CacheStats> {
        Some(self.cache.stats())
    }

    fn clear_cache(&mut self) {
        self.cache.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }
}

/// Detailed cache statistics for the CachedFactStore
#[derive(Debug, Clone)]
pub struct CachedStoreStats {
    pub cache_stats: CacheStats,
    pub hit_rate: f64,
    pub total_hits: usize,
    pub total_misses: usize,
}

/// Partitioned fact store for memory-efficient large datasets
///
/// This implementation distributes facts across multiple partitions to reduce
/// memory pressure and improve performance on very large datasets.
#[derive(Debug)]
pub struct PartitionedFactStore {
    partitions: Vec<VecFactStore>,
    partition_count: usize,
    total_facts: usize,
}

impl PartitionedFactStore {
    /// Create a new partitioned fact store with the specified number of partitions
    pub fn new(partition_count: usize) -> Self {
        let partitions = (0..partition_count).map(|_| VecFactStore::new()).collect();

        Self { partitions, partition_count, total_facts: 0 }
    }

    /// Create with capacity hint per partition
    pub fn with_capacity(partition_count: usize, capacity_per_partition: usize) -> Self {
        let partitions = (0..partition_count)
            .map(|_| VecFactStore::with_capacity(capacity_per_partition))
            .collect();

        Self { partitions, partition_count, total_facts: 0 }
    }

    /// Get the partition index for a given fact ID
    fn partition_for_id(&self, fact_id: FactId) -> usize {
        (fact_id as usize) % self.partition_count
    }

    /// Get partition for a field value (for efficient querying)
    #[allow(dead_code)]
    fn partition_for_field_value(&self, field_value: &FactValue) -> Option<usize> {
        // Hash the field value to determine partition
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        // Use the existing Hash implementation for FactValue
        field_value.hash(&mut hasher);

        Some((hasher.finish() as usize) % self.partition_count)
    }

    /// Get statistics for each partition
    pub fn partition_stats(&self) -> Vec<(usize, usize)> {
        self.partitions
            .iter()
            .enumerate()
            .map(|(i, partition)| (i, partition.len()))
            .collect()
    }

    /// Get the least loaded partition index
    #[allow(dead_code)]
    fn least_loaded_partition(&self) -> usize {
        self.partitions
            .iter()
            .enumerate()
            .min_by_key(|(_, partition)| partition.len())
            .map(|(index, _)| index)
            .unwrap_or(0)
    }
}

impl FactStore for PartitionedFactStore {
    fn insert(&mut self, fact: Fact) -> FactId {
        // Use fact ID to determine partition for consistent lookup
        let global_id = self.total_facts as FactId;
        let partition_index = self.partition_for_id(global_id);

        // Insert into the appropriate partition with predetermined global ID
        self.partitions[partition_index].insert_with_id(fact, global_id);
        self.total_facts += 1;

        global_id
    }

    fn get(&self, id: FactId) -> Option<&Fact> {
        let partition_index = self.partition_for_id(id);

        // Search in the determined partition for the global ID
        self.partitions[partition_index].facts().iter().find(|fact| fact.id == id)
    }

    fn extend_from_vec(&mut self, facts: Vec<Fact>) {
        // Distribute facts across partitions for load balancing
        for fact in facts {
            self.insert(fact);
        }
    }

    fn len(&self) -> usize {
        self.total_facts
    }

    fn clear(&mut self) {
        for partition in &mut self.partitions {
            partition.clear();
        }
        self.total_facts = 0;
    }

    fn find_by_field(&self, field: &str, value: &FactValue) -> Vec<&Fact> {
        let mut results = Vec::new();

        // For partitioned search, we need to search all partitions
        // since facts are distributed by ID, not by field value
        for partition in &self.partitions {
            results.extend(partition.find_by_field(field, value));
        }

        results
    }

    fn find_by_criteria(&self, criteria: &[(String, FactValue)]) -> Vec<&Fact> {
        let mut results = Vec::new();

        // For multi-criteria search, search all partitions
        // TODO: Optimize this by using partition hints from criteria
        for partition in &self.partitions {
            results.extend(partition.find_by_criteria(criteria));
        }

        results
    }
}

/// Factory for creating optimized fact stores based on use case
pub struct FactStoreFactory;

impl FactStoreFactory {
    /// Create the best fact store for the given capacity and performance requirements
    pub fn create_optimized(capacity_hint: usize) -> Box<dyn FactStore> {
        // Use the large dataset optimization logic for proper memory efficiency
        Self::create_for_large_dataset(capacity_hint)
    }

    /// Create a simple Vec-based store for development and testing
    pub fn create_simple() -> Box<dyn FactStore> {
        Box::new(VecFactStore::new())
    }

    /// Create a cached fact store for scenarios with repeated fact access
    pub fn create_cached(capacity_hint: usize, cache_size: usize) -> Box<dyn FactStore> {
        Box::new(CachedFactStore::with_capacity(capacity_hint, cache_size))
    }

    /// Create an arena-based store for maximum performance (requires arena-alloc feature)
    #[cfg(all(feature = "arena-alloc", not(target_arch = "wasm32")))]
    pub fn create_arena(capacity: usize) -> Box<dyn FactStore> {
        Box::new(ArenaFactStore::with_capacity(capacity))
    }

    /// Create a partitioned store for very large datasets
    pub fn create_partitioned(
        partition_count: usize,
        capacity_per_partition: usize,
    ) -> Box<dyn FactStore> {
        Box::new(PartitionedFactStore::with_capacity(
            partition_count,
            capacity_per_partition,
        ))
    }

    /// Create the best store for large dataset scenarios
    pub fn create_for_large_dataset(estimated_facts: usize) -> Box<dyn FactStore> {
        if estimated_facts > 1_000_000 {
            // Use partitioned store for very large datasets
            let partition_count = (estimated_facts / 100_000).clamp(4, 16); // 4-16 partitions
            let capacity_per_partition = estimated_facts / partition_count + 1000;
            Self::create_partitioned(partition_count, capacity_per_partition)
        } else if estimated_facts > 10_000 {
            // Use cached store for medium datasets
            let cache_size = (estimated_facts / 10).clamp(100, 1000);
            Self::create_cached(estimated_facts, cache_size)
        } else {
            // Use simple store for small datasets
            Box::new(VecFactStore::with_capacity(estimated_facts))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FactData, FactValue};
    use std::collections::HashMap;

    fn create_test_fact(id: u64) -> Fact {
        let mut fields = HashMap::new();
        fields.insert("test_field".to_string(), FactValue::Integer(id as i64));

        Fact { id, data: FactData { fields } }
    }

    #[test]
    fn test_vec_fact_store() {
        let mut store = VecFactStore::new();

        let fact1 = create_test_fact(1);
        let fact2 = create_test_fact(2);

        let id1 = store.insert(fact1.clone());
        let id2 = store.insert(fact2.clone());

        assert_eq!(store.len(), 2);
        assert_eq!(store.get(id1).unwrap().id, id1); // ID gets overridden to index
        assert_eq!(store.get(id2).unwrap().id, id2); // ID gets overridden to index

        let facts = vec![create_test_fact(3), create_test_fact(4)];
        store.extend_from_vec(facts);

        assert_eq!(store.len(), 4);
    }

    #[test]
    fn test_vec_fact_store_with_capacity() {
        let store = VecFactStore::with_capacity(100);
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }
}
