//! Metrics and observability for the Bingo Rules Engine API
//!
//! This module provides structured metrics export for monitoring API performance,
//! cache efficiency, and resource utilization in production environments.

use prometheus::{
    Encoder, IntCounter, IntGauge, TextEncoder, register_int_counter_with_registry,
    register_int_gauge_with_registry,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Metrics collector for the Bingo API
#[derive(Clone, Debug)]
pub struct ApiMetrics {
    // Performance tracking
    performance_data: Arc<Mutex<PerformanceData>>,

    // Prometheus registry
    prometheus_registry: prometheus::Registry,

    // Custom gauges and counters
    /// Gauge for current memory usage in MB.
    pub memory_usage_mb: IntGauge,
    /// Counter for how many times incremental processing is activated.
    pub incremental_processing_activated: IntCounter,
    /// Counter for total security violations.
    pub security_violations_total: IntCounter,
    /// Counter for total security rejections.
    pub security_rejections_total: IntCounter,
}

/// Performance data for metrics calculation
#[derive(Debug, Default)]
struct PerformanceData {
    facts_per_second: f64,
    average_memory_per_request: f64,
    active_requests: u64,
    total_memory_usage: u64,
}

impl ApiMetrics {
    /// Initialize metrics
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing API metrics");

        // Create Prometheus registry
        let prometheus_registry = prometheus::Registry::new();

        // Register custom metrics
        let memory_usage_mb = register_int_gauge_with_registry!(
            "bingo_memory_usage_mb",
            "Current process memory usage in megabytes (RSS).",
            &prometheus_registry
        )?;
        let incremental_processing_activated = register_int_counter_with_registry!(
            "bingo_incremental_processing_activated_total",
            "Total number of times incremental processing has been activated.",
            &prometheus_registry
        )?;
        let security_violations_total = register_int_counter_with_registry!(
            "bingo_security_violations_total",
            "Total number of security violations detected.",
            &prometheus_registry
        )?;
        let security_rejections_total = register_int_counter_with_registry!(
            "bingo_security_rejections_total",
            "Total number of requests rejected due to security policies.",
            &prometheus_registry
        )?;

        Ok(Self {
            performance_data: Arc::new(Mutex::new(PerformanceData::default())),
            prometheus_registry,
            memory_usage_mb,
            incremental_processing_activated,
            security_violations_total,
            security_rejections_total,
        })
    }

    /// Record a request with its metadata
    pub fn record_request(&self, method: &str, path: &str, status_code: u16, duration: Duration) {
        debug!(
            method = method,
            path = path,
            status_code = status_code,
            duration_ms = duration.as_millis(),
            "Recorded request metrics"
        );
    }

    /// Record evaluation metrics
    pub fn record_evaluation(
        &self,
        rules_processed: usize,
        facts_processed: usize,
        rules_fired: usize,
        duration: Duration,
        is_streaming: bool,
    ) {
        // Calculate facts/second
        let facts_per_second = if duration.as_secs_f64() > 0.0 {
            facts_processed as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        // Update performance data
        if let Ok(mut perf_data) = self.performance_data.lock() {
            perf_data.facts_per_second = facts_per_second;
        }

        debug!(
            rules_processed = rules_processed,
            facts_processed = facts_processed,
            rules_fired = rules_fired,
            duration_ms = duration.as_millis(),
            facts_per_second = facts_per_second,
            is_streaming = is_streaming,
            "Recorded evaluation metrics"
        );
    }

    /// Record cache metrics
    pub fn record_cache_hit(&self, cache_type: &str) {
        debug!(cache_type = cache_type, "Recorded cache hit");
    }

    /// Record cache miss
    pub fn record_cache_miss(&self, cache_type: &str) {
        debug!(cache_type = cache_type, "Recorded cache miss");
    }

    /// Record security violation
    pub fn record_security_violation(&self, violation_type: &str, severity: &str) {
        self.security_violations_total.inc();
        if severity == "error" || severity == "critical" || violation_type.contains("rejected") {
            self.security_rejections_total.inc();
        }
        debug!(
            violation_type = violation_type,
            severity = severity,
            "Recorded security violation"
        );
    }

    /// Record memory usage per request
    pub fn record_memory_usage(&self, memory_bytes: u64) {
        if let Ok(mut perf_data) = self.performance_data.lock() {
            perf_data.total_memory_usage = memory_bytes;
            // Calculate average memory per active request
            if perf_data.active_requests > 0 {
                perf_data.average_memory_per_request =
                    memory_bytes as f64 / perf_data.active_requests as f64;
            }
        }

        debug!(memory_bytes = memory_bytes, "Recorded memory usage");
    }

    /// Increment active requests counter
    pub fn increment_active_requests(&self) {
        if let Ok(mut perf_data) = self.performance_data.lock() {
            perf_data.active_requests += 1;
        }
    }

    /// Decrement active requests counter
    pub fn decrement_active_requests(&self) {
        if let Ok(mut perf_data) = self.performance_data.lock() {
            perf_data.active_requests = perf_data.active_requests.saturating_sub(1);
        }
    }

    /// Get current performance statistics
    pub fn get_performance_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if let Ok(perf_data) = self.performance_data.lock() {
            stats.insert("facts_per_second".to_string(), perf_data.facts_per_second);
            stats.insert(
                "average_memory_per_request_bytes".to_string(),
                perf_data.average_memory_per_request,
            );
            stats.insert(
                "active_requests".to_string(),
                perf_data.active_requests as f64,
            );
            stats.insert(
                "total_memory_usage_bytes".to_string(),
                perf_data.total_memory_usage as f64,
            );
        }

        stats
    }

    /// Export Prometheus metrics
    pub fn export_prometheus(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Use proper Prometheus encoder for production-grade metrics export
        let stats = self.get_performance_stats();

        let encoder = TextEncoder::new();
        let metric_families = self.prometheus_registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;

        // Add our runtime stats to the Prometheus output
        let mut output = String::from_utf8(buffer)?;

        output.push_str("# HELP bingo_api_info API information\n");
        output.push_str("# TYPE bingo_api_info gauge\n");
        output.push_str("bingo_api_info{version=\"1.0.0\"} 1\n");

        for (key, value) in stats {
            output.push_str(&format!("# HELP bingo_{} Bingo API runtime metric\n", key));
            output.push_str(&format!("# TYPE bingo_{} gauge\n", key));
            output.push_str(&format!("bingo_{} {}\n", key, value));
        }

        Ok(output)
    }
}

