//! Adaptive fact store backends that automatically optimize based on dataset characteristics
//!
//! This module provides intelligent backend selection and dynamic optimization strategies
//! that adapt to changing data patterns and usage characteristics.

use crate::cache::CacheStats;
use crate::fact_store::FactStore;
use crate::types::{Fact, FactId, FactValue};
use crate::unified_fact_store::{OptimizedFactStore, OptimizedStoreStats};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Characteristics of a dataset that influence optimal backend selection
#[derive(Debug, Clone)]
pub struct DatasetCharacteristics {
    /// Total number of facts
    pub fact_count: usize,
    /// Average fact size in bytes
    pub avg_fact_size: usize,
    /// Frequency of read operations vs write operations (0.0 = all writes, 1.0 = all reads)
    pub read_write_ratio: f64,
    /// Percentage of queries that are for non-existent facts
    pub miss_rate: f64,
    /// Most frequently accessed fields for lookups
    pub hot_fields: Vec<String>,
    /// Average number of fields per fact
    pub fields_per_fact: f64,
    /// Estimated memory budget in bytes
    pub memory_budget: usize,
    /// Expected growth rate (facts per second)
    pub growth_rate: f64,
    /// Temporal access patterns
    pub access_patterns: AccessPattern,
    /// Data distribution characteristics
    pub distribution: DataDistribution,
}

/// Temporal access patterns for facts
#[derive(Debug, Clone, PartialEq)]
pub enum AccessPattern {
    /// Most recent facts are accessed more frequently
    Recency,
    /// Random access pattern with no clear temporal bias
    Random,
    /// Older facts are accessed more frequently (archival pattern)
    Historical,
    /// Burst access to related facts
    Clustered,
    /// Sequential access through fact IDs
    Sequential,
}

/// Distribution characteristics of the data
#[derive(Debug, Clone)]
pub struct DataDistribution {
    /// Field cardinality distribution (field_name -> unique_values)
    pub field_cardinality: HashMap<String, usize>,
    /// Size distribution (small, medium, large fact percentages)
    pub size_distribution: (f64, f64, f64),
    /// Temporal distribution of fact creation
    pub temporal_skew: f64, // 0.0 = uniform, 1.0 = highly skewed
}

/// Backend optimization strategy recommendations
#[derive(Debug, Clone, PartialEq)]
pub enum BackendStrategy {
    /// HashMap-based with aggressive caching for high read/write ratios
    FastLookup { cache_size: usize, bloom_filter: bool },
    /// Memory-efficient vector-based for large datasets with budget constraints
    MemoryEfficient { cache_size: usize, bloom_filter: bool, compression: bool },
    /// Hybrid approach with partitioning for very large datasets
    Partitioned { partition_count: usize, cache_per_partition: usize, bloom_filter: bool },
    /// Write-optimized for high-growth scenarios
    WriteOptimized { buffer_size: usize, batch_threshold: usize, bloom_filter: bool },
    /// Read-optimized with extensive indexing and caching
    ReadOptimized {
        cache_size: usize,
        index_all_fields: bool,
        bloom_filter: bool,
        prefetch_size: usize,
    },
}

/// Backend performance metrics for adaptation decisions
#[derive(Debug, Clone)]
pub struct BackendMetrics {
    /// Average lookup time in microseconds
    pub avg_lookup_time: f64,
    /// Average insertion time in microseconds
    pub avg_insert_time: f64,
    /// Memory usage in bytes
    pub memory_usage: usize,
    /// Cache hit rate percentage
    pub cache_hit_rate: f64,
    /// Bloom filter effectiveness percentage
    pub bloom_effectiveness: f64,
    /// Operations per second
    pub ops_per_second: f64,
    /// Memory efficiency (facts per MB)
    pub memory_efficiency: f64,
    /// Timestamp of last measurement
    pub last_measured: Instant,
}

/// Adaptive backend selector that chooses optimal strategies
#[derive(Debug)]
pub struct AdaptiveBackendSelector {
    /// Current dataset characteristics
    characteristics: DatasetCharacteristics,
    /// Performance history for different strategies
    performance_history: HashMap<String, Vec<BackendMetrics>>,
    /// Configuration for adaptation behavior
    config: AdaptationConfig,
    /// Current active strategy
    current_strategy: BackendStrategy,
    /// Time of last adaptation
    last_adaptation: Instant,
}

