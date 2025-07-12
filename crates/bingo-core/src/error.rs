//! Comprehensive error handling for the Bingo Core engine
//!
//! This module provides structured error types for all core engine operations,
//! enabling better error handling, debugging, and integration with higher-level systems.

use std::fmt;
use thiserror::Error;

/// Comprehensive error type for Bingo Core engine operations
#[derive(Error, Debug, Clone)]
pub enum BingoError {
    /// Rule compilation and validation errors
    #[error("Rule error: {message}")]
    Rule {
        message: String,
        rule_id: Option<u64>,
        rule_name: Option<String>,
        details: Option<String>,
    },

    /// Rule condition parsing and validation errors
    #[error("Condition error: {message}")]
    Condition {
        message: String,
        field: Option<String>,
        operator: Option<String>,
        value: Option<String>,
    },

    /// Fact store operation errors
    #[error("Fact store error: {message}")]
    FactStore {
        message: String,
        fact_id: Option<u64>,
        operation: Option<String>,
        details: Option<String>,
    },

    /// RETE network compilation and processing errors
    #[error("RETE network error: {message}")]
    ReteNetwork {
        message: String,
        node_type: Option<String>,
        network_state: Option<String>,
        details: Option<String>,
    },

    /// Calculator and expression evaluation errors
    #[error("Calculator error: {message}")]
    Calculator {
        message: String,
        expression: Option<String>,
        variable: Option<String>,
        operation: Option<String>,
    },

    /// Aggregation processing errors
    #[error("Aggregation error: {message}")]
    Aggregation {
        message: String,
        aggregation_type: Option<String>,
        source_field: Option<String>,
        group_by: Option<Vec<String>>,
        details: Option<String>,
    },

    /// Memory and resource management errors
    #[error("Memory error: {message}")]
    Memory {
        message: String,
        pool_type: Option<String>,
        requested_size: Option<usize>,
        available_size: Option<usize>,
    },

    /// Serialization and deserialization errors
    #[error("Serialization error: {message}")]
    Serialization { message: String, data_type: Option<String>, operation: Option<String> },

    /// Configuration and initialization errors
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        setting: Option<String>,
        expected: Option<String>,
        actual: Option<String>,
    },

    /// Performance and timeout errors
    #[error("Performance error: {message}")]
    Performance {
        message: String,
        operation: Option<String>,
        duration_ms: Option<u64>,
        limit_ms: Option<u64>,
    },

    /// External dependency errors (I/O, network, etc.)
    #[error("External error: {message}")]
    External { message: String, service: Option<String>, source_details: Option<String> },

    /// Generic internal errors
    #[error("Internal error: {message}")]
    Internal { message: String, component: Option<String>, source_details: Option<String> },
}

impl BingoError {
    /// Get the error category for logging and metrics
    pub fn category(&self) -> &'static str {
        match self {
            BingoError::Rule { .. } => "rule",
            BingoError::Condition { .. } => "condition",
            BingoError::FactStore { .. } => "fact_store",
            BingoError::ReteNetwork { .. } => "rete_network",
            BingoError::Calculator { .. } => "calculator",
            BingoError::Aggregation { .. } => "aggregation",
            BingoError::Memory { .. } => "memory",
            BingoError::Serialization { .. } => "serialization",
            BingoError::Configuration { .. } => "configuration",
            BingoError::Performance { .. } => "performance",
            BingoError::External { .. } => "external",
            BingoError::Internal { .. } => "internal",
        }
    }

    /// Get the error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            BingoError::Rule { .. } => ErrorSeverity::Medium,
            BingoError::Condition { .. } => ErrorSeverity::Medium,
            BingoError::FactStore { .. } => ErrorSeverity::High,
            BingoError::ReteNetwork { .. } => ErrorSeverity::High,
            BingoError::Calculator { .. } => ErrorSeverity::Medium,
            BingoError::Aggregation { .. } => ErrorSeverity::Medium,
            BingoError::Memory { .. } => ErrorSeverity::Critical,
            BingoError::Serialization { .. } => ErrorSeverity::Low,
            BingoError::Configuration { .. } => ErrorSeverity::Critical,
            BingoError::Performance { .. } => ErrorSeverity::High,
            BingoError::External { .. } => ErrorSeverity::Medium,
            BingoError::Internal { .. } => ErrorSeverity::Critical,
        }
    }

    /// Get structured context information for debugging
    pub fn context(&self) -> ErrorContext {
        match self {
            BingoError::Rule { rule_id, rule_name, .. } => ErrorContext {
                rule_id: *rule_id,
                rule_name: rule_name.clone(),
                ..Default::default()
            },
            BingoError::Condition { field, operator, value, .. } => ErrorContext {
                field: field.clone(),
                operator: operator.clone(),
                value: value.clone(),
                ..Default::default()
            },
            BingoError::FactStore { fact_id, operation, .. } => ErrorContext {
                fact_id: *fact_id,
                operation: operation.clone(),
                ..Default::default()
            },
            BingoError::Calculator { expression, variable, operation, .. } => ErrorContext {
                expression: expression.clone(),
                variable: variable.clone(),
                operation: operation.clone(),
                ..Default::default()
            },
            BingoError::Aggregation { aggregation_type, source_field, .. } => ErrorContext {
                aggregation_type: aggregation_type.clone(),
                field: source_field.clone(),
                ..Default::default()
            },
            _ => ErrorContext::default(),
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            BingoError::Rule { .. } => true,
            BingoError::Condition { .. } => true,
            BingoError::FactStore { .. } => false, // Data integrity concerns
            BingoError::ReteNetwork { .. } => false, // Network state corruption
            BingoError::Calculator { .. } => true,
            BingoError::Aggregation { .. } => true,
            BingoError::Memory { .. } => false, // Memory issues require restart
            BingoError::Serialization { .. } => true,
            BingoError::Configuration { .. } => false, // Config errors need fixing
            BingoError::Performance { .. } => true,
            BingoError::External { .. } => true, // External deps may recover
            BingoError::Internal { .. } => false, // Unknown internal state
        }
    }
}

