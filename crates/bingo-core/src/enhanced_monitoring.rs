//! Enhanced Monitoring System for Bingo RETE Engine
//!
//! This module provides comprehensive, granular performance monitoring capabilities
//! for production environments, including real-time metrics, alerting, and
//! operational visibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use tracing::{debug, error, info, instrument, warn};

/// Enhanced monitoring system for comprehensive observability
#[derive(Debug, Clone)]
pub struct EnhancedMonitoring {
    /// Real-time performance metrics
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,

    /// Resource utilization tracking
    resource_metrics: Arc<RwLock<ResourceMetrics>>,

    /// Business metrics for operational insights
    business_metrics: Arc<RwLock<BusinessMetrics>>,

    /// Alert configuration and state
    alert_manager: Arc<RwLock<AlertManager>>,

    /// Historical data for trend analysis
    historical_data: Arc<RwLock<HistoricalData>>,

    /// Configuration for monitoring behavior
    config: MonitoringConfig,
}

/// Comprehensive performance metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Engine-level performance
    pub engine_performance: EnginePerformanceMetrics,

    /// RETE network specific metrics
    pub rete_performance: RetePerformanceMetrics,

    /// Memory pool performance
    pub memory_pool_performance: MemoryPoolMetrics,

    /// Parallel processing performance
    pub parallel_performance: ParallelProcessingMetrics,

    /// Cache performance metrics
    pub cache_performance: CachePerformanceMetrics,
}

/// Engine-level performance metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EnginePerformanceMetrics {
    /// Total facts processed per second
    pub facts_per_second: f64,

    /// Average rule execution time (microseconds)
    pub avg_rule_execution_time_us: f64,

    /// Rule compilation throughput (rules per second)
    pub rules_compiled_per_second: f64,

    /// Success rate percentage
    pub success_rate_percent: f64,

    /// Average memory per operation (bytes)
    pub avg_memory_per_operation: u64,

    /// Current CPU usage percentage
    pub cpu_usage_percent: f64,

    /// Garbage collection frequency (per minute)
    pub gc_frequency_per_minute: f64,
}

/// RETE network specific performance metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RetePerformanceMetrics {
    /// Alpha node hit rate percentage
    pub alpha_node_hit_rate: f64,

    /// Beta node efficiency percentage
    pub beta_node_efficiency: f64,

    /// Working memory utilization percentage
    pub working_memory_utilization: f64,

    /// Rule firing rate (rules per second)
    pub rule_firing_rate: f64,

    /// Conflict resolution time (microseconds)
    pub conflict_resolution_time_us: f64,

    /// Pattern matching efficiency
    pub pattern_matching_efficiency: f64,
}

/// Memory pool performance metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MemoryPoolMetrics {
    /// Overall pool hit rate percentage
    pub overall_hit_rate: f64,

    /// Memory saved through pooling (bytes)
    pub memory_saved_bytes: u64,

    /// Pool utilization percentage
    pub pool_utilization: f64,

    /// Average allocation time (nanoseconds)
    pub avg_allocation_time_ns: f64,

    /// Pool contention rate (contentions per second)
    pub contention_rate: f64,
}

/// Parallel processing performance metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ParallelProcessingMetrics {
    /// Parallel efficiency percentage
    pub parallel_efficiency: f64,

    /// Worker thread utilization percentage
    pub worker_utilization: f64,

    /// Load balancing effectiveness
    pub load_balancing_score: f64,

    /// Thread contention rate
    pub thread_contention_rate: f64,

    /// Parallel speedup factor
    pub speedup_factor: f64,
}

/// Cache performance metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CachePerformanceMetrics {
    /// Rule cache hit rate percentage
    pub rule_cache_hit_rate: f64,

    /// Calculator cache hit rate percentage
    pub calculator_cache_hit_rate: f64,

    /// Fact lookup cache hit rate percentage
    pub fact_cache_hit_rate: f64,

    /// Average cache lookup time (nanoseconds)
    pub avg_cache_lookup_time_ns: f64,

    /// Cache eviction rate (evictions per minute)
    pub cache_eviction_rate: f64,
}

/// Resource utilization metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// Current memory usage (bytes)
    pub memory_usage_bytes: u64,

    /// Peak memory usage (bytes)
    pub peak_memory_bytes: u64,

    /// Memory growth rate (bytes per second)
    pub memory_growth_rate: f64,

    /// File descriptors in use
    pub file_descriptors_used: u32,

    /// Network connections active
    pub active_connections: u32,

    /// Disk I/O operations per second
    pub disk_io_ops_per_second: f64,

    /// Thread count
    pub thread_count: u32,
}

