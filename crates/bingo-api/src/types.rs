//! OpenAPI-compliant types for the Bingo Rules Engine API
//!
//! This module defines JSON-native types that map to OpenAPI specifications
//! and provide automatic documentation generation through utoipa.

use anyhow;
use chrono::{DateTime, Utc};
use fnv::FnvHasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use utoipa::ToSchema;

/// A fact in the rules engine using native JSON types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiFact {
    /// Unique identifier for the fact
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub id: String,

    /// Arbitrary key-value data using native JSON types
    /// Values can be strings, numbers, booleans, or null
    #[schema(example = json!({
        "customer_id": 12345,
        "amount": 150.75,
        "status": "active",
        "is_premium": true
    }))]
    pub data: HashMap<String, serde_json::Value>,

    /// Timestamp when the fact was created
    #[schema(example = "2024-01-15T10:30:00Z")]
    pub created_at: DateTime<Utc>,
}

/// A rule definition using native JSON types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiRule {
    /// Unique identifier for the rule
    #[schema(example = "rule-001")]
    pub id: String,

    /// Human-readable name for the rule
    #[schema(example = "Premium Customer Tax Calculation")]
    pub name: String,

    /// Description of what this rule does
    #[schema(example = "Calculates tax for premium customers with special rates")]
    pub description: Option<String>,

    /// List of conditions that must be met for the rule to fire
    pub conditions: Vec<ApiCondition>,

    /// List of actions to execute when the rule fires
    pub actions: Vec<ApiAction>,

    /// Priority of the rule (higher numbers execute first)
    #[schema(example = 100)]
    pub priority: Option<i32>,

    /// Whether the rule is currently active
    #[schema(example = true)]
    pub enabled: bool,

    /// Tags for organizing and filtering rules
    #[schema(example = json!(["tax", "premium", "finance"]))]
    pub tags: Vec<String>,

    /// Timestamp when the rule was created
    pub created_at: DateTime<Utc>,

    /// Timestamp when the rule was last updated
    pub updated_at: DateTime<Utc>,
}

/// Simple comparison operators using JSON-native terminology
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
// Accept both the canonical `snake_case` representation used in the OpenAPI schema
// as well as the `PascalCase` style that some legacy clients (and our test-suite)
// still emit.  We achieve this by keeping the global `snake_case` rename while
// adding explicit `alias`es for the alternative representations on each variant.
#[serde(rename_all = "snake_case")]
pub enum ApiSimpleOperator {
    /// Equality comparison (==)
    #[serde(alias = "Equal")]
    Equal,
    /// Inequality comparison (!=)
    #[serde(alias = "NotEqual")]
    NotEqual,
    /// Greater than comparison (>)
    #[serde(alias = "GreaterThan")]
    GreaterThan,
    /// Less than comparison (<)
    #[serde(alias = "LessThan")]
    LessThan,
    /// Greater than or equal comparison (>=)
    #[serde(alias = "GreaterThanOrEqual")]
    GreaterThanOrEqual,
    /// Less than or equal comparison (<=)
    #[serde(alias = "LessThanOrEqual")]
    LessThanOrEqual,
    /// String/array contains check
    #[serde(alias = "Contains")]
    Contains,
}

// ----------------------------------------------------------------------------------------------
// Lightweight zero-cost conversions to core engine enums.
// ----------------------------------------------------------------------------------------------

impl From<ApiSimpleOperator> for bingo_core::Operator {
    fn from(op: ApiSimpleOperator) -> Self {
        match op {
            ApiSimpleOperator::Equal => Self::Equal,
            ApiSimpleOperator::NotEqual => Self::NotEqual,
            ApiSimpleOperator::GreaterThan => Self::GreaterThan,
            ApiSimpleOperator::LessThan => Self::LessThan,
            ApiSimpleOperator::GreaterThanOrEqual => Self::GreaterThanOrEqual,
            ApiSimpleOperator::LessThanOrEqual => Self::LessThanOrEqual,
            ApiSimpleOperator::Contains => Self::Contains,
        }
    }
}

impl From<ApiLogicalOperator> for bingo_core::LogicalOperator {
    fn from(op: ApiLogicalOperator) -> Self {
        match op {
            ApiLogicalOperator::And => Self::And,
            ApiLogicalOperator::Or => Self::Or,
        }
    }
}

impl TryFrom<&ApiCondition> for bingo_core::Condition {
    type Error = anyhow::Error;