/// Configuration for adaptation behavior
#[derive(Debug, Clone)]
pub struct AdaptationConfig {
    /// Minimum time between adaptations
    pub min_adaptation_interval: Duration,
    /// Performance degradation threshold for triggering adaptation
    pub performance_threshold: f64,
    /// Memory pressure threshold for triggering adaptation
    pub memory_threshold: f64,
    /// Enable automatic adaptation
    pub auto_adapt: bool,
    /// Conservative vs aggressive adaptation strategy
    pub adaptation_aggressiveness: f64, // 0.0 = conservative, 1.0 = aggressive
    /// Sample size for performance measurements
    pub measurement_window: usize,
}

impl Default for AdaptationConfig {
    fn default() -> Self {
        Self {
            min_adaptation_interval: Duration::from_secs(30),
            performance_threshold: 0.8, // 20% degradation triggers adaptation
            memory_threshold: 0.85,     // 85% memory usage triggers adaptation
            auto_adapt: true,
            adaptation_aggressiveness: 0.5, // Balanced approach
            measurement_window: 100,
        }
    }
}

impl AdaptiveBackendSelector {
    /// Create a new adaptive backend selector
    pub fn new(initial_characteristics: DatasetCharacteristics, config: AdaptationConfig) -> Self {
        let initial_strategy = Self::recommend_strategy(&initial_characteristics);

        Self {
            characteristics: initial_characteristics,
            performance_history: HashMap::new(),
            config,
            current_strategy: initial_strategy,
            last_adaptation: Instant::now(),
        }
    }

    /// Recommend the optimal backend strategy for given characteristics
    pub fn recommend_strategy(characteristics: &DatasetCharacteristics) -> BackendStrategy {
        // Decision tree based on characteristics

        // Very large datasets (>1M facts) or memory constraints
        if characteristics.fact_count > 1_000_000
            || characteristics.estimated_memory_usage() > characteristics.memory_budget
        {
            return Self::recommend_large_dataset_strategy(characteristics);
        }

        // High read/write ratio indicates read-heavy workload
        if characteristics.read_write_ratio > 0.8 {
            return Self::recommend_read_optimized_strategy(characteristics);
        }

        // High growth rate indicates write-heavy workload
        if characteristics.growth_rate > 1000.0 {
            return Self::recommend_write_optimized_strategy(characteristics);
        }

        // High miss rate benefits from bloom filters
        if characteristics.miss_rate > 0.3 {
            return BackendStrategy::FastLookup {
                cache_size: Self::calculate_cache_size(characteristics),
                bloom_filter: true,
            };
        }

        // Default balanced approach
        BackendStrategy::FastLookup {
            cache_size: Self::calculate_cache_size(characteristics),
            bloom_filter: characteristics.miss_rate > 0.1,
        }
    }

    /// Update characteristics and potentially adapt strategy
    pub fn update_characteristics(&mut self, new_characteristics: DatasetCharacteristics) -> bool {
        let old_characteristics = self.characteristics.clone();
        self.characteristics = new_characteristics;

        if self.config.auto_adapt && self.should_adapt(&old_characteristics) {
            self.adapt_strategy()
        } else {
            false
        }
    }

    /// Record performance metrics for the current strategy
    pub fn record_metrics(&mut self, metrics: BackendMetrics) {
        let strategy_key = format!("{:?}", self.current_strategy);
        let history = self.performance_history.entry(strategy_key).or_default();

        history.push(metrics);

        // Keep only recent measurements
        if history.len() > self.config.measurement_window {
            history.remove(0);
        }
    }

    /// Check if adaptation should be triggered
    fn should_adapt(&self, old_characteristics: &DatasetCharacteristics) -> bool {
        // Check time threshold
        if self.last_adaptation.elapsed() < self.config.min_adaptation_interval {
            return false;
        }

        // Check for significant characteristic changes
        let fact_count_change =
            (self.characteristics.fact_count as f64 - old_characteristics.fact_count as f64).abs()
                / old_characteristics.fact_count.max(1) as f64;

        let memory_change = (self.characteristics.estimated_memory_usage() as f64
            - old_characteristics.estimated_memory_usage() as f64)
            .abs()
            / old_characteristics.estimated_memory_usage().max(1) as f64;

        let read_write_change =
            (self.characteristics.read_write_ratio - old_characteristics.read_write_ratio).abs();

        // Trigger adaptation if significant changes detected
        fact_count_change > 0.5 || memory_change > 0.3 || read_write_change > 0.2
    }