/// Business-oriented metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    /// Rules processed in last hour
    pub rules_processed_last_hour: u64,

    /// Compliance checks performed
    pub compliance_checks_performed: u64,

    /// Payroll calculations completed
    pub payroll_calculations_completed: u64,

    /// TRONC distributions processed
    pub tronc_distributions_processed: u64,

    /// Error rate percentage
    pub error_rate_percent: f64,

    /// Business rule violations detected
    pub rule_violations_detected: u64,

    /// Average processing latency (milliseconds)
    pub avg_processing_latency_ms: f64,
}

/// Alert management system
#[derive(Debug)]
pub struct AlertManager {
    /// Active alerts
    active_alerts: HashMap<AlertType, Alert>,

    /// Alert thresholds
    thresholds: AlertThresholds,
}

/// Types of monitoring alerts
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum AlertType {
    HighCpuUsage,
    HighMemoryUsage,
    LowCacheHitRate,
    HighErrorRate,
    SlowResponseTime,
    HighRuleFailureRate,
    MemoryLeak,
    ThreadDeadlock,
    ResourceExhaustion,
    PerformanceDegradation,
}

/// Individual alert details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: SystemTime,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub resolved: bool,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Alert event for historical tracking
#[derive(Debug, Clone)]
pub struct AlertEvent {
    pub alert: Alert,
    pub event_type: AlertEventType,
    pub timestamp: SystemTime,
}

/// Types of alert events
#[derive(Debug, Clone)]
pub enum AlertEventType {
    Triggered,
    Resolved,
    Escalated,
}

/// Configurable alert thresholds
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub cpu_usage_warning: f64,
    pub cpu_usage_critical: f64,
    pub memory_usage_warning: f64,
    pub memory_usage_critical: f64,
    pub cache_hit_rate_warning: f64,
    pub cache_hit_rate_critical: f64,
    pub error_rate_warning: f64,
    pub error_rate_critical: f64,
    pub response_time_warning_ms: f64,
    pub response_time_critical_ms: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_usage_warning: 70.0,
            cpu_usage_critical: 90.0,
            memory_usage_warning: 80.0,
            memory_usage_critical: 95.0,
            cache_hit_rate_warning: 85.0,
            cache_hit_rate_critical: 70.0,
            error_rate_warning: 5.0,
            error_rate_critical: 10.0,
            response_time_warning_ms: 100.0,
            response_time_critical_ms: 500.0,
        }
    }
}

/// Historical data for trend analysis
#[derive(Debug, Default)]
pub struct HistoricalData {
    /// Performance samples over time
    performance_samples: Vec<PerformanceSample>,

    /// Resource usage samples
    resource_samples: Vec<ResourceSample>,

    /// Business metrics samples
    business_samples: Vec<BusinessSample>,

    /// Maximum samples to retain
    max_samples: usize,
}

/// Performance data point with timestamp
#[derive(Debug, Clone)]
pub struct PerformanceSample {
    pub timestamp: SystemTime,
    pub metrics: PerformanceMetrics,
}

/// Resource usage data point
#[derive(Debug, Clone)]
pub struct ResourceSample {
    pub timestamp: SystemTime,
    pub metrics: ResourceMetrics,
}

/// Business metrics data point
#[derive(Debug, Clone)]
pub struct BusinessSample {
    pub timestamp: SystemTime,
    pub metrics: BusinessMetrics,
}

/// Configuration for enhanced monitoring
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Enable/disable enhanced monitoring
    pub enabled: bool,

    /// Sampling interval for metrics collection (seconds)
    pub sampling_interval_seconds: u64,

    /// Maximum historical samples to retain
    pub max_historical_samples: usize,

    /// Performance profiler integration
    pub enable_profiler_integration: bool,

    /// Export metrics in Prometheus format
    pub enable_prometheus_export: bool,

    /// Enable detailed tracing
    pub enable_detailed_tracing: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_interval_seconds: 60,
            max_historical_samples: 1440, // 24 hours at 1-minute intervals
            enable_profiler_integration: true,
            enable_prometheus_export: true,
            enable_detailed_tracing: false,
        }
    }
}