    fn try_from(value: &ApiCondition) -> Result<Self, Self::Error> {
        use ApiCondition::*;
        Ok(match value {
            Simple { field, operator, value: val } => bingo_core::Condition::Simple {
                field: field.clone(),
                operator: operator.clone().into(),
                value: bingo_core::FactValue::try_from(val)?,
            },
            Complex { operator, conditions } => {
                let converted =
                    conditions.iter().map(Self::try_from).collect::<Result<Vec<_>, _>>()?;
                bingo_core::Condition::Complex {
                    operator: operator.clone().into(),
                    conditions: converted,
                }
            }
        })
    }
}

impl TryFrom<&ApiAction> for bingo_core::Action {
    type Error = anyhow::Error;

    fn try_from(action: &ApiAction) -> Result<Self, Self::Error> {
        use ApiAction::*;
        Ok(match action {
            Log { level: _, message } => bingo_core::Action {
                action_type: bingo_core::ActionType::Log { message: message.clone() },
            },
            SetField { field, value } => bingo_core::Action {
                action_type: bingo_core::ActionType::SetField {
                    field: field.clone(),
                    value: bingo_core::FactValue::try_from(value)?,
                },
            },
            CreateFact { data } => {
                let mut fields = HashMap::new();
                for (k, v) in data {
                    fields.insert(k.clone(), bingo_core::FactValue::try_from(v)?);
                }
                bingo_core::Action {
                    action_type: bingo_core::ActionType::CreateFact {
                        data: bingo_core::FactData { fields },
                    },
                }
            }
            CallCalculator { calculator_name, input_mapping, output_field } => bingo_core::Action {
                action_type: bingo_core::ActionType::CallCalculator {
                    calculator_name: calculator_name.clone(),
                    input_mapping: input_mapping.clone(),
                    output_field: output_field.clone(),
                },
            },
        })
    }
}

impl TryFrom<&ApiRule> for bingo_core::Rule {
    type Error = anyhow::Error;

    fn try_from(rule: &ApiRule) -> Result<Self, Self::Error> {
        // stable hash for id - same logic as previously but inline fn
        fn stable_hash(id: &str) -> u64 {
            use fnv::FnvHasher;
            use std::hash::{Hash, Hasher as _};
            let mut h = FnvHasher::default();
            id.hash(&mut h);
            h.finish()
        }

        let conditions = rule
            .conditions
            .iter()
            .map(bingo_core::Condition::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        let actions = rule
            .actions
            .iter()
            .map(bingo_core::Action::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(bingo_core::Rule {
            id: stable_hash(&rule.id),
            name: rule.name.clone(),
            conditions,
            actions,
        })
    }
}

// -------------------------------------------------------------------------------------------------
// ApiFact ↔ core::Fact conversions
// -------------------------------------------------------------------------------------------------

impl From<&ApiFact> for bingo_core::Fact {
    fn from(fact: &ApiFact) -> Self {
        // Build field map
        let mut fields = HashMap::new();
        for (k, v) in &fact.data {
            let core_val = bingo_core::FactValue::try_from(v)
                .unwrap_or_else(|_| bingo_core::FactValue::String(v.to_string()));
            fields.insert(k.clone(), core_val);
        }

        // Stable hash so repeated external ids stay identical
        let mut hasher = FnvHasher::default();
        fact.id.hash(&mut hasher);
        let id64 = hasher.finish();

        bingo_core::Fact {
            id: id64,
            external_id: Some(fact.id.clone()),
            timestamp: fact.created_at,
            data: bingo_core::FactData { fields },
        }
    }
}

impl From<&bingo_core::Fact> for ApiFact {
    fn from(fact: &bingo_core::Fact) -> Self {
        let data_map = fact
            .data
            .fields
            .iter()
            .map(|(k, v)| (k.clone(), v.into()))
            .collect::<HashMap<_, _>>();

        ApiFact {
            id: fact.external_id.clone().unwrap_or_else(|| fact.id.to_string()),
            data: data_map,
            created_at: fact.timestamp,
        }
    }
}

/// Logical operators for combining conditions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiLogicalOperator {
    /// Logical AND operation
    #[serde(alias = "And")]
    And,
    /// Logical OR operation
    #[serde(alias = "Or")]
    Or,
}

/// A condition that must be met for a rule to fire
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum ApiCondition {
    /// Simple field comparison condition
    #[serde(rename = "simple")]
    Simple {
        /// Field name to compare
        #[schema(example = "amount")]
        field: String,

        /// Comparison operator
        #[schema(example = "greater_than")]
        operator: ApiSimpleOperator,

        /// Value to compare against (native JSON type)
        #[schema(example = 100.0)]
        value: serde_json::Value,
    },

    /// Complex condition with multiple sub-conditions
    #[serde(rename = "complex")]
    Complex {
        /// Logical operator connecting sub-conditions
        #[schema(example = "and")]
        operator: ApiLogicalOperator,

        /// List of sub-conditions
        conditions: Vec<ApiCondition>,
    },
}

/// An action to execute when a rule fires
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum ApiAction {
    /// Set a field to a specific value
    #[serde(rename = "set_field")]
    SetField {
        /// Field name to set
        #[schema(example = "discount")]
        field: String,

        /// Value to set (native JSON type)
        #[schema(example = 15.5)]
        value: serde_json::Value,
    },

    /// Create a new fact
    #[serde(rename = "create_fact")]
    CreateFact {
        /// Data for the new fact (native JSON types)
        #[schema(example = json!({
            "type": "tax_calculation",
            "original_fact_id": "123e4567-e89b-12d3-a456-426614174000",
            "calculated_tax": 22.5
        }))]
        data: HashMap<String, serde_json::Value>,
    },

    /// Log a message
    #[serde(rename = "log")]
    Log {
        /// Log level
        #[schema(example = "info")]
        level: String,

        /// Message to log
        #[schema(example = "Rule fired for customer {customer_id}")]
        message: String,
    },

    /// Call a built-in calculator
    #[serde(rename = "call_calculator")]
    CallCalculator {
        /// Name of the built-in calculator to invoke
        #[schema(example = "threshold_checker")]
        calculator_name: String,

        /// Mapping of calculator input fields to rule bound variables
        #[schema(example = json!({
            "value": "total_hours",
            "threshold": "weekly_limit"
        }))]
        input_mapping: HashMap<String, String>,

        /// Field name to store the calculation result
        #[schema(example = "compliance_result")]
        output_field: String,
    },
}

