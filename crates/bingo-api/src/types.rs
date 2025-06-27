//! OpenAPI-compliant types for the Bingo Rules Engine API
//!
//! This module defines JSON-native types that map to OpenAPI specifications
//! and provide automatic documentation generation through utoipa.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
        operator: String,

        /// Value to compare against (native JSON type)
        #[schema(example = 100.0)]
        value: serde_json::Value,
    },

    /// Complex condition with multiple sub-conditions
    #[serde(rename = "complex")]
    Complex {
        /// Logical operator connecting sub-conditions
        #[schema(example = "and")]
        operator: String,

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

    /// Calculate a field using a formula expression
    #[serde(rename = "formula")]
    Formula {
        /// Target field name for the calculated result
        #[schema(example = "tax_amount")]
        field: String,

        /// Mathematical expression using calculator DSL
        #[schema(example = "amount * 0.15")]
        expression: String,
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

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Engine statistics
    pub engine_stats: EngineStats,

    /// Timestamp of the health check
    pub timestamp: DateTime<Utc>,
}

/// Standard API error response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiError {
    /// Error code
    #[schema(example = "VALIDATION_ERROR")]
    pub code: String,

    /// Human-readable error message
    #[schema(example = "Invalid rule condition: field 'amount' not found")]
    pub message: String,

    /// Additional error details
    pub details: Option<serde_json::Value>,

    /// Request ID for tracking
    pub request_id: Option<String>,

    /// Timestamp when the error occurred
    pub timestamp: DateTime<Utc>,
}

impl ApiError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
            request_id: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
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
            ApiCondition::Simple { field, operator, .. } => {
                if field.trim().is_empty() {
                    return Err("Field name cannot be empty".to_string());
                }

                let valid_operators = [
                    "equal",
                    "not_equal",
                    "greater_than",
                    "less_than",
                    "greater_than_or_equal",
                    "less_than_or_equal",
                    "contains",
                ];

                if !valid_operators.contains(&operator.as_str()) {
                    return Err(format!("Invalid operator: {}", operator));
                }

                Ok(())
            }
            ApiCondition::Complex { operator, conditions } => {
                let valid_operators = ["and", "or", "not"];
                if !valid_operators.contains(&operator.as_str()) {
                    return Err(format!("Invalid logical operator: {}", operator));
                }

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
            ApiAction::Formula { field, expression } => {
                if field.trim().is_empty() {
                    return Err("Field name cannot be empty".to_string());
                }
                if expression.trim().is_empty() {
                    return Err("Expression cannot be empty".to_string());
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
}

/// Response from the evaluate endpoint - YOUR API!
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvaluateResponse {
    /// Unique request identifier for tracking
    pub request_id: String,

    /// Results of rule execution
    pub results: Vec<ApiRuleExecutionResult>,

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
    FieldSet { field: String, value: serde_json::Value },

    /// Calculator was executed with result - YOUR PREDEFINED CALCULATOR!
    #[serde(rename = "calculator_result")]
    CalculatorResult { calculator: String, result: String },

    /// Message was logged
    #[serde(rename = "logged")]
    Logged { message: String },
}

// Validation implementation for RegisterRulesetRequest
impl RegisterRulesetRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.ruleset_id.trim().is_empty() {
            return Err("Ruleset ID cannot be empty".to_string());
        }

        if self.rules.is_empty() {
            return Err("Ruleset must contain at least one rule".to_string());
        }

        // Validate each rule
        for (i, rule) in self.rules.iter().enumerate() {
            rule.validate().map_err(|e| format!("Rule {}: {}", i, e))?;
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
                    rule.validate().map_err(|e| format!("Rule {}: {}", i, e))?;
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
                return Err(format!("Fact {} must contain at least one data field", i));
            }
        }

        Ok(())
    }
}