/// Metrics middleware for tracking request metrics
#[derive(Clone, Debug)]
pub struct MetricsMiddleware {
    metrics: Arc<ApiMetrics>,
}

impl MetricsMiddleware {
    pub fn new(metrics: Arc<ApiMetrics>) -> Self {
        Self { metrics }
    }

    /// Start tracking a request
    pub fn start_request(&self) -> RequestTracker {
        self.metrics.increment_active_requests();
        RequestTracker { metrics: self.metrics.clone(), start_time: Instant::now() }
    }
}

/// Request tracker for measuring request duration and cleanup
pub struct RequestTracker {
    metrics: Arc<ApiMetrics>,
    start_time: Instant,
}

impl RequestTracker {
    /// Finish tracking a request
    pub fn finish(self, method: &str, path: &str, status_code: u16) {
        let duration = self.start_time.elapsed();
        self.metrics.record_request(method, path, status_code, duration);
        self.metrics.decrement_active_requests();
    }

    /// Record evaluation metrics for this request
    pub fn record_evaluation(
        &self,
        rules_processed: usize,
        facts_processed: usize,
        rules_fired: usize,
        evaluation_duration: Duration,
        is_streaming: bool,
    ) {
        self.metrics.record_evaluation(
            rules_processed,
            facts_processed,
            rules_fired,
            evaluation_duration,
            is_streaming,
        );
    }

    /// Record cache metrics for this request
    pub fn record_cache_activity(&self, cache_type: &str, hit: bool) {
        if hit {
            self.metrics.record_cache_hit(cache_type);
        } else {
            self.metrics.record_cache_miss(cache_type);
        }
    }

    /// Record security violation for this request
    pub fn record_security_violation(&self, violation_type: &str) {
        self.metrics.record_security_violation(violation_type, "warning");
    }
}

impl Drop for RequestTracker {
    fn drop(&mut self) {
        // Ensure active requests count is decremented even if finish() wasn't called
        self.metrics.decrement_active_requests();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_metrics_initialization() {
        let _metrics = ApiMetrics::new().expect("Failed to create metrics");
        // Test that metrics can be created without panicking
    }

    #[test]
    fn test_request_tracking() {
        let metrics = Arc::new(ApiMetrics::new().expect("Failed to create metrics"));
        let middleware = MetricsMiddleware::new(metrics.clone());

        let tracker = middleware.start_request();

        // Simulate some work with computation instead of sleep
        let mut result = 0;
        for i in 0..100 {
            result += i * i;
        }

        tracker.finish("POST", "/evaluate", 200);

        // Verify metrics were recorded (basic sanity check)
        let stats = metrics.get_performance_stats();
        assert_eq!(stats.get("active_requests").unwrap_or(&1.0), &0.0);
        
        // Use the result to prevent compiler optimization
        assert!(result > 0);
    }

    #[test]
    fn test_evaluation_metrics() {
        let metrics = Arc::new(ApiMetrics::new().expect("Failed to create metrics"));

        metrics.record_evaluation(
            10,  // rules_processed
            100, // facts_processed
            5,   // rules_fired
            Duration::from_millis(50),
            false, // streaming
        );

        let stats = metrics.get_performance_stats();
        assert!(stats.contains_key("facts_per_second"));
        assert!(*stats.get("facts_per_second").unwrap() > 0.0);
    }

    #[test]
    fn test_cache_metrics() {
        let metrics = Arc::new(ApiMetrics::new().expect("Failed to create metrics"));

        metrics.record_cache_hit("ruleset");
        metrics.record_cache_miss("calculator");
        // Basic test - just ensure no panics
    }

    #[test]
    fn test_security_metrics() {
        let metrics = Arc::new(ApiMetrics::new().expect("Failed to create metrics"));

        metrics.record_security_violation("expression_complexity", "high");
        // Basic test - just ensure no panics
    }
}