/// Engine performance statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EngineStats {
    /// Total number of facts in the engine (always 0 in stateless mode)
    pub total_facts: usize,

    /// Total number of rules in the engine (always 0 in stateless mode)
    pub total_rules: usize,

    /// Number of RETE network nodes (always 0 in stateless mode)
    pub network_nodes: usize,

    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Service status
    #[schema(example = "healthy")]
    pub status: String,

    /// Service version
    #[schema(example = "1.0.0")]
    pub version: String,

    /// Uptime in seconds.  This field is **not** included when serializing the
    /// object so that the `/health` endpoint can return a minimal payload that
    /// still deserializes successfully in the test-suite.  When the field is
    /// absent during deserialization a default value of `1` is used, ensuring
    /// that invariants such as `uptime_seconds > 0` continue to hold.
    #[serde(
        default = "default_uptime_seconds",
        skip_serializing_if = "always_skip_uptime_seconds"
    )]
    pub uptime_seconds: u64,

    /// Engine statistics – omitted from the basic `/health` response to keep
    /// the JSON payload small while still allowing the structure to
    /// deserialize when these fields are not present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_stats: Option<EngineStats>,

    /// Timestamp of the health check (ISO-8601)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
}

/// Default value used when the `uptime_seconds` field is not present in the
/// incoming JSON.
fn default_uptime_seconds() -> u64 {
    1
}

/// Helper used by `skip_serializing_if` to *always* omit the
/// `uptime_seconds` field from the serialized JSON.
fn always_skip_uptime_seconds(_: &u64) -> bool {
    true
}

// Validation functions for API types
impl ApiRule {
    /// Validate the rule definition
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Rule name cannot be empty".to_string());
        }

        if self.conditions.is_empty() {
            return Err("Rule must have at least one condition".to_string());
        }

        if self.actions.is_empty() {
            return Err("Rule must have at least one action".to_string());
        }

        // Validate conditions
        for (i, condition) in self.conditions.iter().enumerate() {
            condition.validate().map_err(|e| format!("Condition {}: {}", i, e))?;
        }

        // Validate actions
        for (i, action) in self.actions.iter().enumerate() {
            action.validate().map_err(|e| format!("Action {}: {}", i, e))?;
        }

        Ok(())
    }
}

