//! Comprehensive Memory Profiler for RETE Network Components
//!
//! This module provides detailed memory usage tracking and adaptive sizing
//! capabilities for all major RETE network components including:
//! - Alpha/Beta/Terminal node collections
//! - Token pools and memory pools  
//! - Pattern caches and fact stores
//! - Node sharing registries
//!
//! The profiler enables automatic memory optimization based on usage patterns,
//! memory pressure detection, and intelligent capacity planning.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Comprehensive memory usage statistics for a single component
#[derive(Debug, Clone)]
pub struct ComponentMemoryStats {
    /// Component identifier
    pub component_name: String,
    /// Current allocated memory in bytes
    pub allocated_bytes: usize,
    /// Peak allocated memory in bytes
    pub peak_allocated_bytes: usize,
    /// Number of allocations
    pub allocation_count: usize,
    /// Number of deallocations
    pub deallocation_count: usize,
    /// Current utilization as percentage (0-100)
    pub utilization_percent: f64,
    /// Memory growth rate (bytes per second)
    pub growth_rate_bytes_per_sec: f64,
    /// Time since last measurement
    pub measurement_timestamp: Instant,
    /// Average object size in bytes
    pub average_object_size: usize,
    /// Fragmentation score (0-100, higher = more fragmented)
    pub fragmentation_score: f64,
}

/// Memory pressure levels for triggering different optimization strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryPressureLevel {
    /// Normal operation, no memory concerns
    Normal,
    /// Moderate pressure, start conservative cleanup
    Moderate,
    /// High pressure, aggressive cleanup needed
    High,
    /// Critical pressure, emergency cleanup required
    Critical,
}

/// Memory profiling configuration
#[derive(Debug, Clone)]
pub struct MemoryProfilerConfig {
    /// How often to collect memory statistics
    pub collection_interval: Duration,
    /// Memory pressure thresholds in bytes
    pub pressure_thresholds: MemoryPressureThresholds,
    /// Whether to enable automatic memory optimization
    pub enable_auto_optimization: bool,
    /// Maximum memory growth rate before triggering optimization (bytes/sec)
    pub max_growth_rate: f64,
    /// History retention duration for trend analysis
    pub history_retention: Duration,
}

#[derive(Debug, Clone)]
pub struct MemoryPressureThresholds {
    /// Threshold for moderate pressure (bytes)
    pub moderate_threshold: usize,
    /// Threshold for high pressure (bytes)
    pub high_threshold: usize,
    /// Threshold for critical pressure (bytes)
    pub critical_threshold: usize,
}

impl Default for MemoryProfilerConfig {
    fn default() -> Self {
        Self {
            collection_interval: Duration::from_secs(10),
            pressure_thresholds: MemoryPressureThresholds {
                moderate_threshold: 100 * 1024 * 1024, // 100MB
                high_threshold: 250 * 1024 * 1024,     // 250MB
                critical_threshold: 500 * 1024 * 1024, // 500MB
            },
            enable_auto_optimization: true,
            max_growth_rate: 10.0 * 1024.0 * 1024.0, // 10MB/sec
            history_retention: <Duration as DurationExt>::from_mins(30),
        }
    }
}

/// Historical memory usage sample for trend analysis
#[derive(Debug, Clone)]
pub struct MemoryUsageSample {
    pub timestamp: Instant,
    pub total_allocated: usize,
    pub components: HashMap<String, ComponentMemoryStats>,
}

/// Central memory profiler for the entire RETE network
#[derive(Debug)]
pub struct ReteMemoryProfiler {
    config: MemoryProfilerConfig,
    component_stats: HashMap<String, ComponentMemoryStats>,
    historical_samples: Vec<MemoryUsageSample>,
    last_collection_time: Instant,
    pressure_level: MemoryPressureLevel,
    optimization_events: Vec<MemoryOptimizationEvent>,
}

