//! Enhanced Error Diagnostics and Debugging Tools
//!
//! This module provides advanced error diagnostics, contextual help,
//! and interactive debugging capabilities for better error handling.

use crate::error::{BingoError, BingoResult, ErrorContext, ErrorSeverity};
use crate::types::RuleId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use uuid::Uuid;

/// Enhanced error diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDiagnostic {
    /// Unique diagnostic ID
    pub diagnostic_id: Uuid,
    /// Original error
    pub error: String,
    /// Error category
    pub category: String,
    /// Severity level
    pub severity: ErrorSeverity,
    /// Timestamp when error occurred
    pub timestamp: DateTime<Utc>,
    /// Contextual information
    pub context: ErrorContext,
    /// Suggested fixes
    pub suggestions: Vec<ErrorSuggestion>,
    /// Related documentation links
    pub documentation_links: Vec<DocumentationLink>,
    /// Error location in code
    pub location: Option<ErrorLocation>,
    /// Stack trace if available
    pub stack_trace: Option<String>,
    /// Similar errors seen recently
    pub similar_errors: Vec<SimilarError>,
    /// User context when error occurred
    pub user_context: Option<UserContext>,
    /// System state when error occurred
    pub system_state: Option<SystemState>,
}

/// Suggested fix for an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSuggestion {
    /// Suggestion ID
    pub id: String,
    /// Priority (0.0 to 1.0)
    pub priority: f64,
    /// Brief description
    pub title: String,
    /// Detailed explanation
    pub description: String,
    /// Steps to implement the fix
    pub steps: Vec<FixStep>,
    /// Code examples if applicable
    pub code_examples: Vec<CodeExample>,
    /// Estimated effort to implement
    pub effort: EffortLevel,
    /// Risk level of implementing this fix
    pub risk: RiskLevel,
}

/// Individual step in a fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixStep {
    /// Step number
    pub step_number: usize,
    /// Step description
    pub description: String,
    /// Optional command to run
    pub command: Option<String>,
    /// Expected outcome
    pub expected_outcome: String,
}

/// Code example for a suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    /// Language
    pub language: String,
    /// Code snippet
    pub code: String,
    /// Explanation
    pub explanation: String,
}

/// Documentation link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationLink {
    /// Link title
    pub title: String,
    /// URL
    pub url: String,
    /// Brief description
    pub description: String,
    /// Relevance score (0.0 to 1.0)
    pub relevance: f64,
}

/// Error location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    /// File name
    pub file: String,
    /// Line number
    pub line: Option<u32>,
    /// Column number
    pub column: Option<u32>,
    /// Function name
    pub function: Option<String>,
    /// Module path
    pub module: Option<String>,
}

/// Similar error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarError {
    /// When the similar error occurred
    pub occurred_at: DateTime<Utc>,
    /// Error message
    pub message: String,
    /// Resolution if known
    pub resolution: Option<String>,
    /// How similar (0.0 to 1.0)
    pub similarity: f64,
}

/// User context when error occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    /// User ID if available
    pub user_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Current operation
    pub operation: Option<String>,
    /// Request parameters
    pub parameters: HashMap<String, String>,
    /// User agent / client info
    pub client_info: Option<String>,
}

/// System state when error occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    /// CPU usage percentage
    pub cpu_usage: Option<f64>,
    /// Memory usage in bytes
    pub memory_usage: Option<usize>,
    /// Number of active connections
    pub active_connections: Option<usize>,
    /// Number of rules loaded
    pub rules_loaded: Option<usize>,
    /// Facts in working memory
    pub facts_in_memory: Option<usize>,
    /// System uptime in seconds
    pub uptime_seconds: Option<u64>,
}

/// Effort level for implementing a fix
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EffortLevel {
    /// Quick fix (< 5 minutes)
    Quick,
    /// Low effort (< 30 minutes)
    Low,
    /// Medium effort (< 2 hours)
    Medium,
    /// High effort (< 1 day)
    High,
    /// Major effort (> 1 day)
    Major,
}

