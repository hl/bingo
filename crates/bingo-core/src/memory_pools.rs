//! Memory Pool Management for High-Frequency Allocations
//!
//! This module provides memory pools for frequently allocated objects to reduce
//! garbage collection pressure and improve performance in high-throughput scenarios.

use crate::rete_nodes::{FactIdSet, Token};
use crate::types::{Fact, FactData, FactValue};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

/// Object pool for frequent allocations with configurable growth strategies
pub struct ObjectPool<T> {
    /// Available objects ready for reuse
    available: VecDeque<T>,
    /// Factory function to create new objects when pool is empty
    factory: Box<dyn Fn() -> T + Send + Sync>,
    /// Maximum pool size to prevent unbounded growth
    max_size: usize,
    /// Current pool utilization statistics
    pub allocated_count: usize,
    pub returned_count: usize,
    pub pool_hits: usize,
    pub pool_misses: usize,
    pub peak_size: usize,
}

impl<T> ObjectPool<T> {
    /// Create a new object pool with factory function and max size
    pub fn new<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            available: VecDeque::with_capacity(max_size.min(64)),
            factory: Box::new(factory),
            max_size,
            allocated_count: 0,
            returned_count: 0,
            pool_hits: 0,
            pool_misses: 0,
            peak_size: 0,
        }
    }

    /// Get an object from the pool, creating new if none available
    pub fn get(&mut self) -> T {
        self.allocated_count += 1;

        if let Some(obj) = self.available.pop_front() {
            self.pool_hits += 1;
            obj
        } else {
            self.pool_misses += 1;
            (self.factory)()
        }
    }

    /// Return an object to the pool for reuse
    pub fn return_object(&mut self, obj: T) {
        self.returned_count += 1;

        if self.available.len() < self.max_size {
            self.available.push_back(obj);
            self.peak_size = self.peak_size.max(self.available.len());
        }
        // If pool is full, object is dropped (garbage collected)
    }

    /// Get pool utilization statistics
    pub fn utilization(&self) -> f64 {
        let total_requests = self.pool_hits + self.pool_misses;
        if total_requests == 0 {
            0.0
        } else {
            (self.pool_hits as f64 / total_requests as f64) * 100.0
        }
    }

    /// Get hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        self.utilization()
    }

    /// Clear all pooled objects
    pub fn clear(&mut self) {
        self.available.clear();
        self.allocated_count = 0;
        self.returned_count = 0;
        self.pool_hits = 0;
        self.pool_misses = 0;
        self.peak_size = 0;
    }

    /// Get current pool size
    pub fn size(&self) -> usize {
        self.available.len()
    }
}

/// Thread-safe wrapper for object pools
pub struct ThreadSafePool<T> {
    pool: Arc<Mutex<ObjectPool<T>>>,
}

impl<T> ThreadSafePool<T> {
    /// Create a new thread-safe object pool
    pub fn new<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self { pool: Arc::new(Mutex::new(ObjectPool::new(factory, max_size))) }
    }

    /// Get an object from the pool
    pub fn get(&self) -> T {
        self.pool.lock().unwrap().get()
    }

    /// Return an object to the pool
    pub fn return_object(&self, obj: T) {
        self.pool.lock().unwrap().return_object(obj);
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let pool = self.pool.lock().unwrap();
        PoolStats {
            allocated_count: pool.allocated_count,
            returned_count: pool.returned_count,
            pool_hits: pool.pool_hits,
            pool_misses: pool.pool_misses,
            hit_rate: pool.hit_rate(),
            current_size: pool.size(),
            peak_size: pool.peak_size,
            memory_usage_bytes: 0, // Placeholder, actual calculation depends on T
        }
    }

    /// Clear the pool
    pub fn clear(&self) {
        self.pool.lock().unwrap().clear();
    }
}

impl<T> Clone for ThreadSafePool<T> {
    fn clone(&self) -> Self {
        Self { pool: Arc::clone(&self.pool) }
    }
}

/// Statistics for memory pool performance monitoring
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub allocated_count: usize,
    pub returned_count: usize,
    pub pool_hits: usize,
    pub pool_misses: usize,
    pub hit_rate: f64,
    pub current_size: usize,
    pub peak_size: usize,
    pub memory_usage_bytes: usize,
}

impl PoolStats {
    /// Get total pool operations
    pub fn total_operations(&self) -> usize {
        self.pool_hits + self.pool_misses
    }

