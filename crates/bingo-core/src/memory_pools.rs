//! Comprehensive memory pooling system for frequently allocated objects
//!
//! This module provides various memory pools to reduce allocation overhead
//! and improve performance in high-throughput scenarios.

use crate::beta_network::Token;
use crate::rete_nodes::RuleExecutionResult;
use crate::types::{FactId, FactValue, RuleId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Pool for `Vec<RuleExecutionResult>` to reduce allocation overhead during rule processing
/// Thread-safe implementation using Arc<Mutex<T>> and atomic counters
#[derive(Debug, Clone)]
pub struct RuleExecutionResultPool {
    pool: Arc<Mutex<Vec<Vec<RuleExecutionResult>>>>,
    hits: Arc<AtomicUsize>,
    misses: Arc<AtomicUsize>,
    max_pool_size: usize,
}

impl RuleExecutionResultPool {
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_size / 4))),
            hits: Arc::new(AtomicUsize::new(0)),
            misses: Arc::new(AtomicUsize::new(0)),
            max_pool_size: max_size,
        }
    }

    /// Get a `Vec<RuleExecutionResult>` from the pool
    pub fn get(&self) -> Vec<RuleExecutionResult> {
        if let Ok(mut pool) = self.pool.lock() {
            if let Some(mut vec) = pool.pop() {
                vec.clear();
                self.hits.fetch_add(1, Ordering::Relaxed);
                vec
            } else {
                self.misses.fetch_add(1, Ordering::Relaxed);
                Vec::new()
            }
        } else {
            // If lock is poisoned, create a new vector
            self.misses.fetch_add(1, Ordering::Relaxed);
            Vec::new()
        }
    }

    /// Return a `Vec<RuleExecutionResult>` to the pool
    pub fn return_vec(&self, vec: Vec<RuleExecutionResult>) {
        if let Ok(mut pool) = self.pool.lock() {
            if pool.len() < self.max_pool_size {
                pool.push(vec);
            }
        }
        // If lock is poisoned, just drop the vector
    }

    /// Get pool statistics (hits, misses, pool_size)
    pub fn stats(&self) -> (usize, usize, usize) {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let pool_size = self.pool.lock().map(|pool| pool.len()).unwrap_or(0);
        (hits, misses, pool_size)
    }

    /// Calculate hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let total = hits + self.misses.load(Ordering::Relaxed) as f64;
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

/// Pool for `Vec<RuleId>` to reduce allocation overhead during rule matching
#[derive(Debug, Clone)]
pub struct RuleIdVecPool {
    pool: Arc<Mutex<Vec<Vec<RuleId>>>>,
    hits: Arc<AtomicUsize>,
    misses: Arc<AtomicUsize>,
    max_pool_size: usize,
}

impl RuleIdVecPool {
    pub fn new() -> Self {
        Self::with_capacity(200)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_size / 4))),
            hits: Arc::new(AtomicUsize::new(0)),
            misses: Arc::new(AtomicUsize::new(0)),
            max_pool_size: max_size,
        }
    }

    /// Get a `Vec<RuleId>` from the pool
    pub fn get(&self) -> Vec<RuleId> {
        if let Ok(mut pool) = self.pool.lock() {
            if let Some(mut vec) = pool.pop() {
                vec.clear();
                self.hits.fetch_add(1, Ordering::Relaxed);
                vec
            } else {
                self.misses.fetch_add(1, Ordering::Relaxed);
                Vec::new()
            }
        } else {
            // If lock is poisoned, create a new vector
            self.misses.fetch_add(1, Ordering::Relaxed);
            Vec::new()
        }
    }

    /// Return a `Vec<RuleId>` to the pool
    pub fn return_vec(&self, vec: Vec<RuleId>) {
        if let Ok(mut pool) = self.pool.lock() {
            if pool.len() < self.max_pool_size {
                pool.push(vec);
            }
        }
        // If lock is poisoned, just drop the vector
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let pool_size = self.pool.lock().map(|pool| pool.len()).unwrap_or(0);
        (hits, misses, pool_size)
    }

    /// Calculate hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let total = hits + self.misses.load(Ordering::Relaxed) as f64;
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

/// Pool for `Vec<FactId>` to reduce allocation overhead during fact processing
#[derive(Debug, Clone)]
pub struct FactIdVecPool {
    pool: Arc<Mutex<Vec<Vec<FactId>>>>,
    hits: Arc<AtomicUsize>,
    misses: Arc<AtomicUsize>,
    max_pool_size: usize,
}

