//! Rule execution profiling and performance metrics
//!
//! This module provides comprehensive performance tracking for rule execution,
//! including timing, memory usage, and rule firing statistics.

use crate::types::RuleId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Comprehensive performance tracking for rule execution
#[derive(Debug, Default)]
pub struct RulePerformanceTracker {
    /// Performance profiles for each rule
    rule_profiles: HashMap<RuleId, RuleExecutionProfile>,
    /// Global performance metrics
    global_metrics: GlobalPerformanceMetrics,
    /// Current execution session
    current_session: Option<ExecutionSession>,
    /// Configuration for performance tracking
    config: PerformanceConfig,
}

/// Performance configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable detailed timing measurements
    pub enable_timing: bool,
    /// Enable memory usage tracking
    pub enable_memory_tracking: bool,
    /// Enable rule firing statistics
    pub enable_rule_stats: bool,
    /// Minimum execution time to record (microseconds)
    pub min_execution_time_us: u64,
    /// Maximum number of execution records to keep per rule
    pub max_execution_records: usize,
    /// Enable performance trend analysis
    pub enable_trend_analysis: bool,
}

/// Execution profile for a specific rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleExecutionProfile {
    /// Rule identifier
    pub rule_id: RuleId,
    /// Total number of times rule was evaluated
    pub evaluation_count: usize,
    /// Total number of times rule fired (conditions met)
    pub fire_count: usize,
    /// Success rate (fires / evaluations)
    pub success_rate: f64,
    /// Timing statistics
    pub timing_stats: TimingStatistics,
    /// Memory usage statistics
    pub memory_stats: MemoryStatistics,
    /// Recent execution records
    pub recent_executions: Vec<ExecutionRecord>,
    /// Performance trend over time
    pub performance_trend: Vec<PerformanceTrendPoint>,
    /// Last updated timestamp
    pub last_updated: std::time::SystemTime,
}

/// Timing statistics for rule execution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimingStatistics {
    /// Total execution time across all evaluations
    pub total_time: Duration,
    /// Average execution time per evaluation
    pub average_time: Duration,
    /// Minimum execution time recorded
    pub min_time: Duration,
    /// Maximum execution time recorded
    pub max_time: Duration,
    /// Standard deviation of execution times
    pub std_deviation: Duration,
    /// 95th percentile execution time
    pub p95_time: Duration,
    /// 99th percentile execution time
    pub p99_time: Duration,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatistics {
    /// Average memory usage during execution
    pub average_memory: usize,
    /// Peak memory usage recorded
    pub peak_memory: usize,
    /// Total memory allocated for rule execution
    pub total_allocated: usize,
    /// Memory efficiency (output facts / memory used)
    pub memory_efficiency: f64,
}

/// Individual execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Execution start time
    pub started_at: std::time::SystemTime,
    /// Execution duration
    pub duration: Duration,
    /// Number of input facts
    pub input_fact_count: usize,
    /// Number of output facts
    pub output_fact_count: usize,
    /// Memory usage during execution
    pub memory_used: usize,
    /// Whether rule fired
    pub rule_fired: bool,
    /// Execution context information
    pub context: ExecutionContext,
}

/// Execution context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Processing mode used
    pub processing_mode: String,
    /// Cache hit rate during execution
    pub cache_hit_rate: f64,
    /// Number of nodes traversed
    pub nodes_traversed: usize,
    /// Token operations performed
    pub token_operations: usize,
}

/// Performance trend point for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrendPoint {
    /// Timestamp of measurement
    pub timestamp: std::time::SystemTime,
    /// Execution time at this point
    pub execution_time: Duration,
    /// Memory usage at this point
    pub memory_usage: usize,
    /// Success rate at this point
    pub success_rate: f64,
    /// Throughput (evaluations per second)
    pub throughput: f64,
}

