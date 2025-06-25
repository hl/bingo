//! OpenAPI-compliant types for the Bingo Rules Engine API
//!
//! This module defines JSON-native types that map to OpenAPI specifications
//! and provide automatic documentation generation through utoipa.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::{IntoParams, ToSchema};

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

/// Request to process facts through the rules engine
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProcessFactsRequest {
    /// List of facts to process
    pub facts: Vec<ApiFact>,

    /// Optional rule filter (process only specific rules)
    pub rule_filter: Option<Vec<String>>,

    /// Optional execution mode
    #[schema(example = "parallel")]
    pub execution_mode: Option<String>,
}

/// Response from processing facts
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProcessFactsResponse {
    /// Request ID for tracking
    pub request_id: String,

    /// Facts generated by rule execution
    pub results: Vec<ApiFact>,

    /// Number of input facts processed
    pub facts_processed: usize,

    /// Number of rules evaluated
    pub rules_evaluated: usize,

    /// Number of rules that fired
    pub rules_fired: usize,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Engine statistics
    pub stats: EngineStats,
}

/// Engine performance statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EngineStats {
    /// Total number of facts in the engine
    pub total_facts: usize,

    /// Total number of rules in the engine
    pub total_rules: usize,

    /// Number of RETE network nodes
    pub network_nodes: usize,

    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
}

/// Request to create or update a rule
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateRuleRequest {
    /// Rule definition
    #[serde(flatten)]
    pub rule: ApiRule,
}

/// Response from creating or updating a rule
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateRuleResponse {
    /// The created/updated rule
    pub rule: ApiRule,

    /// Whether this was a new rule (true) or update (false)
    pub created: bool,
}

/// Request to retrieve rules with optional filtering
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ListRulesQuery {
    /// Filter by tags
    pub tags: Option<Vec<String>>,

    /// Filter by enabled status
    pub enabled: Option<bool>,

    /// Search in rule names and descriptions
    pub search: Option<String>,

    /// Maximum number of results
    #[schema(example = 50)]
    pub limit: Option<usize>,

    /// Offset for pagination
    #[schema(example = 0)]
    pub offset: Option<usize>,
}

/// Response with list of rules
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListRulesResponse {
    /// List of rules
    pub rules: Vec<ApiRule>,

    /// Total number of rules (before pagination)
    pub total: usize,

    /// Number of results returned
    pub count: usize,
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