    /// Get memory efficiency (objects reused vs created)
    pub fn memory_efficiency(&self) -> f64 {
        if self.allocated_count == 0 {
            0.0
        } else {
            (self.pool_hits as f64 / self.allocated_count as f64) * 100.0
        }
    }

    /// Estimate memory usage in bytes
    pub fn memory_usage_bytes(&self) -> usize {
        self.memory_usage_bytes
    }
}

/// Specialized pools for RETE engine objects
pub struct ReteMemoryPools {
    /// Pool for frequently created tokens
    pub token_pool: ThreadSafePool<Token>,
    /// Pool for fact data objects
    pub fact_data_pool: ThreadSafePool<FactData>,
    /// Pool for HashMap<String, FactValue> (common in facts)
    pub field_map_pool: ThreadSafePool<HashMap<String, FactValue>>,
    /// Pool for Vec<Fact> collections
    pub fact_vec_pool: ThreadSafePool<Vec<Fact>>,
    /// Pool for FactIdSet objects
    pub fact_id_set_pool: ThreadSafePool<FactIdSet>,
}

use crate::unified_memory_coordinator::MemoryConsumer;

impl ReteMemoryPools {
    /// Create new RETE memory pools with optimized sizes
    pub fn new() -> Self {
        Self {
            token_pool: ThreadSafePool::new(
                || Token::from_facts(vec![]),
                1000, // Pool up to 1000 tokens
            ),
            fact_data_pool: ThreadSafePool::new(
                || FactData { fields: HashMap::with_capacity(8) },
                500, // Pool up to 500 fact data objects
            ),
            field_map_pool: ThreadSafePool::new(
                || HashMap::with_capacity(8),
                750, // Pool up to 750 field maps
            ),
            fact_vec_pool: ThreadSafePool::new(
                || Vec::with_capacity(100),
                200, // Pool up to 200 fact vectors
            ),
            fact_id_set_pool: ThreadSafePool::new(
                || FactIdSet::new(Vec::with_capacity(4)),
                1000, // Pool up to 1000 fact ID sets
            ),
        }
    }
}

impl MemoryConsumer for ReteMemoryPools {
    fn memory_usage_bytes(&self) -> usize {
        self.token_pool.stats().memory_usage_bytes
            + self.fact_data_pool.stats().memory_usage_bytes
            + self.field_map_pool.stats().memory_usage_bytes
            + self.fact_vec_pool.stats().memory_usage_bytes
            + self.fact_id_set_pool.stats().memory_usage_bytes
    }

    fn reduce_memory_usage(&mut self, _reduction_factor: f64) -> usize {
        // For now, we don't have a direct way to reduce capacity of ThreadSafePools
        // This would require adding `reduce_capacity` methods to ThreadSafePool and ObjectPool
        // and then calling them here.
        0
    }

    fn get_stats(&self) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        let stats = self.get_stats();
        map.insert(
            "token_pool_hits".to_string(),
            stats.token_pool.pool_hits as f64,
        );
        map.insert(
            "token_pool_misses".to_string(),
            stats.token_pool.pool_misses as f64,
        );
        map.insert(
            "fact_data_pool_hits".to_string(),
            stats.fact_data_pool.pool_hits as f64,
        );
        map.insert(
            "fact_data_pool_misses".to_string(),
            stats.fact_data_pool.pool_misses as f64,
        );
        map.insert(
            "field_map_pool_hits".to_string(),
            stats.field_map_pool.pool_hits as f64,
        );
        map.insert(
            "field_map_pool_misses".to_string(),
            stats.field_map_pool.pool_misses as f64,
        );
        map.insert(
            "fact_vec_pool_hits".to_string(),
            stats.fact_vec_pool.pool_hits as f64,
        );
        map.insert(
            "fact_vec_pool_misses".to_string(),
            stats.fact_vec_pool.pool_misses as f64,
        );
        map.insert(
            "fact_id_set_pool_hits".to_string(),
            stats.fact_id_set_pool.pool_hits as f64,
        );
        map.insert(
            "fact_id_set_pool_misses".to_string(),
            stats.fact_id_set_pool.pool_misses as f64,
        );
        map.insert("overall_efficiency".to_string(), self.overall_efficiency());
        map.insert(
            "total_pooled_objects".to_string(),
            self.total_pooled_objects() as f64,
        );
        map.insert(
            "total_peak_objects".to_string(),
            self.total_peak_objects() as f64,
        );
        map.insert(
            "memory_usage_bytes".to_string(),
            self.memory_usage_bytes() as f64,
        );
        map
    }

    fn name(&self) -> &str {
        "ReteMemoryPools"
    }
}

