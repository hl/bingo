//! Advanced performance profiling for the Bingo rules engine
//!
//! This module provides comprehensive performance profiling capabilities including
//! real-time timing measurements, operation tracking, bottleneck detection, and
//! detailed performance analytics.

use crate::unified_statistics::UnifiedStats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Real-time performance profiler for engine operations
#[derive(Debug, Clone)]
pub struct EngineProfiler {
    /// Operation timing records
    timings: Arc<Mutex<HashMap<String, Vec<Duration>>>>,
    /// Operation counters
    counters: Arc<Mutex<HashMap<String, u64>>>,
    /// Active operation start times
    active_operations: Arc<Mutex<HashMap<String, Instant>>>,
    /// Performance thresholds for alerting
    thresholds: PerformanceThresholds,
    /// Enable detailed profiling
    enabled: bool,
}

/// Performance thresholds for different operations
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// Maximum acceptable time for rule compilation (milliseconds)
    pub rule_compilation_ms: u64,
    /// Maximum acceptable time for fact processing (milliseconds)
    pub fact_processing_ms: u64,
    /// Maximum acceptable time for RETE network processing (milliseconds)
    pub rete_processing_ms: u64,
    /// Maximum acceptable time for action execution (milliseconds)
    pub action_execution_ms: u64,
    /// Maximum acceptable memory usage (bytes)
    pub max_memory_bytes: u64,
    /// Maximum acceptable cache miss rate (percentage)
    pub max_cache_miss_rate: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            rule_compilation_ms: 100,
            fact_processing_ms: 50,
            rete_processing_ms: 30,
            action_execution_ms: 20,
            max_memory_bytes: 100_000_000, // 100MB
            max_cache_miss_rate: 20.0,     // 20%
        }
    }
}

/// Detailed performance metrics for a specific operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetrics {
    /// Operation name
    pub name: String,
    /// Total number of invocations
    pub invocations: u64,
    /// Minimum execution time (microseconds)
    pub min_duration_us: u64,
    /// Maximum execution time (microseconds)
    pub max_duration_us: u64,
    /// Average execution time (microseconds)
    pub avg_duration_us: u64,
    /// 95th percentile execution time (microseconds)
    pub p95_duration_us: u64,
    /// Total time spent in this operation (microseconds)
    pub total_duration_us: u64,
    /// Standard deviation of execution times (microseconds)
    pub std_deviation_us: u64,
}

/// Performance bottleneck analysis result
#[derive(Debug, Clone)]
pub struct BottleneckAnalysis {
    /// Operation causing the bottleneck
    pub operation: String,
    /// Severity of the bottleneck (1-10)
    pub severity: u8,
    /// Description of the bottleneck
    pub description: String,
    /// Suggested optimization
    pub suggestion: String,
    /// Impact on overall performance (percentage)
    pub performance_impact: f64,
}

/// Comprehensive performance report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// Overall performance score (0-100)
    pub overall_score: f64,
    /// Individual operation metrics
    pub operation_metrics: Vec<OperationMetrics>,
    /// Identified bottlenecks
    pub bottlenecks: Vec<BottleneckAnalysis>,
    /// Unified statistics
    pub unified_stats: UnifiedStats,
    /// Performance alerts
    pub alerts: Vec<PerformanceAlert>,
    /// Report generation timestamp
    pub timestamp: Instant,
}