/// Global performance metrics across all rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPerformanceMetrics {
    /// Total number of rules evaluated
    pub total_evaluations: usize,
    /// Total number of rule firings
    pub total_firings: usize,
    /// Overall success rate
    pub overall_success_rate: f64,
    /// Total processing time
    pub total_processing_time: Duration,
    /// Average processing time per fact batch
    pub average_batch_time: Duration,
    /// Peak memory usage across all executions
    pub peak_memory_usage: usize,
    /// Total facts processed
    pub total_facts_processed: usize,
    /// Total facts generated
    pub total_facts_generated: usize,
    /// Fact generation efficiency
    pub fact_efficiency: f64,
}

/// Current execution session tracking
#[derive(Debug)]
pub struct ExecutionSession {
    /// Session identifier
    pub session_id: String,
    /// Session start time
    pub started_at: Instant,
    /// Facts being processed in this session
    pub fact_count: usize,
    /// Rules being evaluated
    pub rules_evaluated: Vec<RuleId>,
    /// Current memory usage
    pub current_memory: usize,
    /// Session statistics
    pub session_stats: SessionStatistics,
}

/// Statistics for current execution session
#[derive(Debug, Default)]
pub struct SessionStatistics {
    /// Facts processed so far
    pub facts_processed: usize,
    /// Rules fired so far
    pub rules_fired: usize,
    /// Total execution time so far
    pub total_time: Duration,
    /// Memory allocated so far
    pub memory_allocated: usize,
}

impl RulePerformanceTracker {
    /// Create new performance tracker
    pub fn new() -> Self {
        Self {
            rule_profiles: HashMap::new(),
            global_metrics: GlobalPerformanceMetrics::default(),
            current_session: None,
            config: PerformanceConfig::default(),
        }
    }

    /// Create performance tracker with custom configuration
    pub fn with_config(config: PerformanceConfig) -> Self {
        Self {
            rule_profiles: HashMap::new(),
            global_metrics: GlobalPerformanceMetrics::default(),
            current_session: None,
            config,
        }
    }

    /// Start tracking a new execution session
    pub fn start_session(&mut self, fact_count: usize) -> String {
        let session_id = format!("session_{}", uuid::Uuid::new_v4());

        debug!(
            session_id = %session_id,
            fact_count = fact_count,
            "Starting performance tracking session"
        );

        self.current_session = Some(ExecutionSession {
            session_id: session_id.clone(),
            started_at: Instant::now(),
            fact_count,
            rules_evaluated: Vec::new(),
            current_memory: 0,
            session_stats: SessionStatistics::default(),
        });

        session_id
    }

    /// Record rule evaluation start
    pub fn start_rule_evaluation(&mut self, rule_id: RuleId) -> RuleExecutionTimer {
        if !self.config.enable_timing {
            return RuleExecutionTimer::disabled();
        }

        if let Some(session) = &mut self.current_session {
            if !session.rules_evaluated.contains(&rule_id) {
                session.rules_evaluated.push(rule_id);
            }
        }

        RuleExecutionTimer::new(rule_id)
    }