impl ReteMemoryPools {
    /// Create pools with custom sizes for different usage patterns
    pub fn with_sizes(
        token_pool_size: usize,
        fact_data_pool_size: usize,
        field_map_pool_size: usize,
        fact_vec_pool_size: usize,
        fact_id_set_pool_size: usize,
    ) -> Self {
        Self {
            token_pool: ThreadSafePool::new(|| Token::from_facts(vec![]), token_pool_size),
            fact_data_pool: ThreadSafePool::new(
                || FactData { fields: HashMap::with_capacity(8) },
                fact_data_pool_size,
            ),
            field_map_pool: ThreadSafePool::new(|| HashMap::with_capacity(8), field_map_pool_size),
            fact_vec_pool: ThreadSafePool::new(|| Vec::with_capacity(100), fact_vec_pool_size),
            fact_id_set_pool: ThreadSafePool::new(
                || FactIdSet::new(Vec::with_capacity(4)),
                fact_id_set_pool_size,
            ),
        }
    }

    /// Get a token from the pool, ready for reuse
    pub fn get_token(&self) -> Token {
        self.token_pool.get()
    }

    /// Return a token to the pool (automatically clears fact IDs)
    pub fn return_token(&self, mut token: Token) {
        // Clear the token for reuse
        token.fact_ids = FactIdSet::new(Vec::new());
        self.token_pool.return_object(token);
    }

    /// Get a fact data object from the pool
    pub fn get_fact_data(&self) -> FactData {
        let mut fact_data = self.fact_data_pool.get();
        fact_data.fields.clear(); // Ensure clean state
        fact_data
    }

    /// Return a fact data object to the pool
    pub fn return_fact_data(&self, fact_data: FactData) {
        self.fact_data_pool.return_object(fact_data);
    }

    /// Get a field map from the pool
    pub fn get_field_map(&self) -> HashMap<String, FactValue> {
        let mut map = self.field_map_pool.get();
        map.clear(); // Ensure clean state
        map
    }

    /// Return a field map to the pool
    pub fn return_field_map(&self, map: HashMap<String, FactValue>) {
        self.field_map_pool.return_object(map);
    }

    /// Get a fact vector from the pool
    pub fn get_fact_vec(&self) -> Vec<Fact> {
        let mut vec = self.fact_vec_pool.get();
        vec.clear(); // Ensure clean state
        vec
    }

    /// Return a fact vector to the pool
    pub fn return_fact_vec(&self, vec: Vec<Fact>) {
        self.fact_vec_pool.return_object(vec);
    }

    /// Get a fact ID set from the pool
    pub fn get_fact_id_set(&self) -> FactIdSet {
        self.fact_id_set_pool.get()
    }

    /// Return a fact ID set to the pool
    pub fn return_fact_id_set(&self, fact_id_set: FactIdSet) {
        self.fact_id_set_pool.return_object(fact_id_set);
    }

    /// Get comprehensive statistics for all pools
    pub fn get_stats(&self) -> RetePoolStats {
        RetePoolStats {
            token_pool: self.token_pool.stats(),
            fact_data_pool: self.fact_data_pool.stats(),
            field_map_pool: self.field_map_pool.stats(),
            fact_vec_pool: self.fact_vec_pool.stats(),
            fact_id_set_pool: self.fact_id_set_pool.stats(),
        }
    }

    /// Clear all pools
    pub fn clear_all(&self) {
        self.token_pool.clear();
        self.fact_data_pool.clear();
        self.field_map_pool.clear();
        self.fact_vec_pool.clear();
        self.fact_id_set_pool.clear();
    }

    /// Get overall memory pool efficiency
    pub fn overall_efficiency(&self) -> f64 {
        let stats = self.get_stats();
        let total_hits = stats.token_pool.pool_hits
            + stats.fact_data_pool.pool_hits
            + stats.field_map_pool.pool_hits
            + stats.fact_vec_pool.pool_hits
            + stats.fact_id_set_pool.pool_hits;

        let total_operations = stats.token_pool.total_operations()
            + stats.fact_data_pool.total_operations()
            + stats.field_map_pool.total_operations()
            + stats.fact_vec_pool.total_operations()
            + stats.fact_id_set_pool.total_operations();

        if total_operations == 0 {
            0.0
        } else {
            (total_hits as f64 / total_operations as f64) * 100.0
        }
    }