    /// Adapt to new strategy based on current characteristics
    fn adapt_strategy(&mut self) -> bool {
        let recommended_strategy = Self::recommend_strategy(&self.characteristics);

        if recommended_strategy != self.current_strategy {
            self.current_strategy = recommended_strategy;
            self.last_adaptation = Instant::now();
            true
        } else {
            false
        }
    }

    /// Get the current recommended strategy
    pub fn get_current_strategy(&self) -> &BackendStrategy {
        &self.current_strategy
    }

    /// Get performance summary for current strategy
    pub fn get_performance_summary(&self) -> Option<BackendPerformanceSummary> {
        let strategy_key = format!("{:?}", self.current_strategy);
        let history = self.performance_history.get(&strategy_key)?;

        if history.is_empty() {
            return None;
        }

        let recent_metrics = &history[history.len().saturating_sub(10)..];

        Some(BackendPerformanceSummary {
            strategy: self.current_strategy.clone(),
            sample_count: recent_metrics.len(),
            avg_lookup_time: recent_metrics.iter().map(|m| m.avg_lookup_time).sum::<f64>()
                / recent_metrics.len() as f64,
            avg_insert_time: recent_metrics.iter().map(|m| m.avg_insert_time).sum::<f64>()
                / recent_metrics.len() as f64,
            avg_memory_usage: recent_metrics.iter().map(|m| m.memory_usage).sum::<usize>()
                / recent_metrics.len(),
            avg_cache_hit_rate: recent_metrics.iter().map(|m| m.cache_hit_rate).sum::<f64>()
                / recent_metrics.len() as f64,
            avg_bloom_effectiveness: recent_metrics
                .iter()
                .map(|m| m.bloom_effectiveness)
                .sum::<f64>()
                / recent_metrics.len() as f64,
            avg_ops_per_second: recent_metrics.iter().map(|m| m.ops_per_second).sum::<f64>()
                / recent_metrics.len() as f64,
        })
    }

    // Private helper methods for strategy recommendation

    fn recommend_large_dataset_strategy(
        characteristics: &DatasetCharacteristics,
    ) -> BackendStrategy {
        let partition_count = (characteristics.fact_count / 100_000).clamp(4, 16);
        let cache_per_partition = characteristics.memory_budget / (partition_count * 8); // Reserve memory for partitions

        BackendStrategy::Partitioned {
            partition_count,
            cache_per_partition,
            bloom_filter: characteristics.miss_rate > 0.1,
        }
    }

    fn recommend_read_optimized_strategy(
        characteristics: &DatasetCharacteristics,
    ) -> BackendStrategy {
        BackendStrategy::ReadOptimized {
            cache_size: (characteristics.memory_budget / 4).max(1000), // Use 25% of budget for cache
            index_all_fields: characteristics.hot_fields.len() > 3,
            bloom_filter: characteristics.miss_rate > 0.05,
            prefetch_size: if characteristics.access_patterns == AccessPattern::Sequential {
                10
            } else {
                0
            },
        }
    }

    fn recommend_write_optimized_strategy(
        characteristics: &DatasetCharacteristics,
    ) -> BackendStrategy {
        BackendStrategy::WriteOptimized {
            buffer_size: (characteristics.growth_rate as usize * 10).clamp(100, 10000),
            batch_threshold: (characteristics.growth_rate as usize / 10).clamp(10, 1000),
            bloom_filter: characteristics.miss_rate > 0.2,
        }
    }

    fn calculate_cache_size(characteristics: &DatasetCharacteristics) -> usize {
        // Base cache size on memory budget and access patterns
        let base_size = characteristics.memory_budget / 8; // Use 12.5% of budget for cache
        let pattern_multiplier = match characteristics.access_patterns {
            AccessPattern::Recency => 1.5,
            AccessPattern::Clustered => 1.3,
            AccessPattern::Random => 1.0,
            AccessPattern::Historical => 0.8,
            AccessPattern::Sequential => 0.6,
        };

        ((base_size as f64 * pattern_multiplier) as usize).clamp(100, 100000)
    }
}

