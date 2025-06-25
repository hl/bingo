//! Advanced Rule Debugging and Performance Profiling
//!
//! This module provides comprehensive debugging capabilities for the RETE network,
//! including execution traces, performance profiling, and rule analysis tools.

use crate::types::{Fact, RuleId};
use crate::rete_nodes::{Token, NodeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use uuid::Uuid;

/// Unique identifier for debugging sessions
pub type DebugSessionId = Uuid;

/// Unique identifier for execution traces
pub type TraceId = Uuid;

/// Main debugging and profiling manager
#[derive(Debug)]
pub struct DebugManager {
    /// Active debugging sessions
    sessions: HashMap<DebugSessionId, DebugSession>,
    /// Global execution traces
    traces: HashMap<TraceId, ExecutionTrace>,
    /// Performance profiles
    profiles: HashMap<RuleId, RulePerformanceProfile>,
    /// Configuration for debugging
    config: DebugConfig,
    /// Statistics and metrics
    stats: DebugStats,
}

/// Configuration for debugging behavior
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// Enable execution tracing
    pub enable_tracing: bool,
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// Enable memory tracking
    pub enable_memory_tracking: bool,
    /// Maximum number of traces to keep in memory
    pub max_traces: usize,
    /// Trace retention period in milliseconds
    pub trace_retention_ms: u64,
    /// Minimum execution time to record (microseconds)
    pub min_execution_time_us: u64,
    /// Enable detailed token tracking
    pub enable_token_tracking: bool,
    /// Enable rule dependency analysis
    pub enable_dependency_analysis: bool,
}

/// A debugging session for tracking rule execution
#[derive(Debug)]
pub struct DebugSession {
    /// Session identifier
    pub session_id: DebugSessionId,
    /// Rules being debugged in this session
    pub target_rules: Vec<RuleId>,
    /// Session start time
    pub started_at: SystemTime,
    /// Session configuration
    pub config: DebugConfig,
    /// Execution traces for this session
    pub traces: Vec<TraceId>,
    /// Session statistics
    pub stats: SessionStats,
    /// Active breakpoints
    pub breakpoints: HashMap<NodeId, Breakpoint>,
    /// Step-by-step execution state
    pub step_state: Option<StepExecutionState>,
}

/// Statistics for a debugging session
#[derive(Debug, Default)]
pub struct SessionStats {
    /// Total facts processed
    pub facts_processed: usize,
    /// Total tokens generated
    pub tokens_generated: usize,
    /// Total rules fired
    pub rules_fired: usize,
    /// Total execution time
    pub total_execution_time: Duration,
    /// Number of traces recorded
    pub traces_recorded: usize,
    /// Memory usage during session
    pub peak_memory_usage: usize,
}

/// Breakpoint for debugging
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Node where breakpoint is set
    pub node_id: NodeId,
    /// Condition for triggering breakpoint
    pub condition: BreakpointCondition,
    /// Whether breakpoint is active
    pub enabled: bool,
    /// Number of times this breakpoint has been hit
    pub hit_count: usize,
}

/// Conditions for triggering breakpoints
#[derive(Debug, Clone)]
pub enum BreakpointCondition {
    /// Break on any token arrival
    Always,
    /// Break when specific fact is processed
    FactId(u64),
    /// Break when rule fires
    RuleFired(RuleId),
    /// Break after N hits
    HitCount(usize),
    /// Break on custom condition
    Custom(String),
}

/// State for step-by-step execution
#[derive(Debug)]
pub struct StepExecutionState {
    /// Current execution step
    pub current_step: usize,
    /// Paused at node
    pub paused_at: Option<NodeId>,
    /// Execution queue
    pub execution_queue: VecDeque<ExecutionStep>,
    /// Variables available at current step
    pub variables: HashMap<String, String>,
}

/// Individual execution step
#[derive(Debug, Clone)]
pub struct ExecutionStep {
    /// Step identifier
    pub step_id: usize,
    /// Node being executed
    pub node_id: NodeId,
    /// Token being processed
    pub token: Token,
    /// Timestamp
    pub timestamp: SystemTime,
    /// Step type
    pub step_type: StepType,
}