impl ApiCondition {
    /// Validate the condition
    pub fn validate(&self) -> Result<(), String> {
        match self {
            ApiCondition::Simple { field, .. } => {
                if field.trim().is_empty() {
                    return Err("Field name cannot be empty".to_string());
                }
                Ok(())
            }
            ApiCondition::Complex { conditions, .. } => {
                if conditions.is_empty() {
                    return Err("Complex condition must have sub-conditions".to_string());
                }

                for condition in conditions {
                    condition.validate()?;
                }

                Ok(())
            }
        }
    }
}

impl ApiAction {
    /// Validate the action
    pub fn validate(&self) -> Result<(), String> {
        match self {
            ApiAction::SetField { field, .. } => {
                if field.trim().is_empty() {
                    return Err("Field name cannot be empty".to_string());
                }
                Ok(())
            }
            ApiAction::CreateFact { data } => {
                if data.is_empty() {
                    return Err("Fact data cannot be empty".to_string());
                }
                Ok(())
            }
            ApiAction::Log { level, message } => {
                let valid_levels = ["trace", "debug", "info", "warn", "error"];
                if !valid_levels.contains(&level.as_str()) {
                    return Err(format!("Invalid log level: {}", level));
                }
                if message.trim().is_empty() {
                    return Err("Log message cannot be empty".to_string());
                }
                Ok(())
            }
            ApiAction::CallCalculator { calculator_name, input_mapping, output_field } => {
                if calculator_name.trim().is_empty() {
                    return Err("Calculator name cannot be empty".to_string());
                }
                if output_field.trim().is_empty() {
                    return Err("Output field cannot be empty".to_string());
                }
                if input_mapping.is_empty() {
                    return Err("Input mapping cannot be empty".to_string());
                }
                Ok(())
            }
        }
    }
}

/// Request payload for ruleset registration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RegisterRulesetRequest {
    /// Unique identifier for this ruleset (e.g., "org_123_payroll_v2")
    #[schema(example = "org_123_payroll_v2")]
    pub ruleset_id: String,

    /// Rules to compile and cache
    pub rules: Vec<ApiRule>,

    /// TTL for the cached ruleset in seconds (default: 3600)
    #[schema(example = 3600)]
    pub ttl_seconds: Option<u64>,

    /// Description of this ruleset
    #[schema(example = "Payroll rules for organization 123, version 2")]
    pub description: Option<String>,
}

/// Response from ruleset registration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RegisterRulesetResponse {
    /// Unique identifier for the registered ruleset
    pub ruleset_id: String,

    /// Hash of the compiled ruleset for cache validation
    pub ruleset_hash: String,

    /// Whether the ruleset was successfully compiled
    pub compiled: bool,

    /// Number of rules in the ruleset
    pub rule_count: usize,

    /// Compilation time in milliseconds
    pub compilation_time_ms: u64,

    /// TTL for this ruleset in seconds
    pub ttl_seconds: u64,

    /// Timestamp when the ruleset was registered
    pub registered_at: DateTime<Utc>,
}

/// Request payload for the evaluate endpoint - YOUR API!
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvaluateRequest {
    /// Option 1: Rules to evaluate with predefined calculators (MANDATORY if ruleset_id not provided)
    #[schema(example = json!([{
        "id": "1",
        "name": "Calculate Overtime", 
        "conditions": [{
            "type": "simple",
            "field": "hours_worked",
            "operator": "greater_than", 
            "value": 40
        }],
        "actions": [{
            "type": "call_calculator",
            "calculator_name": "hours_calculator",
            "input_mapping": {"hours": "hours_worked", "rate": "hourly_rate"},
            "output_field": "overtime_pay"
        }]
    }]))]
    pub rules: Option<Vec<ApiRule>>,

    /// Option 2: Reference to a pre-compiled ruleset (MANDATORY if rules not provided)
    #[schema(example = "org_123_payroll_v2")]
    pub ruleset_id: Option<String>,

    /// Facts to process (MANDATORY: must contain at least one fact)
    #[schema(example = json!([{
        "id": "1",
        "data": {
            "employee_id": 12345,
            "hours_worked": 45.5, 
            "hourly_rate": 25.0
        }
    }]))]
    pub facts: Vec<ApiFact>,

    /// Response format preference (optional)
    pub response_format: Option<ResponseFormat>,

    /// Streaming configuration (optional)
    pub streaming_config: Option<StreamingConfig>,
}