/// Record of memory optimization actions taken
#[derive(Debug, Clone)]
pub struct MemoryOptimizationEvent {
    pub timestamp: Instant,
    pub event_type: OptimizationEventType,
    pub component: String,
    pub bytes_freed: usize,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum OptimizationEventType {
    /// Automatic cleanup triggered by pressure
    AutoCleanup,
    /// Capacity reduction to save memory
    CapacityReduction,
    /// Cache eviction to free memory
    CacheEviction,
    /// Pool consolidation to reduce fragmentation
    PoolConsolidation,
}

impl Default for ReteMemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl ReteMemoryProfiler {
    /// Create a new memory profiler with default configuration
    pub fn new() -> Self {
        Self::with_config(MemoryProfilerConfig::default())
    }

    /// Create a new memory profiler with custom configuration
    pub fn with_config(config: MemoryProfilerConfig) -> Self {
        Self {
            config,
            component_stats: HashMap::new(),
            historical_samples: Vec::new(),
            last_collection_time: Instant::now(),
            pressure_level: MemoryPressureLevel::Normal,
            optimization_events: Vec::new(),
        }
    }

    /// Record memory allocation for a component
    pub fn record_allocation(&mut self, component: &str, bytes: usize, object_count: usize) {
        let stats = self.component_stats.entry(component.to_string()).or_insert_with(|| {
            ComponentMemoryStats {
                component_name: component.to_string(),
                allocated_bytes: 0,
                peak_allocated_bytes: 0,
                allocation_count: 0,
                deallocation_count: 0,
                utilization_percent: 0.0,
                growth_rate_bytes_per_sec: 0.0,
                measurement_timestamp: Instant::now(),
                average_object_size: 0,
                fragmentation_score: 0.0,
            }
        });

        let previous_bytes = stats.allocated_bytes;
        stats.allocated_bytes += bytes;
        stats.allocation_count += object_count;

        if stats.allocated_bytes > stats.peak_allocated_bytes {
            stats.peak_allocated_bytes = stats.allocated_bytes;
        }

        // Calculate growth rate
        let now = Instant::now();
        let time_delta = now.duration_since(stats.measurement_timestamp).as_secs_f64();
        if time_delta > 0.0 {
            let bytes_delta = stats.allocated_bytes as f64 - previous_bytes as f64;
            stats.growth_rate_bytes_per_sec = bytes_delta / time_delta;
            stats.measurement_timestamp = now;
        }

        // Update average object size
        if stats.allocation_count > 0 {
            stats.average_object_size = stats.allocated_bytes / stats.allocation_count;
        }

        // Check if we need to update pressure level
        self.update_pressure_level();
    }

    /// Record memory deallocation for a component
    pub fn record_deallocation(&mut self, component: &str, bytes: usize, object_count: usize) {
        if let Some(stats) = self.component_stats.get_mut(component) {
            stats.allocated_bytes = stats.allocated_bytes.saturating_sub(bytes);
            stats.deallocation_count += object_count;

            // Recalculate average object size
            let total_objects = stats.allocation_count.saturating_sub(stats.deallocation_count);
            if total_objects > 0 {
                stats.average_object_size = stats.allocated_bytes / total_objects;
            } else {
                stats.average_object_size = 0;
            }

            stats.measurement_timestamp = Instant::now();
        }

        self.update_pressure_level();
    }

    /// Update memory utilization for a component
    pub fn update_utilization(
        &mut self,
        component: &str,
        used_capacity: usize,
        total_capacity: usize,
    ) {
        if let Some(stats) = self.component_stats.get_mut(component) {
            stats.utilization_percent = if total_capacity > 0 {
                (used_capacity as f64 / total_capacity as f64) * 100.0
            } else {
                0.0
            };

            // Calculate fragmentation score based on utilization and allocation patterns
            let fragmentation_score = Self::calculate_fragmentation_score_static(stats);
            stats.fragmentation_score = fragmentation_score;
            stats.measurement_timestamp = Instant::now();
        }
    }