/// Performance alert for threshold violations
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    /// Alert severity level
    pub severity: AlertSeverity,
    /// Operation that triggered the alert
    pub operation: String,
    /// Alert message
    pub message: String,
    /// Actual value that exceeded threshold
    pub actual_value: f64,
    /// Threshold value that was exceeded
    pub threshold_value: f64,
    /// Timestamp when alert was generated
    pub timestamp: Instant,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl EngineProfiler {
    /// Create a new performance profiler
    pub fn new() -> Self {
        Self {
            timings: Arc::new(Mutex::new(HashMap::new())),
            counters: Arc::new(Mutex::new(HashMap::new())),
            active_operations: Arc::new(Mutex::new(HashMap::new())),
            thresholds: PerformanceThresholds::default(),
            enabled: true,
        }
    }

    /// Create a profiler with custom thresholds
    pub fn with_thresholds(thresholds: PerformanceThresholds) -> Self {
        Self {
            timings: Arc::new(Mutex::new(HashMap::new())),
            counters: Arc::new(Mutex::new(HashMap::new())),
            active_operations: Arc::new(Mutex::new(HashMap::new())),
            thresholds,
            enabled: true,
        }
    }

    /// Enable or disable profiling
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Start timing an operation
    pub fn start_operation(&self, operation: &str) {
        if !self.enabled {
            return;
        }

        if let Ok(mut active) = self.active_operations.lock() {
            active.insert(operation.to_string(), Instant::now());
        }

        if let Ok(mut counters) = self.counters.lock() {
            *counters.entry(operation.to_string()).or_insert(0) += 1;
        }
    }

    /// End timing an operation and record the duration
    pub fn end_operation(&self, operation: &str) -> Option<Duration> {
        if !self.enabled {
            return None;
        }

        let duration = {
            if let Ok(mut active) = self.active_operations.lock() {
                active.remove(operation).map(|start_time| start_time.elapsed())
            } else {
                None
            }
        };

        if let Some(duration) = duration {
            if let Ok(mut timings) = self.timings.lock() {
                timings.entry(operation.to_string()).or_insert_with(Vec::new).push(duration);
            }
        }

        duration
    }

    /// Record a duration for an operation directly
    pub fn record_duration(&self, operation: &str, duration: Duration) {
        if !self.enabled {
            return;
        }

        if let Ok(mut timings) = self.timings.lock() {
            timings.entry(operation.to_string()).or_insert_with(Vec::new).push(duration);
        }

        if let Ok(mut counters) = self.counters.lock() {
            *counters.entry(operation.to_string()).or_insert(0) += 1;
        }
    }

    /// Time a closure and record the duration
    pub fn time_operation<T, F>(&self, operation: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        if !self.enabled {
            return f();
        }

        self.start_operation(operation);
        let result = f();
        self.end_operation(operation);
        result
    }

    /// Get metrics for a specific operation
    pub fn get_operation_metrics(&self, operation: &str) -> Option<OperationMetrics> {
        let timings = self.timings.lock().ok()?;
        let counters = self.counters.lock().ok()?;

        let durations = timings.get(operation)?;
        let invocations = *counters.get(operation).unwrap_or(&0);

        if durations.is_empty() {
            return None;
        }

        let mut sorted_durations = durations.clone();
        sorted_durations.sort();

        let min_duration = *sorted_durations.first().unwrap();
        let max_duration = *sorted_durations.last().unwrap();
        let total_duration: Duration = sorted_durations.iter().sum();
        let avg_duration = total_duration / sorted_durations.len() as u32;

        // Calculate 95th percentile
        let p95_index = (sorted_durations.len() as f64 * 0.95) as usize;
        let p95_duration = sorted_durations
            .get(p95_index.min(sorted_durations.len() - 1))
            .copied()
            .unwrap_or(max_duration);

        // Calculate standard deviation
        let variance: f64 = sorted_durations
            .iter()
            .map(|d| {
                let diff = d.as_secs_f64() - avg_duration.as_secs_f64();
                diff * diff
            })
            .sum::<f64>()
            / sorted_durations.len() as f64;
        let std_deviation = Duration::from_secs_f64(variance.sqrt());

        Some(OperationMetrics {
            name: operation.to_string(),
            invocations,
            min_duration_us: min_duration.as_micros() as u64,
            max_duration_us: max_duration.as_micros() as u64,
            avg_duration_us: avg_duration.as_micros() as u64,
            p95_duration_us: p95_duration.as_micros() as u64,
            total_duration_us: total_duration.as_micros() as u64,
            std_deviation_us: std_deviation.as_micros() as u64,
        })
    }

    /// Get all operation metrics
    pub fn get_all_metrics(&self) -> Vec<OperationMetrics> {
        let timings = self.timings.lock().unwrap();
        let mut metrics = Vec::new();

        for operation in timings.keys() {
            if let Some(metric) = self.get_operation_metrics(operation) {
                metrics.push(metric);
            }
        }

        // Sort by total time spent (descending)
        metrics.sort_by(|a, b| b.total_duration_us.cmp(&a.total_duration_us));
        metrics
    }

    /// Analyze performance bottlenecks
    pub fn analyze_bottlenecks(&self) -> Vec<BottleneckAnalysis> {
        let metrics = self.get_all_metrics();
        let mut bottlenecks = Vec::new();

        if metrics.is_empty() {
            return bottlenecks;
        }

        let total_time_us: u64 = metrics.iter().map(|m| m.total_duration_us).sum();

        for metric in &metrics {
            let impact = if total_time_us > 0 {
                (metric.total_duration_us as f64 / total_time_us as f64) * 100.0
            } else {
                0.0
            };

            // Check for slow operations (high average duration)
            if metric.avg_duration_us > 100_000 {
                // 100ms in microseconds
                let severity = match metric.avg_duration_us {
                    100_001..=500_000 => 3,   // 100-500ms
                    500_001..=1_000_000 => 6, // 500ms-1s
                    _ => 9,                   // >1s
                };

                bottlenecks.push(BottleneckAnalysis {
                    operation: metric.name.clone(),
                    severity,
                    description: format!(
                        "Operation '{}' has high average duration: {}μs",
                        metric.name, metric.avg_duration_us
                    ),
                    suggestion: "Consider optimizing the algorithm or caching results".to_string(),
                    performance_impact: impact,
                });
            }

            // Check for operations with high variability
            if metric.std_deviation_us > 50_000 {
                // 50ms in microseconds
                bottlenecks.push(BottleneckAnalysis {
                    operation: metric.name.clone(),
                    severity: 4,
                    description: format!(
                        "Operation '{}' has high timing variability (std dev: {}μs)",
                        metric.name, metric.std_deviation_us
                    ),
                    suggestion: "Investigate inconsistent performance causes".to_string(),
                    performance_impact: impact * 0.5, // Lower impact for variability
                });
            }

            // Check for operations consuming significant total time
            if impact > 30.0 {
                bottlenecks.push(BottleneckAnalysis {
                    operation: metric.name.clone(),
                    severity: 7,
                    description: format!(
                        "Operation '{}' consumes {:.1}% of total execution time",
                        metric.name, impact
                    ),
                    suggestion: "Primary optimization target - focus optimization efforts here"
                        .to_string(),
                    performance_impact: impact,
                });
            }
        }

        // Sort by severity (descending)
        bottlenecks.sort_by(|a, b| b.severity.cmp(&a.severity));
        bottlenecks
    }

    /// Generate performance alerts based on thresholds
    pub fn generate_alerts(&self) -> Vec<PerformanceAlert> {
        let mut alerts = Vec::new();
        let metrics = self.get_all_metrics();

        for metric in metrics {
            // Check rule compilation threshold
            if metric.name.contains("rule_compilation")
                && metric.avg_duration_us > (self.thresholds.rule_compilation_ms * 1000)
            {
                alerts.push(PerformanceAlert {
                    severity: AlertSeverity::Warning,
                    operation: metric.name.clone(),
                    message: "Rule compilation exceeds threshold".to_string(),
                    actual_value: metric.avg_duration_us as f64 / 1000.0, // Convert to ms
                    threshold_value: self.thresholds.rule_compilation_ms as f64,
                    timestamp: Instant::now(),
                });
            }

            // Check fact processing threshold
            if metric.name.contains("fact_processing")
                && metric.avg_duration_us > (self.thresholds.fact_processing_ms * 1000)
            {
                alerts.push(PerformanceAlert {
                    severity: AlertSeverity::Critical,
                    operation: metric.name.clone(),
                    message: "Fact processing exceeds critical threshold".to_string(),
                    actual_value: metric.avg_duration_us as f64 / 1000.0,
                    threshold_value: self.thresholds.fact_processing_ms as f64,
                    timestamp: Instant::now(),
                });
            }

            // Check RETE processing threshold
            if metric.name.contains("rete_processing")
                && metric.avg_duration_us > (self.thresholds.rete_processing_ms * 1000)
            {
                alerts.push(PerformanceAlert {
                    severity: AlertSeverity::Warning,
                    operation: metric.name.clone(),
                    message: "RETE network processing is slow".to_string(),
                    actual_value: metric.avg_duration_us as f64 / 1000.0,
                    threshold_value: self.thresholds.rete_processing_ms as f64,
                    timestamp: Instant::now(),
                });
            }
        }

        alerts
    }

    /// Generate comprehensive performance report
    pub fn generate_report(&self, unified_stats: UnifiedStats) -> PerformanceReport {
        let operation_metrics = self.get_all_metrics();
        let bottlenecks = self.analyze_bottlenecks();
        let alerts = self.generate_alerts();

        // Calculate overall performance score
        let mut score = 100.0;

        // Deduct points for bottlenecks
        for bottleneck in &bottlenecks {
            score -= (bottleneck.severity as f64) * 2.0;
        }

        // Deduct points for alerts
        for alert in &alerts {
            let deduction = match alert.severity {
                AlertSeverity::Info => 1.0,
                AlertSeverity::Warning => 5.0,
                AlertSeverity::Critical => 15.0,
            };
            score -= deduction;
        }

        // Factor in cache hit rates
        let cache_hit_rate = unified_stats.overall_cache_hit_rate();
        if cache_hit_rate < 80.0 {
            score -= (80.0 - cache_hit_rate) * 0.5;
        }

        score = score.clamp(0.0, 100.0);

        PerformanceReport {
            overall_score: score,
            operation_metrics,
            bottlenecks,
            unified_stats,
            alerts,
            timestamp: Instant::now(),
        }
    }

    /// Reset all profiling data
    pub fn reset(&self) {
        if let Ok(mut timings) = self.timings.lock() {
            timings.clear();
        }
        if let Ok(mut counters) = self.counters.lock() {
            counters.clear();
        }
        if let Ok(mut active) = self.active_operations.lock() {
            active.clear();
        }
    }

    /// Export profiling data as JSON
    pub fn export_json(&self) -> serde_json::Result<String> {
        let metrics = self.get_all_metrics();
        serde_json::to_string_pretty(&metrics)
    }
}

