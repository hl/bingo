//! Debugging hooks for rule firing and token propagation
//!
//! This module provides comprehensive debugging capabilities for the RETE network,
//! including rule firing events, token propagation tracking, and execution tracing.

use crate::rete_nodes::{NodeId, Token};
use crate::types::{Fact, FactValue, RuleId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use tracing::{debug, info, instrument, trace, warn};
use uuid::Uuid;

/// Debug session identifier
pub type DebugSessionId = Uuid;

/// Trace identifier for execution tracking
pub type TraceId = Uuid;

/// Event identifier for individual debug events
pub type EventId = Uuid;

/// Comprehensive debugging hook manager
#[derive(Debug)]
pub struct DebugHookManager {
    /// Active debugging sessions
    sessions: HashMap<DebugSessionId, DebugSession>,
    /// Global event hooks
    event_hooks: Vec<Box<dyn EventHook>>,
    /// Rule firing hooks
    rule_hooks: HashMap<RuleId, Vec<Box<dyn RuleFireHook>>>,
    /// Token propagation hooks
    token_hooks: Vec<Box<dyn TokenPropagationHook>>,
    /// Debug configuration
    config: DebugConfig,
    /// Event buffer for recent events
    event_buffer: Arc<Mutex<VecDeque<DebugEvent>>>,
    /// Statistics for debugging overhead
    debug_stats: DebugOverheadStats,
}

/// Configuration for debugging behavior
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// Enable rule firing hooks
    pub enable_rule_hooks: bool,
    /// Enable token propagation hooks
    pub enable_token_hooks: bool,
    /// Enable detailed event tracing
    pub enable_event_tracing: bool,
    /// Maximum number of events to keep in buffer
    pub max_event_buffer_size: usize,
    /// Enable performance impact tracking
    pub track_debug_overhead: bool,
    /// Minimum execution time to trigger hooks (microseconds)
    pub min_execution_time_us: u64,
    /// Enable conditional debugging based on fact content
    pub enable_conditional_debugging: bool,
    /// Sample rate for events (0.0 to 1.0, 1.0 = all events)
    pub event_sample_rate: f64,
}

/// Debug session for tracking execution
#[derive(Debug)]
pub struct DebugSession {
    /// Session identifier
    pub session_id: DebugSessionId,
    /// Session start time
    pub started_at: SystemTime,
    /// Rules being monitored in this session
    pub monitored_rules: Vec<RuleId>,
    /// Fact patterns to watch for
    pub fact_patterns: Vec<FactPattern>,
    /// Execution traces collected
    pub traces: Vec<ExecutionTrace>,
    /// Session configuration
    pub config: DebugConfig,
    /// Session statistics
    pub session_stats: SessionDebugStats,
    /// Active breakpoints
    pub breakpoints: HashMap<NodeId, Breakpoint>,
}

/// Pattern for matching facts during debugging
#[derive(Debug, Clone)]
pub struct FactPattern {
    /// Field name to match
    pub field_name: String,
    /// Expected value (None means any value)
    pub expected_value: Option<FactValue>,
    /// Match operator
    pub operator: PatternOperator,
}

/// Pattern matching operators
#[derive(Debug, Clone)]
pub enum PatternOperator {
    /// Exact equality
    Equals,
    /// Contains substring (for strings)
    Contains,
    /// Greater than (for numbers)
    GreaterThan,
    /// Less than (for numbers)
    LessThan,
    /// Value exists (ignores expected_value)
    Exists,
    /// Regex match (for strings)
    Regex(String),
}

/// Execution trace for a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Trace identifier
    pub trace_id: TraceId,
    /// Rule being traced
    pub rule_id: RuleId,
    /// Trace start time
    pub started_at: SystemTime,
    /// Input facts that triggered the trace
    pub input_facts: Vec<Fact>,
    /// Execution steps
    pub execution_steps: Vec<ExecutionStep>,
    /// Final result
    pub result: ExecutionResult,
    /// Total execution time
    pub total_time: Duration,
    /// Debug events captured during execution
    pub debug_events: Vec<DebugEvent>,
}

/// Individual execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// Step identifier
    pub step_id: Uuid,
    /// Step timestamp
    pub timestamp: SystemTime,
    /// Node where step occurred
    pub node_id: NodeId,
    /// Node type
    pub node_type: String,
    /// Step type
    pub step_type: StepType,
    /// Input tokens
    pub input_tokens: Vec<Token>,
    /// Output tokens
    pub output_tokens: Vec<Token>,
    /// Step duration
    pub duration: Duration,
    /// Additional step data
    pub step_data: StepData,
}