    /// Get total number of pooled objects across all pools
    pub fn total_pooled_objects(&self) -> usize {
        let stats = self.get_stats();
        stats.token_pool.current_size
            + stats.fact_data_pool.current_size
            + stats.field_map_pool.current_size
            + stats.fact_vec_pool.current_size
            + stats.fact_id_set_pool.current_size
    }

    /// Get total peak objects across all pools
    pub fn total_peak_objects(&self) -> usize {
        let stats = self.get_stats();
        stats.token_pool.peak_size
            + stats.fact_data_pool.peak_size
            + stats.field_map_pool.peak_size
            + stats.fact_vec_pool.peak_size
            + stats.fact_id_set_pool.peak_size
    }
}

impl Default for ReteMemoryPools {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive statistics for all RETE memory pools
#[derive(Debug, Clone)]
pub struct RetePoolStats {
    pub token_pool: PoolStats,
    pub fact_data_pool: PoolStats,
    pub field_map_pool: PoolStats,
    pub fact_vec_pool: PoolStats,
    pub fact_id_set_pool: PoolStats,
}

impl RetePoolStats {
    /// Get total memory operations across all pools
    pub fn total_operations(&self) -> usize {
        self.token_pool.total_operations()
            + self.fact_data_pool.total_operations()
            + self.field_map_pool.total_operations()
            + self.fact_vec_pool.total_operations()
            + self.fact_id_set_pool.total_operations()
    }

    /// Get total objects currently pooled
    pub fn total_pooled_objects(&self) -> usize {
        self.token_pool.current_size
            + self.fact_data_pool.current_size
            + self.field_map_pool.current_size
            + self.fact_vec_pool.current_size
            + self.fact_id_set_pool.current_size
    }

    /// Get average hit rate across all pools
    pub fn average_hit_rate(&self) -> f64 {
        let hit_rates = [
            self.token_pool.hit_rate,
            self.fact_data_pool.hit_rate,
            self.field_map_pool.hit_rate,
            self.fact_vec_pool.hit_rate,
            self.fact_id_set_pool.hit_rate,
        ];

        hit_rates.iter().sum::<f64>() / hit_rates.len() as f64
    }

