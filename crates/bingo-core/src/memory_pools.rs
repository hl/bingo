//! Comprehensive memory pooling system for frequently allocated objects
//!
//! This module provides various memory pools to reduce allocation overhead
//! and improve performance in high-throughput scenarios.

use crate::rete_nodes::RuleExecutionResult;
use crate::types::{FactId, FactValue, RuleId};
use std::cell::RefCell;
use std::collections::HashMap;

/// Pool for Vec<RuleExecutionResult> to reduce allocation overhead during rule processing
#[derive(Debug, Clone)]
pub struct RuleExecutionResultPool {
    pool: RefCell<Vec<Vec<RuleExecutionResult>>>,
    hits: RefCell<usize>,
    misses: RefCell<usize>,
    max_pool_size: usize,
}

impl RuleExecutionResultPool {
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            pool: RefCell::new(Vec::with_capacity(max_size / 4)),
            hits: RefCell::new(0),
            misses: RefCell::new(0),
            max_pool_size: max_size,
        }
    }

    /// Get a Vec<RuleExecutionResult> from the pool
    pub fn get(&self) -> Vec<RuleExecutionResult> {
        if let Some(mut vec) = self.pool.borrow_mut().pop() {
            vec.clear();
            *self.hits.borrow_mut() += 1;
            vec
        } else {
            *self.misses.borrow_mut() += 1;
            Vec::new()
        }
    }

    /// Return a Vec<RuleExecutionResult> to the pool
    pub fn return_vec(&self, vec: Vec<RuleExecutionResult>) {
        if self.pool.borrow().len() < self.max_pool_size {
            self.pool.borrow_mut().push(vec);
        }
    }

    /// Get pool statistics (hits, misses, pool_size)
    pub fn stats(&self) -> (usize, usize, usize) {
        (
            *self.hits.borrow(),
            *self.misses.borrow(),
            self.pool.borrow().len(),
        )
    }

    /// Calculate hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let hits = *self.hits.borrow() as f64;
        let total = hits + *self.misses.borrow() as f64;
        if total > 0.0 {
            (hits / total) * 100.0
        } else {
            0.0
        }
    }
}

impl Default for RuleExecutionResultPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Pool for Vec<RuleId> to reduce allocation overhead during rule matching
#[derive(Debug, Clone)]
pub struct RuleIdVecPool {
    pool: RefCell<Vec<Vec<RuleId>>>,
    hits: RefCell<usize>,
    misses: RefCell<usize>,
    max_pool_size: usize,
}

impl RuleIdVecPool {
    pub fn new() -> Self {
        Self::with_capacity(200)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            pool: RefCell::new(Vec::with_capacity(max_size / 4)),
            hits: RefCell::new(0),
            misses: RefCell::new(0),
            max_pool_size: max_size,
        }
    }

    /// Get a Vec<RuleId> from the pool
    pub fn get(&self) -> Vec<RuleId> {
        if let Some(mut vec) = self.pool.borrow_mut().pop() {
            vec.clear();
            *self.hits.borrow_mut() += 1;
            vec
        } else {
            *self.misses.borrow_mut() += 1;
            Vec::new()
        }
    }

    /// Return a Vec<RuleId> to the pool
    pub fn return_vec(&self, vec: Vec<RuleId>) {
        if self.pool.borrow().len() < self.max_pool_size {
            self.pool.borrow_mut().push(vec);
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        (
            *self.hits.borrow(),
            *self.misses.borrow(),
            self.pool.borrow().len(),
        )
    }

    /// Calculate hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let hits = *self.hits.borrow() as f64;
        let total = hits + *self.misses.borrow() as f64;
        if total > 0.0 {
            (hits / total) * 100.0
        } else {
            0.0
        }
    }
}

impl Default for RuleIdVecPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Pool for Vec<FactId> to reduce allocation overhead during fact processing
#[derive(Debug, Clone)]
pub struct FactIdVecPool {
    pool: RefCell<Vec<Vec<FactId>>>,
    hits: RefCell<usize>,
    misses: RefCell<usize>,
    max_pool_size: usize,
}