/// Types of execution steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    /// Token arrived at alpha node
    AlphaNodeEntry,
    /// Alpha node condition evaluation
    AlphaNodeEvaluation,
    /// Alpha node produced tokens
    AlphaNodeOutput,
    /// Token arrived at beta node
    BetaNodeEntry,
    /// Beta node join operation
    BetaNodeJoin,
    /// Beta node produced tokens
    BetaNodeOutput,
    /// Token arrived at terminal node
    TerminalNodeEntry,
    /// Rule condition evaluation
    RuleEvaluation,
    /// Rule action execution
    RuleActionExecution,
    /// Rule fired successfully
    RuleFired,
}

/// Additional data for execution steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepData {
    /// Condition that was evaluated (if applicable)
    pub condition_evaluated: Option<String>,
    /// Condition result (if applicable)
    pub condition_result: Option<bool>,
    /// Action executed (if applicable)
    pub action_executed: Option<String>,
    /// Action result (if applicable)
    pub action_result: Option<String>,
    /// Memory state before step
    pub memory_before: Option<usize>,
    /// Memory state after step
    pub memory_after: Option<usize>,
    /// Custom debug data
    pub custom_data: HashMap<String, String>,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionResult {
    /// Rule fired successfully
    Success {
        /// Number of facts created
        facts_created: usize,
        /// Actions executed
        actions_executed: Vec<String>,
    },
    /// Rule conditions not met
    ConditionsNotMet {
        /// Failed conditions
        failed_conditions: Vec<String>,
    },
    /// Execution failed with error
    Failed {
        /// Error message
        error: String,
        /// Node where failure occurred
        failed_at_node: Option<NodeId>,
    },
}

/// Debug event for detailed tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugEvent {
    /// Event identifier
    pub event_id: EventId,
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Event type
    pub event_type: DebugEventType,
    /// Associated rule (if applicable)
    pub rule_id: Option<RuleId>,
    /// Associated node (if applicable)
    pub node_id: Option<NodeId>,
    /// Event description
    pub description: String,
    /// Event data
    pub event_data: HashMap<String, String>,
    /// Event severity
    pub severity: EventSeverity,
}

/// Types of debug events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugEventType {
    /// Rule started evaluation
    RuleEvaluationStarted,
    /// Rule finished evaluation
    RuleEvaluationFinished,
    /// Rule fired
    RuleFired,
    /// Rule conditions failed
    RuleConditionsFailed,
    /// Token created
    TokenCreated,
    /// Token propagated between nodes
    TokenPropagated,
    /// Token consumed
    TokenConsumed,
    /// Node memory updated
    NodeMemoryUpdated,
    /// Condition evaluated
    ConditionEvaluated,
    /// Action executed
    ActionExecuted,
    /// Performance threshold exceeded
    PerformanceThresholdExceeded,
    /// Custom debug event
    Custom(String),
}

/// Event severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    /// Trace level - very detailed
    Trace,
    /// Debug level - detailed
    Debug,
    /// Info level - informational
    Info,
    /// Warn level - potential issues
    Warn,
    /// Error level - actual problems
    Error,
}

/// Session debugging statistics
#[derive(Debug, Default, Clone)]
pub struct SessionDebugStats {
    /// Total events captured
    pub total_events: usize,
    /// Events by type
    pub events_by_type: HashMap<String, usize>,
    /// Rules traced
    pub rules_traced: usize,
    /// Total trace time
    pub total_trace_time: Duration,
    /// Average trace time
    pub average_trace_time: Duration,
    /// Debugging overhead time
    pub debug_overhead: Duration,
}

/// Statistics for debugging overhead
#[derive(Debug, Default, Clone)]
pub struct DebugOverheadStats {
    /// Total time spent in debugging hooks
    pub total_debug_time: Duration,
    /// Number of hook invocations
    pub hook_invocations: usize,
    /// Average time per hook invocation
    pub average_hook_time: Duration,
    /// Percentage of total execution time spent in debugging
    pub overhead_percentage: f64,
}

/// Breakpoint for debugging
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Breakpoint identifier
    pub id: Uuid,
    /// Node where breakpoint is set
    pub node_id: NodeId,
    /// Breakpoint condition
    pub condition: BreakpointCondition,
    /// Whether breakpoint is enabled
    pub enabled: bool,
    /// Number of times hit
    pub hit_count: usize,
    /// Action to take when hit
    pub action: BreakpointAction,
}