/// Error severity levels for logging and alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "LOW"),
            ErrorSeverity::Medium => write!(f, "MEDIUM"),
            ErrorSeverity::High => write!(f, "HIGH"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Structured error context for debugging and telemetry
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ErrorContext {
    pub rule_id: Option<u64>,
    pub rule_name: Option<String>,
    pub fact_id: Option<u64>,
    pub field: Option<String>,
    pub operator: Option<String>,
    pub value: Option<String>,
    pub expression: Option<String>,
    pub variable: Option<String>,
    pub operation: Option<String>,
    pub aggregation_type: Option<String>,
}

/// Result type alias for core engine operations
pub type BingoResult<T> = Result<T, BingoError>;

/// Convenience constructors for common error scenarios
impl BingoError {
    /// Create a rule compilation error
    pub fn rule_compilation(rule_id: u64, rule_name: &str, message: impl Into<String>) -> Self {
        Self::Rule {
            message: message.into(),
            rule_id: Some(rule_id),
            rule_name: Some(rule_name.to_string()),
            details: None,
        }
    }

    /// Create a rule validation error
    pub fn rule_validation(message: impl Into<String>) -> Self {
        Self::Rule { message: message.into(), rule_id: None, rule_name: None, details: None }
    }

    /// Create a condition parsing error
    pub fn condition_parse(
        field: &str,
        operator: &str,
        value: &str,
        message: impl Into<String>,
    ) -> Self {
        Self::Condition {
            message: message.into(),
            field: Some(field.to_string()),
            operator: Some(operator.to_string()),
            value: Some(value.to_string()),
        }
    }

    /// Create a fact store operation error
    pub fn fact_store(operation: &str, message: impl Into<String>) -> Self {
        Self::FactStore {
            message: message.into(),
            fact_id: None,
            operation: Some(operation.to_string()),
            details: None,
        }
    }

    /// Create a fact store error with fact ID
    pub fn fact_store_with_id(fact_id: u64, operation: &str, message: impl Into<String>) -> Self {
        Self::FactStore {
            message: message.into(),
            fact_id: Some(fact_id),
            operation: Some(operation.to_string()),
            details: None,
        }
    }

    /// Create a RETE network error
    pub fn rete_network(node_type: &str, message: impl Into<String>) -> Self {
        Self::ReteNetwork {
            message: message.into(),
            node_type: Some(node_type.to_string()),
            network_state: None,
            details: None,
        }
    }

    /// Create a calculator error
    pub fn calculator(expression: &str, message: impl Into<String>) -> Self {
        Self::Calculator {
            message: message.into(),
            expression: Some(expression.to_string()),
            variable: None,
            operation: None,
        }
    }

    /// Create an aggregation error
    pub fn aggregation(
        aggregation_type: &str,
        source_field: &str,
        message: impl Into<String>,
    ) -> Self {
        Self::Aggregation {
            message: message.into(),
            aggregation_type: Some(aggregation_type.to_string()),
            source_field: Some(source_field.to_string()),
            group_by: None,
            details: None,
        }
    }

    /// Create a memory allocation error
    pub fn memory_allocation(
        pool_type: &str,
        requested: usize,
        available: usize,
        message: impl Into<String>,
    ) -> Self {
        Self::Memory {
            message: message.into(),
            pool_type: Some(pool_type.to_string()),
            requested_size: Some(requested),
            available_size: Some(available),
        }
    }

    /// Create a serialization error
    pub fn serialization(data_type: &str, operation: &str, message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
            data_type: Some(data_type.to_string()),
            operation: Some(operation.to_string()),
        }
    }

    /// Create a configuration error
    pub fn configuration(
        setting: &str,
        expected: &str,
        actual: &str,
        message: impl Into<String>,
    ) -> Self {
        Self::Configuration {
            message: message.into(),
            setting: Some(setting.to_string()),
            expected: Some(expected.to_string()),
            actual: Some(actual.to_string()),
        }
    }

    /// Create a performance timeout error
    pub fn performance_timeout(
        operation: &str,
        duration_ms: u64,
        limit_ms: u64,
        message: impl Into<String>,
    ) -> Self {
        Self::Performance {
            message: message.into(),
            operation: Some(operation.to_string()),
            duration_ms: Some(duration_ms),
            limit_ms: Some(limit_ms),
        }
    }

    /// Create an external dependency error
    pub fn external_service(service: &str, message: impl Into<String>) -> Self {
        Self::External {
            message: message.into(),
            service: Some(service.to_string()),
            source_details: None,
        }
    }

    /// Create an internal error with component context
    pub fn internal_component(component: &str, message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            component: Some(component.to_string()),
            source_details: None,
        }
    }

    /// Create a generic internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal { message: message.into(), component: None, source_details: None }
    }
}