/// Types of execution steps
#[derive(Debug, Clone)]
pub enum StepType {
    /// Token arrival at node
    TokenArrival,
    /// Token processing
    TokenProcessing,
    /// Rule condition evaluation
    ConditionEvaluation,
    /// Rule action execution
    ActionExecution,
    /// Token propagation
    TokenPropagation,
}

/// Comprehensive execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Trace identifier
    pub trace_id: TraceId,
    /// Rule that triggered this execution
    pub rule_id: RuleId,
    /// Input fact that started the trace
    pub input_fact: Fact,
    /// Timestamp when trace started
    pub started_at: SystemTime,
    /// Timestamp when trace completed
    pub completed_at: Option<SystemTime>,
    /// Execution path through RETE network
    pub execution_path: Vec<NodeExecution>,
    /// Final result of execution
    pub result: ExecutionResult,
    /// Performance metrics for this trace
    pub performance: TracePerformance,
    /// Memory usage during execution
    pub memory_usage: MemoryUsage,
    /// Dependencies discovered during execution
    pub dependencies: Vec<RuleDependency>,
}

/// Execution of a single RETE node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecution {
    /// Node identifier
    pub node_id: NodeId,
    /// Node type (alpha, beta, terminal)
    pub node_type: String,
    /// Input tokens
    pub input_tokens: Vec<Token>,
    /// Output tokens
    pub output_tokens: Vec<Token>,
    /// Execution start time
    pub started_at: SystemTime,
    /// Execution duration
    pub duration: Duration,
    /// Memory allocated during execution
    pub memory_allocated: usize,
    /// Whether this node caused rule firing
    pub fired_rule: bool,
    /// Condition evaluation details
    pub condition_evaluation: Option<ConditionEvaluation>,
    /// Action execution details
    pub action_execution: Option<ActionExecution>,
}

/// Details of condition evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionEvaluation {
    /// Condition expression
    pub expression: String,
    /// Evaluation result
    pub result: bool,
    /// Evaluation time
    pub evaluation_time: Duration,
    /// Variables used in evaluation
    pub variables: HashMap<String, String>,
    /// Sub-conditions evaluated
    pub sub_conditions: Vec<SubConditionResult>,
}

/// Result of sub-condition evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubConditionResult {
    /// Sub-condition expression
    pub expression: String,
    /// Result
    pub result: bool,
    /// Evaluation time
    pub evaluation_time: Duration,
}

/// Details of action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionExecution {
    /// Action type
    pub action_type: String,
    /// Action parameters
    pub parameters: HashMap<String, String>,
    /// Execution result
    pub result: ActionResult,
    /// Execution time
    pub execution_time: Duration,
    /// Facts created by action
    pub facts_created: Vec<Fact>,
    /// External calls made
    pub external_calls: Vec<ExternalCall>,
}

/// Result of action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    /// Action succeeded
    Success,
    /// Action failed with error
    Failed(String),
    /// Action was skipped
    Skipped(String),
}

/// External call made during action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalCall {
    /// Call type (HTTP, database, etc.)
    pub call_type: String,
    /// Target endpoint/resource
    pub target: String,
    /// Call duration
    pub duration: Duration,
    /// Call result
    pub result: String,
}

/// Final result of trace execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionResult {
    /// Rule fired successfully
    RuleFired {
        /// Number of actions executed
        actions_executed: usize,
        /// Facts created
        facts_created: Vec<Fact>,
    },
    /// Rule conditions not met
    ConditionsNotMet {
        /// Failed condition details
        failed_conditions: Vec<String>,
    },
    /// Execution failed
    ExecutionFailed {
        /// Error details
        error: String,
        /// Node where failure occurred
        failed_at_node: Option<NodeId>,
    },
}

/// Performance metrics for a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracePerformance {
    /// Total execution time
    pub total_time: Duration,
    /// Time spent in alpha nodes
    pub alpha_time: Duration,
    /// Time spent in beta nodes
    pub beta_time: Duration,
    /// Time spent in terminal nodes
    pub terminal_time: Duration,
    /// Time spent in condition evaluation
    pub condition_evaluation_time: Duration,
    /// Time spent in action execution
    pub action_execution_time: Duration,
    /// Number of nodes traversed
    pub nodes_traversed: usize,
    /// Number of token operations
    pub token_operations: usize,
    /// Cache hit ratio
    pub cache_hit_ratio: f64,
}