/// Breakpoint conditions
#[derive(Debug, Clone)]
pub enum BreakpointCondition {
    /// Always break
    Always,
    /// Break when specific fact ID is processed
    FactId(u64),
    /// Break when rule fires
    RuleFired(RuleId),
    /// Break after N hits
    HitCount(usize),
    /// Break when fact matches pattern
    FactPattern(FactPattern),
    /// Break when custom condition is met
    Custom(String),
}

/// Breakpoint actions
#[derive(Debug, Clone)]
pub enum BreakpointAction {
    /// Pause execution (if interactive)
    Pause,
    /// Log event and continue
    Log,
    /// Execute custom callback
    Callback(String),
    /// Collect detailed trace
    CollectTrace,
}

// Trait definitions for hooks

/// General event hook trait
pub trait EventHook: Send + Sync + std::fmt::Debug {
    /// Called when any debug event occurs
    fn on_event(&self, event: &DebugEvent, context: &DebugContext);

    /// Get hook name for identification
    fn name(&self) -> &str;

    /// Check if hook should process this event
    fn should_process(&self, event: &DebugEvent) -> bool {
        true // Default: process all events
    }
}

/// Rule firing hook trait
pub trait RuleFireHook: Send + Sync + std::fmt::Debug {
    /// Called before rule evaluation
    fn before_rule_evaluation(&self, rule_id: RuleId, facts: &[Fact], context: &DebugContext);

    /// Called after rule evaluation
    fn after_rule_evaluation(
        &self,
        rule_id: RuleId,
        result: &ExecutionResult,
        context: &DebugContext,
    );

    /// Called when rule fires
    fn on_rule_fired(
        &self,
        rule_id: RuleId,
        input_facts: &[Fact],
        output_facts: &[Fact],
        context: &DebugContext,
    );

    /// Get hook name
    fn name(&self) -> &str;
}

/// Token propagation hook trait
pub trait TokenPropagationHook: Send + Sync + std::fmt::Debug {
    /// Called when token is created
    fn on_token_created(&self, token: &Token, node_id: NodeId, context: &DebugContext);

    /// Called when token is propagated between nodes
    fn on_token_propagated(
        &self,
        token: &Token,
        from_node: NodeId,
        to_node: NodeId,
        context: &DebugContext,
    );

    /// Called when token is consumed
    fn on_token_consumed(&self, token: &Token, node_id: NodeId, context: &DebugContext);

    /// Get hook name
    fn name(&self) -> &str;
}

/// Debug context provided to hooks
#[derive(Debug, Clone)]
pub struct DebugContext {
    /// Current session ID (if any)
    pub session_id: Option<DebugSessionId>,
    /// Current trace ID (if any)
    pub trace_id: Option<TraceId>,
    /// Timestamp when context was created
    pub timestamp: SystemTime,
    /// Additional context data
    pub context_data: HashMap<String, String>,
}