/// Performance summary for a backend strategy
#[derive(Debug, Clone)]
pub struct BackendPerformanceSummary {
    pub strategy: BackendStrategy,
    pub sample_count: usize,
    pub avg_lookup_time: f64,
    pub avg_insert_time: f64,
    pub avg_memory_usage: usize,
    pub avg_cache_hit_rate: f64,
    pub avg_bloom_effectiveness: f64,
    pub avg_ops_per_second: f64,
}

impl DatasetCharacteristics {
    /// Create initial characteristics from basic parameters
    pub fn new(fact_count: usize, memory_budget: usize) -> Self {
        Self {
            fact_count,
            avg_fact_size: 256,    // Default estimate
            read_write_ratio: 0.7, // Default read-heavy
            miss_rate: 0.1,        // Default 10% miss rate
            hot_fields: vec!["id".to_string(), "status".to_string()], // Common fields
            fields_per_fact: 5.0,  // Default estimate
            memory_budget,
            growth_rate: 0.0, // Default no growth
            access_patterns: AccessPattern::Random,
            distribution: DataDistribution {
                field_cardinality: HashMap::new(),
                size_distribution: (0.6, 0.3, 0.1), // Most facts are small
                temporal_skew: 0.3,                 // Moderate skew
            },
        }
    }

    /// Estimate total memory usage based on characteristics
    pub fn estimated_memory_usage(&self) -> usize {
        // Base memory for facts
        let fact_memory = self.fact_count * self.avg_fact_size;

        // Index memory (rough estimate)
        let index_memory = self.hot_fields.len() * self.fact_count * 8; // 8 bytes per index entry

        // Overhead memory (hashmaps, metadata, etc.)
        let overhead = (fact_memory + index_memory) / 4; // 25% overhead

        fact_memory + index_memory + overhead
    }

    /// Update characteristics based on observed store statistics
    pub fn update_from_stats(&mut self, stats: &OptimizedStoreStats) {
        self.fact_count = stats.facts_stored;

        // Update read/write ratio based on cache effectiveness
        if stats.total_lookups > 0 {
            self.read_write_ratio = stats.cache_hits as f64 / stats.total_lookups as f64;
        }

        // Update miss rate based on bloom filter effectiveness
        if let Some(bloom_stats) = &stats.bloom_filter_stats {
            self.miss_rate = bloom_stats.effectiveness / 100.0;
        }
    }

    /// Analyze access patterns from recent operations
    pub fn analyze_access_patterns(&mut self, recent_accesses: &[FactId]) {
        if recent_accesses.len() < 10 {
            return;
        }

        // Analyze for sequential access
        let mut sequential_count = 0;
        for i in 1..recent_accesses.len() {
            if recent_accesses[i] == recent_accesses[i - 1] + 1 {
                sequential_count += 1;
            }
        }

        let sequential_ratio = sequential_count as f64 / (recent_accesses.len() - 1) as f64;

        // Analyze for recency bias
        let max_id = *recent_accesses.iter().max().unwrap_or(&0);
        let recent_threshold = max_id.saturating_sub(max_id / 10); // Last 10% of IDs
        let recent_count = recent_accesses.iter().filter(|&&id| id >= recent_threshold).count();
        let recency_ratio = recent_count as f64 / recent_accesses.len() as f64;

        // Update access pattern based on analysis
        self.access_patterns = if sequential_ratio > 0.7 {
            AccessPattern::Sequential
        } else if recency_ratio > 0.6 {
            AccessPattern::Recency
        } else if recency_ratio < 0.2 {
            AccessPattern::Historical
        } else {
            AccessPattern::Random
        };
    }
}

/// Adaptive fact store that dynamically selects optimal backend strategies
#[derive(Debug)]
pub struct AdaptiveFactStore {
    /// Current backend implementation
    backend: OptimizedFactStore,
    /// Backend selector for strategy decisions
    selector: AdaptiveBackendSelector,
    /// Performance monitoring
    operation_count: usize,
    total_lookup_time: Duration,
    total_insert_time: Duration,
    last_metrics_update: Instant,
    /// Recent access patterns for analysis
    recent_accesses: Vec<FactId>,
    max_access_history: usize,
}