impl FactIdVecPool {
    pub fn new() -> Self {
        Self::with_capacity(300)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            pool: RefCell::new(Vec::with_capacity(max_size / 4)),
            hits: RefCell::new(0),
            misses: RefCell::new(0),
            max_pool_size: max_size,
        }
    }

    /// Get a Vec<FactId> from the pool
    pub fn get(&self) -> Vec<FactId> {
        if let Some(mut vec) = self.pool.borrow_mut().pop() {
            vec.clear();
            *self.hits.borrow_mut() += 1;
            vec
        } else {
            *self.misses.borrow_mut() += 1;
            Vec::new()
        }
    }

    /// Return a Vec<FactId> to the pool
    pub fn return_vec(&self, vec: Vec<FactId>) {
        if self.pool.borrow().len() < self.max_pool_size {
            self.pool.borrow_mut().push(vec);
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        (
            *self.hits.borrow(),
            *self.misses.borrow(),
            self.pool.borrow().len(),
        )
    }

    /// Calculate hit rate as percentage  
    pub fn hit_rate(&self) -> f64 {
        let hits = *self.hits.borrow() as f64;
        let total = hits + *self.misses.borrow() as f64;
        if total > 0.0 {
            (hits / total) * 100.0
        } else {
            0.0
        }
    }
}

impl Default for FactIdVecPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Pool for HashMap<String, FactValue> used in fact processing and aggregations
#[derive(Debug, Clone)]
pub struct FactFieldMapPool {
    pool: RefCell<Vec<HashMap<String, FactValue>>>,
    hits: RefCell<usize>,
    misses: RefCell<usize>,
    max_pool_size: usize,
}

impl FactFieldMapPool {
    pub fn new() -> Self {
        Self::with_capacity(150)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            pool: RefCell::new(Vec::with_capacity(max_size / 4)),
            hits: RefCell::new(0),
            misses: RefCell::new(0),
            max_pool_size: max_size,
        }
    }

    /// Get a HashMap<String, FactValue> from the pool
    pub fn get(&self) -> HashMap<String, FactValue> {
        if let Some(mut map) = self.pool.borrow_mut().pop() {
            map.clear();
            *self.hits.borrow_mut() += 1;
            map
        } else {
            *self.misses.borrow_mut() += 1;
            HashMap::new()
        }
    }

    /// Return a HashMap<String, FactValue> to the pool
    pub fn return_map(&self, map: HashMap<String, FactValue>) {
        if self.pool.borrow().len() < self.max_pool_size {
            self.pool.borrow_mut().push(map);
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        (
            *self.hits.borrow(),
            *self.misses.borrow(),
            self.pool.borrow().len(),
        )
    }

    /// Calculate hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let hits = *self.hits.borrow() as f64;
        let total = hits + *self.misses.borrow() as f64;
        if total > 0.0 {
            (hits / total) * 100.0
        } else {
            0.0
        }
    }
}

impl Default for FactFieldMapPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Pool for Vec<f64> used in aggregation calculations
#[derive(Debug, Clone)]
pub struct NumericVecPool {
    pool: RefCell<Vec<Vec<f64>>>,
    hits: RefCell<usize>,
    misses: RefCell<usize>,
    max_pool_size: usize,
}

impl NumericVecPool {
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            pool: RefCell::new(Vec::with_capacity(max_size / 4)),
            hits: RefCell::new(0),
            misses: RefCell::new(0),
            max_pool_size: max_size,
        }
    }

    /// Get a Vec<f64> from the pool
    pub fn get(&self) -> Vec<f64> {
        if let Some(mut vec) = self.pool.borrow_mut().pop() {
            vec.clear();
            *self.hits.borrow_mut() += 1;
            vec
        } else {
            *self.misses.borrow_mut() += 1;
            Vec::new()
        }
    }

    /// Return a Vec<f64> to the pool
    pub fn return_vec(&self, vec: Vec<f64>) {
        if self.pool.borrow().len() < self.max_pool_size {
            self.pool.borrow_mut().push(vec);
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        (
            *self.hits.borrow(),
            *self.misses.borrow(),
            self.pool.borrow().len(),
        )
    }

    /// Calculate hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let hits = *self.hits.borrow() as f64;
        let total = hits + *self.misses.borrow() as f64;
        if total > 0.0 {
            (hits / total) * 100.0
        } else {
            0.0
        }
    }
}