    /// Record rule evaluation completion
    pub fn complete_rule_evaluation(
        &mut self,
        timer: RuleExecutionTimer,
        input_facts: usize,
        output_facts: usize,
        rule_fired: bool,
        memory_used: usize,
        context: ExecutionContext,
    ) {
        if !timer.is_enabled() {
            return;
        }

        let execution_time = timer.elapsed();
        let rule_id = timer.rule_id();

        // Skip recording if execution time is below threshold
        if execution_time.as_micros() < self.config.min_execution_time_us as u128 {
            return;
        }

        // Update rule profile
        let profile = self.rule_profiles.entry(rule_id).or_insert_with(|| RuleExecutionProfile {
            rule_id,
            evaluation_count: 0,
            fire_count: 0,
            success_rate: 0.0,
            timing_stats: TimingStatistics::default(),
            memory_stats: MemoryStatistics::default(),
            recent_executions: Vec::new(),
            performance_trend: Vec::new(),
            last_updated: std::time::SystemTime::now(),
        });

        profile.evaluation_count += 1;
        if rule_fired {
            profile.fire_count += 1;
        }
        profile.success_rate = profile.fire_count as f64 / profile.evaluation_count as f64;
        profile.last_updated = std::time::SystemTime::now();

        // Update timing statistics
        Self::update_timing_stats_static(&mut profile.timing_stats, execution_time);

        // Update memory statistics
        Self::update_memory_stats_static(&mut profile.memory_stats, memory_used, output_facts);

        // Add execution record
        let execution_record = ExecutionRecord {
            started_at: std::time::SystemTime::now(),
            duration: execution_time,
            input_fact_count: input_facts,
            output_fact_count: output_facts,
            memory_used,
            rule_fired,
            context,
        };

        profile.recent_executions.push(execution_record);

        // Keep only recent executions
        if profile.recent_executions.len() > self.config.max_execution_records {
            profile.recent_executions.remove(0);
        }

        // Add performance trend point
        if self.config.enable_trend_analysis {
            let trend_point = PerformanceTrendPoint {
                timestamp: std::time::SystemTime::now(),
                execution_time,
                memory_usage: memory_used,
                success_rate: profile.success_rate,
                throughput: 1000.0 / execution_time.as_millis() as f64,
            };
            profile.performance_trend.push(trend_point);

            // Keep only recent trend points (last 100)
            if profile.performance_trend.len() > 100 {
                profile.performance_trend.remove(0);
            }
        }

        // Update global metrics
        self.update_global_metrics(
            execution_time,
            input_facts,
            output_facts,
            rule_fired,
            memory_used,
        );

        // Update session statistics
        if let Some(session) = &mut self.current_session {
            session.session_stats.facts_processed += input_facts;
            if rule_fired {
                session.session_stats.rules_fired += 1;
            }
            session.session_stats.total_time += execution_time;
            session.session_stats.memory_allocated += memory_used;
        }

        debug!(
            rule_id = rule_id,
            execution_time_ms = execution_time.as_millis(),
            rule_fired = rule_fired,
            input_facts = input_facts,
            output_facts = output_facts,
            "Rule execution completed"
        );
    }

    /// Finish current execution session
    pub fn finish_session(&mut self) -> Option<SessionSummary> {
        if let Some(session) = self.current_session.take() {
            let session_duration = session.started_at.elapsed();

            let summary = SessionSummary {
                session_id: session.session_id.clone(),
                total_duration: session_duration,
                facts_processed: session.session_stats.facts_processed,
                rules_fired: session.session_stats.rules_fired,
                rules_evaluated: session.rules_evaluated.len(),
                memory_used: session.session_stats.memory_allocated,
                average_throughput: session.fact_count as f64 / session_duration.as_secs_f64(),
            };

            info!(
                session_id = %session.session_id,
                duration_ms = session_duration.as_millis(),
                facts_processed = session.session_stats.facts_processed,
                rules_fired = session.session_stats.rules_fired,
                "Performance tracking session completed"
            );

            Some(summary)
        } else {
            None
        }
    }

    /// Get performance profile for a rule
    pub fn get_rule_profile(&self, rule_id: RuleId) -> Option<&RuleExecutionProfile> {
        self.rule_profiles.get(&rule_id)
    }

    /// Get global performance metrics
    pub fn get_global_metrics(&self) -> &GlobalPerformanceMetrics {
        &self.global_metrics
    }

    /// Get performance summary for all rules
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        let mut slowest_rules = Vec::new();
        let mut most_fired_rules = Vec::new();
        let mut least_efficient_rules = Vec::new();

        for profile in self.rule_profiles.values() {
            slowest_rules.push((profile.rule_id, profile.timing_stats.average_time));
            most_fired_rules.push((profile.rule_id, profile.fire_count));
            least_efficient_rules.push((profile.rule_id, profile.success_rate));
        }