impl FactIdVecPool {
    pub fn new() -> Self {
        Self::with_capacity(300)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_size / 4))),
            hits: Arc::new(AtomicUsize::new(0)),
            misses: Arc::new(AtomicUsize::new(0)),
            max_pool_size: max_size,
        }
    }

    /// Get a `Vec<FactId>` from the pool
    pub fn get(&self) -> Vec<FactId> {
        if let Ok(mut pool) = self.pool.lock() {
            if let Some(mut vec) = pool.pop() {
                vec.clear();
                self.hits.fetch_add(1, Ordering::Relaxed);
                vec
            } else {
                self.misses.fetch_add(1, Ordering::Relaxed);
                Vec::new()
            }
        } else {
            // If lock is poisoned, create a new vector
            self.misses.fetch_add(1, Ordering::Relaxed);
            Vec::new()
        }
    }

    /// Return a `Vec<FactId>` to the pool
    pub fn return_vec(&self, vec: Vec<FactId>) {
        if let Ok(mut pool) = self.pool.lock() {
            if pool.len() < self.max_pool_size {
                pool.push(vec);
            }
        }
        // If lock is poisoned, just drop the vector
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let pool_size = self.pool.lock().map(|pool| pool.len()).unwrap_or(0);
        (hits, misses, pool_size)
    }

    /// Calculate hit rate as percentage  
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let total = hits + self.misses.load(Ordering::Relaxed) as f64;
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
    pool: Arc<Mutex<Vec<HashMap<String, FactValue>>>>,
    hits: Arc<AtomicUsize>,
    misses: Arc<AtomicUsize>,
    max_pool_size: usize,
}

impl FactFieldMapPool {
    pub fn new() -> Self {
        Self::with_capacity(150)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_size / 4))),
            hits: Arc::new(AtomicUsize::new(0)),
            misses: Arc::new(AtomicUsize::new(0)),
            max_pool_size: max_size,
        }
    }

    /// Get a HashMap<String, FactValue> from the pool
    pub fn get(&self) -> HashMap<String, FactValue> {
        if let Ok(mut pool) = self.pool.lock() {
            if let Some(mut map) = pool.pop() {
                map.clear();
                self.hits.fetch_add(1, Ordering::Relaxed);
                map
            } else {
                self.misses.fetch_add(1, Ordering::Relaxed);
                HashMap::new()
            }
        } else {
            // If lock is poisoned, create a new map
            self.misses.fetch_add(1, Ordering::Relaxed);
            HashMap::new()
        }
    }

    /// Return a HashMap<String, FactValue> to the pool
    pub fn return_map(&self, map: HashMap<String, FactValue>) {
        if let Ok(mut pool) = self.pool.lock() {
            if pool.len() < self.max_pool_size {
                pool.push(map);
            }
        }
        // If lock is poisoned, just drop the map
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let pool_size = self.pool.lock().map(|pool| pool.len()).unwrap_or(0);
        (hits, misses, pool_size)
    }

    /// Calculate hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let total = hits + self.misses.load(Ordering::Relaxed) as f64;
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

/// Pool for `Vec<f64>` used in aggregation calculations
#[derive(Debug, Clone)]
pub struct NumericVecPool {
    pool: Arc<Mutex<Vec<Vec<f64>>>>,
    hits: Arc<AtomicUsize>,
    misses: Arc<AtomicUsize>,
    max_pool_size: usize,
}

impl NumericVecPool {
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_size / 4))),
            hits: Arc::new(AtomicUsize::new(0)),
            misses: Arc::new(AtomicUsize::new(0)),
            max_pool_size: max_size,
        }
    }

    /// Get a `Vec<f64>` from the pool
    pub fn get(&self) -> Vec<f64> {
        if let Ok(mut pool) = self.pool.lock() {
            if let Some(mut vec) = pool.pop() {
                vec.clear();
                self.hits.fetch_add(1, Ordering::Relaxed);
                vec
            } else {
                self.misses.fetch_add(1, Ordering::Relaxed);
                Vec::new()
            }
        } else {
            // If lock is poisoned, create a new vector
            self.misses.fetch_add(1, Ordering::Relaxed);
            Vec::new()
        }
    }

    /// Return a `Vec<f64>` to the pool
    pub fn return_vec(&self, vec: Vec<f64>) {
        if let Ok(mut pool) = self.pool.lock() {
            if pool.len() < self.max_pool_size {
                pool.push(vec);
            }
        }
        // If lock is poisoned, just drop the vector
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let pool_size = self.pool.lock().map(|pool| pool.len()).unwrap_or(0);
        (hits, misses, pool_size)
    }

    /// Calculate hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let total = hits + self.misses.load(Ordering::Relaxed) as f64;
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