impl Default for NumericVecPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Centralized memory pool manager that coordinates all pools
#[derive(Debug, Clone)]
pub struct MemoryPoolManager {
    pub rule_execution_results: RuleExecutionResultPool,
    pub rule_id_vecs: RuleIdVecPool,
    pub fact_id_vecs: FactIdVecPool,
    pub fact_field_maps: FactFieldMapPool,
    pub numeric_vecs: NumericVecPool,
}

impl MemoryPoolManager {
    /// Create a new memory pool manager with default capacities
    pub fn new() -> Self {
        Self {
            rule_execution_results: RuleExecutionResultPool::new(),
            rule_id_vecs: RuleIdVecPool::new(),
            fact_id_vecs: FactIdVecPool::new(),
            fact_field_maps: FactFieldMapPool::new(),
            numeric_vecs: NumericVecPool::new(),
        }
    }

    /// Create a memory pool manager with custom capacities for high-throughput scenarios
    pub fn with_high_throughput_config() -> Self {
        Self {
            rule_execution_results: RuleExecutionResultPool::with_capacity(500),
            rule_id_vecs: RuleIdVecPool::with_capacity(1000),
            fact_id_vecs: FactIdVecPool::with_capacity(1500),
            fact_field_maps: FactFieldMapPool::with_capacity(800),
            numeric_vecs: NumericVecPool::with_capacity(400),
        }
    }

    /// Get comprehensive statistics for all pools
    pub fn get_comprehensive_stats(&self) -> MemoryPoolStats {
        let (rer_hits, rer_misses, rer_size) = self.rule_execution_results.stats();
        let (rid_hits, rid_misses, rid_size) = self.rule_id_vecs.stats();
        let (fid_hits, fid_misses, fid_size) = self.fact_id_vecs.stats();
        let (ffm_hits, ffm_misses, ffm_size) = self.fact_field_maps.stats();
        let (nv_hits, nv_misses, nv_size) = self.numeric_vecs.stats();

        MemoryPoolStats {
            rule_execution_result_pool: PoolStats {
                hits: rer_hits,
                misses: rer_misses,
                pool_size: rer_size,
                hit_rate: self.rule_execution_results.hit_rate(),
            },
            rule_id_vec_pool: PoolStats {
                hits: rid_hits,
                misses: rid_misses,
                pool_size: rid_size,
                hit_rate: self.rule_id_vecs.hit_rate(),
            },
            fact_id_vec_pool: PoolStats {
                hits: fid_hits,
                misses: fid_misses,
                pool_size: fid_size,
                hit_rate: self.fact_id_vecs.hit_rate(),
            },
            fact_field_map_pool: PoolStats {
                hits: ffm_hits,
                misses: ffm_misses,
                pool_size: ffm_size,
                hit_rate: self.fact_field_maps.hit_rate(),
            },
            numeric_vec_pool: PoolStats {
                hits: nv_hits,
                misses: nv_misses,
                pool_size: nv_size,
                hit_rate: self.numeric_vecs.hit_rate(),
            },
        }
    }

    /// Calculate overall memory pool efficiency
    pub fn overall_efficiency(&self) -> f64 {
        let stats = self.get_comprehensive_stats();
        let total_hits = stats.rule_execution_result_pool.hits
            + stats.rule_id_vec_pool.hits
            + stats.fact_id_vec_pool.hits
            + stats.fact_field_map_pool.hits
            + stats.numeric_vec_pool.hits;

        let total_requests = total_hits
            + stats.rule_execution_result_pool.misses
            + stats.rule_id_vec_pool.misses
            + stats.fact_id_vec_pool.misses
            + stats.fact_field_map_pool.misses
            + stats.numeric_vec_pool.misses;

        if total_requests > 0 {
            (total_hits as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        }
    }
}

impl Default for MemoryPoolManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for individual memory pools
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub hits: usize,
    pub misses: usize,
    pub pool_size: usize,
    pub hit_rate: f64,
}

