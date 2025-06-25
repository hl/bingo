//! Optimization coordinator that orchestrates all optimization strategies
//!
//! This module provides a unified interface for managing memory optimization,
//! adaptive backends, bloom filters, advanced indexing, and performance monitoring.

// Note: These imports are available for future integration
// use crate::adaptive_backends::{AdaptiveFactStore, DatasetCharacteristics, AdaptationConfig};
// use crate::advanced_indexing::{AdvancedFieldIndexer, AdvancedIndexingStats};
// use crate::bloom_filter::{FactBloomFilter, FactBloomStats};
use crate::memory_profiler::{MemoryPressureLevel, ReteMemoryProfiler};
use crate::rete_network::ReteNetwork;
use crate::unified_memory_coordinator::{MemoryCoordinatorConfig, UnifiedMemoryCoordinator};
use std::time::{Duration, Instant};

/// Comprehensive optimization coordinator for RETE network performance
pub struct OptimizationCoordinator {
    /// Memory profiler for system-wide memory monitoring
    memory_profiler: ReteMemoryProfiler,
    /// Unified memory coordinator for cross-component optimization
    #[allow(dead_code)]
    memory_coordinator: UnifiedMemoryCoordinator,
    /// Configuration for optimization behavior
    config: OptimizationConfig,
    /// Last optimization run timestamp
    last_optimization: Instant,
    /// Performance metrics history
    performance_history: Vec<OptimizationMetrics>,
    /// Optimization statistics
    stats: OptimizationStats,
}

/// Configuration for optimization coordinator behavior
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// Enable automatic optimization
    pub auto_optimize: bool,
    /// Interval between optimization runs
    pub optimization_interval: Duration,
    /// Memory pressure threshold for triggering optimization
    pub memory_pressure_threshold: MemoryPressureLevel,
    /// Enable adaptive backend optimization
    pub enable_adaptive_backends: bool,
    /// Enable advanced indexing optimization
    pub enable_advanced_indexing: bool,
    /// Enable bloom filter optimization
    pub enable_bloom_filters: bool,
    /// Maximum memory usage before aggressive optimization
    pub max_memory_usage: usize,
    /// Target performance improvement percentage
    pub target_improvement: f64,
    /// Performance monitoring window size
    pub monitoring_window: usize,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            auto_optimize: true,
            optimization_interval: Duration::from_secs(60),
            memory_pressure_threshold: MemoryPressureLevel::Moderate,
            enable_adaptive_backends: true,
            enable_advanced_indexing: true,
            enable_bloom_filters: true,
            max_memory_usage: 1024 * 1024 * 1024, // 1GB default
            target_improvement: 0.15,             // 15% improvement target
            monitoring_window: 100,
        }
    }
}

/// Optimization metrics for performance tracking
#[derive(Debug, Clone)]
pub struct OptimizationMetrics {
    /// Timestamp of measurement
    pub timestamp: Instant,
    /// Memory usage in bytes
    pub memory_usage: usize,
    /// Memory pressure level
    pub memory_pressure: MemoryPressureLevel,
    /// Average lookup time in microseconds
    pub avg_lookup_time: f64,
    /// Average insertion time in microseconds
    pub avg_insertion_time: f64,
    /// Cache hit rate percentage
    pub cache_hit_rate: f64,
    /// Bloom filter effectiveness percentage
    pub bloom_effectiveness: f64,
    /// Index efficiency score
    pub index_efficiency: f64,
    /// Operations per second
    pub ops_per_second: f64,
    /// Total facts stored
    pub total_facts: usize,
}

/// Statistics about optimization system performance
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    /// Total number of optimization runs
    pub optimization_runs: usize,
    /// Total time spent in optimization
    pub total_optimization_time: Duration,
    /// Number of successful optimizations
    pub successful_optimizations: usize,
    /// Memory reclaimed through optimization (bytes)
    pub memory_reclaimed: usize,
    /// Performance improvements achieved
    pub performance_improvements: Vec<f64>,
    /// Last optimization result
    pub last_result: OptimizationResult,
}