impl Default for EngineProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience macro for timing operations
#[macro_export]
macro_rules! profile_operation {
    ($profiler:expr, $operation:expr, $code:block) => {
        $profiler.time_operation($operation, || $code)
    };
}

/// Scoped profiling guard that automatically ends profiling when dropped
pub struct ProfileGuard<'a> {
    profiler: &'a EngineProfiler,
    operation: String,
}

impl<'a> ProfileGuard<'a> {
    pub fn new(profiler: &'a EngineProfiler, operation: String) -> Self {
        profiler.start_operation(&operation);
        Self { profiler, operation }
    }
}

impl<'a> Drop for ProfileGuard<'a> {
    fn drop(&mut self) {
        self.profiler.end_operation(&self.operation);
    }
}

/// Create a scoped profiling guard
#[macro_export]
macro_rules! profile_scope {
    ($profiler:expr, $operation:expr) => {
        let _guard = $crate::profiler::ProfileGuard::new($profiler, $operation.to_string());
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_basic_profiling() {
        let profiler = EngineProfiler::new();

        profiler.start_operation("test_op");
        // Simulate work with a simple computation instead of sleep
        let mut sum = 0;
        for i in 0..1000 {
            sum += i;
        }
        let duration = profiler.end_operation("test_op");

        assert!(duration.is_some());
        assert!(duration.unwrap() > Duration::from_nanos(0));

        let metrics = profiler.get_operation_metrics("test_op");
        assert!(metrics.is_some());
        assert_eq!(metrics.unwrap().invocations, 1);

        // Use the sum to prevent compiler optimization
        assert!(sum > 0);
    }

    #[test]
    fn test_time_operation_macro() {
        let profiler = EngineProfiler::new();

        let result = profiler.time_operation("test_op", || {
            // Simulate work with computation instead of sleep
            let mut work_result = 0;
            for i in 0..500 {
                work_result += i * i;
            }
            // Use work_result to prevent optimization
            assert!(work_result > 0);
            42
        });

        assert_eq!(result, 42);

        let metrics = profiler.get_operation_metrics("test_op");
        assert!(metrics.is_some());
        assert!(metrics.unwrap().avg_duration_us > 0); // Should have some duration
    }

    #[test]
    fn test_profiler_disable() {
        let mut profiler = EngineProfiler::new();
        profiler.set_enabled(false);

        profiler.start_operation("test_op");
        // Simulate work with computation instead of sleep
        let mut sum = 0;
        for i in 0..1000 {
            sum += i;
        }
        let duration = profiler.end_operation("test_op");

        assert!(duration.is_none());

        let metrics = profiler.get_operation_metrics("test_op");
        assert!(metrics.is_none());

        // Use the sum to prevent compiler optimization
        assert!(sum > 0);
    }
}
