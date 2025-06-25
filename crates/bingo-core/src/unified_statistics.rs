//! Unified statistics system for all optimization layers
//!
//! This module consolidates performance statistics from fact stores, caches,
//! memory pools, calculators, and other optimization components into a single
//! unified reporting system.

use crate::cache::CacheStats;
use crate::field_indexing::FieldIndexStats;
use std::collections::HashMap;
use std::fmt;

/// Unified statistics collector for all optimization components
#[derive(Debug, Clone)]
pub struct UnifiedStats {
    /// Statistics for fact storage optimization
    pub fact_storage: FactStorageStats,
    /// Statistics for caching layers
    pub caching: CachingStats,
    /// Statistics for memory management
    pub memory: MemoryStats,
    /// Statistics for calculator optimization
    pub calculator: CalculatorStats,
    /// Statistics for field indexing
    pub indexing: IndexingStats,
    /// Component-specific counters
    pub component_counters: HashMap<String, ComponentStats>,
}

/// Statistics for fact storage operations
#[derive(Debug, Clone, Default)]
pub struct FactStorageStats {
    /// Total facts stored across all stores
    pub total_facts: usize,
    /// Total fact lookups performed
    pub total_lookups: usize,
    /// Facts stored by backend type
    pub facts_by_backend: HashMap<String, usize>,
    /// Average lookup time in microseconds
    pub avg_lookup_time_us: f64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: usize,
}

/// Statistics for all caching layers
#[derive(Debug, Clone, Default)]
pub struct CachingStats {
    /// Total cache hits across all caches
    pub total_hits: usize,
    /// Total cache misses across all caches
    pub total_misses: usize,
    /// Cache statistics by component
    pub cache_by_component: HashMap<String, CacheStats>,
    /// LRU evictions performed
    pub total_evictions: usize,
    /// Cache memory usage in bytes
    pub cache_memory_bytes: usize,
}

/// Statistics for memory management
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Object pool hits across all pools
    pub pool_hits: usize,
    /// Object pool misses across all pools
    pub pool_misses: usize,
    /// Objects currently allocated from pools
    pub pool_allocated: usize,
    /// Objects returned to pools
    pub pool_returned: usize,
    /// Peak pool sizes by object type
    pub peak_pool_sizes: HashMap<String, usize>,
    /// Total memory allocated in bytes
    pub total_allocated_bytes: usize,
}

/// Statistics for calculator optimization
#[derive(Debug, Clone, Default)]
pub struct CalculatorStats {
    /// Expression compilation cache hits
    pub compilation_hits: usize,
    /// Expression compilation cache misses
    pub compilation_misses: usize,
    /// Result cache hits
    pub result_hits: usize,
    /// Result cache misses
    pub result_misses: usize,
    /// Total expressions evaluated
    pub total_evaluations: usize,
    /// Average evaluation time in microseconds
    pub avg_evaluation_time_us: f64,
}

/// Statistics for field indexing optimization
#[derive(Debug, Clone, Default)]
pub struct IndexingStats {
    /// Number of indexed fields
    pub indexed_fields: usize,
    /// Total index entries across all fields
    pub total_index_entries: usize,
    /// Index memory usage in bytes
    pub index_memory_bytes: usize,
    /// Index lookup operations performed
    pub index_lookups: usize,
    /// Average facts per index entry
    pub avg_facts_per_entry: f64,
}

/// Generic component statistics
#[derive(Debug, Clone, Default)]
pub struct ComponentStats {
    /// Operations performed
    pub operations: usize,
    /// Successes
    pub successes: usize,
    /// Failures
    pub failures: usize,
    /// Average operation time in microseconds
    pub avg_time_us: f64,
    /// Component-specific metrics
    pub custom_metrics: HashMap<String, f64>,
}

impl UnifiedStats {
    /// Create a new unified statistics collector
    pub fn new() -> Self {
        Self {
            fact_storage: FactStorageStats::default(),
            caching: CachingStats::default(),
            memory: MemoryStats::default(),
            calculator: CalculatorStats::default(),
            indexing: IndexingStats::default(),
            component_counters: HashMap::new(),
        }
    }

    /// Register fact storage statistics
    pub fn register_fact_storage(
        &mut self,
        backend_type: &str,
        facts_count: usize,
        lookups: usize,
    ) {
        self.fact_storage.total_facts += facts_count;
        self.fact_storage.total_lookups += lookups;
        self.fact_storage.facts_by_backend.insert(backend_type.to_string(), facts_count);
    }

    /// Register cache statistics for a component
    pub fn register_cache(
        &mut self,
        component: &str,
        cache_stats: CacheStats,
        hits: usize,
        misses: usize,
    ) {
        self.caching.total_hits += hits;
        self.caching.total_misses += misses;
        self.caching.cache_by_component.insert(component.to_string(), cache_stats);
    }