impl EnhancedMonitoring {
    /// Create a new enhanced monitoring instance
    pub fn new(config: MonitoringConfig) -> Self {
        info!("Initializing enhanced monitoring system");

        Self {
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            resource_metrics: Arc::new(RwLock::new(ResourceMetrics::default())),
            business_metrics: Arc::new(RwLock::new(BusinessMetrics::default())),
            alert_manager: Arc::new(RwLock::new(AlertManager::new())),
            historical_data: Arc::new(RwLock::new(HistoricalData {
                max_samples: config.max_historical_samples,
                ..Default::default()
            })),
            config,
        }
    }
}

impl Default for EnhancedMonitoring {
    fn default() -> Self {
        Self::new(MonitoringConfig::default())
    }
}

impl EnhancedMonitoring {
    /// Record engine performance metrics
    #[instrument(skip(self))]
    pub fn record_engine_performance(
        &self,
        metrics: EnginePerformanceMetrics,
    ) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut perf_metrics = self
            .performance_metrics
            .write()
            .map_err(|e| format!("Failed to lock performance metrics: {e}"))?;

        perf_metrics.engine_performance = metrics.clone();

        info!(
            "Recorded engine performance metrics: {:.2} facts/sec, {:.2}% success rate",
            metrics.facts_per_second, metrics.success_rate_percent
        );

        // Check for performance alerts
        self.check_performance_alerts(&metrics)?;