/// Result of an optimization run
#[derive(Debug, Clone)]
pub enum OptimizationResult {
    /// Optimization completed successfully
    Success { improvements: OptimizationImprovements, actions_taken: Vec<OptimizationAction> },
    /// Optimization skipped (not needed)
    Skipped(String),
    /// Optimization failed
    Failed(String),
}

/// Improvements achieved through optimization
#[derive(Debug, Clone)]
pub struct OptimizationImprovements {
    /// Memory usage reduction in bytes
    pub memory_reduction: usize,
    /// Lookup time improvement percentage
    pub lookup_improvement: f64,
    /// Insertion time improvement percentage
    pub insertion_improvement: f64,
    /// Cache hit rate improvement
    pub cache_improvement: f64,
    /// Overall performance score improvement
    pub overall_improvement: f64,
}

/// Actions taken during optimization
#[derive(Debug, Clone)]
pub enum OptimizationAction {
    /// Memory cleanup was performed
    MemoryCleanup { bytes_freed: usize },
    /// Backend strategy was adapted
    BackendAdaptation { old_strategy: String, new_strategy: String },
    /// Indexing strategy was optimized
    IndexingOptimization { fields_optimized: usize },
    /// Bloom filter was resized or optimized
    BloomFilterOptimization { new_size: usize },
    /// Cache was resized or optimized
    CacheOptimization { new_size: usize },
    /// Network topology was optimized
    NetworkOptimization { nodes_optimized: usize },
}

impl OptimizationCoordinator {
    /// Create a new optimization coordinator
    pub fn new(config: OptimizationConfig) -> anyhow::Result<Self> {
        let memory_profiler = ReteMemoryProfiler::new();
        let memory_coordinator = UnifiedMemoryCoordinator::new(MemoryCoordinatorConfig::default());

        Ok(Self {
            memory_profiler,
            memory_coordinator,
            config,
            last_optimization: Instant::now(),
            performance_history: Vec::new(),
            stats: OptimizationStats {
                optimization_runs: 0,
                total_optimization_time: Duration::ZERO,
                successful_optimizations: 0,
                memory_reclaimed: 0,
                performance_improvements: Vec::new(),
                last_result: OptimizationResult::Skipped("Initial state".to_string()),
            },
        })
    }

    /// Run optimization if needed
    pub fn optimize_if_needed(
        &mut self,
        network: &mut ReteNetwork,
    ) -> anyhow::Result<OptimizationResult> {
        if !self.config.auto_optimize {
            return Ok(OptimizationResult::Skipped(
                "Auto-optimization disabled".to_string(),
            ));
        }

        if self.last_optimization.elapsed() < self.config.optimization_interval {
            return Ok(OptimizationResult::Skipped(
                "Too soon since last optimization".to_string(),
            ));
        }

        self.run_optimization(network)
    }

    /// Force run optimization
    pub fn run_optimization(
        &mut self,
        network: &mut ReteNetwork,
    ) -> anyhow::Result<OptimizationResult> {
        let start_time = Instant::now();
        self.stats.optimization_runs += 1;

        // Collect current metrics
        let before_metrics = self.collect_metrics(network)?;

        // Determine if optimization is needed
        if !self.should_optimize(&before_metrics) {
            return Ok(OptimizationResult::Skipped(
                "Optimization not needed".to_string(),
            ));
        }

        let mut actions_taken = Vec::new();

        // 1. Memory optimization
        if self.should_optimize_memory(&before_metrics) {
            let memory_actions = self.optimize_memory(network)?;
            actions_taken.extend(memory_actions);
        }

        // 2. Adaptive backend optimization
        if self.config.enable_adaptive_backends {
            let backend_actions = self.optimize_backends(network)?;
            actions_taken.extend(backend_actions);
        }

        // 3. Advanced indexing optimization
        if self.config.enable_advanced_indexing {
            let indexing_actions = self.optimize_indexing(network)?;
            actions_taken.extend(indexing_actions);
        }

        // 4. Bloom filter optimization
        if self.config.enable_bloom_filters {
            let bloom_actions = self.optimize_bloom_filters(network)?;
            actions_taken.extend(bloom_actions);
        }

        // 5. Network topology optimization
        let network_actions = self.optimize_network_topology(network)?;
        actions_taken.extend(network_actions);

        // Collect metrics after optimization
        let after_metrics = self.collect_metrics(network)?;
        let improvements = self.calculate_improvements(&before_metrics, &after_metrics);

        // Update statistics
        let optimization_time = start_time.elapsed();
        self.stats.total_optimization_time += optimization_time;
        self.last_optimization = Instant::now();

        if improvements.overall_improvement > 0.0 {
            self.stats.successful_optimizations += 1;
            self.stats.performance_improvements.push(improvements.overall_improvement);
            self.stats.memory_reclaimed += improvements.memory_reduction;
        }

        let result = OptimizationResult::Success { improvements, actions_taken };

        self.stats.last_result = result.clone();

        // Store metrics for trend analysis
        self.performance_history.push(after_metrics);
        if self.performance_history.len() > self.config.monitoring_window {
            self.performance_history.remove(0);
        }

        Ok(result)
    }