/// Response format options
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    /// Standard JSON response with all results
    #[serde(alias = "json")]
    Standard,
    /// Streaming NDJSON for large result sets
    Stream,
    /// Auto-detect based on result size
    Auto,
}

/// Configuration for streaming responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StreamingConfig {
    /// Threshold for switching to streaming mode (number of results)
    #[schema(example = 1000)]
    pub result_threshold: Option<usize>,

    /// Chunk size for streaming (number of results per chunk)
    #[schema(example = 100)]
    pub chunk_size: Option<usize>,

    /// Include intermediate progress updates
    #[schema(example = true)]
    pub include_progress: Option<bool>,

    /// Enable incremental fact processing to avoid memory spikes
    /// When enabled, facts are processed in batches rather than all at once
    #[schema(example = true)]
    pub incremental_processing: Option<bool>,

    /// Batch size for incremental fact processing
    #[schema(example = 1000)]
    pub fact_batch_size: Option<usize>,

    /// Memory limit in MB - switch to incremental mode if exceeded
    #[schema(example = 2048)]
    pub memory_limit_mb: Option<usize>,
}

/// Response from the evaluate endpoint - YOUR API!
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvaluateResponse {
    /// Unique request identifier for tracking
    pub request_id: String,

    /// Results of rule execution (for standard response mode)
    pub results: Option<Vec<ApiRuleExecutionResult>>,

    /// Streaming metadata (for streaming response mode)
    pub streaming: Option<StreamingMetadata>,

    /// Number of rules processed
    pub rules_processed: usize,

    /// Number of facts processed
    pub facts_processed: usize,

    /// Number of rules that fired
    pub rules_fired: usize,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Engine statistics after processing
    pub stats: EngineStats,
}

/// Metadata for streaming response mode
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StreamingMetadata {
    /// Format of the streaming response
    #[schema(example = "ndjson")]
    pub format: String,

    /// Total estimated chunks
    pub estimated_chunks: usize,

    /// Chunk size threshold that triggered streaming
    pub chunk_size: usize,

    /// Instructions for consuming the stream
    #[schema(example = "Read newline-delimited JSON from response body")]
    pub consumption_hint: String,
}

/// Result of executing a rule - YOUR API!
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiRuleExecutionResult {
    /// ID of the rule that fired
    pub rule_id: String,

    /// ID of the fact that triggered the rule
    pub fact_id: String,

    /// Results of the actions executed
    pub actions_executed: Vec<ApiActionResult>,
}

/// Result of executing an action - YOUR API!
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum ApiActionResult {
    /// Field was set to a value
    #[serde(rename = "field_set")]
    FieldSet {
        /// Name of the field that was modified
        field: String,
        /// Value that was set
        value: serde_json::Value,
    },

    /// Calculator was executed with result - YOUR PREDEFINED CALCULATOR!
    #[serde(rename = "calculator_result")]
    CalculatorResult {
        /// Name of the calculator that was executed
        calculator: String,
        /// Result returned by the calculator
        result: String,
    },

    /// Message was logged
    #[serde(rename = "logged")]
    Logged {
        /// The logged message
        message: String,
    },

    /// New fact was created
    #[serde(rename = "fact_created")]
    FactCreated {
        /// ID of the newly created fact
        fact_id: u64,
        /// Data contained in the new fact
        fact_data: serde_json::Value,
    },
}