/// Risk level of implementing a fix
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Very low risk
    VeryLow,
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
    /// Critical risk (major changes)
    Critical,
}

/// Error analytics and patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAnalytics {
    /// Error frequency by category
    pub frequency_by_category: HashMap<String, usize>,
    /// Error trends over time
    pub error_trends: Vec<ErrorTrendPoint>,
    /// Most common error patterns
    pub common_patterns: Vec<ErrorPattern>,
    /// Resolution success rates
    pub resolution_rates: HashMap<String, f64>,
    /// Top error sources
    pub top_error_sources: Vec<ErrorSource>,
}

/// Error trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTrendPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Error count at this point
    pub error_count: usize,
    /// Category breakdown
    pub category_breakdown: HashMap<String, usize>,
}

/// Common error pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    /// Pattern description
    pub description: String,
    /// Number of occurrences
    pub occurrences: usize,
    /// Affected rules
    pub affected_rules: Vec<RuleId>,
    /// Common resolution
    pub common_resolution: Option<String>,
    /// Pattern confidence (0.0 to 1.0)
    pub confidence: f64,
}

/// Error source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSource {
    /// Source identifier (rule, node, module)
    pub source_id: String,
    /// Source type
    pub source_type: String,
    /// Error count from this source
    pub error_count: usize,
    /// Error rate (errors per execution)
    pub error_rate: f64,
}

/// Interactive debugging session
#[derive(Debug)]
pub struct InteractiveDebugSession {
    /// Session ID
    pub session_id: Uuid,
    /// Current error being debugged
    pub current_error: Option<ErrorDiagnostic>,
    /// Error history in this session
    pub error_history: VecDeque<ErrorDiagnostic>,
    /// Active suggestions
    pub active_suggestions: Vec<ErrorSuggestion>,
    /// User feedback on suggestions
    pub suggestion_feedback: HashMap<String, SuggestionFeedback>,
    /// Session configuration
    pub config: DebugSessionConfig,
}

/// Configuration for debugging session
#[derive(Debug, Clone)]
pub struct DebugSessionConfig {
    /// Maximum errors to keep in history
    pub max_error_history: usize,
    /// Enable automatic suggestion generation
    pub auto_suggestions: bool,
    /// Include system state in diagnostics
    pub include_system_state: bool,
    /// Verbosity level
    pub verbosity: VerbosityLevel,
}

/// Verbosity level for debugging
#[derive(Debug, Clone, Copy)]
pub enum VerbosityLevel {
    /// Minimal information
    Minimal,
    /// Basic information
    Basic,
    /// Detailed information
    Detailed,
    /// Maximum information
    Verbose,
}

/// User feedback on error suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionFeedback {
    /// Suggestion ID
    pub suggestion_id: String,
    /// Was the suggestion helpful?
    pub helpful: bool,
    /// Was the suggestion implemented?
    pub implemented: bool,
    /// Did it resolve the issue?
    pub resolved_issue: bool,
    /// User comments
    pub comments: Option<String>,
    /// Timestamp of feedback
    pub feedback_time: DateTime<Utc>,
}

/// Error diagnostics manager
#[derive(Debug)]
pub struct ErrorDiagnosticsManager {
    /// Recent error diagnostics
    diagnostics: VecDeque<ErrorDiagnostic>,
    /// Error analytics
    analytics: ErrorAnalytics,
    /// Active debugging sessions
    debug_sessions: HashMap<Uuid, InteractiveDebugSession>,
    /// Configuration
    config: DiagnosticsConfig,
    /// Suggestion templates
    suggestion_templates: HashMap<String, Vec<ErrorSuggestion>>,
}

/// Configuration for diagnostics manager
#[derive(Debug, Clone)]
pub struct DiagnosticsConfig {
    /// Maximum diagnostics to keep
    pub max_diagnostics: usize,
    /// Enable analytics collection
    pub enable_analytics: bool,
    /// Enable automatic documentation linking
    pub auto_documentation_links: bool,
    /// Default suggestion priority threshold
    pub suggestion_priority_threshold: f64,
}