        Ok(())
    }

    /// Record RETE network performance
    #[instrument(skip(self))]
    pub fn record_rete_performance(&self, metrics: RetePerformanceMetrics) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut perf_metrics = self
            .performance_metrics
            .write()
            .map_err(|e| format!("Failed to lock performance metrics: {e}"))?;

        perf_metrics.rete_performance = metrics.clone();

        debug!(
            "Recorded RETE performance: {:.2}% alpha hit rate, {:.2}% beta efficiency",
            metrics.alpha_node_hit_rate, metrics.beta_node_efficiency
        );

        Ok(())
    }

    /// Record memory pool performance
    #[instrument(skip(self))]
    pub fn record_memory_pool_performance(&self, metrics: MemoryPoolMetrics) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut perf_metrics = self
            .performance_metrics
            .write()
            .map_err(|e| format!("Failed to lock performance metrics: {e}"))?;

        perf_metrics.memory_pool_performance = metrics.clone();

        debug!(
            "Recorded memory pool performance: {:.2}% hit rate, {} bytes saved",
            metrics.overall_hit_rate, metrics.memory_saved_bytes
        );

        Ok(())
    }

    /// Record parallel processing performance
    #[instrument(skip(self))]
    pub fn record_parallel_performance(
        &self,
        metrics: ParallelProcessingMetrics,
    ) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut perf_metrics = self
            .performance_metrics
            .write()
            .map_err(|e| format!("Failed to lock performance metrics: {e}"))?;

        perf_metrics.parallel_performance = metrics.clone();

        info!(
            "Recorded parallel performance: {:.2}% efficiency, {:.2}x speedup",
            metrics.parallel_efficiency, metrics.speedup_factor
        );

        Ok(())
    }

    /// Record resource utilization
    #[instrument(skip(self))]
    pub fn record_resource_metrics(&self, metrics: ResourceMetrics) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut resource_metrics = self
            .resource_metrics
            .write()
            .map_err(|e| format!("Failed to lock resource metrics: {e}"))?;

        *resource_metrics = metrics.clone();

        debug!(
            "Recorded resource metrics: {} MB memory, {} threads",
            metrics.memory_usage_bytes / 1024 / 1024,
            metrics.thread_count
        );

        // Check for resource alerts
        self.check_resource_alerts(&metrics)?;

        Ok(())
    }

    /// Record business metrics
    #[instrument(skip(self))]
    pub fn record_business_metrics(&self, metrics: BusinessMetrics) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut business_metrics = self
            .business_metrics
            .write()
            .map_err(|e| format!("Failed to lock business metrics: {e}"))?;

        *business_metrics = metrics.clone();

        info!(
            "Recorded business metrics: {} rules processed, {:.2}% error rate",
            metrics.rules_processed_last_hour, metrics.error_rate_percent
        );

        Ok(())
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> Result<PerformanceMetrics, String> {
        self.performance_metrics
            .read()
            .map(|metrics| metrics.clone())
            .map_err(|e| format!("Failed to read performance metrics: {e}"))
    }

    /// Get current resource metrics
    pub fn get_resource_metrics(&self) -> Result<ResourceMetrics, String> {
        self.resource_metrics
            .read()
            .map(|metrics| metrics.clone())
            .map_err(|e| format!("Failed to read resource metrics: {e}"))
    }

    /// Get current business metrics
    pub fn get_business_metrics(&self) -> Result<BusinessMetrics, String> {
        self.business_metrics
            .read()
            .map(|metrics| metrics.clone())
            .map_err(|e| format!("Failed to read business metrics: {e}"))
    }

    /// Generate comprehensive monitoring report
    #[instrument(skip(self))]
    pub fn generate_monitoring_report(&self) -> Result<MonitoringReport, String> {
        let performance_metrics = self.get_performance_metrics()?;
        let resource_metrics = self.get_resource_metrics()?;
        let business_metrics = self.get_business_metrics()?;

        let alert_manager = self
            .alert_manager
            .read()
            .map_err(|e| format!("Failed to read alert manager: {e}"))?;

        let active_alerts = alert_manager.active_alerts.values().cloned().collect();

        let report = MonitoringReport {
            timestamp: SystemTime::now(),
            performance_metrics,
            resource_metrics,
            business_metrics,
            active_alerts,
            system_health_score: self.calculate_system_health_score()?,
        };

        info!(
            "Generated comprehensive monitoring report with health score: {:.2}",
            report.system_health_score
        );

        Ok(report)
    }

    /// Calculate overall system health score (0-100)
    fn calculate_system_health_score(&self) -> Result<f64, String> {
        let performance_metrics = self.get_performance_metrics()?;
        let resource_metrics = self.get_resource_metrics()?;
        let business_metrics = self.get_business_metrics()?;

        // Weight different components of system health
        let performance_weight = 0.4;
        let resource_weight = 0.3;
        let business_weight = 0.3;

        // Performance score (0-100)
        let performance_score = (performance_metrics.engine_performance.success_rate_percent
            + performance_metrics.rete_performance.alpha_node_hit_rate
            + performance_metrics.memory_pool_performance.overall_hit_rate
            + performance_metrics.parallel_performance.parallel_efficiency)
            / 4.0;

        // Resource score (0-100, inverted for usage metrics)
        let memory_usage_percent = (resource_metrics.memory_usage_bytes as f64
            / resource_metrics.peak_memory_bytes as f64)
            * 100.0;
        let resource_score = 100.0 - (memory_usage_percent / 2.0).min(50.0); // Cap at 50% penalty

        // Business score (0-100, inverted for error rate)
        let business_score = 100.0 - business_metrics.error_rate_percent.min(20.0) * 5.0; // Cap error impact

        let health_score = (performance_score * performance_weight
            + resource_score * resource_weight
            + business_score * business_weight)
            .clamp(0.0, 100.0);

        Ok(health_score)
    }

    /// Check for performance-related alerts
    fn check_performance_alerts(&self, metrics: &EnginePerformanceMetrics) -> Result<(), String> {
        let alert_manager = self
            .alert_manager
            .read()
            .map_err(|e| format!("Failed to read alert manager: {e}"))?;

        let thresholds = &alert_manager.thresholds;

        // Check CPU usage
        if metrics.cpu_usage_percent > thresholds.cpu_usage_critical {
            self.trigger_alert(
                AlertType::HighCpuUsage,
                AlertSeverity::Critical,
                format!("CPU usage at {:.2}%", metrics.cpu_usage_percent),
                metrics.cpu_usage_percent,
                thresholds.cpu_usage_critical,
            )?;
        } else if metrics.cpu_usage_percent > thresholds.cpu_usage_warning {
            self.trigger_alert(
                AlertType::HighCpuUsage,
                AlertSeverity::Warning,
                format!("CPU usage at {:.2}%", metrics.cpu_usage_percent),
                metrics.cpu_usage_percent,
                thresholds.cpu_usage_warning,
            )?;
        }

        // Check error rate
        let error_rate = 100.0 - metrics.success_rate_percent;
        if error_rate > thresholds.error_rate_critical {
            self.trigger_alert(
                AlertType::HighErrorRate,
                AlertSeverity::Critical,
                format!("Error rate at {error_rate:.2}%"),
                error_rate,
                thresholds.error_rate_critical,
            )?;
        }

        Ok(())
    }

    /// Check for resource-related alerts
    fn check_resource_alerts(&self, metrics: &ResourceMetrics) -> Result<(), String> {
        let alert_manager = self
            .alert_manager
            .read()
            .map_err(|e| format!("Failed to read alert manager: {e}"))?;

        let thresholds = &alert_manager.thresholds;

        // Check memory usage
        let memory_usage_percent =
            (metrics.memory_usage_bytes as f64 / metrics.peak_memory_bytes as f64) * 100.0;
        if memory_usage_percent > thresholds.memory_usage_critical {
            self.trigger_alert(
                AlertType::HighMemoryUsage,
                AlertSeverity::Critical,
                format!("Memory usage at {memory_usage_percent:.2}%"),
                memory_usage_percent,
                thresholds.memory_usage_critical,
            )?;
        }

        Ok(())
    }

    /// Trigger an alert
    fn trigger_alert(
        &self,
        alert_type: AlertType,
        severity: AlertSeverity,
        message: String,
        metric_value: f64,
        threshold_value: f64,
    ) -> Result<(), String> {
        let alert = Alert {
            alert_type: alert_type.clone(),
            severity,
            message: message.clone(),
            timestamp: SystemTime::now(),
            metric_value,
            threshold_value,
            resolved: false,
        };

        match alert.severity {
            AlertSeverity::Critical | AlertSeverity::Emergency => {
                error!("ALERT: {:?} - {}", alert_type, message);
            }
            AlertSeverity::Warning => {
                warn!("ALERT: {:?} - {}", alert_type, message);
            }
            AlertSeverity::Info => {
                info!("ALERT: {:?} - {}", alert_type, message);
            }
        }

        let mut alert_manager = self
            .alert_manager
            .write()
            .map_err(|e| format!("Failed to write alert manager: {e}"))?;

        alert_manager.active_alerts.insert(alert_type, alert);

        Ok(())
    }

    /// Get configuration for testing
    pub fn get_config(&self) -> &MonitoringConfig {
        &self.config
    }

    /// Get historical data for testing (read-only access)
    pub fn get_historical_data_reader(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<HistoricalData>, String> {
        self.historical_data
            .read()
            .map_err(|e| format!("Failed to read historical data: {e}"))
    }

    /// Add sample to historical data
    pub fn add_historical_sample(&self) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        let performance_metrics = self.get_performance_metrics()?;
        let resource_metrics = self.get_resource_metrics()?;
        let business_metrics = self.get_business_metrics()?;

        let mut historical_data = self
            .historical_data
            .write()
            .map_err(|e| format!("Failed to lock historical data: {e}"))?;

        let timestamp = SystemTime::now();

        // Add performance sample
        historical_data
            .performance_samples
            .push(PerformanceSample { timestamp, metrics: performance_metrics });

        // Add resource sample
        historical_data
            .resource_samples
            .push(ResourceSample { timestamp, metrics: resource_metrics });

        // Add business sample
        historical_data
            .business_samples
            .push(BusinessSample { timestamp, metrics: business_metrics });

        // Maintain sample limits
        if historical_data.performance_samples.len() > historical_data.max_samples {
            historical_data.performance_samples.remove(0);
        }
        if historical_data.resource_samples.len() > historical_data.max_samples {
            historical_data.resource_samples.remove(0);
        }
        if historical_data.business_samples.len() > historical_data.max_samples {
            historical_data.business_samples.remove(0);
        }

        debug!(
            "Added historical sample, {} samples stored",
            historical_data.performance_samples.len()
        );

        Ok(())
    }
}