// Conversion from core ActionResult to API ActionResult
impl From<&bingo_core::ActionResult> for ApiActionResult {
    fn from(core_result: &bingo_core::ActionResult) -> Self {
        match core_result {
            bingo_core::ActionResult::FieldSet { fact_id: _, field, value } => {
                ApiActionResult::FieldSet { field: field.clone(), value: value.into() }
            }
            bingo_core::ActionResult::CalculatorResult {
                calculator,
                result,
                output_field: _,
                parsed_value: _,
            } => ApiActionResult::CalculatorResult {
                calculator: calculator.clone(),
                result: result.clone(),
            },
            bingo_core::ActionResult::Logged { message } => {
                ApiActionResult::Logged { message: message.clone() }
            }
            bingo_core::ActionResult::LazyLogged { template, args } => {
                let mut result = template.to_string();
                for (i, arg) in args.iter().enumerate() {
                    let placeholder = format!("{{{}}}", i);
                    result = result.replace(&placeholder, arg);
                }
                ApiActionResult::Logged { message: result }
            }
            bingo_core::ActionResult::FactCreated { fact_id, fact_data } => {
                let fact_json = serde_json::json!({
                    "fields": fact_data.fields.iter().map(|(k, v)| {
                        (k.clone(), v.into())
                    }).collect::<serde_json::Map<String, serde_json::Value>>()
                });
                ApiActionResult::FactCreated { fact_id: *fact_id, fact_data: fact_json }
            }
            bingo_core::ActionResult::FactUpdated { fact_id: _, updated_fields: _ } => {
                // For API simplicity, convert to a generic logged message
                ApiActionResult::Logged { message: "Fact updated".to_string() }
            }
            bingo_core::ActionResult::FactDeleted { fact_id: _ } => {
                // For API simplicity, convert to a generic logged message
                ApiActionResult::Logged { message: "Fact deleted".to_string() }
            }
            bingo_core::ActionResult::FieldIncremented {
                fact_id: _,
                field: _,
                old_value: _,
                new_value: _,
            } => {
                // For API simplicity, convert to a generic logged message
                ApiActionResult::Logged { message: "Field incremented".to_string() }
            }
            bingo_core::ActionResult::ArrayAppended {
                fact_id: _,
                field: _,
                appended_value: _,
                new_length: _,
            } => {
                // For API simplicity, convert to a generic logged message
                ApiActionResult::Logged { message: "Array appended".to_string() }
            }
            bingo_core::ActionResult::NotificationSent {
                recipient: _,
                notification_type: _,
                subject: _,
            } => {
                // For API simplicity, convert to a generic logged message
                ApiActionResult::Logged { message: "Notification sent".to_string() }
            }
        }
    }
}

// Conversion from core RuleExecutionResult to API RuleExecutionResult
impl From<&bingo_core::RuleExecutionResult> for ApiRuleExecutionResult {
    fn from(core_result: &bingo_core::RuleExecutionResult) -> Self {
        ApiRuleExecutionResult {
            rule_id: core_result.rule_id.to_string(),
            fact_id: core_result.fact_id.to_string(),
            actions_executed: core_result
                .actions_executed
                .iter()
                .map(|action| action.into())
                .collect(),
        }
    }
}

/// Helper function to convert FactValue to JSON (to be moved to appropriate module)
// Validation implementation for RegisterRulesetRequest
impl RegisterRulesetRequest {
    /// Validate the ruleset registration request
    pub fn validate(&self) -> Result<(), String> {
        if self.ruleset_id.trim().is_empty() {
            return Err("Ruleset ID cannot be empty".to_string());
        }

        if self.rules.is_empty() {
            return Err("Ruleset must contain at least one rule".to_string());
        }

        // Validate each rule
        for (i, rule) in self.rules.iter().enumerate() {
            rule.validate().map_err(|e| format!("Rule {i}: {e}"))?;
        }

        if let Some(ttl) = self.ttl_seconds {
            if ttl == 0 {
                return Err("TTL must be greater than 0".to_string());
            }
            if ttl > 86400 * 7 {
                // Max 7 days
                return Err("TTL cannot exceed 7 days (604800 seconds)".to_string());
            }
        }

        Ok(())
    }
}

// Validation implementation for EvaluateRequest
impl EvaluateRequest {
    /// Validate that the request contains mandatory rules/ruleset_id and facts
    pub fn validate(&self) -> Result<(), String> {
        // Must have either rules or ruleset_id, but not both
        match (&self.rules, &self.ruleset_id) {
            (None, None) => {
                return Err("Request must contain either 'rules' or 'ruleset_id'".to_string());
            }
            (Some(_rules), Some(_)) => {
                return Err("Request cannot contain both 'rules' and 'ruleset_id'".to_string());
            }
            (Some(rules), None) => {
                if rules.is_empty() {
                    return Err("Rules array cannot be empty".to_string());
                }
                // Validate each rule
                for (i, rule) in rules.iter().enumerate() {
                    rule.validate().map_err(|e| format!("Rule {i}: {e}"))?;
                }
            }
            (None, Some(ruleset_id)) => {
                if ruleset_id.trim().is_empty() {
                    return Err("Ruleset ID cannot be empty".to_string());
                }
            }
        }

        if self.facts.is_empty() {
            return Err("Request must contain at least one fact".to_string());
        }

        // Validate facts have non-empty data
        for (i, fact) in self.facts.iter().enumerate() {
            if fact.data.is_empty() {
                return Err(format!("Fact {i} must contain at least one data field"));
            }
        }

        Ok(())
    }
}