    /// Register memory pool statistics
    pub fn register_memory_pool(
        &mut self,
        pool_type: &str,
        hits: usize,
        misses: usize,
        allocated: usize,
        returned: usize,
        peak_size: usize,
    ) {
        self.memory.pool_hits += hits;
        self.memory.pool_misses += misses;
        self.memory.pool_allocated += allocated;
        self.memory.pool_returned += returned;
        self.memory.peak_pool_sizes.insert(pool_type.to_string(), peak_size);
    }

    /// Register calculator statistics
    pub fn register_calculator(
        &mut self,
        comp_hits: usize,
        comp_misses: usize,
        result_hits: usize,
        result_misses: usize,
        evaluations: usize,
    ) {
        self.calculator.compilation_hits += comp_hits;
        self.calculator.compilation_misses += comp_misses;
        self.calculator.result_hits += result_hits;
        self.calculator.result_misses += result_misses;
        self.calculator.total_evaluations += evaluations;
    }

    /// Register field indexing statistics
    pub fn register_indexing(&mut self, field_stats: FieldIndexStats) {
        self.indexing.indexed_fields = field_stats.indexed_fields;
        self.indexing.total_index_entries = field_stats.total_entries;
        self.indexing.index_memory_bytes = field_stats.memory_usage_bytes;
        self.indexing.avg_facts_per_entry = field_stats.avg_facts_per_entry();
    }

    /// Register custom component statistics
    pub fn register_component(&mut self, component: &str, stats: ComponentStats) {
        self.component_counters.insert(component.to_string(), stats);
    }

    /// Calculate overall cache hit rate percentage
    pub fn overall_cache_hit_rate(&self) -> f64 {
        let total_operations = self.caching.total_hits + self.caching.total_misses;
        if total_operations == 0 {
            0.0
        } else {
            (self.caching.total_hits as f64 / total_operations as f64) * 100.0
        }
    }

    /// Calculate overall memory pool utilization percentage
    pub fn overall_pool_utilization(&self) -> f64 {
        let total_requests = self.memory.pool_hits + self.memory.pool_misses;
        if total_requests == 0 {
            0.0
        } else {
            (self.memory.pool_hits as f64 / total_requests as f64) * 100.0
        }
    }

    /// Calculate calculator compilation efficiency percentage
    pub fn calculator_compilation_efficiency(&self) -> f64 {
        let total_compilations =
            self.calculator.compilation_hits + self.calculator.compilation_misses;
        if total_compilations == 0 {
            0.0
        } else {
            (self.calculator.compilation_hits as f64 / total_compilations as f64) * 100.0
        }
    }

    /// Calculate calculator result cache efficiency percentage
    pub fn calculator_result_efficiency(&self) -> f64 {
        let total_results = self.calculator.result_hits + self.calculator.result_misses;
        if total_results == 0 {
            0.0
        } else {
            (self.calculator.result_hits as f64 / total_results as f64) * 100.0
        }
    }

    /// Calculate average facts per storage backend
    pub fn avg_facts_per_backend(&self) -> f64 {
        if self.fact_storage.facts_by_backend.is_empty() {
            0.0
        } else {
            let total_facts: usize = self.fact_storage.facts_by_backend.values().sum();
            total_facts as f64 / self.fact_storage.facts_by_backend.len() as f64
        }
    }