impl HistoricalData {
    /// Get performance samples count
    pub fn performance_samples_len(&self) -> usize {
        self.performance_samples.len()
    }

    /// Get latest performance sample
    pub fn get_latest_performance_sample(&self) -> Option<&PerformanceSample> {
        self.performance_samples.last()
    }
}

impl AlertManager {
    fn new() -> Self {
        Self { active_alerts: HashMap::new(), thresholds: AlertThresholds::default() }
    }
}

/// Comprehensive monitoring report
#[derive(Debug, Serialize, Deserialize)]
pub struct MonitoringReport {
    pub timestamp: SystemTime,
    pub performance_metrics: PerformanceMetrics,
    pub resource_metrics: ResourceMetrics,
    pub business_metrics: BusinessMetrics,
    pub active_alerts: Vec<Alert>,
    pub system_health_score: f64,
}

impl MonitoringReport {
    /// Export report as JSON
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize monitoring report: {e}"))
    }

    /// Get summary statistics
    pub fn get_summary(&self) -> MonitoringSummary {
        MonitoringSummary {
            health_score: self.system_health_score,
            active_alert_count: self.active_alerts.len(),
            critical_alert_count: self
                .active_alerts
                .iter()
                .filter(|a| {
                    matches!(
                        a.severity,
                        AlertSeverity::Critical | AlertSeverity::Emergency
                    )
                })
                .count(),
            facts_per_second: self.performance_metrics.engine_performance.facts_per_second,
            memory_usage_mb: self.resource_metrics.memory_usage_bytes / 1024 / 1024,
            error_rate_percent: self.business_metrics.error_rate_percent,
        }
    }
}