impl fmt::Display for EffortLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EffortLevel::Quick => write!(f, "Quick"),
            EffortLevel::Low => write!(f, "Low"),
            EffortLevel::Medium => write!(f, "Medium"),
            EffortLevel::High => write!(f, "High"),
            EffortLevel::Major => write!(f, "Major"),
        }
    }
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::VeryLow => write!(f, "Very Low"),
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::High => write!(f, "High"),
            RiskLevel::Critical => write!(f, "Critical"),
        }
    }
}

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self {
            max_diagnostics: 1000,
            enable_analytics: true,
            auto_documentation_links: true,
            suggestion_priority_threshold: 0.5,
        }
    }
}

impl Default for DebugSessionConfig {
    fn default() -> Self {
        Self {
            max_error_history: 50,
            auto_suggestions: true,
            include_system_state: true,
            verbosity: VerbosityLevel::Detailed,
        }
    }
}

impl ErrorDiagnosticsManager {
    /// Create new diagnostics manager
    pub fn new(config: DiagnosticsConfig) -> Self {
        let mut manager = Self {
            diagnostics: VecDeque::new(),
            analytics: ErrorAnalytics {
                frequency_by_category: HashMap::new(),
                error_trends: Vec::new(),
                common_patterns: Vec::new(),
                resolution_rates: HashMap::new(),
                top_error_sources: Vec::new(),
            },
            debug_sessions: HashMap::new(),
            config,
            suggestion_templates: HashMap::new(),
        };

        manager.load_suggestion_templates();
        manager
    }

    /// Create enhanced diagnostic from BingoError
    pub fn create_diagnostic(&mut self, error: BingoError) -> ErrorDiagnostic {
        let diagnostic_id = Uuid::new_v4();
        let category = error.category().to_string();
        let severity = error.severity();
        let context = error.context();

        let suggestions = self.generate_suggestions(&error);
        let documentation_links = self.generate_documentation_links(&error);

        let diagnostic = ErrorDiagnostic {
            diagnostic_id,
            error: error.to_string(),
            category: category.clone(),
            severity,
            timestamp: Utc::now(),
            context,
            suggestions,
            documentation_links,
            location: None, // Would be populated by call site
            stack_trace: None,
            similar_errors: self.find_similar_errors(&error),
            user_context: None,
            system_state: None,
        };

        // Update analytics
        if self.config.enable_analytics {
            self.update_analytics(&diagnostic);
        }

        // Store diagnostic
        self.diagnostics.push_back(diagnostic.clone());
        if self.diagnostics.len() > self.config.max_diagnostics {
            self.diagnostics.pop_front();
        }

        diagnostic
    }