/// Pool for `Vec<Token>` used in beta network processing
#[derive(Debug, Clone)]
pub struct TokenVecPool {
    pool: Arc<Mutex<Vec<Vec<Token>>>>,
    hits: Arc<AtomicUsize>,
    misses: Arc<AtomicUsize>,
    max_pool_size: usize,
}

impl TokenVecPool {
    pub fn new() -> Self {
        Self::with_capacity(200)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_size / 4))),
            hits: Arc::new(AtomicUsize::new(0)),
            misses: Arc::new(AtomicUsize::new(0)),
            max_pool_size: max_size,
        }
    }

    /// Get a `Vec<Token>` from the pool
    pub fn get(&self) -> Vec<Token> {
        if let Ok(mut pool) = self.pool.lock() {
            if let Some(mut vec) = pool.pop() {
                vec.clear();
                self.hits.fetch_add(1, Ordering::Relaxed);
                vec
            } else {
                self.misses.fetch_add(1, Ordering::Relaxed);
                Vec::new()
            }
        } else {
            // If lock is poisoned, create a new vector
            self.misses.fetch_add(1, Ordering::Relaxed);
            Vec::new()
        }
    }

    /// Return a `Vec<Token>` to the pool
    pub fn return_vec(&self, vec: Vec<Token>) {
        if let Ok(mut pool) = self.pool.lock() {
            if pool.len() < self.max_pool_size {
                pool.push(vec);
            }
        }
        // If lock is poisoned, just drop the vector
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let pool_size = self.pool.lock().map(|pool| pool.len()).unwrap_or(0);
        (hits, misses, pool_size)
    }

    /// Calculate hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let total = hits + self.misses.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            (hits / total) * 100.0
        } else {
            0.0
        }
    }
}

impl Default for TokenVecPool {
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
    pub token_vecs: TokenVecPool,
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
            token_vecs: TokenVecPool::new(),
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
            token_vecs: TokenVecPool::with_capacity(600),
        }
    }

    /// Get comprehensive statistics for all pools
    pub fn get_comprehensive_stats(&self) -> MemoryPoolStats {
        let (rer_hits, rer_misses, rer_size) = self.rule_execution_results.stats();
        let (rid_hits, rid_misses, rid_size) = self.rule_id_vecs.stats();
        let (fid_hits, fid_misses, fid_size) = self.fact_id_vecs.stats();
        let (ffm_hits, ffm_misses, ffm_size) = self.fact_field_maps.stats();
        let (nv_hits, nv_misses, nv_size) = self.numeric_vecs.stats();
        let (tv_hits, tv_misses, tv_size) = self.token_vecs.stats();

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
            token_vec_pool: PoolStats {
                hits: tv_hits,
                misses: tv_misses,
                pool_size: tv_size,
                hit_rate: self.token_vecs.hit_rate(),
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
            + stats.numeric_vec_pool.hits
            + stats.token_vec_pool.hits;

        let total_requests = total_hits
            + stats.rule_execution_result_pool.misses
            + stats.rule_id_vec_pool.misses
            + stats.fact_id_vec_pool.misses
            + stats.fact_field_map_pool.misses
            + stats.numeric_vec_pool.misses
            + stats.token_vec_pool.misses;

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
    pub token_vec_pool: PoolStats,
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
        let tv_saved = self.token_vec_pool.hits * 80; // ~80 bytes per Token vec

        rer_saved + rid_saved + fid_saved + ffm_saved + nv_saved + tv_saved
    }

    /// Get total pool utilization
    pub fn total_pool_utilization(&self) -> f64 {
        let total_capacity = 100 + 200 + 300 + 150 + 100 + 200; // Default capacities including token pool
        let total_used = self.rule_execution_result_pool.pool_size
            + self.rule_id_vec_pool.pool_size
            + self.fact_id_vec_pool.pool_size
            + self.fact_field_map_pool.pool_size
            + self.numeric_vec_pool.pool_size
            + self.token_vec_pool.pool_size;

        (total_used as f64 / total_capacity as f64) * 100.0
    }
}

// ============================================================================
// CONCURRENT MEMORY POOLS
// ============================================================================
// Thread-safe memory pools for concurrent access in parallel processing scenarios

/// Thread-safe pool for `Vec<RuleExecutionResult>` with atomic statistics
#[derive(Debug)]
pub struct ConcurrentRuleExecutionResultPool {
    pool: Arc<Mutex<Vec<Vec<RuleExecutionResult>>>>,
    hits: Arc<AtomicUsize>,
    misses: Arc<AtomicUsize>,
    max_pool_size: usize,
}