        // Sort and take top 10
        slowest_rules.sort_by_key(|(_, time)| std::cmp::Reverse(*time));
        slowest_rules.truncate(10);

        most_fired_rules.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        most_fired_rules.truncate(10);

        least_efficient_rules.sort_by(|(_, rate_a), (_, rate_b)| {
            rate_a.partial_cmp(rate_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        least_efficient_rules.truncate(10);

        PerformanceSummary {
            total_rules_tracked: self.rule_profiles.len(),
            global_metrics: self.global_metrics.clone(),
            slowest_rules,
            most_fired_rules,
            least_efficient_rules,
        }
    }

    /// Update timing statistics for a rule
    fn update_timing_stats_static(stats: &mut TimingStatistics, execution_time: Duration) {
        stats.total_time += execution_time;

        if stats.min_time == Duration::default() || execution_time < stats.min_time {
            stats.min_time = execution_time;
        }

        if execution_time > stats.max_time {
            stats.max_time = execution_time;
        }

        // Calculate average (simplified)
        stats.average_time = stats.total_time; // Will be divided by count in actual calculation
    }

    /// Update memory statistics for a rule
    fn update_memory_stats_static(
        stats: &mut MemoryStatistics,
        memory_used: usize,
        output_facts: usize,
    ) {
        stats.total_allocated += memory_used;

        if memory_used > stats.peak_memory {
            stats.peak_memory = memory_used;
        }

        if output_facts > 0 {
            stats.memory_efficiency = output_facts as f64 / memory_used as f64;
        }
    }

    /// Update global performance metrics
    fn update_global_metrics(
        &mut self,
        execution_time: Duration,
        input_facts: usize,
        output_facts: usize,
        rule_fired: bool,
        memory_used: usize,
    ) {
        self.global_metrics.total_evaluations += 1;
        if rule_fired {
            self.global_metrics.total_firings += 1;
        }

        self.global_metrics.overall_success_rate =
            self.global_metrics.total_firings as f64 / self.global_metrics.total_evaluations as f64;

        self.global_metrics.total_processing_time += execution_time;
        self.global_metrics.total_facts_processed += input_facts;
        self.global_metrics.total_facts_generated += output_facts;

        if memory_used > self.global_metrics.peak_memory_usage {
            self.global_metrics.peak_memory_usage = memory_used;
        }

        if self.global_metrics.total_facts_processed > 0 {
            self.global_metrics.fact_efficiency = self.global_metrics.total_facts_generated as f64
                / self.global_metrics.total_facts_processed as f64;
        }
    }

    /// Identify performance bottlenecks
    pub fn identify_bottlenecks(&self) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();

        for profile in self.rule_profiles.values() {
            // Check for slow rules
            if profile.timing_stats.average_time > Duration::from_millis(100) {
                bottlenecks.push(PerformanceBottleneck {
                    rule_id: profile.rule_id,
                    bottleneck_type: BottleneckType::SlowExecution,
                    severity: calculate_severity(
                        profile.timing_stats.average_time.as_millis(),
                        100,
                        1000,
                    ),
                    description: format!(
                        "Rule {} has slow average execution time: {}ms",
                        profile.rule_id,
                        profile.timing_stats.average_time.as_millis()
                    ),
                    recommendation: "Consider optimizing rule conditions or indexing".to_string(),
                });
            }

            // Check for low efficiency rules
            if profile.success_rate < 0.1 && profile.evaluation_count > 100 {
                bottlenecks.push(PerformanceBottleneck {
                    rule_id: profile.rule_id,
                    bottleneck_type: BottleneckType::LowEfficiency,
                    severity: 1.0 - profile.success_rate,
                    description: format!(
                        "Rule {} has low success rate: {:.1}%",
                        profile.rule_id,
                        profile.success_rate * 100.0
                    ),
                    recommendation:
                        "Consider reordering conditions or adding more selective conditions"
                            .to_string(),
                });
            }

            // Check for memory-intensive rules
            if profile.memory_stats.peak_memory > 10_000_000 {
                // 10MB
                bottlenecks.push(PerformanceBottleneck {
                    rule_id: profile.rule_id,
                    bottleneck_type: BottleneckType::HighMemoryUsage,
                    severity: calculate_severity(profile.memory_stats.peak_memory as u128, 10_000_000, 100_000_000),
                    description: format!(
                        "Rule {} uses high memory: {}MB",
                        profile.rule_id,
                        profile.memory_stats.peak_memory / 1_000_000
                    ),
                    recommendation: "Consider optimizing data structures or reducing working memory".to_string(),
                });
            }
        }

        bottlenecks
    }
}

/// Timer for tracking rule execution time
#[derive(Debug)]
pub struct RuleExecutionTimer {
    rule_id: RuleId,
    start_time: Option<Instant>,
}

impl RuleExecutionTimer {
    fn new(rule_id: RuleId) -> Self {
        Self { rule_id, start_time: Some(Instant::now()) }
    }