    /// Collect current performance metrics
    fn collect_metrics(&mut self, _network: &ReteNetwork) -> anyhow::Result<OptimizationMetrics> {
        self.memory_profiler.collect_statistics();
        let memory_report = self.memory_profiler.generate_report();

        Ok(OptimizationMetrics {
            timestamp: Instant::now(),
            memory_usage: memory_report.total_allocated_bytes,
            memory_pressure: self.memory_profiler.get_pressure_level(),
            avg_lookup_time: 0.0, // Would need to be collected from network stats
            avg_insertion_time: 0.0, // Would need to be collected from network stats
            cache_hit_rate: 0.0,  // Would need to be collected from fact stores
            bloom_effectiveness: 0.0, // Would need to be collected from bloom filters
            index_efficiency: 0.0, // Would need to be collected from indexers
            ops_per_second: 0.0,  // Would need to be calculated from operation counts
            total_facts: 0,       // Would need to be implemented in ReteNetwork
        })
    }

    /// Determine if optimization should run
    fn should_optimize(&self, metrics: &OptimizationMetrics) -> bool {
        // Check memory pressure
        if metrics.memory_pressure >= self.config.memory_pressure_threshold {
            return true;
        }

        // Check memory usage threshold
        if metrics.memory_usage > self.config.max_memory_usage {
            return true;
        }

        // Check performance degradation
        if let Some(last_metrics) = self.performance_history.last() {
            let lookup_degradation = (metrics.avg_lookup_time - last_metrics.avg_lookup_time)
                / last_metrics.avg_lookup_time.max(1.0);
            if lookup_degradation > 0.2 {
                // 20% degradation
                return true;
            }
        }

        false
    }

    /// Check if memory optimization is needed
    fn should_optimize_memory(&self, metrics: &OptimizationMetrics) -> bool {
        metrics.memory_pressure >= MemoryPressureLevel::Moderate
            || metrics.memory_usage > self.config.max_memory_usage / 2
    }

    /// Optimize memory usage
    fn optimize_memory(
        &mut self,
        network: &mut ReteNetwork,
    ) -> anyhow::Result<Vec<OptimizationAction>> {
        let mut actions = Vec::new();

        // Perform adaptive memory sizing
        let before_memory = self.memory_profiler.get_total_allocated_memory();
        network.perform_adaptive_memory_sizing()?;
        let after_memory = self.memory_profiler.get_total_allocated_memory();

        if before_memory > after_memory {
            actions.push(OptimizationAction::MemoryCleanup {
                bytes_freed: before_memory - after_memory,
            });
        }

        Ok(actions)
    }

    /// Optimize backend strategies
    fn optimize_backends(
        &mut self,
        _network: &mut ReteNetwork,
    ) -> anyhow::Result<Vec<OptimizationAction>> {
        let actions = Vec::new();

        // This would integrate with adaptive fact stores
        // For now, return empty actions as this would require deeper integration

        Ok(actions)
    }

    /// Optimize indexing strategies
    fn optimize_indexing(
        &mut self,
        _network: &mut ReteNetwork,
    ) -> anyhow::Result<Vec<OptimizationAction>> {
        let actions = Vec::new();

        // This would optimize field indexing strategies
        // For now, return empty actions as this would require integration with fact stores

        Ok(actions)
    }