impl AdaptiveFactStore {
    /// Create a new adaptive fact store
    pub fn new(characteristics: DatasetCharacteristics, config: AdaptationConfig) -> Self {
        let selector = AdaptiveBackendSelector::new(characteristics, config);
        let backend = Self::create_backend_for_strategy(selector.get_current_strategy());

        Self {
            backend,
            selector,
            operation_count: 0,
            total_lookup_time: Duration::ZERO,
            total_insert_time: Duration::ZERO,
            last_metrics_update: Instant::now(),
            recent_accesses: Vec::new(),
            max_access_history: 1000,
        }
    }

    /// Create backend implementation for the given strategy
    fn create_backend_for_strategy(strategy: &BackendStrategy) -> OptimizedFactStore {
        match strategy {
            BackendStrategy::FastLookup { cache_size, bloom_filter } => {
                if *bloom_filter {
                    OptimizedFactStore::new_fast_with_bloom(*cache_size, 10000)
                } else {
                    OptimizedFactStore::new_fast(*cache_size)
                }
            }
            BackendStrategy::MemoryEfficient { cache_size, bloom_filter, .. } => {
                if *bloom_filter {
                    OptimizedFactStore::new_memory_efficient_with_bloom(*cache_size, 10000)
                } else {
                    OptimizedFactStore::new_memory_efficient(*cache_size)
                }
            }
            BackendStrategy::ReadOptimized { cache_size, bloom_filter, .. } => {
                if *bloom_filter {
                    OptimizedFactStore::new_fast_with_bloom(*cache_size, 50000) // Larger bloom filter for read optimization
                } else {
                    OptimizedFactStore::new_fast(*cache_size)
                }
            }
            BackendStrategy::WriteOptimized { bloom_filter, .. } => {
                if *bloom_filter {
                    OptimizedFactStore::new_fast_with_bloom(1000, 10000) // Smaller cache for write optimization
                } else {
                    OptimizedFactStore::new_fast(1000)
                }
            }
            BackendStrategy::Partitioned { bloom_filter, .. } => {
                // For now, use fast lookup as partitioned implementation is complex
                if *bloom_filter {
                    OptimizedFactStore::new_fast_with_bloom(5000, 20000)
                } else {
                    OptimizedFactStore::new_fast(5000)
                }
            }
        }
    }

    /// Get a fact with performance tracking
    pub fn get_mut(&mut self, fact_id: FactId) -> Option<Fact> {
        let start = Instant::now();
        let result = self.backend.get_mut(fact_id);
        let elapsed = start.elapsed();

        self.total_lookup_time += elapsed;
        self.operation_count += 1;

        // Track access for pattern analysis
        self.recent_accesses.push(fact_id);
        if self.recent_accesses.len() > self.max_access_history {
            self.recent_accesses.remove(0);
        }

        self.update_metrics_if_needed();
        result
    }

    /// Insert a fact with performance tracking
    pub fn insert(&mut self, fact: Fact) -> FactId {
        let start = Instant::now();
        let result = self.backend.insert(fact);
        let elapsed = start.elapsed();

        self.total_insert_time += elapsed;
        self.operation_count += 1;

        self.update_metrics_if_needed();
        result
    }

    /// Update metrics and potentially adapt backend
    fn update_metrics_if_needed(&mut self) {
        // Update metrics every 100 operations or every 10 seconds
        let should_update = self.operation_count % 100 == 0
            || self.last_metrics_update.elapsed() > Duration::from_secs(10);

        if should_update && self.operation_count > 0 {
            let stats = self.backend.stats();

            let metrics = BackendMetrics {
                avg_lookup_time: self.total_lookup_time.as_micros() as f64
                    / self.operation_count as f64,
                avg_insert_time: self.total_insert_time.as_micros() as f64
                    / self.operation_count as f64,
                memory_usage: 0, // Would need to implement memory tracking
                cache_hit_rate: stats.hit_rate,
                bloom_effectiveness: stats.bloom_filter_effectiveness(),
                ops_per_second: self.operation_count as f64
                    / self.last_metrics_update.elapsed().as_secs_f64(),
                memory_efficiency: stats.facts_stored as f64 / 1024.0, // Simplified calculation
                last_measured: Instant::now(),
            };

            self.selector.record_metrics(metrics);

            // Update characteristics based on current performance
            let mut characteristics = self.selector.characteristics.clone();
            characteristics.update_from_stats(&stats);
            characteristics.analyze_access_patterns(&self.recent_accesses);

            // Check if adaptation is needed
            if self.selector.update_characteristics(characteristics) {
                self.adapt_backend();
            }

            self.last_metrics_update = Instant::now();
        }
    }