    /// Get optimization efficiency score (0-100)
    /// Composite score based on cache hit rates, pool utilization, and indexing efficiency
    pub fn optimization_efficiency_score(&self) -> f64 {
        let cache_score = self.overall_cache_hit_rate();
        let pool_score = self.overall_pool_utilization();
        let calc_score =
            (self.calculator_compilation_efficiency() + self.calculator_result_efficiency()) / 2.0;

        // Weighted average: caching (40%), memory pools (30%), calculator (30%)
        (cache_score * 0.4) + (pool_score * 0.3) + (calc_score * 0.3)
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Merge statistics from another UnifiedStats instance
    pub fn merge(&mut self, other: &UnifiedStats) {
        // Merge fact storage
        self.fact_storage.total_facts += other.fact_storage.total_facts;
        self.fact_storage.total_lookups += other.fact_storage.total_lookups;
        for (backend, count) in &other.fact_storage.facts_by_backend {
            *self.fact_storage.facts_by_backend.entry(backend.clone()).or_insert(0) += count;
        }

        // Merge caching
        self.caching.total_hits += other.caching.total_hits;
        self.caching.total_misses += other.caching.total_misses;
        self.caching.total_evictions += other.caching.total_evictions;
        for (component, cache_stats) in &other.caching.cache_by_component {
            self.caching.cache_by_component.insert(component.clone(), cache_stats.clone());
        }

        // Merge memory
        self.memory.pool_hits += other.memory.pool_hits;
        self.memory.pool_misses += other.memory.pool_misses;
        self.memory.pool_allocated += other.memory.pool_allocated;
        self.memory.pool_returned += other.memory.pool_returned;
        for (pool_type, peak_size) in &other.memory.peak_pool_sizes {
            let current_peak = self.memory.peak_pool_sizes.entry(pool_type.clone()).or_insert(0);
            *current_peak = (*current_peak).max(*peak_size);
        }

        // Merge calculator
        self.calculator.compilation_hits += other.calculator.compilation_hits;
        self.calculator.compilation_misses += other.calculator.compilation_misses;
        self.calculator.result_hits += other.calculator.result_hits;
        self.calculator.result_misses += other.calculator.result_misses;
        self.calculator.total_evaluations += other.calculator.total_evaluations;

        // Merge indexing
        self.indexing.indexed_fields += other.indexing.indexed_fields;
        self.indexing.total_index_entries += other.indexing.total_index_entries;
        self.indexing.index_memory_bytes += other.indexing.index_memory_bytes;
        self.indexing.index_lookups += other.indexing.index_lookups;

        // Merge component counters
        for (component, stats) in &other.component_counters {
            self.component_counters.insert(component.clone(), stats.clone());
        }
    }
}

impl Default for UnifiedStats {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UnifiedStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== Unified Optimization Statistics ===")?;
        writeln!(f)?;

        writeln!(
            f,
            "📊 Overall Efficiency Score: {:.1}%",
            self.optimization_efficiency_score()
        )?;
        writeln!(f)?;

        writeln!(f, "💾 Fact Storage:")?;
        writeln!(f, "  Total Facts: {}", self.fact_storage.total_facts)?;
        writeln!(f, "  Total Lookups: {}", self.fact_storage.total_lookups)?;
        writeln!(
            f,
            "  Avg Facts/Backend: {:.1}",
            self.avg_facts_per_backend()
        )?;
        for (backend, count) in &self.fact_storage.facts_by_backend {
            writeln!(f, "  {}: {} facts", backend, count)?;
        }
        writeln!(f)?;

        writeln!(
            f,
            "⚡ Caching (Hit Rate: {:.1}%):",
            self.overall_cache_hit_rate()
        )?;
        writeln!(f, "  Total Hits: {}", self.caching.total_hits)?;
        writeln!(f, "  Total Misses: {}", self.caching.total_misses)?;
        writeln!(f, "  Total Evictions: {}", self.caching.total_evictions)?;
        for (component, cache_stats) in &self.caching.cache_by_component {
            writeln!(
                f,
                "  {}: {:.1}% utilization",
                component,
                cache_stats.utilization()
            )?;
        }
        writeln!(f)?;

        writeln!(
            f,
            "🧠 Memory Pools (Utilization: {:.1}%):",
            self.overall_pool_utilization()
        )?;
        writeln!(f, "  Pool Hits: {}", self.memory.pool_hits)?;
        writeln!(f, "  Pool Misses: {}", self.memory.pool_misses)?;
        writeln!(f, "  Objects Allocated: {}", self.memory.pool_allocated)?;
        writeln!(f, "  Objects Returned: {}", self.memory.pool_returned)?;
        for (pool_type, peak_size) in &self.memory.peak_pool_sizes {
            writeln!(f, "  {} Peak Size: {}", pool_type, peak_size)?;
        }
        writeln!(f)?;

        writeln!(f, "🧮 Calculator Optimization:")?;
        writeln!(
            f,
            "  Compilation Efficiency: {:.1}%",
            self.calculator_compilation_efficiency()
        )?;
        writeln!(
            f,
            "  Result Cache Efficiency: {:.1}%",
            self.calculator_result_efficiency()
        )?;
        writeln!(
            f,
            "  Total Evaluations: {}",
            self.calculator.total_evaluations
        )?;
        writeln!(
            f,
            "  Avg Eval Time: {:.1}μs",
            self.calculator.avg_evaluation_time_us
        )?;
        writeln!(f)?;

        writeln!(f, "🗂️ Field Indexing:")?;
        writeln!(f, "  Indexed Fields: {}", self.indexing.indexed_fields)?;
        writeln!(
            f,
            "  Total Index Entries: {}",
            self.indexing.total_index_entries
        )?;
        writeln!(
            f,
            "  Index Memory: {} bytes",
            self.indexing.index_memory_bytes
        )?;
        writeln!(
            f,
            "  Avg Facts/Entry: {:.1}",
            self.indexing.avg_facts_per_entry
        )?;
        writeln!(f)?;