/// Memory usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    /// Memory at start of execution
    pub start_memory: usize,
    /// Peak memory during execution
    pub peak_memory: usize,
    /// Memory at end of execution
    pub end_memory: usize,
    /// Memory allocated for tokens
    pub token_memory: usize,
    /// Memory allocated for facts
    pub fact_memory: usize,
    /// Memory used by working memory
    pub working_memory: usize,
}

/// Rule dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDependency {
    /// Rule that depends on another
    pub dependent_rule: RuleId,
    /// Rule being depended upon
    pub dependency_rule: RuleId,
    /// Type of dependency
    pub dependency_type: DependencyType,
    /// Strength of dependency (0.0 to 1.0)
    pub strength: f64,
}

/// Types of rule dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Data dependency (rule depends on facts created by another)
    DataDependency,
    /// Conflict dependency (rules compete for same resources)
    ConflictDependency,
    /// Temporal dependency (execution order matters)
    TemporalDependency,
    /// Conditional dependency (one rule enables another)
    ConditionalDependency,
}

/// Performance profile for a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePerformanceProfile {
    /// Rule identifier
    pub rule_id: RuleId,
    /// Number of times rule was evaluated
    pub evaluation_count: usize,
    /// Number of times rule fired
    pub fire_count: usize,
    /// Average execution time
    pub average_execution_time: Duration,
    /// Minimum execution time
    pub min_execution_time: Duration,
    /// Maximum execution time
    pub max_execution_time: Duration,
    /// Total execution time
    pub total_execution_time: Duration,
    /// Average memory usage
    pub average_memory_usage: usize,
    /// Peak memory usage
    pub peak_memory_usage: usize,
    /// Success rate (fires / evaluations)
    pub success_rate: f64,
    /// Performance trend over time
    pub performance_trend: Vec<PerformanceDataPoint>,
    /// Bottleneck analysis
    pub bottlenecks: Vec<PerformanceBottleneck>,
}

/// Performance data point for trending
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDataPoint {
    /// Timestamp
    pub timestamp: SystemTime,
    /// Execution time at this point
    pub execution_time: Duration,
    /// Memory usage at this point
    pub memory_usage: usize,
    /// Throughput (evaluations per second)
    pub throughput: f64,
}

/// Performance bottleneck identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    /// Bottleneck type
    pub bottleneck_type: BottleneckType,
    /// Severity (0.0 to 1.0)
    pub severity: f64,
    /// Description
    pub description: String,
    /// Suggested optimization
    pub suggestion: String,
    /// Node where bottleneck occurs
    pub node_id: Option<NodeId>,
}

/// Types of performance bottlenecks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckType {
    /// CPU-bound bottleneck
    CpuBound,
    /// Memory-bound bottleneck
    MemoryBound,
    /// I/O-bound bottleneck
    IoBound,
    /// Condition evaluation bottleneck
    ConditionEvaluation,
    /// Action execution bottleneck
    ActionExecution,
    /// Token propagation bottleneck
    TokenPropagation,
}

/// Overall debugging statistics
#[derive(Debug, Default, Clone)]
pub struct DebugStats {
    /// Total debugging sessions created
    pub total_sessions: usize,
    /// Total traces recorded
    pub total_traces: usize,
    /// Total profiling time
    pub total_profiling_time: Duration,
    /// Memory usage for debugging
    pub debug_memory_usage: usize,
    /// Most frequently debugged rules
    pub top_debugged_rules: Vec<(RuleId, usize)>,
    /// Performance improvement opportunities
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
}

/// Optimization opportunity identified by debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    /// Type of optimization
    pub optimization_type: OptimizationType,
    /// Potential improvement (percentage)
    pub potential_improvement: f64,
    /// Priority (0.0 to 1.0)
    pub priority: f64,
    /// Description
    pub description: String,
    /// Rules affected
    pub affected_rules: Vec<RuleId>,
    /// Implementation complexity
    pub complexity: OptimizationComplexity,
}