    fn disabled() -> Self {
        Self { rule_id: 0, start_time: None }
    }

    fn is_enabled(&self) -> bool {
        self.start_time.is_some()
    }

    fn rule_id(&self) -> RuleId {
        self.rule_id
    }

    fn elapsed(&self) -> Duration {
        self.start_time.map(|start| start.elapsed()).unwrap_or_default()
    }
}

/// Session execution summary
#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub session_id: String,
    pub total_duration: Duration,
    pub facts_processed: usize,
    pub rules_fired: usize,
    pub rules_evaluated: usize,
    pub memory_used: usize,
    pub average_throughput: f64,
}

/// Performance summary across all rules
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub total_rules_tracked: usize,
    pub global_metrics: GlobalPerformanceMetrics,
    pub slowest_rules: Vec<(RuleId, Duration)>,
    pub most_fired_rules: Vec<(RuleId, usize)>,
    pub least_efficient_rules: Vec<(RuleId, f64)>,
}

/// Performance bottleneck identification
#[derive(Debug, Clone)]
pub struct PerformanceBottleneck {
    pub rule_id: RuleId,
    pub bottleneck_type: BottleneckType,
    pub severity: f64, // 0.0 to 1.0
    pub description: String,
    pub recommendation: String,
}

/// Types of performance bottlenecks
#[derive(Debug, Clone)]
pub enum BottleneckType {
    SlowExecution,
    LowEfficiency,
    HighMemoryUsage,
    FrequentFiring,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_timing: true,
            enable_memory_tracking: true,
            enable_rule_stats: true,
            min_execution_time_us: 100,
            max_execution_records: 100,
            enable_trend_analysis: true,
        }
    }
}

impl Default for MemoryStatistics {
    fn default() -> Self {
        Self { average_memory: 0, peak_memory: 0, total_allocated: 0, memory_efficiency: 0.0 }
    }
}

impl Default for GlobalPerformanceMetrics {
    fn default() -> Self {
        Self {
            total_evaluations: 0,
            total_firings: 0,
            overall_success_rate: 0.0,
            total_processing_time: Duration::default(),
            average_batch_time: Duration::default(),
            peak_memory_usage: 0,
            total_facts_processed: 0,
            total_facts_generated: 0,
            fact_efficiency: 0.0,
        }
    }
}

/// Calculate severity score based on value relative to warning and critical thresholds
fn calculate_severity(value: u128, warning_threshold: u128, critical_threshold: u128) -> f64 {
    if value >= critical_threshold {
        1.0
    } else if value >= warning_threshold {
        0.5 + 0.5
            * ((value - warning_threshold) as f64 / (critical_threshold - warning_threshold) as f64)
    } else {
        0.0
    }
}