        if !self.component_counters.is_empty() {
            writeln!(f, "🔧 Component Statistics:")?;
            for (component, stats) in &self.component_counters {
                let success_rate = if stats.operations == 0 {
                    0.0
                } else {
                    (stats.successes as f64 / stats.operations as f64) * 100.0
                };
                writeln!(
                    f,
                    "  {}: {} ops, {:.1}% success, {:.1}μs avg",
                    component, stats.operations, success_rate, stats.avg_time_us
                )?;
            }
        }

        Ok(())
    }
}

/// Builder for creating UnifiedStats with fluent API
pub struct UnifiedStatsBuilder {
    stats: UnifiedStats,
}

impl UnifiedStatsBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self { stats: UnifiedStats::new() }
    }

    /// Add fact storage statistics
    pub fn with_fact_storage(mut self, backend: &str, facts: usize, lookups: usize) -> Self {
        self.stats.register_fact_storage(backend, facts, lookups);
        self
    }

    /// Add cache statistics
    pub fn with_cache(
        mut self,
        component: &str,
        cache_stats: CacheStats,
        hits: usize,
        misses: usize,
    ) -> Self {
        self.stats.register_cache(component, cache_stats, hits, misses);
        self
    }

    /// Add memory pool statistics
    pub fn with_memory_pool(
        mut self,
        pool_type: &str,
        hits: usize,
        misses: usize,
        allocated: usize,
        returned: usize,
        peak_size: usize,
    ) -> Self {
        self.stats
            .register_memory_pool(pool_type, hits, misses, allocated, returned, peak_size);
        self
    }

    /// Build the unified statistics
    pub fn build(self) -> UnifiedStats {
        self.stats
    }
}

impl Default for UnifiedStatsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_stats_creation() {
        let stats = UnifiedStats::new();
        assert_eq!(stats.overall_cache_hit_rate(), 0.0);
        assert_eq!(stats.overall_pool_utilization(), 0.0);
        assert_eq!(stats.optimization_efficiency_score(), 0.0);
    }

    #[test]
    fn test_fact_storage_registration() {
        let mut stats = UnifiedStats::new();
        stats.register_fact_storage("HashMap", 1000, 500);
        stats.register_fact_storage("Vector", 2000, 300);

        assert_eq!(stats.fact_storage.total_facts, 3000);
        assert_eq!(stats.fact_storage.total_lookups, 800);
        assert_eq!(stats.avg_facts_per_backend(), 1500.0);
    }

    #[test]
    fn test_cache_hit_rate_calculation() {
        let mut stats = UnifiedStats::new();
        let cache_stats = CacheStats { capacity: 100, size: 50, access_counter: 200 };

        stats.register_cache("FactStore", cache_stats, 800, 200);
        assert_eq!(stats.overall_cache_hit_rate(), 80.0);
    }

    #[test]
    fn test_optimization_efficiency_score() {
        let mut stats = UnifiedStats::new();

        // Register high-performance statistics
        let cache_stats = CacheStats { capacity: 100, size: 80, access_counter: 1000 };
        stats.register_cache("FactStore", cache_stats, 900, 100); // 90% hit rate
        stats.register_memory_pool("Token", 800, 200, 500, 400, 50); // 80% utilization
        stats.register_calculator(700, 300, 850, 150, 1000); // 70% and 85% efficiency

        let score = stats.optimization_efficiency_score();
        assert!(score > 80.0); // Should be high overall efficiency
    }

    #[test]
    fn test_stats_merge() {
        let mut stats1 = UnifiedStats::new();
        stats1.register_fact_storage("HashMap", 1000, 500);

        let mut stats2 = UnifiedStats::new();
        stats2.register_fact_storage("Vector", 2000, 300);

        stats1.merge(&stats2);
        assert_eq!(stats1.fact_storage.total_facts, 3000);
        assert_eq!(stats1.fact_storage.total_lookups, 800);
    }

    #[test]
    fn test_builder_pattern() {
        let cache_stats = CacheStats { capacity: 100, size: 75, access_counter: 500 };

        let stats = UnifiedStatsBuilder::new()
            .with_fact_storage("HashMap", 1000, 800)
            .with_cache("FactStore", cache_stats, 400, 100)
            .with_memory_pool("Token", 300, 50, 200, 180, 25)
            .build();

        assert_eq!(stats.fact_storage.total_facts, 1000);
        assert_eq!(stats.overall_cache_hit_rate(), 80.0);
        assert_eq!(stats.overall_pool_utilization(), 85.71428571428571);
    }
}