/// Comprehensive statistics for all memory pools
#[derive(Debug, Clone)]
pub struct MemoryPoolStats {
    pub rule_execution_result_pool: PoolStats,
    pub rule_id_vec_pool: PoolStats,
    pub fact_id_vec_pool: PoolStats,
    pub fact_field_map_pool: PoolStats,
    pub numeric_vec_pool: PoolStats,
}

impl MemoryPoolStats {
    /// Calculate total memory saved through pooling (rough estimate)
    pub fn estimated_memory_saved_bytes(&self) -> usize {
        // Rough estimates based on typical allocation sizes
        let rer_saved = self.rule_execution_result_pool.hits * 128; // ~128 bytes per RuleExecutionResult vec
        let rid_saved = self.rule_id_vec_pool.hits * 64; // ~64 bytes per RuleId vec
        let fid_saved = self.fact_id_vec_pool.hits * 64; // ~64 bytes per FactId vec
        let ffm_saved = self.fact_field_map_pool.hits * 256; // ~256 bytes per HashMap
        let nv_saved = self.numeric_vec_pool.hits * 96; // ~96 bytes per numeric vec

        rer_saved + rid_saved + fid_saved + ffm_saved + nv_saved
    }

    /// Get total pool utilization
    pub fn total_pool_utilization(&self) -> f64 {
        let total_capacity = 100 + 200 + 300 + 150 + 100; // Default capacities
        let total_used = self.rule_execution_result_pool.pool_size
            + self.rule_id_vec_pool.pool_size
            + self.fact_id_vec_pool.pool_size
            + self.fact_field_map_pool.pool_size
            + self.numeric_vec_pool.pool_size;

        (total_used as f64 / total_capacity as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_execution_result_pool() {
        let pool = RuleExecutionResultPool::new();

        // First get should be a miss
        let vec1 = pool.get();
        assert!(vec1.is_empty());

        // Return the vec
        pool.return_vec(vec1);

        // Second get should be a hit
        let vec2 = pool.get();
        assert!(vec2.is_empty());

        let (hits, misses, pool_size) = pool.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
        assert_eq!(pool_size, 0); // vec2 is still out

        assert_eq!(pool.hit_rate(), 50.0);
    }

    #[test]
    fn test_memory_pool_manager() {
        let manager = MemoryPoolManager::new();

        // Test each pool
        let _rule_results = manager.rule_execution_results.get();
        let _rule_ids = manager.rule_id_vecs.get();
        let _fact_ids = manager.fact_id_vecs.get();
        let _fact_map = manager.fact_field_maps.get();
        let _numbers = manager.numeric_vecs.get();

        let stats = manager.get_comprehensive_stats();

        // All should be misses initially
        assert_eq!(stats.rule_execution_result_pool.misses, 1);
        assert_eq!(stats.rule_id_vec_pool.misses, 1);
        assert_eq!(stats.fact_id_vec_pool.misses, 1);
        assert_eq!(stats.fact_field_map_pool.misses, 1);
        assert_eq!(stats.numeric_vec_pool.misses, 1);

        // Overall efficiency should be 0% (all misses)
        assert_eq!(manager.overall_efficiency(), 0.0);
    }

    #[test]
    fn test_high_throughput_config() {
        let manager = MemoryPoolManager::with_high_throughput_config();

        // Test that it can handle many allocations efficiently
        for _ in 0..100 {
            let vec = manager.rule_id_vecs.get();
            manager.rule_id_vecs.return_vec(vec);
        }

        let stats = manager.get_comprehensive_stats();
        assert!(stats.rule_id_vec_pool.hit_rate > 90.0); // Should have high hit rate
    }

    #[test]
    fn test_memory_savings_estimation() {
        let manager = MemoryPoolManager::new();

        // Simulate some pool usage
        for _ in 0..10 {
            let vec = manager.rule_execution_results.get();
            manager.rule_execution_results.return_vec(vec);
        }

        let stats = manager.get_comprehensive_stats();
        let memory_saved = stats.estimated_memory_saved_bytes();

        // Should estimate some memory savings
        assert!(memory_saved > 0);
    }
}