    /// Generate contextual suggestions for an error
    fn generate_suggestions(&self, error: &BingoError) -> Vec<ErrorSuggestion> {
        let mut suggestions = Vec::new();

        match error {
            BingoError::Rule { rule_id, rule_name, .. } => {
                suggestions.push(ErrorSuggestion {
                    id: "rule_validation".to_string(),
                    priority: 0.9,
                    title: "Validate Rule Syntax".to_string(),
                    description: "Check rule syntax for common errors and validate against schema"
                        .to_string(),
                    steps: vec![
                        FixStep {
                            step_number: 1,
                            description: "Review rule syntax".to_string(),
                            command: None,
                            expected_outcome: "Identify syntax errors".to_string(),
                        },
                        FixStep {
                            step_number: 2,
                            description: "Validate field names exist".to_string(),
                            command: None,
                            expected_outcome: "Confirm all referenced fields are valid".to_string(),
                        },
                    ],
                    code_examples: vec![CodeExample {
                        language: "json".to_string(),
                        code: r#"{
  "id": 1,
  "name": "example_rule",
  "conditions": [
    {"field": "amount", "operator": "greater_than", "value": 1000}
  ],
  "actions": [
    {"type": "set_field", "field": "category", "value": "high_value"}
  ]
}"#
                        .to_string(),
                        explanation: "Example of correct rule syntax".to_string(),
                    }],
                    effort: EffortLevel::Quick,
                    risk: RiskLevel::VeryLow,
                });

                if let Some(rule_name) = rule_name {
                    suggestions.push(ErrorSuggestion {
                        id: "rule_debugger".to_string(),
                        priority: 0.8,
                        title: "Use Rule Debugger".to_string(),
                        description: format!("Debug rule '{rule_name}' step by step"),
                        steps: vec![FixStep {
                            step_number: 1,
                            description: "Start debugging session".to_string(),
                            command: Some(format!("debug_rule {}", rule_id.unwrap_or(0))),
                            expected_outcome: "Interactive debugging session started".to_string(),
                        }],
                        code_examples: Vec::new(),
                        effort: EffortLevel::Low,
                        risk: RiskLevel::VeryLow,
                    });
                }
            }

            BingoError::Condition { field, operator, .. } => {
                suggestions.push(ErrorSuggestion {
                    id: "condition_fix".to_string(),
                    priority: 0.9,
                    title: "Fix Condition Syntax".to_string(),
                    description: "Correct the rule condition syntax".to_string(),
                    steps: vec![
                        FixStep {
                            step_number: 1,
                            description: format!(
                                "Check field '{}' exists",
                                field.as_ref().unwrap_or(&"unknown".to_string())
                            ),
                            command: None,
                            expected_outcome: "Field validation passes".to_string(),
                        },
                        FixStep {
                            step_number: 2,
                            description: format!(
                                "Verify operator '{}' is valid",
                                operator.as_ref().unwrap_or(&"unknown".to_string())
                            ),
                            command: None,
                            expected_outcome: "Operator is supported".to_string(),
                        },
                    ],
                    code_examples: vec![CodeExample {
                        language: "json".to_string(),
                        code: r#"// Supported operators:
"equals", "not_equals", "greater_than", "less_than", 
"greater_than_or_equal", "less_than_or_equal", 
"contains", "starts_with", "ends_with", "regex""#
                            .to_string(),
                        explanation: "List of supported condition operators".to_string(),
                    }],
                    effort: EffortLevel::Quick,
                    risk: RiskLevel::VeryLow,
                });
            }

            BingoError::FactStore { .. } => {
                suggestions.push(ErrorSuggestion {
                    id: "fact_store_recovery".to_string(),
                    priority: 0.8,
                    title: "Recover Fact Store".to_string(),
                    description: "Attempt to recover from fact store corruption".to_string(),
                    steps: vec![
                        FixStep {
                            step_number: 1,
                            description: "Check fact store integrity".to_string(),
                            command: Some("check_fact_store_integrity".to_string()),
                            expected_outcome: "Integrity status reported".to_string(),
                        },
                        FixStep {
                            step_number: 2,
                            description: "Backup current state".to_string(),
                            command: Some("backup_fact_store".to_string()),
                            expected_outcome: "Backup created successfully".to_string(),
                        },
                        FixStep {
                            step_number: 3,
                            description: "Attempt repair".to_string(),
                            command: Some("repair_fact_store".to_string()),
                            expected_outcome: "Fact store repaired".to_string(),
                        },
                    ],
                    code_examples: Vec::new(),
                    effort: EffortLevel::Medium,
                    risk: RiskLevel::Medium,
                });
            }

            BingoError::Performance { operation, duration_ms, limit_ms, .. } => {
                if let (Some(duration), Some(limit)) = (duration_ms, limit_ms) {
                    suggestions.push(ErrorSuggestion {
                        id: "performance_optimization".to_string(),
                        priority: 0.7,
                        title: "Optimize Performance".to_string(),
                        description: format!(
                            "Operation '{}' took {}ms, exceeding limit of {}ms",
                            operation.as_ref().unwrap_or(&"unknown".to_string()),
                            duration,
                            limit
                        ),
                        steps: vec![
                            FixStep {
                                step_number: 1,
                                description: "Profile the operation".to_string(),
                                command: Some("profile_operation".to_string()),
                                expected_outcome: "Performance profile generated".to_string(),
                            },
                            FixStep {
                                step_number: 2,
                                description: "Identify bottlenecks".to_string(),
                                command: None,
                                expected_outcome: "Bottlenecks identified".to_string(),
                            },
                            FixStep {
                                step_number: 3,
                                description: "Apply optimizations".to_string(),
                                command: None,
                                expected_outcome: "Performance improved".to_string(),
                            },
                        ],
                        code_examples: Vec::new(),
                        effort: EffortLevel::High,
                        risk: RiskLevel::Medium,
                    });
                }
            }

            _ => {
                // Generic suggestions for other error types
                suggestions.push(ErrorSuggestion {
                    id: "generic_debug".to_string(),
                    priority: 0.5,
                    title: "Enable Debug Logging".to_string(),
                    description: "Enable detailed logging to get more information".to_string(),
                    steps: vec![FixStep {
                        step_number: 1,
                        description: "Set log level to debug".to_string(),
                        command: Some("set_log_level debug".to_string()),
                        expected_outcome: "Debug logging enabled".to_string(),
                    }],
                    code_examples: Vec::new(),
                    effort: EffortLevel::Quick,
                    risk: RiskLevel::VeryLow,
                });
            }
        }