/// Types of optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    /// Rule reordering
    RuleReordering,
    /// Index optimization
    IndexOptimization,
    /// Memory optimization
    MemoryOptimization,
    /// Condition simplification
    ConditionSimplification,
    /// Node sharing optimization
    NodeSharing,
    /// Caching optimization
    CachingOptimization,
}

/// Complexity of implementing an optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationComplexity {
    /// Simple change, low risk
    Low,
    /// Moderate change, medium risk
    Medium,
    /// Complex change, high risk
    High,
    /// Major architectural change
    Critical,
}

impl DebugManager {
    /// Create new debug manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            traces: HashMap::new(),
            profiles: HashMap::new(),
            config: DebugConfig::default(),
            stats: DebugStats::default(),
        }
    }

    /// Create new debugging session
    pub fn create_session(&mut self, target_rules: Vec<RuleId>, config: Option<DebugConfig>) -> DebugSessionId {
        let session_id = Uuid::new_v4();
        let session = DebugSession {
            session_id,
            target_rules,
            started_at: SystemTime::now(),
            config: config.unwrap_or_else(|| self.config.clone()),
            traces: Vec::new(),
            stats: SessionStats::default(),
            breakpoints: HashMap::new(),
            step_state: None,
        };

        self.sessions.insert(session_id, session);
        self.stats.total_sessions += 1;

        tracing::info!(
            session_id = %session_id,
            "Created new debugging session"
        );

        session_id
    }

    /// Start execution trace for a rule
    pub fn start_trace(&mut self, rule_id: RuleId, input_fact: Fact) -> TraceId {
        let trace_id = Uuid::new_v4();
        let trace = ExecutionTrace {
            trace_id,
            rule_id,
            input_fact,
            started_at: SystemTime::now(),
            completed_at: None,
            execution_path: Vec::new(),
            result: ExecutionResult::ConditionsNotMet {
                failed_conditions: Vec::new(),
            },
            performance: TracePerformance::default(),
            memory_usage: MemoryUsage::default(),
            dependencies: Vec::new(),
        };

        self.traces.insert(trace_id, trace);
        self.stats.total_traces += 1;

        tracing::debug!(
            trace_id = %trace_id,
            rule_id = %rule_id,
            "Started execution trace"
        );

        trace_id
    }

    /// Record node execution in trace
    pub fn record_node_execution(&mut self, trace_id: TraceId, node_execution: NodeExecution) {
        if let Some(trace) = self.traces.get_mut(&trace_id) {
            trace.execution_path.push(node_execution);
        }
    }

    /// Complete execution trace
    pub fn complete_trace(&mut self, trace_id: TraceId, result: ExecutionResult) {
        let rule_id = if let Some(trace) = self.traces.get_mut(&trace_id) {
            trace.completed_at = Some(SystemTime::now());
            trace.result = result;
            trace.rule_id
        } else {
            return;
        };
        
        // Calculate performance metrics
        self.calculate_trace_performance(trace_id);
        
        // Update profiles
        if let Some(trace) = self.traces.get(&trace_id).cloned() {
            self.update_rule_profile(rule_id, &trace);
        }
    }

    /// Calculate performance metrics for a trace
    fn calculate_trace_performance(&mut self, trace_id: TraceId) {
        if let Some(trace) = self.traces.get_mut(&trace_id) {
            let mut performance = TracePerformance::default();
            
            performance.nodes_traversed = trace.execution_path.len();
            performance.total_time = trace.completed_at
                .unwrap_or(SystemTime::now())
                .duration_since(trace.started_at)
                .unwrap_or_default();

            for node_exec in &trace.execution_path {
                match node_exec.node_type.as_str() {
                    "alpha" => performance.alpha_time += node_exec.duration,
                    "beta" => performance.beta_time += node_exec.duration,
                    "terminal" => performance.terminal_time += node_exec.duration,
                    _ => {}
                }

                if let Some(cond_eval) = &node_exec.condition_evaluation {
                    performance.condition_evaluation_time += cond_eval.evaluation_time;
                }

                if let Some(action_exec) = &node_exec.action_execution {
                    performance.action_execution_time += action_exec.execution_time;
                }

                performance.token_operations += node_exec.input_tokens.len() + node_exec.output_tokens.len();
            }

            trace.performance = performance;
        }
    }

    /// Update rule performance profile
    fn update_rule_profile(&mut self, rule_id: RuleId, trace: &ExecutionTrace) {
        let profile = self.profiles.entry(rule_id).or_insert_with(|| {
            RulePerformanceProfile {
                rule_id,
                evaluation_count: 0,
                fire_count: 0,
                average_execution_time: Duration::default(),
                min_execution_time: Duration::from_secs(3600), // Start with high value
                max_execution_time: Duration::default(),
                total_execution_time: Duration::default(),
                average_memory_usage: 0,
                peak_memory_usage: 0,
                success_rate: 0.0,
                performance_trend: Vec::new(),
                bottlenecks: Vec::new(),
            }
        });

        profile.evaluation_count += 1;
        
        if matches!(trace.result, ExecutionResult::RuleFired { .. }) {
            profile.fire_count += 1;
        }

        let execution_time = trace.performance.total_time;
        profile.total_execution_time += execution_time;
        profile.average_execution_time = profile.total_execution_time / profile.evaluation_count as u32;

        if execution_time < profile.min_execution_time {
            profile.min_execution_time = execution_time;
        }
        if execution_time > profile.max_execution_time {
            profile.max_execution_time = execution_time;
        }

        profile.success_rate = profile.fire_count as f64 / profile.evaluation_count as f64;

        // Add performance data point
        profile.performance_trend.push(PerformanceDataPoint {
            timestamp: trace.started_at,
            execution_time,
            memory_usage: trace.memory_usage.peak_memory,
            throughput: 1000.0 / execution_time.as_millis() as f64, // Evaluations per second
        });

        // Keep only recent data points (last 100)
        if profile.performance_trend.len() > 100 {
            profile.performance_trend.remove(0);
        }
    }

    /// Set breakpoint
    pub fn set_breakpoint(&mut self, session_id: DebugSessionId, node_id: NodeId, condition: BreakpointCondition) -> anyhow::Result<()> {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            let breakpoint = Breakpoint {
                node_id,
                condition,
                enabled: true,
                hit_count: 0,
            };
            session.breakpoints.insert(node_id, breakpoint);
            
            tracing::info!(
                session_id = %session_id,
                node_id = %node_id,
                "Set breakpoint"
            );
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Debug session not found: {}", session_id))
        }
    }

    /// Check if execution should break at a node
    pub fn should_break(&mut self, session_id: DebugSessionId, node_id: NodeId, token: &Token) -> bool {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            if let Some(breakpoint) = session.breakpoints.get_mut(&node_id) {
                if !breakpoint.enabled {
                    return false;
                }

                let should_break = match &breakpoint.condition {
                    BreakpointCondition::Always => true,
                    BreakpointCondition::FactId(fact_id) => {
                        token.fact_ids.iter().any(|&id| id == *fact_id)
                    }
                    BreakpointCondition::RuleFired(_rule_id) => {
                        // This would be checked when rule fires
                        false
                    }
                    BreakpointCondition::HitCount(target_count) => {
                        breakpoint.hit_count >= *target_count
                    }
                    BreakpointCondition::Custom(_condition) => {
                        // Custom condition evaluation would go here
                        false
                    }
                };

                if should_break {
                    breakpoint.hit_count += 1;
                    tracing::info!(
                        session_id = %session_id,
                        node_id = %node_id,
                        hit_count = breakpoint.hit_count,
                        "Breakpoint hit"
                    );
                }

                should_break
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get performance profile for a rule
    pub fn get_rule_profile(&self, rule_id: RuleId) -> Option<&RulePerformanceProfile> {
        self.profiles.get(&rule_id)
    }

    /// Get execution traces for a session
    pub fn get_session_traces(&self, session_id: DebugSessionId) -> Vec<&ExecutionTrace> {
        if let Some(session) = self.sessions.get(&session_id) {
            session.traces.iter()
                .filter_map(|trace_id| self.traces.get(trace_id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Analyze performance bottlenecks
    pub fn analyze_bottlenecks(&mut self, rule_id: RuleId) -> Vec<PerformanceBottleneck> {
        if let Some(profile) = self.profiles.get_mut(&rule_id) {
            let mut bottlenecks = Vec::new();

            // Analyze execution time trends
            if profile.performance_trend.len() >= 10 {
                let recent_times: Vec<_> = profile.performance_trend.iter()
                    .rev()
                    .take(10)
                    .map(|dp| dp.execution_time.as_micros())
                    .collect();

                let avg_time = recent_times.iter().sum::<u128>() / recent_times.len() as u128;
                let max_time = *recent_times.iter().max().unwrap();

                if max_time > avg_time * 2 {
                    bottlenecks.push(PerformanceBottleneck {
                        bottleneck_type: BottleneckType::CpuBound,
                        severity: 0.7,
                        description: "Execution time variance is high".to_string(),
                        suggestion: "Consider optimizing condition evaluation".to_string(),
                        node_id: None,
                    });
                }
            }

            // Analyze success rate
            if profile.success_rate < 0.1 && profile.evaluation_count > 100 {
                bottlenecks.push(PerformanceBottleneck {
                    bottleneck_type: BottleneckType::ConditionEvaluation,
                    severity: 0.8,
                    description: "Low success rate indicates inefficient conditions".to_string(),
                    suggestion: "Consider reordering conditions or adding more selective conditions first".to_string(),
                    node_id: None,
                });
            }

            profile.bottlenecks = bottlenecks.clone();
            bottlenecks
        } else {
            Vec::new()
        }
    }

    /// Generate optimization recommendations
    pub fn generate_optimizations(&self) -> Vec<OptimizationOpportunity> {
        let mut opportunities = Vec::new();

        for profile in self.profiles.values() {
            // Low success rate optimization
            if profile.success_rate < 0.2 && profile.evaluation_count > 50 {
                opportunities.push(OptimizationOpportunity {
                    optimization_type: OptimizationType::RuleReordering,
                    potential_improvement: (1.0 - profile.success_rate) * 50.0,
                    priority: 0.8,
                    description: format!("Rule {} has low success rate ({}%), consider reordering conditions", 
                                       profile.rule_id, (profile.success_rate * 100.0) as u32),
                    affected_rules: vec![profile.rule_id],
                    complexity: OptimizationComplexity::Low,
                });
            }

            // High execution time optimization
            if profile.average_execution_time > Duration::from_millis(100) {
                opportunities.push(OptimizationOpportunity {
                    optimization_type: OptimizationType::IndexOptimization,
                    potential_improvement: 30.0,
                    priority: 0.7,
                    description: format!("Rule {} has high execution time ({}ms), consider adding indexes", 
                                       profile.rule_id, profile.average_execution_time.as_millis()),
                    affected_rules: vec![profile.rule_id],
                    complexity: OptimizationComplexity::Medium,
                });
            }
        }

        opportunities
    }

    /// Clean up old traces
    pub fn cleanup_old_traces(&mut self) {
        let cutoff = SystemTime::now() - Duration::from_millis(self.config.trace_retention_ms);
        
        self.traces.retain(|_, trace| {
            trace.started_at > cutoff
        });
    }

    /// Get debugging statistics
    pub fn get_stats(&self) -> &DebugStats {
        &self.stats
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enable_tracing: true,
            enable_profiling: true,
            enable_memory_tracking: true,
            max_traces: 1000,
            trace_retention_ms: 3600_000, // 1 hour
            min_execution_time_us: 100,   // 100 microseconds
            enable_token_tracking: true,
            enable_dependency_analysis: false,
        }
    }
}

impl Default for TracePerformance {
    fn default() -> Self {
        Self {
            total_time: Duration::default(),
            alpha_time: Duration::default(),
            beta_time: Duration::default(),
            terminal_time: Duration::default(),
            condition_evaluation_time: Duration::default(),
            action_execution_time: Duration::default(),
            nodes_traversed: 0,
            token_operations: 0,
            cache_hit_ratio: 0.0,
        }
    }
}

impl Default for MemoryUsage {
    fn default() -> Self {
        Self {
            start_memory: 0,
            peak_memory: 0,
            end_memory: 0,
            token_memory: 0,
            fact_memory: 0,
            working_memory: 0,
        }
    }
}

/// Get current timestamp in milliseconds since Unix epoch
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}