impl ConcurrentRuleExecutionResultPool {
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_size / 4))),
            hits: Arc::new(AtomicUsize::new(0)),
            misses: Arc::new(AtomicUsize::new(0)),
            max_pool_size: max_size,
        }
    }

    /// Get a `Vec<RuleExecutionResult>` from the pool (thread-safe)
    pub fn get(&self) -> Vec<RuleExecutionResult> {
        if let Ok(mut pool) = self.pool.lock() {
            if let Some(mut vec) = pool.pop() {
                vec.clear();
                self.hits.fetch_add(1, Ordering::Relaxed);
                return vec;
            }
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        Vec::new()
    }

    /// Return a `Vec<RuleExecutionResult>` to the pool (thread-safe)
    pub fn return_vec(&self, vec: Vec<RuleExecutionResult>) {
        if let Ok(mut pool) = self.pool.lock() {
            if pool.len() < self.max_pool_size {
                pool.push(vec);
            }
        }
    }

    /// Get pool statistics (hits, misses, pool_size)
    pub fn stats(&self) -> (usize, usize, usize) {
        let pool_size = self.pool.lock().map(|p| p.len()).unwrap_or(0);
        (
            self.hits.load(Ordering::Relaxed),
            self.misses.load(Ordering::Relaxed),
            pool_size,
        )
    }

    /// Calculate hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let misses = self.misses.load(Ordering::Relaxed) as f64;
        let total = hits + misses;
        if total > 0.0 {
            (hits / total) * 100.0
        } else {
            0.0
        }
    }
}

impl Clone for ConcurrentRuleExecutionResultPool {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
            hits: Arc::clone(&self.hits),
            misses: Arc::clone(&self.misses),
            max_pool_size: self.max_pool_size,
        }
    }
}

impl Default for ConcurrentRuleExecutionResultPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe pool for `Vec<RuleId>` with atomic statistics
#[derive(Debug)]
pub struct ConcurrentRuleIdVecPool {
    pool: Arc<Mutex<Vec<Vec<RuleId>>>>,
    hits: Arc<AtomicUsize>,
    misses: Arc<AtomicUsize>,
    max_pool_size: usize,
}

impl ConcurrentRuleIdVecPool {
    pub fn new() -> Self {
        Self::with_capacity(200)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_size / 4))),
            hits: Arc::new(AtomicUsize::new(0)),
            misses: Arc::new(AtomicUsize::new(0)),
            max_pool_size: max_size,
        }
    }

    /// Get a `Vec<RuleId>` from the pool (thread-safe)
    pub fn get(&self) -> Vec<RuleId> {
        if let Ok(mut pool) = self.pool.lock() {
            if let Some(mut vec) = pool.pop() {
                vec.clear();
                self.hits.fetch_add(1, Ordering::Relaxed);
                return vec;
            }
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        Vec::new()
    }

    /// Return a `Vec<RuleId>` to the pool (thread-safe)
    pub fn return_vec(&self, vec: Vec<RuleId>) {
        if let Ok(mut pool) = self.pool.lock() {
            if pool.len() < self.max_pool_size {
                pool.push(vec);
            }
        }
    }

    /// Get pool statistics (hits, misses, pool_size)
    pub fn stats(&self) -> (usize, usize, usize) {
        let pool_size = self.pool.lock().map(|p| p.len()).unwrap_or(0);
        (
            self.hits.load(Ordering::Relaxed),
            self.misses.load(Ordering::Relaxed),
            pool_size,
        )
    }

    /// Calculate hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let misses = self.misses.load(Ordering::Relaxed) as f64;
        let total = hits + misses;
        if total > 0.0 {
            (hits / total) * 100.0
        } else {
            0.0
        }
    }
}

impl Clone for ConcurrentRuleIdVecPool {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
            hits: Arc::clone(&self.hits),
            misses: Arc::clone(&self.misses),
            max_pool_size: self.max_pool_size,
        }
    }
}

impl Default for ConcurrentRuleIdVecPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe centralized memory pool manager for concurrent scenarios
#[derive(Debug)]
pub struct ConcurrentMemoryPoolManager {
    pub rule_execution_results: ConcurrentRuleExecutionResultPool,
    pub rule_id_vecs: ConcurrentRuleIdVecPool,
    pub enabled: Arc<AtomicUsize>, // 1 = enabled, 0 = disabled
}