/// Monitoring summary for quick overview
#[derive(Debug, Serialize, Deserialize)]
pub struct MonitoringSummary {
    pub health_score: f64,
    pub active_alert_count: usize,
    pub critical_alert_count: usize,
    pub facts_per_second: f64,
    pub memory_usage_mb: u64,
    pub error_rate_percent: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_monitoring_creation() {
        let monitoring = EnhancedMonitoring::default();
        assert!(monitoring.config.enabled);
        assert_eq!(monitoring.config.sampling_interval_seconds, 60);
    }

    #[test]
    fn test_performance_metrics_recording() {
        let monitoring = EnhancedMonitoring::default();

        let metrics = EnginePerformanceMetrics {
            facts_per_second: 1000.0,
            success_rate_percent: 99.5,
            avg_rule_execution_time_us: 50.0,
            ..Default::default()
        };

        monitoring.record_engine_performance(metrics).unwrap();

        let recorded = monitoring.get_performance_metrics().unwrap();
        assert_eq!(recorded.engine_performance.facts_per_second, 1000.0);
        assert_eq!(recorded.engine_performance.success_rate_percent, 99.5);
    }

    #[test]
    fn test_health_score_calculation() {
        let monitoring = EnhancedMonitoring::default();

        // Record good metrics
        let perf_metrics =
            EnginePerformanceMetrics { success_rate_percent: 99.0, ..Default::default() };
        monitoring.record_engine_performance(perf_metrics).unwrap();

        // Record RETE performance metrics to ensure health score calculation has all components
        let rete_metrics = RetePerformanceMetrics {
            alpha_node_hit_rate: 95.0,
            beta_node_efficiency: 90.0,
            ..Default::default()
        };
        monitoring.record_rete_performance(rete_metrics).unwrap();

        // Record memory pool performance metrics
        let memory_pool_metrics =
            MemoryPoolMetrics { overall_hit_rate: 92.0, ..Default::default() };
        monitoring.record_memory_pool_performance(memory_pool_metrics).unwrap();

        // Record parallel performance metrics
        let parallel_metrics =
            ParallelProcessingMetrics { parallel_efficiency: 85.0, ..Default::default() };
        monitoring.record_parallel_performance(parallel_metrics).unwrap();

        let resource_metrics = ResourceMetrics {
            memory_usage_bytes: 1024 * 1024 * 100, // 100MB
            peak_memory_bytes: 1024 * 1024 * 1000, // 1GB
            ..Default::default()
        };
        monitoring.record_resource_metrics(resource_metrics).unwrap();

        let business_metrics = BusinessMetrics { error_rate_percent: 0.5, ..Default::default() };
        monitoring.record_business_metrics(business_metrics).unwrap();

        let health_score = monitoring.calculate_system_health_score().unwrap();
        assert!(health_score > 80.0); // Should be high with good metrics
    }

    #[test]
    fn test_monitoring_report_generation() {
        let monitoring = EnhancedMonitoring::default();

        // Add some sample data
        let perf_metrics = EnginePerformanceMetrics {
            facts_per_second: 500.0,
            success_rate_percent: 98.5,
            ..Default::default()
        };
        monitoring.record_engine_performance(perf_metrics).unwrap();

        let report = monitoring.generate_monitoring_report().unwrap();

        assert!(report.system_health_score >= 0.0);
        assert!(report.system_health_score <= 100.0);
        assert_eq!(
            report.performance_metrics.engine_performance.facts_per_second,
            500.0
        );
    }
}