        suggestions
    }

    /// Generate relevant documentation links
    fn generate_documentation_links(&self, error: &BingoError) -> Vec<DocumentationLink> {
        let mut links = Vec::new();

        match error {
            BingoError::Rule { .. } => {
                links.push(DocumentationLink {
                    title: "Rule Definition Guide".to_string(),
                    url: "https://docs.bingo-engine.com/rules/definition".to_string(),
                    description: "Complete guide to defining rules".to_string(),
                    relevance: 0.9,
                });
                links.push(DocumentationLink {
                    title: "Rule Troubleshooting".to_string(),
                    url: "https://docs.bingo-engine.com/troubleshooting/rules".to_string(),
                    description: "Common rule issues and solutions".to_string(),
                    relevance: 0.8,
                });
            }

            BingoError::Condition { .. } => {
                links.push(DocumentationLink {
                    title: "Condition Syntax Reference".to_string(),
                    url: "https://docs.bingo-engine.com/conditions/syntax".to_string(),
                    description: "Complete reference for condition syntax".to_string(),
                    relevance: 0.9,
                });
            }

            BingoError::Performance { .. } => {
                links.push(DocumentationLink {
                    title: "Performance Optimization Guide".to_string(),
                    url: "https://docs.bingo-engine.com/performance/optimization".to_string(),
                    description: "Guide to optimizing rule engine performance".to_string(),
                    relevance: 0.9,
                });
            }

            _ => {
                links.push(DocumentationLink {
                    title: "General Troubleshooting".to_string(),
                    url: "https://docs.bingo-engine.com/troubleshooting/general".to_string(),
                    description: "General troubleshooting guide".to_string(),
                    relevance: 0.6,
                });
            }
        }

        links
    }

    /// Find similar errors that occurred recently
    fn find_similar_errors(&self, error: &BingoError) -> Vec<SimilarError> {
        let current_category = error.category();
        let current_message = error.to_string();

        self.diagnostics
            .iter()
            .filter(|diag| diag.category == current_category)
            .filter_map(|diag| {
                let similarity = calculate_message_similarity(&current_message, &diag.error);
                if similarity > 0.7 {
                    Some(SimilarError {
                        occurred_at: diag.timestamp,
                        message: diag.error.clone(),
                        resolution: None, // Would track successful resolutions
                        similarity,
                    })
                } else {
                    None
                }
            })
            .take(5)
            .collect()
    }

    /// Update error analytics
    fn update_analytics(&mut self, diagnostic: &ErrorDiagnostic) {
        // Update frequency by category
        *self
            .analytics
            .frequency_by_category
            .entry(diagnostic.category.clone())
            .or_insert(0) += 1;

        // Update error trends
        let now = Utc::now();
        if let Some(last_trend) = self.analytics.error_trends.last_mut() {
            if now.signed_duration_since(last_trend.timestamp).num_minutes() < 5 {
                last_trend.error_count += 1;
                *last_trend.category_breakdown.entry(diagnostic.category.clone()).or_insert(0) += 1;
            } else {
                self.analytics.error_trends.push(ErrorTrendPoint {
                    timestamp: now,
                    error_count: 1,
                    category_breakdown: {
                        let mut breakdown = HashMap::new();
                        breakdown.insert(diagnostic.category.clone(), 1);
                        breakdown
                    },
                });
            }
        } else {
            self.analytics.error_trends.push(ErrorTrendPoint {
                timestamp: now,
                error_count: 1,
                category_breakdown: {
                    let mut breakdown = HashMap::new();
                    breakdown.insert(diagnostic.category.clone(), 1);
                    breakdown
                },
            });
        }

        // Keep only recent trend points (last 24 hours)
        let cutoff = now - chrono::Duration::hours(24);
        self.analytics.error_trends.retain(|trend| trend.timestamp > cutoff);
    }

    /// Load suggestion templates
    fn load_suggestion_templates(&mut self) {
        // This would typically load from configuration or database
        // For now, we'll populate with some basic templates
        self.suggestion_templates.insert("rule".to_string(), Vec::new());
        self.suggestion_templates.insert("condition".to_string(), Vec::new());
        self.suggestion_templates.insert("fact_store".to_string(), Vec::new());
    }

    /// Start interactive debugging session
    pub fn start_debug_session(&mut self, config: DebugSessionConfig) -> Uuid {
        let session_id = Uuid::new_v4();
        let session = InteractiveDebugSession {
            session_id,
            current_error: None,
            error_history: VecDeque::new(),
            active_suggestions: Vec::new(),
            suggestion_feedback: HashMap::new(),
            config,
        };

        self.debug_sessions.insert(session_id, session);
        session_id
    }

    /// Add error to debugging session
    pub fn add_error_to_session(&mut self, session_id: Uuid, diagnostic: ErrorDiagnostic) {
        if let Some(session) = self.debug_sessions.get_mut(&session_id) {
            session.current_error = Some(diagnostic.clone());
            session.error_history.push_back(diagnostic.clone());
            session.active_suggestions = diagnostic.suggestions;

            if session.error_history.len() > session.config.max_error_history {
                session.error_history.pop_front();
            }
        }
    }

    /// Get debugging session
    pub fn get_debug_session(&self, session_id: Uuid) -> Option<&InteractiveDebugSession> {
        self.debug_sessions.get(&session_id)
    }

    /// Get error analytics
    pub fn get_analytics(&self) -> &ErrorAnalytics {
        &self.analytics
    }

    /// Generate error report
    pub fn generate_error_report(&self, time_range: Option<chrono::Duration>) -> ErrorReport {
        let cutoff = match time_range {
            Some(duration) => Utc::now() - duration,
            None => Utc::now() - chrono::Duration::hours(24),
        };

        let relevant_diagnostics: Vec<_> =
            self.diagnostics.iter().filter(|diag| diag.timestamp > cutoff).collect();

        let total_errors = relevant_diagnostics.len();
        let mut errors_by_category = HashMap::new();
        let mut errors_by_severity = HashMap::new();

        for diagnostic in &relevant_diagnostics {
            *errors_by_category.entry(diagnostic.category.clone()).or_insert(0) += 1;
            *errors_by_severity.entry(format!("{:?}", diagnostic.severity)).or_insert(0) += 1;
        }

        ErrorReport {
            time_range: time_range.unwrap_or(chrono::Duration::hours(24)),
            total_errors,
            errors_by_category,
            errors_by_severity,
            most_common_errors: self.get_most_common_errors(&relevant_diagnostics),
            resolution_suggestions: self.get_top_resolution_suggestions(&relevant_diagnostics),
        }
    }

    /// Get most common errors
    fn get_most_common_errors(&self, diagnostics: &[&ErrorDiagnostic]) -> Vec<(String, usize)> {
        let mut error_counts = HashMap::new();
        for diagnostic in diagnostics {
            *error_counts.entry(diagnostic.error.clone()).or_insert(0) += 1;
        }

        let mut sorted_errors: Vec<_> = error_counts.into_iter().collect();
        sorted_errors.sort_by(|a, b| b.1.cmp(&a.1));
        sorted_errors.into_iter().take(10).collect()
    }

    /// Get top resolution suggestions
    fn get_top_resolution_suggestions(
        &self,
        diagnostics: &[&ErrorDiagnostic],
    ) -> Vec<ErrorSuggestion> {
        let mut all_suggestions = Vec::new();
        for diagnostic in diagnostics {
            all_suggestions.extend(diagnostic.suggestions.clone());
        }

        all_suggestions.sort_by(|a, b| {
            b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal)
        });
        all_suggestions.into_iter().take(5).collect()
    }
}