    /// Optimize bloom filters
    fn optimize_bloom_filters(
        &mut self,
        _network: &mut ReteNetwork,
    ) -> anyhow::Result<Vec<OptimizationAction>> {
        let actions = Vec::new();

        // This would optimize bloom filter sizing and effectiveness
        // For now, return empty actions as this would require integration with fact stores

        Ok(actions)
    }

    /// Optimize network topology
    fn optimize_network_topology(
        &mut self,
        network: &mut ReteNetwork,
    ) -> anyhow::Result<Vec<OptimizationAction>> {
        let mut actions = Vec::new();

        // Optimize pattern cache
        let pattern_stats = network.get_pattern_cache_stats();
        if pattern_stats.pattern_cache_misses > pattern_stats.pattern_cache_hits {
            // Pattern cache is not effective, might need resizing
            actions.push(OptimizationAction::CacheOptimization {
                new_size: pattern_stats.patterns_cached * 2,
            });
        }

        Ok(actions)
    }

    /// Calculate improvements between before/after metrics
    fn calculate_improvements(
        &self,
        before: &OptimizationMetrics,
        after: &OptimizationMetrics,
    ) -> OptimizationImprovements {
        let memory_reduction = before.memory_usage.saturating_sub(after.memory_usage);

        let lookup_improvement = if before.avg_lookup_time > 0.0 {
            (before.avg_lookup_time - after.avg_lookup_time) / before.avg_lookup_time * 100.0
        } else {
            0.0
        };

        let insertion_improvement = if before.avg_insertion_time > 0.0 {
            (before.avg_insertion_time - after.avg_insertion_time) / before.avg_insertion_time
                * 100.0
        } else {
            0.0
        };

        let cache_improvement = after.cache_hit_rate - before.cache_hit_rate;

        // Calculate overall improvement as weighted average
        let overall_improvement = lookup_improvement * 0.3
            + insertion_improvement * 0.2
            + cache_improvement * 0.2
            + (memory_reduction as f64 / before.memory_usage.max(1) as f64) * 100.0 * 0.3;

        OptimizationImprovements {
            memory_reduction,
            lookup_improvement: lookup_improvement.max(0.0),
            insertion_improvement: insertion_improvement.max(0.0),
            cache_improvement: cache_improvement.max(0.0),
            overall_improvement: overall_improvement.max(0.0),
        }
    }

    /// Get optimization statistics
    pub fn get_stats(&self) -> &OptimizationStats {
        &self.stats
    }

    /// Get performance history
    pub fn get_performance_history(&self) -> &[OptimizationMetrics] {
        &self.performance_history
    }

    /// Get current memory pressure level
    pub fn get_memory_pressure(&mut self) -> MemoryPressureLevel {
        self.memory_profiler.collect_statistics();
        self.memory_profiler.get_pressure_level()
    }

    /// Generate optimization report
    pub fn generate_optimization_report(&self) -> OptimizationReport {
        let avg_improvement = if !self.stats.performance_improvements.is_empty() {
            self.stats.performance_improvements.iter().sum::<f64>()
                / self.stats.performance_improvements.len() as f64
        } else {
            0.0
        };

        let success_rate = if self.stats.optimization_runs > 0 {
            (self.stats.successful_optimizations as f64 / self.stats.optimization_runs as f64)
                * 100.0
        } else {
            0.0
        };

        OptimizationReport {
            total_runs: self.stats.optimization_runs,
            successful_runs: self.stats.successful_optimizations,
            success_rate,
            total_optimization_time: self.stats.total_optimization_time,
            average_improvement: avg_improvement,
            total_memory_reclaimed: self.stats.memory_reclaimed,
            last_result: self.stats.last_result.clone(),
            performance_trend: self.calculate_performance_trend(),
        }
    }