impl DebugHookManager {
    /// Create new debug hook manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            event_hooks: Vec::new(),
            rule_hooks: HashMap::new(),
            token_hooks: Vec::new(),
            config: DebugConfig::default(),
            event_buffer: Arc::new(Mutex::new(VecDeque::new())),
            debug_stats: DebugOverheadStats::default(),
        }
    }

    /// Create debug hook manager with custom configuration
    pub fn with_config(config: DebugConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            event_hooks: Vec::new(),
            rule_hooks: HashMap::new(),
            token_hooks: Vec::new(),
            config,
            event_buffer: Arc::new(Mutex::new(VecDeque::new())),
            debug_stats: DebugOverheadStats::default(),
        }
    }

    /// Start a new debugging session
    #[instrument(skip(self))]
    pub fn start_session(
        &mut self,
        monitored_rules: Vec<RuleId>,
        fact_patterns: Vec<FactPattern>,
    ) -> DebugSessionId {
        let session_id = Uuid::new_v4();

        let session = DebugSession {
            session_id,
            started_at: SystemTime::now(),
            monitored_rules,
            fact_patterns,
            traces: Vec::new(),
            config: self.config.clone(),
            session_stats: SessionDebugStats::default(),
            breakpoints: HashMap::new(),
        };

        self.sessions.insert(session_id, session);

        info!(
            session_id = %session_id,
            "Started debugging session"
        );

        session_id
    }

    /// Add event hook
    pub fn add_event_hook(&mut self, hook: Box<dyn EventHook>) {
        debug!(hook_name = hook.name(), "Adding event hook");
        self.event_hooks.push(hook);
    }

    /// Add rule firing hook
    pub fn add_rule_hook(&mut self, rule_id: RuleId, hook: Box<dyn RuleFireHook>) {
        debug!(
            rule_id = rule_id,
            hook_name = hook.name(),
            "Adding rule firing hook"
        );
        self.rule_hooks.entry(rule_id).or_insert_with(Vec::new).push(hook);
    }

    /// Add token propagation hook
    pub fn add_token_hook(&mut self, hook: Box<dyn TokenPropagationHook>) {
        debug!(hook_name = hook.name(), "Adding token propagation hook");
        self.token_hooks.push(hook);
    }

    /// Trigger rule evaluation start event
    #[instrument(skip(self, facts))]
    pub fn trigger_rule_evaluation_started(&mut self, rule_id: RuleId, facts: &[Fact]) {
        if !self.config.enable_rule_hooks {
            return;
        }

        let start_time = Instant::now();
        let context = self.create_debug_context(None, None);

        // Trigger rule hooks
        if let Some(hooks) = self.rule_hooks.get(&rule_id) {
            for hook in hooks {
                hook.before_rule_evaluation(rule_id, facts, &context);
            }
        }

        // Create debug event
        self.emit_event(DebugEvent {
            event_id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            event_type: DebugEventType::RuleEvaluationStarted,
            rule_id: Some(rule_id),
            node_id: None,
            description: format!(
                "Rule {} evaluation started with {} facts",
                rule_id,
                facts.len()
            ),
            event_data: HashMap::new(),
            severity: EventSeverity::Debug,
        });

        self.track_debug_overhead(start_time.elapsed());
    }

    /// Trigger rule fired event
    #[instrument(skip(self, input_facts, output_facts))]
    pub fn trigger_rule_fired(
        &mut self,
        rule_id: RuleId,
        input_facts: &[Fact],
        output_facts: &[Fact],
    ) {
        if !self.config.enable_rule_hooks {
            return;
        }

        let start_time = Instant::now();
        let context = self.create_debug_context(None, None);

        // Trigger rule hooks
        if let Some(hooks) = self.rule_hooks.get(&rule_id) {
            for hook in hooks {
                hook.on_rule_fired(rule_id, input_facts, output_facts, &context);
            }
        }

        // Create debug event
        let mut event_data = HashMap::new();
        event_data.insert(
            "input_fact_count".to_string(),
            input_facts.len().to_string(),
        );
        event_data.insert(
            "output_fact_count".to_string(),
            output_facts.len().to_string(),
        );

        self.emit_event(DebugEvent {
            event_id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            event_type: DebugEventType::RuleFired,
            rule_id: Some(rule_id),
            node_id: None,
            description: format!(
                "Rule {} fired: {} input facts → {} output facts",
                rule_id,
                input_facts.len(),
                output_facts.len()
            ),
            event_data,
            severity: EventSeverity::Info,
        });

        self.track_debug_overhead(start_time.elapsed());
    }

    /// Trigger token created event
    #[instrument(skip(self, token))]
    pub fn trigger_token_created(&mut self, token: &Token, node_id: NodeId) {
        if !self.config.enable_token_hooks {
            return;
        }

        let start_time = Instant::now();
        let context = self.create_debug_context(None, None);

        // Trigger token hooks
        for hook in &self.token_hooks {
            hook.on_token_created(token, node_id, &context);
        }

        // Create debug event
        let mut event_data = HashMap::new();
        event_data.insert("fact_count".to_string(), token.fact_ids.len().to_string());
        event_data.insert("node_id".to_string(), node_id.to_string());

        self.emit_event(DebugEvent {
            event_id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            event_type: DebugEventType::TokenCreated,
            rule_id: None,
            node_id: Some(node_id),
            description: format!(
                "Token created at node {} with {} facts",
                node_id,
                token.fact_ids.len()
            ),
            event_data,
            severity: EventSeverity::Trace,
        });

        self.track_debug_overhead(start_time.elapsed());
    }

    /// Trigger token propagated event
    #[instrument(skip(self, token))]
    pub fn trigger_token_propagated(&mut self, token: &Token, from_node: NodeId, to_node: NodeId) {
        if !self.config.enable_token_hooks {
            return;
        }

        let start_time = Instant::now();
        let context = self.create_debug_context(None, None);

        // Trigger token hooks
        for hook in &self.token_hooks {
            hook.on_token_propagated(token, from_node, to_node, &context);
        }

        // Create debug event
        let mut event_data = HashMap::new();
        event_data.insert("from_node".to_string(), from_node.to_string());
        event_data.insert("to_node".to_string(), to_node.to_string());
        event_data.insert("fact_count".to_string(), token.fact_ids.len().to_string());

        self.emit_event(DebugEvent {
            event_id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            event_type: DebugEventType::TokenPropagated,
            rule_id: None,
            node_id: Some(from_node),
            description: format!(
                "Token propagated from node {} to node {}",
                from_node, to_node
            ),
            event_data,
            severity: EventSeverity::Trace,
        });

        self.track_debug_overhead(start_time.elapsed());
    }

    /// Set breakpoint at a node
    pub fn set_breakpoint(
        &mut self,
        session_id: DebugSessionId,
        node_id: NodeId,
        condition: BreakpointCondition,
        action: BreakpointAction,
    ) -> anyhow::Result<Uuid> {
        let breakpoint_id = Uuid::new_v4();

        let breakpoint = Breakpoint {
            id: breakpoint_id,
            node_id,
            condition,
            enabled: true,
            hit_count: 0,
            action,
        };

        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.breakpoints.insert(node_id, breakpoint);

            debug!(
                session_id = %session_id,
                node_id = node_id,
                breakpoint_id = %breakpoint_id,
                "Set breakpoint"
            );

            Ok(breakpoint_id)
        } else {
            Err(anyhow::anyhow!("Debug session not found: {}", session_id))
        }
    }

    /// Get recent events from buffer
    pub fn get_recent_events(&self, limit: Option<usize>) -> Vec<DebugEvent> {
        if let Ok(buffer) = self.event_buffer.lock() {
            let limit = limit.unwrap_or(buffer.len());
            buffer.iter().rev().take(limit).cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get session statistics
    pub fn get_session_stats(&self, session_id: DebugSessionId) -> Option<&SessionDebugStats> {
        self.sessions.get(&session_id).map(|s| &s.session_stats)
    }

    /// Get debugging overhead statistics
    pub fn get_debug_overhead_stats(&self) -> &DebugOverheadStats {
        &self.debug_stats
    }

    /// Emit debug event
    fn emit_event(&mut self, event: DebugEvent) {
        // Sample events if configured
        if self.config.event_sample_rate < 1.0 {
            if rand::random::<f64>() > self.config.event_sample_rate {
                return;
            }
        }

        // Trigger event hooks
        let context = self.create_debug_context(None, None);
        for hook in &self.event_hooks {
            if hook.should_process(&event) {
                hook.on_event(&event, &context);
            }
        }

        // Add to event buffer
        if let Ok(mut buffer) = self.event_buffer.lock() {
            buffer.push_back(event.clone());

            // Maintain buffer size limit
            while buffer.len() > self.config.max_event_buffer_size {
                buffer.pop_front();
            }
        }

        // Log event if tracing is enabled
        if self.config.enable_event_tracing {
            match event.severity {
                EventSeverity::Trace => trace!(
                    event_id = %event.event_id,
                    event_type = ?event.event_type,
                    rule_id = ?event.rule_id,
                    node_id = ?event.node_id,
                    description = %event.description,
                    "Debug event"
                ),
                EventSeverity::Debug => debug!(
                    event_id = %event.event_id,
                    event_type = ?event.event_type,
                    rule_id = ?event.rule_id,
                    node_id = ?event.node_id,
                    description = %event.description,
                    "Debug event"
                ),
                EventSeverity::Info => info!(
                    event_id = %event.event_id,
                    event_type = ?event.event_type,
                    rule_id = ?event.rule_id,
                    node_id = ?event.node_id,
                    description = %event.description,
                    "Debug event"
                ),
                EventSeverity::Warn => warn!(
                    event_id = %event.event_id,
                    event_type = ?event.event_type,
                    rule_id = ?event.rule_id,
                    node_id = ?event.node_id,
                    description = %event.description,
                    "Debug event"
                ),
                EventSeverity::Error => tracing::error!(
                    event_id = %event.event_id,
                    event_type = ?event.event_type,
                    rule_id = ?event.rule_id,
                    node_id = ?event.node_id,
                    description = %event.description,
                    "Debug event"
                ),
            }
        }
    }

    /// Create debug context
    fn create_debug_context(
        &self,
        session_id: Option<DebugSessionId>,
        trace_id: Option<TraceId>,
    ) -> DebugContext {
        DebugContext {
            session_id,
            trace_id,
            timestamp: SystemTime::now(),
            context_data: HashMap::new(),
        }
    }

    /// Track debugging overhead
    fn track_debug_overhead(&mut self, duration: Duration) {
        if self.config.track_debug_overhead {
            self.debug_stats.total_debug_time += duration;
            self.debug_stats.hook_invocations += 1;
            self.debug_stats.average_hook_time =
                self.debug_stats.total_debug_time / self.debug_stats.hook_invocations as u32;
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: DebugConfig) {
        self.config = config;
    }

    /// Clear all hooks and sessions
    pub fn clear(&mut self) {
        self.sessions.clear();
        self.event_hooks.clear();
        self.rule_hooks.clear();
        self.token_hooks.clear();

        if let Ok(mut buffer) = self.event_buffer.lock() {
            buffer.clear();
        }

        self.debug_stats = DebugOverheadStats::default();
    }

    /// Alias for clear() - clear all debug sessions
    pub fn clear_all_sessions(&mut self) {
        self.clear();
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enable_rule_hooks: true,
            enable_token_hooks: true,
            enable_event_tracing: true,
            max_event_buffer_size: 1000,
            track_debug_overhead: true,
            min_execution_time_us: 10,
            enable_conditional_debugging: false,
            event_sample_rate: 1.0,
        }
    }
}

impl Default for StepData {
    fn default() -> Self {
        Self {
            condition_evaluated: None,
            condition_result: None,
            action_executed: None,
            action_result: None,
            memory_before: None,
            memory_after: None,
            custom_data: HashMap::new(),
        }
    }
}

// Built-in hook implementations

/// Console logging hook for events
#[derive(Debug)]
pub struct ConsoleEventHook {
    name: String,
}

impl ConsoleEventHook {
    pub fn new() -> Self {
        Self { name: "ConsoleEventHook".to_string() }
    }
}

impl EventHook for ConsoleEventHook {
    fn on_event(&self, event: &DebugEvent, _context: &DebugContext) {
        println!(
            "[DEBUG] {} - {} - {}",
            event
                .timestamp
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            format!("{:?}", event.event_type),
            event.description
        );
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// File logging hook for events
#[derive(Debug)]
pub struct FileEventHook {
    name: String,
    file_path: String,
}

impl FileEventHook {
    pub fn new(file_path: String) -> Self {
        Self { name: "FileEventHook".to_string(), file_path }
    }
}

impl EventHook for FileEventHook {
    fn on_event(&self, event: &DebugEvent, _context: &DebugContext) {
        // In a real implementation, this would write to a file
        // For now, we'll just use the tracing system
        debug!(
            target: "debug_file_hook",
            file_path = %self.file_path,
            event = ?event,
            "Debug event logged to file"
        );
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Performance monitoring rule hook
#[derive(Debug)]
pub struct PerformanceRuleHook {
    name: String,
    threshold_ms: u64,
}

impl PerformanceRuleHook {
    pub fn new(threshold_ms: u64) -> Self {
        Self { name: "PerformanceRuleHook".to_string(), threshold_ms }
    }
}

impl RuleFireHook for PerformanceRuleHook {
    fn before_rule_evaluation(&self, rule_id: RuleId, facts: &[Fact], _context: &DebugContext) {
        debug!(
            rule_id = rule_id,
            fact_count = facts.len(),
            "Starting rule evaluation performance monitoring"
        );
    }

    fn after_rule_evaluation(
        &self,
        rule_id: RuleId,
        result: &ExecutionResult,
        _context: &DebugContext,
    ) {
        match result {
            ExecutionResult::Success { facts_created, .. } => {
                info!(
                    rule_id = rule_id,
                    facts_created = facts_created,
                    "Rule evaluation completed successfully"
                );
            }
            ExecutionResult::Failed { error, .. } => {
                warn!(
                    rule_id = rule_id,
                    error = %error,
                    "Rule evaluation failed"
                );
            }
            _ => {}
        }
    }

    fn on_rule_fired(
        &self,
        rule_id: RuleId,
        input_facts: &[Fact],
        output_facts: &[Fact],
        _context: &DebugContext,
    ) {
        info!(
            rule_id = rule_id,
            input_count = input_facts.len(),
            output_count = output_facts.len(),
            "Rule fired successfully"
        );
    }

    fn name(&self) -> &str {
        &self.name
    }
}