/// Convert from anyhow::Error to BingoError
impl From<anyhow::Error> for BingoError {
    fn from(err: anyhow::Error) -> Self {
        // Try to downcast to known error types first
        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            return BingoError::External {
                message: format!("I/O operation failed: {io_err}"),
                service: Some("filesystem".to_string()),
                source_details: Some(format!("IO Error kind: {:?}", io_err.kind())),
            };
        }

        if let Some(serde_err) = err.downcast_ref::<serde_json::Error>() {
            return BingoError::serialization(
                "json",
                if serde_err.is_syntax() {
                    "parse"
                } else if serde_err.is_data() {
                    "validate"
                } else {
                    "unknown"
                },
                format!("JSON serialization error: {serde_err}"),
            );
        }

        // Default to internal error
        BingoError::internal(format!("Unhandled error: {err}"))
    }
}

/// Convert from standard error types
impl From<std::io::Error> for BingoError {
    fn from(err: std::io::Error) -> Self {
        BingoError::External {
            message: format!("I/O error: {err}"),
            service: Some("filesystem".to_string()),
            source_details: Some(format!("IO Error kind: {:?}", err.kind())),
        }
    }
}

impl From<serde_json::Error> for BingoError {
    fn from(err: serde_json::Error) -> Self {
        BingoError::serialization(
            "json",
            if err.is_syntax() {
                "parse"
            } else if err.is_data() {
                "validate"
            } else {
                "unknown"
            },
            format!("JSON error: {err}"),
        )
    }
}

/// Extension trait for adding context to Results
pub trait ResultExt<T> {
    /// Add rule context to an error
    fn with_rule_context(self, rule_id: u64, rule_name: &str) -> BingoResult<T>;

    /// Add fact context to an error
    fn with_fact_context(self, fact_id: u64) -> BingoResult<T>;

    /// Add operation context to an error
    fn with_operation_context(self, operation: &str) -> BingoResult<T>;
}

impl<T> ResultExt<T> for BingoResult<T> {
    fn with_rule_context(self, rule_id: u64, rule_name: &str) -> BingoResult<T> {
        self.map_err(|mut err| {
            if let BingoError::Rule { rule_id: id, rule_name: name, .. } = &mut err {
                *id = Some(rule_id);
                *name = Some(rule_name.to_string());
            }
            err
        })
    }

    fn with_fact_context(self, fact_id: u64) -> BingoResult<T> {
        self.map_err(|mut err| {
            if let BingoError::FactStore { fact_id: id, .. } = &mut err {
                *id = Some(fact_id);
            }
            err
        })
    }

    fn with_operation_context(self, operation: &str) -> BingoResult<T> {
        self.map_err(|mut err| {
            match &mut err {
                BingoError::FactStore { operation: op, .. } => {
                    *op = Some(operation.to_string());
                }
                BingoError::Calculator { operation: op, .. } => {
                    *op = Some(operation.to_string());
                }
                _ => {} // Keep other error types unchanged
            }
            err
        })
    }
}