    /// Get total peak memory usage across all pools
    pub fn total_peak_objects(&self) -> usize {
        self.token_pool.peak_size
            + self.fact_data_pool.peak_size
            + self.field_map_pool.peak_size
            + self.fact_vec_pool.peak_size
            + self.fact_id_set_pool.peak_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FactValue;

    #[test]
    fn test_object_pool_basic_operations() {
        let mut pool = ObjectPool::new(Vec::<i32>::new, 5);

        // Get object from empty pool (should create new)
        let mut vec1 = pool.get();
        assert_eq!(pool.pool_hits, 0);
        assert_eq!(pool.pool_misses, 1);

        // Use the object
        vec1.push(42);
        assert_eq!(vec1.len(), 1);

        // Return object to pool
        pool.return_object(vec1);
        assert_eq!(pool.returned_count, 1);
        assert_eq!(pool.size(), 1);

        // Get object from pool (should reuse)
        let vec2 = pool.get();
        assert_eq!(pool.pool_hits, 1);
        assert_eq!(pool.pool_misses, 1);
        assert_eq!(vec2.len(), 1); // Should have previous state

        // Test utilization
        assert_eq!(pool.utilization(), 50.0); // 1 hit / 2 total = 50%
    }

    #[test]
    fn test_object_pool_max_size() {
        let mut pool = ObjectPool::new(String::new, 2);

        // Fill pool to capacity
        pool.return_object(String::from("test1"));
        pool.return_object(String::from("test2"));
        assert_eq!(pool.size(), 2);

        // Try to add one more (should be dropped)
        pool.return_object(String::from("test3"));
        assert_eq!(pool.size(), 2); // Still 2, third was dropped

        // Peak size should track maximum
        assert_eq!(pool.peak_size, 2);
    }

    #[test]
    fn test_thread_safe_pool() {
        let pool = ThreadSafePool::new(Vec::<String>::new, 10);

        // Get and return objects
        let mut vec = pool.get();
        vec.push("test".to_string());
        pool.return_object(vec);

        // Check statistics
        let stats = pool.stats();
        assert_eq!(stats.allocated_count, 1);
        assert_eq!(stats.returned_count, 1);
        assert_eq!(stats.pool_misses, 1); // First get is always a miss
    }

    #[test]
    fn test_rete_memory_pools() {
        let pools = ReteMemoryPools::new();

        // Test token pool
        let mut token = pools.get_token();
        token.fact_ids = FactIdSet::new(vec![1, 2, 3]);
        pools.return_token(token);

        // Get another token (should be reused and cleared)
        let reused_token = pools.get_token();
        assert!(reused_token.fact_ids.is_empty());

        // Test fact data pool
        let mut fact_data = pools.get_fact_data();
        fact_data.fields.insert("test".to_string(), FactValue::Integer(42));
        pools.return_fact_data(fact_data);

        let reused_fact_data = pools.get_fact_data();
        assert!(reused_fact_data.fields.is_empty());

        // Test field map pool
        let mut field_map = pools.get_field_map();
        field_map.insert("key".to_string(), FactValue::String("value".to_string()));
        pools.return_field_map(field_map);

        let reused_map = pools.get_field_map();
        assert!(reused_map.is_empty());

        // Check overall efficiency
        let efficiency = pools.overall_efficiency();
        assert!((0.0..=100.0).contains(&efficiency));
    }

    #[test]
    fn test_pool_statistics() {
        let pools = ReteMemoryPools::new();

        // Generate some activity
        for _ in 0..10 {
            let token = pools.get_token();
            pools.return_token(token);

            let fact_data = pools.get_fact_data();
            pools.return_fact_data(fact_data);
        }

        let stats = pools.get_stats();

        // Should have operations recorded
        assert!(stats.total_operations() > 0);
        assert!(stats.average_hit_rate() >= 0.0);
        assert!(stats.total_pooled_objects() > 0);

        // Test individual pool stats
        assert!(stats.token_pool.memory_efficiency() >= 0.0);
        assert!(stats.fact_data_pool.total_operations() > 0);
    }

    #[test]
    fn test_pool_clearing() {
        let pools = ReteMemoryPools::new();

        // Generate activity
        let token = pools.get_token();
        pools.return_token(token);

        let fact_data = pools.get_fact_data();
        pools.return_fact_data(fact_data);

        // Verify pools have objects
        let stats_before = pools.get_stats();
        assert!(stats_before.total_pooled_objects() > 0);

        // Clear all pools
        pools.clear_all();

        // Verify pools are empty
        let stats_after = pools.get_stats();
        assert_eq!(stats_after.total_pooled_objects(), 0);
        assert_eq!(stats_after.total_operations(), 0);
    }

    #[test]
    fn test_custom_pool_sizes() {
        let pools = ReteMemoryPools::with_sizes(100, 50, 75, 25, 200);

        // Test that pools respect custom sizes by filling them
        for _ in 0..150 {
            let token = pools.get_token();
            pools.return_token(token);
        }

        let stats = pools.get_stats();

        // Token pool should be limited to 100
        assert!(stats.token_pool.current_size <= 100);
        assert!(stats.token_pool.peak_size <= 100);
    }

    #[test]
    fn test_memory_efficiency_calculation() {
        use tracing::debug;
        let mut pool = ObjectPool::new(Vec::<i32>::new, 5);

        // All gets should be misses initially
        for _ in 0..3 {
            let vec = pool.get();
            pool.return_object(vec);
        }

        // Now gets should be hits
        for _ in 0..2 {
            let vec = pool.get();
            pool.return_object(vec);
        }

        // Total: 5 allocated, 2 hits from pool
        // Memory efficiency should be 2/5 = 40%
        let stats = PoolStats {
            allocated_count: pool.allocated_count,
            returned_count: pool.returned_count,
            pool_hits: pool.pool_hits,
            pool_misses: pool.pool_misses,
            hit_rate: pool.hit_rate(),
            memory_usage_bytes: 0,
            current_size: pool.size(),
            peak_size: pool.peak_size,
        };

        debug!(
            allocated = stats.allocated_count,
            hits = stats.pool_hits,
            misses = stats.pool_misses,
            efficiency = %stats.memory_efficiency(),
            "Memory pool statistics"
        );

        // The actual efficiency is: 4 hits / 5 allocated = 80%
        // First 3 operations: miss, hit, hit (because objects are returned and reused)
        // Next 2 operations: hit, hit
        assert_eq!(stats.memory_efficiency(), 80.0);
    }
}