    /// Collect memory statistics from all tracked components
    pub fn collect_statistics(&mut self) {
        let now = Instant::now();

        // Only collect if enough time has passed
        if now.duration_since(self.last_collection_time) < self.config.collection_interval {
            return;
        }

        // Create a snapshot of current memory usage
        let sample = MemoryUsageSample {
            timestamp: now,
            total_allocated: self.get_total_allocated_memory(),
            components: self.component_stats.clone(),
        };

        self.historical_samples.push(sample);
        self.last_collection_time = now;

        // Clean up old historical data
        let cutoff = now - self.config.history_retention;
        self.historical_samples.retain(|sample| sample.timestamp >= cutoff);

        // Update pressure level and trigger optimization if needed
        self.update_pressure_level();

        if self.config.enable_auto_optimization {
            self.trigger_auto_optimization();
        }

        debug!(
            total_memory = self.get_total_allocated_memory(),
            pressure_level = ?self.pressure_level,
            components = self.component_stats.len(),
            "Memory statistics collected"
        );
    }

    /// Get current memory pressure level
    pub fn get_pressure_level(&self) -> MemoryPressureLevel {
        self.pressure_level
    }

    /// Get total allocated memory across all components
    pub fn get_total_allocated_memory(&self) -> usize {
        self.component_stats.values().map(|stats| stats.allocated_bytes).sum()
    }

    /// Get memory statistics for a specific component
    pub fn get_component_stats(&self, component: &str) -> Option<&ComponentMemoryStats> {
        self.component_stats.get(component)
    }

    /// Get memory statistics for all components
    pub fn get_all_component_stats(&self) -> &HashMap<String, ComponentMemoryStats> {
        &self.component_stats
    }