    /// Calculate performance trend from history
    fn calculate_performance_trend(&self) -> PerformanceTrend {
        if self.performance_history.len() < 2 {
            return PerformanceTrend::Stable;
        }

        let recent_count = (self.performance_history.len() / 2).max(1);
        let recent_metrics =
            &self.performance_history[self.performance_history.len() - recent_count..];
        let older_metrics =
            &self.performance_history[..self.performance_history.len() - recent_count];

        let recent_avg_lookup =
            recent_metrics.iter().map(|m| m.avg_lookup_time).sum::<f64>() / recent_count as f64;
        let older_avg_lookup = older_metrics.iter().map(|m| m.avg_lookup_time).sum::<f64>()
            / older_metrics.len() as f64;

        let improvement = (older_avg_lookup - recent_avg_lookup) / older_avg_lookup.max(1.0);

        if improvement > 0.1 {
            PerformanceTrend::Improving
        } else if improvement < -0.1 {
            PerformanceTrend::Degrading
        } else {
            PerformanceTrend::Stable
        }
    }
}

/// Comprehensive optimization report
#[derive(Debug, Clone)]
pub struct OptimizationReport {
    pub total_runs: usize,
    pub successful_runs: usize,
    pub success_rate: f64,
    pub total_optimization_time: Duration,
    pub average_improvement: f64,
    pub total_memory_reclaimed: usize,
    pub last_result: OptimizationResult,
    pub performance_trend: PerformanceTrend,
}

/// Performance trend analysis
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Degrading,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_config_creation() {
        let config = OptimizationConfig::default();
        assert!(config.auto_optimize);
        assert!(config.enable_adaptive_backends);
        assert!(config.enable_advanced_indexing);
        assert!(config.enable_bloom_filters);
    }

    #[test]
    fn test_optimization_coordinator_creation() {
        let config = OptimizationConfig::default();
        let coordinator = OptimizationCoordinator::new(config);
        assert!(coordinator.is_ok());
    }

    #[test]
    fn test_performance_trend_calculation() {
        let config = OptimizationConfig::default();
        let mut coordinator = OptimizationCoordinator::new(config).unwrap();

        // Add some performance history
        coordinator.performance_history.push(OptimizationMetrics {
            timestamp: Instant::now(),
            memory_usage: 1000,
            memory_pressure: MemoryPressureLevel::Normal,
            avg_lookup_time: 100.0,
            avg_insertion_time: 50.0,
            cache_hit_rate: 80.0,
            bloom_effectiveness: 70.0,
            index_efficiency: 90.0,
            ops_per_second: 1000.0,
            total_facts: 1000,
        });

        coordinator.performance_history.push(OptimizationMetrics {
            timestamp: Instant::now(),
            memory_usage: 900,
            memory_pressure: MemoryPressureLevel::Normal,
            avg_lookup_time: 80.0, // Improved
            avg_insertion_time: 45.0,
            cache_hit_rate: 85.0,
            bloom_effectiveness: 75.0,
            index_efficiency: 92.0,
            ops_per_second: 1200.0,
            total_facts: 1100,
        });

        let trend = coordinator.calculate_performance_trend();
        assert_eq!(trend, PerformanceTrend::Improving);
    }

    #[test]
    fn test_optimization_improvements_calculation() {
        let config = OptimizationConfig::default();
        let coordinator = OptimizationCoordinator::new(config).unwrap();

        let before = OptimizationMetrics {
            timestamp: Instant::now(),
            memory_usage: 1000,
            memory_pressure: MemoryPressureLevel::Normal,
            avg_lookup_time: 100.0,
            avg_insertion_time: 50.0,
            cache_hit_rate: 80.0,
            bloom_effectiveness: 70.0,
            index_efficiency: 90.0,
            ops_per_second: 1000.0,
            total_facts: 1000,
        };

        let after = OptimizationMetrics {
            timestamp: Instant::now(),
            memory_usage: 800,
            memory_pressure: MemoryPressureLevel::Normal,
            avg_lookup_time: 80.0,
            avg_insertion_time: 40.0,
            cache_hit_rate: 90.0,
            bloom_effectiveness: 80.0,
            index_efficiency: 95.0,
            ops_per_second: 1250.0,
            total_facts: 1000,
        };

        let improvements = coordinator.calculate_improvements(&before, &after);

        assert_eq!(improvements.memory_reduction, 200);
        assert_eq!(improvements.lookup_improvement, 20.0);
        assert_eq!(improvements.insertion_improvement, 20.0);
        assert_eq!(improvements.cache_improvement, 10.0);
        assert!(improvements.overall_improvement > 0.0);
    }
}