/// Error report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    /// Time range covered by report
    pub time_range: chrono::Duration,
    /// Total number of errors
    pub total_errors: usize,
    /// Errors grouped by category
    pub errors_by_category: HashMap<String, usize>,
    /// Errors grouped by severity
    pub errors_by_severity: HashMap<String, usize>,
    /// Most frequently occurring errors
    pub most_common_errors: Vec<(String, usize)>,
    /// Top resolution suggestions
    pub resolution_suggestions: Vec<ErrorSuggestion>,
}

/// Calculate similarity between two error messages
fn calculate_message_similarity(msg1: &str, msg2: &str) -> f64 {
    // Simple similarity calculation based on common words
    let words1: std::collections::HashSet<&str> = msg1.split_whitespace().collect();
    let words2: std::collections::HashSet<&str> = msg2.split_whitespace().collect();

    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Extension trait to convert BingoError to diagnostic
pub trait ErrorToDiagnostic {
    fn to_diagnostic(self, manager: &mut ErrorDiagnosticsManager) -> ErrorDiagnostic;
}

impl ErrorToDiagnostic for BingoError {
    fn to_diagnostic(self, manager: &mut ErrorDiagnosticsManager) -> ErrorDiagnostic {
        manager.create_diagnostic(self)
    }
}

/// Extension trait for BingoResult to automatically create diagnostics
pub trait ResultDiagnosticExt<T> {
    fn with_diagnostics(
        self,
        manager: &mut ErrorDiagnosticsManager,
    ) -> Result<T, Box<ErrorDiagnostic>>;
}

impl<T> ResultDiagnosticExt<T> for BingoResult<T> {
    fn with_diagnostics(
        self,
        manager: &mut ErrorDiagnosticsManager,
    ) -> Result<T, Box<ErrorDiagnostic>> {
        self.map_err(|err| Box::new(manager.create_diagnostic(err)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_diagnostics_creation() {
        let mut manager = ErrorDiagnosticsManager::new(DiagnosticsConfig::default());
        let error = BingoError::rule_validation("Test rule error");
        let diagnostic = manager.create_diagnostic(error);

        assert!(!diagnostic.suggestions.is_empty());
        assert_eq!(diagnostic.category, "rule");
    }

    #[test]
    fn test_message_similarity() {
        let msg1 = "Rule compilation failed due to invalid syntax";
        let msg2 = "Rule compilation failed due to missing field";
        let similarity = calculate_message_similarity(msg1, msg2);

        assert!(similarity > 0.5);
    }
}