    /// Get optimization events log
    pub fn get_optimization_events(&self) -> &[MemoryOptimizationEvent] {
        &self.optimization_events
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &MemoryProfilerConfig {
        &self.config
    }

    /// Get the last collection time
    pub fn get_last_collection_time(&self) -> std::time::Instant {
        self.last_collection_time
    }

    /// Generate memory usage report
    pub fn generate_report(&self) -> MemoryUsageReport {
        let total_allocated = self.get_total_allocated_memory();
        let total_peak =
            self.component_stats.values().map(|stats| stats.peak_allocated_bytes).sum();

        let components_by_usage: Vec<_> =
            self.component_stats.values().cloned().collect::<Vec<_>>();

        // Calculate trends from historical data
        let growth_trend = self.calculate_memory_growth_trend();
        let allocation_trend = self.calculate_allocation_trend();

        MemoryUsageReport {
            timestamp: Instant::now(),
            total_allocated_bytes: total_allocated,
            total_peak_bytes: total_peak,
            pressure_level: self.pressure_level,
            component_count: self.component_stats.len(),
            components_by_usage,
            memory_growth_trend: growth_trend,
            allocation_trend,
            optimization_events_count: self.optimization_events.len(),
            recommendations: self.generate_optimization_recommendations(),
        }
    }

    /// Clear all statistics and reset the profiler
    pub fn reset(&mut self) {
        self.component_stats.clear();
        self.historical_samples.clear();
        self.optimization_events.clear();
        self.pressure_level = MemoryPressureLevel::Normal;
        self.last_collection_time = Instant::now();

        info!("Memory profiler reset");
    }

    // Private helper methods

    fn update_pressure_level(&mut self) {
        let total_memory = self.get_total_allocated_memory();
        let thresholds = &self.config.pressure_thresholds;

        let new_level = if total_memory >= thresholds.critical_threshold {
            MemoryPressureLevel::Critical
        } else if total_memory >= thresholds.high_threshold {
            MemoryPressureLevel::High
        } else if total_memory >= thresholds.moderate_threshold {
            MemoryPressureLevel::Moderate
        } else {
            MemoryPressureLevel::Normal
        };

        if new_level != self.pressure_level {
            warn!(
                old_level = ?self.pressure_level,
                new_level = ?new_level,
                total_memory = total_memory,
                "Memory pressure level changed"
            );
            self.pressure_level = new_level;
        }
    }

    #[allow(dead_code)]
    fn calculate_fragmentation_score(&self, stats: &ComponentMemoryStats) -> f64 {
        Self::calculate_fragmentation_score_static(stats)
    }

    fn calculate_fragmentation_score_static(stats: &ComponentMemoryStats) -> f64 {
        // Simple fragmentation heuristic based on allocation patterns
        if stats.allocation_count == 0 {
            return 0.0;
        }

        let alloc_dealloc_ratio = if stats.deallocation_count > 0 {
            stats.allocation_count as f64 / stats.deallocation_count as f64
        } else {
            1.0
        };

        // Higher fragmentation when there are many allocations/deallocations
        // and low utilization
        let allocation_churn = (stats.allocation_count + stats.deallocation_count) as f64;
        let utilization_factor = (100.0 - stats.utilization_percent) / 100.0;

        (allocation_churn / 1000.0 * utilization_factor * alloc_dealloc_ratio).min(100.0)
    }

    fn trigger_auto_optimization(&mut self) {
        match self.pressure_level {
            MemoryPressureLevel::Critical => {
                self.perform_emergency_cleanup();
            }
            MemoryPressureLevel::High => {
                self.perform_aggressive_cleanup();
            }
            MemoryPressureLevel::Moderate => {
                self.perform_conservative_cleanup();
            }
            MemoryPressureLevel::Normal => {
                // Check for excessive growth rates
                self.check_growth_rates();
            }
        }
    }

    fn perform_emergency_cleanup(&mut self) {
        info!("Performing emergency memory cleanup due to critical pressure");

        // Record optimization event
        self.optimization_events.push(MemoryOptimizationEvent {
            timestamp: Instant::now(),
            event_type: OptimizationEventType::AutoCleanup,
            component: "all".to_string(),
            bytes_freed: 0, // Will be updated by actual cleanup
            description: "Emergency cleanup triggered by critical memory pressure".to_string(),
        });
    }

    fn perform_aggressive_cleanup(&mut self) {
        info!("Performing aggressive memory cleanup due to high pressure");

        self.optimization_events.push(MemoryOptimizationEvent {
            timestamp: Instant::now(),
            event_type: OptimizationEventType::AutoCleanup,
            component: "all".to_string(),
            bytes_freed: 0,
            description: "Aggressive cleanup triggered by high memory pressure".to_string(),
        });
    }

    fn perform_conservative_cleanup(&mut self) {
        debug!("Performing conservative memory cleanup due to moderate pressure");

        self.optimization_events.push(MemoryOptimizationEvent {
            timestamp: Instant::now(),
            event_type: OptimizationEventType::AutoCleanup,
            component: "all".to_string(),
            bytes_freed: 0,
            description: "Conservative cleanup triggered by moderate memory pressure".to_string(),
        });
    }

    fn check_growth_rates(&mut self) {
        for (component, stats) in &self.component_stats {
            if stats.growth_rate_bytes_per_sec > self.config.max_growth_rate {
                warn!(
                    component = component,
                    growth_rate = stats.growth_rate_bytes_per_sec,
                    max_rate = self.config.max_growth_rate,
                    "Component exceeding maximum growth rate"
                );

                self.optimization_events.push(MemoryOptimizationEvent {
                    timestamp: Instant::now(),
                    event_type: OptimizationEventType::CapacityReduction,
                    component: component.clone(),
                    bytes_freed: 0,
                    description: format!(
                        "Growth rate limit exceeded: {:.1} bytes/sec",
                        stats.growth_rate_bytes_per_sec
                    ),
                });
            }
        }
    }

    fn calculate_memory_growth_trend(&self) -> f64 {
        if self.historical_samples.len() < 2 {
            return 0.0;
        }

        let recent = &self.historical_samples[self.historical_samples.len() - 1];
        let older = &self.historical_samples[0];

        let time_delta = recent.timestamp.duration_since(older.timestamp).as_secs_f64();
        if time_delta == 0.0 {
            return 0.0;
        }

        let memory_delta = recent.total_allocated as f64 - older.total_allocated as f64;
        memory_delta / time_delta
    }

    fn calculate_allocation_trend(&self) -> f64 {
        if self.historical_samples.len() < 2 {
            return 0.0;
        }

        let recent = &self.historical_samples[self.historical_samples.len() - 1];
        let older = &self.historical_samples[0];

        let recent_total_allocs: usize =
            recent.components.values().map(|stats| stats.allocation_count).sum();
        let older_total_allocs: usize =
            older.components.values().map(|stats| stats.allocation_count).sum();

        let time_delta = recent.timestamp.duration_since(older.timestamp).as_secs_f64();
        if time_delta == 0.0 {
            return 0.0;
        }

        let alloc_delta = recent_total_allocs as f64 - older_total_allocs as f64;
        alloc_delta / time_delta
    }

    fn generate_optimization_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check for high fragmentation components
        for (component, stats) in &self.component_stats {
            if stats.fragmentation_score > 50.0 {
                recommendations.push(format!(
                    "Consider consolidating {} - fragmentation score: {:.1}%",
                    component, stats.fragmentation_score
                ));
            }

            if stats.utilization_percent < 30.0 && stats.allocated_bytes > 1024 * 1024 {
                recommendations.push(format!(
                    "Reduce capacity for {} - low utilization: {:.1}%",
                    component, stats.utilization_percent
                ));
            }

            if stats.growth_rate_bytes_per_sec > self.config.max_growth_rate * 0.8 {
                recommendations.push(format!(
                    "Monitor growth rate for {} - approaching limit: {:.1} bytes/sec",
                    component, stats.growth_rate_bytes_per_sec
                ));
            }
        }

        recommendations
    }
}

/// Comprehensive memory usage report
#[derive(Debug, Clone)]
pub struct MemoryUsageReport {
    pub timestamp: Instant,
    pub total_allocated_bytes: usize,
    pub total_peak_bytes: usize,
    pub pressure_level: MemoryPressureLevel,
    pub component_count: usize,
    pub components_by_usage: Vec<ComponentMemoryStats>,
    pub memory_growth_trend: f64, // bytes per second
    pub allocation_trend: f64,    // allocations per second
    pub optimization_events_count: usize,
    pub recommendations: Vec<String>,
}

impl std::fmt::Display for MemoryUsageReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Memory Usage Report\n\
             ==================\n\
             Timestamp: {:?}\n\
             Total Allocated: {:.2} MB\n\
             Total Peak: {:.2} MB\n\
             Pressure Level: {:?}\n\
             Components: {}\n\
             Growth Trend: {:.1} bytes/sec\n\
             Allocation Trend: {:.1} allocs/sec\n\
             Optimization Events: {}\n\
             Recommendations: {}\n",
            self.timestamp,
            self.total_allocated_bytes as f64 / 1024.0 / 1024.0,
            self.total_peak_bytes as f64 / 1024.0 / 1024.0,
            self.pressure_level,
            self.component_count,
            self.memory_growth_trend,
            self.allocation_trend,
            self.optimization_events_count,
            self.recommendations.len()
        )
    }
}

// Extension trait for Duration to add minutes
trait DurationExt {
    fn from_mins(mins: u64) -> Duration;
}

impl DurationExt for Duration {
    fn from_mins(mins: u64) -> Duration {
        Duration::from_secs(mins * 60)
    }
}