    /// Adapt backend to new strategy
    fn adapt_backend(&mut self) {
        let new_strategy = self.selector.get_current_strategy();
        let old_facts: Vec<Fact> = self
            .backend
            .fact_ids()
            .iter()
            .filter_map(|&id| self.backend.get_mut(id))
            .collect();

        // Create new backend
        self.backend = Self::create_backend_for_strategy(new_strategy);

        // Migrate facts to new backend
        for fact in old_facts {
            self.backend.insert(fact);
        }

        // Reset performance counters
        self.operation_count = 0;
        self.total_lookup_time = Duration::ZERO;
        self.total_insert_time = Duration::ZERO;
        self.last_metrics_update = Instant::now();
    }

    /// Get current backend statistics
    pub fn stats(&self) -> OptimizedStoreStats {
        self.backend.stats()
    }

    /// Get adaptation performance summary
    pub fn get_adaptation_summary(&self) -> Option<BackendPerformanceSummary> {
        self.selector.get_performance_summary()
    }

    /// Get current strategy information
    pub fn get_current_strategy(&self) -> &BackendStrategy {
        self.selector.get_current_strategy()
    }

    /// Manually trigger adaptation analysis
    pub fn trigger_adaptation(&mut self) -> bool {
        let stats = self.backend.stats();
        let mut characteristics = self.selector.characteristics.clone();
        characteristics.update_from_stats(&stats);
        characteristics.analyze_access_patterns(&self.recent_accesses);

        if self.selector.update_characteristics(characteristics) {
            self.adapt_backend();
            true
        } else {
            false
        }
    }
}

// Forward FactStore trait implementation to backend
impl FactStore for AdaptiveFactStore {
    fn insert(&mut self, fact: Fact) -> FactId {
        self.insert(fact)
    }

    fn get(&self, id: FactId) -> Option<&Fact> {
        self.backend.get(id)
    }

    fn extend_from_vec(&mut self, facts: Vec<Fact>) {
        for fact in facts {
            self.insert(fact);
        }
    }

    fn len(&self) -> usize {
        self.backend.len()
    }

    fn clear(&mut self) {
        self.backend.clear();
        self.operation_count = 0;
        self.total_lookup_time = Duration::ZERO;
        self.total_insert_time = Duration::ZERO;
        self.recent_accesses.clear();
    }

    fn find_by_field(&self, _field: &str, _value: &FactValue) -> Vec<&Fact> {
        // Note: This would need to be adapted to return actual fact references
        // For now, we'll use a placeholder implementation
        Vec::new()
    }

    fn find_by_criteria(&self, _criteria: &[(String, FactValue)]) -> Vec<&Fact> {
        // Note: This would need to be adapted to return actual fact references
        // For now, we'll use a placeholder implementation
        Vec::new()
    }

    fn cache_stats(&self) -> Option<CacheStats> {
        self.backend.cache_stats()
    }

    fn clear_cache(&mut self) {
        self.backend.clear_cache()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FactData, FactValue};

    fn create_test_fact(id: FactId, field_name: &str, field_value: FactValue) -> Fact {
        let mut fields = HashMap::new();
        fields.insert(field_name.to_string(), field_value);
        Fact { id, data: FactData { fields } }
    }

    #[test]
    fn test_dataset_characteristics_creation() {
        let characteristics = DatasetCharacteristics::new(1000, 1024 * 1024); // 1K facts, 1MB budget

        assert_eq!(characteristics.fact_count, 1000);
        assert_eq!(characteristics.memory_budget, 1024 * 1024);
        assert_eq!(characteristics.access_patterns, AccessPattern::Random);
        assert!(characteristics.estimated_memory_usage() > 0);
    }