impl ConcurrentMemoryPoolManager {
    /// Create a new concurrent memory pool manager with default capacities
    pub fn new() -> Self {
        Self {
            rule_execution_results: ConcurrentRuleExecutionResultPool::new(),
            rule_id_vecs: ConcurrentRuleIdVecPool::new(),
            enabled: Arc::new(AtomicUsize::new(1)),
        }
    }

    /// Create a concurrent memory pool manager with high-throughput configuration
    pub fn with_high_throughput_config() -> Self {
        Self {
            rule_execution_results: ConcurrentRuleExecutionResultPool::with_capacity(1000),
            rule_id_vecs: ConcurrentRuleIdVecPool::with_capacity(2000),
            enabled: Arc::new(AtomicUsize::new(1)),
        }
    }

    /// Enable or disable concurrent memory pooling
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(if enabled { 1 } else { 0 }, Ordering::Relaxed);
    }

    /// Check if concurrent memory pooling is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed) == 1
    }

    /// Get comprehensive statistics for all concurrent pools
    pub fn get_concurrent_stats(&self) -> ConcurrentMemoryPoolStats {
        let (rer_hits, rer_misses, rer_size) = self.rule_execution_results.stats();
        let (rid_hits, rid_misses, rid_size) = self.rule_id_vecs.stats();

        ConcurrentMemoryPoolStats {
            rule_execution_result_pool: ConcurrentPoolStats {
                hits: rer_hits,
                misses: rer_misses,
                pool_size: rer_size,
                hit_rate: self.rule_execution_results.hit_rate(),
            },
            rule_id_vec_pool: ConcurrentPoolStats {
                hits: rid_hits,
                misses: rid_misses,
                pool_size: rid_size,
                hit_rate: self.rule_id_vecs.hit_rate(),
            },
            enabled: self.is_enabled(),
        }
    }

    /// Calculate total memory efficiency across all pools
    pub fn total_efficiency(&self) -> f64 {
        let stats = self.get_concurrent_stats();
        let total_hits = stats.rule_execution_result_pool.hits + stats.rule_id_vec_pool.hits;
        let total_requests =
            total_hits + stats.rule_execution_result_pool.misses + stats.rule_id_vec_pool.misses;

        if total_requests > 0 {
            (total_hits as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        }
    }
}

impl Clone for ConcurrentMemoryPoolManager {
    fn clone(&self) -> Self {
        Self {
            rule_execution_results: self.rule_execution_results.clone(),
            rule_id_vecs: self.rule_id_vecs.clone(),
            enabled: Arc::clone(&self.enabled),
        }
    }
}

impl Default for ConcurrentMemoryPoolManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for individual concurrent pools
#[derive(Debug, Clone)]
pub struct ConcurrentPoolStats {
    pub hits: usize,
    pub misses: usize,
    pub pool_size: usize,
    pub hit_rate: f64,
}

/// Comprehensive statistics for all concurrent memory pools
#[derive(Debug, Clone)]
pub struct ConcurrentMemoryPoolStats {
    pub rule_execution_result_pool: ConcurrentPoolStats,
    pub rule_id_vec_pool: ConcurrentPoolStats,
    pub enabled: bool,
}

impl ConcurrentMemoryPoolStats {
    /// Calculate total memory saved through concurrent pooling
    pub fn estimated_memory_saved_bytes(&self) -> usize {
        let rer_saved = self.rule_execution_result_pool.hits * 128; // ~128 bytes per vec
        let rid_saved = self.rule_id_vec_pool.hits * 64; // ~64 bytes per vec
        rer_saved + rid_saved
    }

    /// Get average hit rate across all concurrent pools
    pub fn average_hit_rate(&self) -> f64 {
        (self.rule_execution_result_pool.hit_rate + self.rule_id_vec_pool.hit_rate) / 2.0
    }

    /// Get total concurrent pool utilization
    pub fn total_pool_utilization(&self) -> f64 {
        let total_capacity = 100 + 200; // Default capacities for concurrent pools
        let total_used =
            self.rule_execution_result_pool.pool_size + self.rule_id_vec_pool.pool_size;
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
        let _tokens = manager.token_vecs.get();

        let stats = manager.get_comprehensive_stats();

        // All should be misses initially
        assert_eq!(stats.rule_execution_result_pool.misses, 1);
        assert_eq!(stats.rule_id_vec_pool.misses, 1);
        assert_eq!(stats.fact_id_vec_pool.misses, 1);
        assert_eq!(stats.fact_field_map_pool.misses, 1);
        assert_eq!(stats.numeric_vec_pool.misses, 1);
        assert_eq!(stats.token_vec_pool.misses, 1);

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