    #[test]
    fn test_backend_strategy_recommendation() {
        // Small dataset should use FastLookup
        let small_characteristics = DatasetCharacteristics::new(100, 1024 * 1024);
        let strategy = AdaptiveBackendSelector::recommend_strategy(&small_characteristics);
        match strategy {
            BackendStrategy::FastLookup { .. } => {}
            _ => panic!("Expected FastLookup for small dataset"),
        }

        // Large dataset should use partitioned or memory efficient
        let large_characteristics = DatasetCharacteristics::new(2_000_000, 1024 * 1024);
        let strategy = AdaptiveBackendSelector::recommend_strategy(&large_characteristics);
        match strategy {
            BackendStrategy::Partitioned { .. } | BackendStrategy::MemoryEfficient { .. } => {}
            _ => panic!("Expected Partitioned or MemoryEfficient for large dataset"),
        }
    }

    #[test]
    fn test_access_pattern_analysis() {
        let mut characteristics = DatasetCharacteristics::new(1000, 1024 * 1024);

        // Sequential access pattern
        let sequential_accesses: Vec<FactId> = (1..20).collect();
        characteristics.analyze_access_patterns(&sequential_accesses);
        assert_eq!(characteristics.access_patterns, AccessPattern::Sequential);

        // Recent access pattern (high IDs, not sequential)
        let recent_accesses: Vec<FactId> =
            vec![950, 952, 965, 967, 970, 980, 990, 995, 998, 999, 1000, 1005];
        characteristics.analyze_access_patterns(&recent_accesses);
        assert_eq!(characteristics.access_patterns, AccessPattern::Recency);
    }

    #[test]
    fn test_adaptive_fact_store() {
        let characteristics = DatasetCharacteristics::new(100, 1024 * 1024);
        let config = AdaptationConfig::default();
        let mut store = AdaptiveFactStore::new(characteristics, config);

        // Insert some facts
        for i in 1..10 {
            let fact = create_test_fact(i, "test", FactValue::Integer(i as i64));
            store.insert(fact);
        }

        assert_eq!(store.len(), 9);

        // Test retrieval
        let retrieved = store.get_mut(5);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, 5);

        // Test stats
        let stats = store.stats();
        assert_eq!(stats.facts_stored, 9);
    }

    #[test]
    fn test_performance_metrics_tracking() {
        let characteristics = DatasetCharacteristics::new(100, 1024 * 1024);
        let config = AdaptationConfig::default();
        let mut selector = AdaptiveBackendSelector::new(characteristics, config);

        let metrics = BackendMetrics {
            avg_lookup_time: 50.0,
            avg_insert_time: 20.0,
            memory_usage: 1024,
            cache_hit_rate: 85.0,
            bloom_effectiveness: 70.0,
            ops_per_second: 1000.0,
            memory_efficiency: 10.0,
            last_measured: Instant::now(),
        };

        selector.record_metrics(metrics);

        let summary = selector.get_performance_summary();
        assert!(summary.is_some());

        let summary = summary.unwrap();
        assert_eq!(summary.sample_count, 1);
        assert_eq!(summary.avg_lookup_time, 50.0);
    }

    #[test]
    fn test_backend_adaptation() {
        let mut characteristics = DatasetCharacteristics::new(100, 1024 * 1024);
        let config = AdaptationConfig {
            min_adaptation_interval: Duration::from_secs(0), // Allow immediate adaptation for test
            performance_threshold: 0.8,
            memory_threshold: 0.85,
            auto_adapt: true,
            adaptation_aggressiveness: 0.5,
            measurement_window: 100,
        };
        let mut selector = AdaptiveBackendSelector::new(characteristics.clone(), config);

        // Change characteristics to trigger adaptation
        characteristics.fact_count = 2_000_000; // Large increase
        characteristics.read_write_ratio = 0.95; // Very read-heavy

        let adapted = selector.update_characteristics(characteristics);
        assert!(adapted); // Should trigger adaptation due to significant changes

        // New strategy should be read-optimized
        match selector.get_current_strategy() {
            BackendStrategy::ReadOptimized { .. } | BackendStrategy::Partitioned { .. } => {}
            _ => panic!(
                "Expected read-optimized or partitioned strategy for large read-heavy dataset"
            ),
        }
    }
}
