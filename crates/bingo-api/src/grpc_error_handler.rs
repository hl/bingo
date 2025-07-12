//! Enhanced gRPC Error Handling with Diagnostics
//!
//! This module provides comprehensive error handling for gRPC services,
//! integrating with the error diagnostics system for better debugging.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bingo_core::{
    BingoError, ErrorDiagnostic, ErrorDiagnosticsManager, ErrorToDiagnostic, ResultDiagnosticExt,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tonic::{Code, Status};
use tracing::{debug, error, warn};
use uuid::Uuid;

/// gRPC error handler with diagnostics integration
#[derive(Debug)]
pub struct GrpcErrorHandler {
    /// Diagnostics manager for enhanced error handling
    diagnostics_manager: Arc<Mutex<ErrorDiagnosticsManager>>,
    /// Configuration for error handling
    config: GrpcErrorConfig,
    /// Error mapping rules
    error_mappings: HashMap<String, GrpcErrorMapping>,
}

/// Configuration for gRPC error handling
#[derive(Debug, Clone)]
pub struct GrpcErrorConfig {
    /// Include detailed error messages in responses
    pub include_detailed_messages: bool,
    /// Include error diagnostics in metadata
    pub include_diagnostics_in_metadata: bool,
    /// Include error suggestions for recoverable errors
    pub include_suggestions: bool,
    /// Maximum length for error messages
    pub max_message_length: usize,
    /// Enable error analytics collection
    pub enable_analytics: bool,
    /// Include debug information in non-production environments
    pub include_debug_info: bool,
}

/// Mapping configuration for specific error types
#[derive(Debug, Clone)]
pub struct GrpcErrorMapping {
    /// gRPC status code to use
    pub status_code: Code,
    /// Whether to include detailed message
    pub include_details: bool,
    /// Whether this error type is user-actionable
    pub user_actionable: bool,
    /// Custom message template
    pub message_template: Option<String>,
}

/// Enhanced gRPC error response with diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedGrpcError {
    /// Error code
    pub code: String,
    /// Primary error message
    pub message: String,
    /// Diagnostic ID for tracking
    pub diagnostic_id: Option<String>,
    /// Error category
    pub category: String,
    /// Severity level
    pub severity: String,
    /// Contextual information
    pub context: HashMap<String, String>,
    /// Suggested fixes (for recoverable errors)
    pub suggestions: Vec<ErrorSuggestionSummary>,
    /// Documentation links
    pub documentation_links: Vec<DocumentationLinkSummary>,
    /// Request ID for correlation
    pub request_id: Option<String>,
}

/// Simplified error suggestion for gRPC responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSuggestionSummary {
    /// Suggestion ID
    pub id: String,
    /// Priority (0.0 to 1.0)
    pub priority: f64,
    /// Brief title
    pub title: String,
    /// Short description
    pub description: String,
    /// Effort level
    pub effort: String,
}

/// Simplified documentation link for gRPC responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationLinkSummary {
    /// Link title
    pub title: String,
    /// URL
    pub url: String,
    /// Relevance score
    pub relevance: f64,
}

/// gRPC request context for error handling
#[derive(Debug, Clone)]
pub struct GrpcRequestContext {
    /// Request ID
    pub request_id: Option<String>,
    /// Service method name
    pub method: String,
    /// Client information
    pub client_info: Option<String>,
    /// Request metadata
    pub metadata: HashMap<String, String>,
    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for GrpcErrorConfig {
    fn default() -> Self {
        Self {
            include_detailed_messages: true,
            include_diagnostics_in_metadata: true,
            include_suggestions: true,
            max_message_length: 500,
            enable_analytics: true,
            include_debug_info: false,
        }
    }
}

impl GrpcErrorHandler {
    /// Create new gRPC error handler
    pub fn new(
        diagnostics_manager: Arc<Mutex<ErrorDiagnosticsManager>>,
        config: GrpcErrorConfig,
    ) -> Self {
        let mut handler = Self { diagnostics_manager, config, error_mappings: HashMap::new() };

        handler.setup_default_mappings();
        handler
    }

    /// Setup default error mappings for common error types
    fn setup_default_mappings(&mut self) {
        // Rule errors - client errors
        self.error_mappings.insert(
            "rule".to_string(),
            GrpcErrorMapping {
                status_code: Code::InvalidArgument,
                include_details: true,
                user_actionable: true,
                message_template: Some("Rule validation failed: {message}".to_string()),
            },
        );

        // Condition errors - client errors
        self.error_mappings.insert(
            "condition".to_string(),
            GrpcErrorMapping {
                status_code: Code::InvalidArgument,
                include_details: true,
                user_actionable: true,
                message_template: Some("Rule condition error: {message}".to_string()),
            },
        );

        // Fact store errors - server errors (usually)
        self.error_mappings.insert(
            "fact_store".to_string(),
            GrpcErrorMapping {
                status_code: Code::Internal,
                include_details: false,
                user_actionable: false,
                message_template: Some("Data storage error occurred".to_string()),
            },
        );

        // RETE network errors - server errors
        self.error_mappings.insert(
            "rete_network".to_string(),
            GrpcErrorMapping {
                status_code: Code::Internal,
                include_details: false,
                user_actionable: false,
                message_template: Some("Rule processing error occurred".to_string()),
            },
        );

        // Calculator errors - client errors
        self.error_mappings.insert(
            "calculator".to_string(),
            GrpcErrorMapping {
                status_code: Code::InvalidArgument,
                include_details: true,
                user_actionable: true,
                message_template: Some("Calculation error: {message}".to_string()),
            },
        );

        // Performance errors - server errors
        self.error_mappings.insert(
            "performance".to_string(),
            GrpcErrorMapping {
                status_code: Code::DeadlineExceeded,
                include_details: true,
                user_actionable: true,
                message_template: Some("Operation timeout: {message}".to_string()),
            },
        );

        // Memory errors - server errors
        self.error_mappings.insert(
            "memory".to_string(),
            GrpcErrorMapping {
                status_code: Code::ResourceExhausted,
                include_details: false,
                user_actionable: false,
                message_template: Some("Insufficient resources available".to_string()),
            },
        );

        // Configuration errors - server errors
        self.error_mappings.insert(
            "configuration".to_string(),
            GrpcErrorMapping {
                status_code: Code::FailedPrecondition,
                include_details: false,
                user_actionable: false,
                message_template: Some("Service configuration error".to_string()),
            },
        );

        // External service errors
        self.error_mappings.insert(
            "external".to_string(),
            GrpcErrorMapping {
                status_code: Code::Unavailable,
                include_details: true,
                user_actionable: false,
                message_template: Some("External service unavailable: {message}".to_string()),
            },
        );

        // Generic internal errors
        self.error_mappings.insert(
            "internal".to_string(),
            GrpcErrorMapping {
                status_code: Code::Internal,
                include_details: false,
                user_actionable: false,
                message_template: Some("Internal server error occurred".to_string()),
            },
        );
    }

    /// Convert BingoError to gRPC Status with enhanced diagnostics
    pub fn handle_error(&self, error: BingoError, context: Option<GrpcRequestContext>) -> Status {
        // Create diagnostic
        let diagnostic = {
            let mut manager = self.diagnostics_manager.lock().unwrap();
            error.to_diagnostic(&mut manager)
        };

        // Get error mapping
        let category = diagnostic.category.clone();
        let mapping = self.error_mappings.get(&category).cloned().unwrap_or_else(|| {
            // Default mapping for unknown error types
            GrpcErrorMapping {
                status_code: Code::Internal,
                include_details: false,
                user_actionable: false,
                message_template: Some("An error occurred".to_string()),
            }
        });

        // Generate user-friendly message
        let message = self.generate_error_message(&diagnostic, &mapping);

        // Create enhanced error response
        let enhanced_error = self.create_enhanced_error(&diagnostic, context.as_ref());

        // Create gRPC status with metadata
        let mut status = Status::new(mapping.status_code, message);

        // Add diagnostic information to metadata if enabled
        if self.config.include_diagnostics_in_metadata {
            self.add_diagnostic_metadata(&mut status, &enhanced_error);
        }

        // Log error for internal tracking
        self.log_error(&diagnostic, &mapping, context.as_ref());

        status
    }

    /// Generate user-friendly error message
    fn generate_error_message(
        &self,
        diagnostic: &ErrorDiagnostic,
        mapping: &GrpcErrorMapping,
    ) -> String {
        let mut message = if mapping.include_details && self.config.include_detailed_messages {
            diagnostic.error.clone()
        } else {
            mapping
                .message_template
                .as_ref()
                .map(|template| template.replace("{message}", &diagnostic.error))
                .unwrap_or_else(|| "An error occurred".to_string())
        };

        // Truncate message if too long
        if message.len() > self.config.max_message_length {
            message.truncate(self.config.max_message_length - 3);
            message.push_str("...");
        }

        message
    }

    /// Create enhanced error response for metadata
    fn create_enhanced_error(
        &self,
        diagnostic: &ErrorDiagnostic,
        context: Option<&GrpcRequestContext>,
    ) -> EnhancedGrpcError {
        let mut context_map = HashMap::new();

        // Add diagnostic context
        if let Some(rule_id) = diagnostic.context.rule_id {
            context_map.insert("rule_id".to_string(), rule_id.to_string());
        }
        if let Some(rule_name) = &diagnostic.context.rule_name {
            context_map.insert("rule_name".to_string(), rule_name.clone());
        }
        if let Some(field) = &diagnostic.context.field {
            context_map.insert("field".to_string(), field.clone());
        }
        if let Some(operator) = &diagnostic.context.operator {
            context_map.insert("operator".to_string(), operator.clone());
        }

        // Add request context
        let request_id = context.and_then(|ctx| ctx.request_id.clone());

        // Filter suggestions based on configuration
        let suggestions = if self.config.include_suggestions {
            diagnostic
                .suggestions
                .iter()
                .filter(|s| s.priority >= 0.5) // Only include high-priority suggestions
                .take(3) // Limit to top 3 suggestions
                .map(|s| ErrorSuggestionSummary {
                    id: s.id.clone(),
                    priority: s.priority,
                    title: s.title.clone(),
                    description: s.description.clone(),
                    effort: s.effort.to_string(),
                })
                .collect()
        } else {
            Vec::new()
        };

        // Filter documentation links
        let documentation_links = diagnostic
            .documentation_links
            .iter()
            .filter(|link| link.relevance >= 0.7) // Only include highly relevant links
            .take(2) // Limit to top 2 links
            .map(|link| DocumentationLinkSummary {
                title: link.title.clone(),
                url: link.url.clone(),
                relevance: link.relevance,
            })
            .collect();

        EnhancedGrpcError {
            code: format!("BINGO_{}", diagnostic.category.to_uppercase()),
            message: diagnostic.error.clone(),
            diagnostic_id: Some(diagnostic.diagnostic_id.to_string()),
            category: diagnostic.category.clone(),
            severity: format!("{:?}", diagnostic.severity),
            context: context_map,
            suggestions,
            documentation_links,
            request_id,
        }
    }

    /// Add diagnostic metadata to gRPC status
    fn add_diagnostic_metadata(&self, status: &mut Status, enhanced_error: &EnhancedGrpcError) {
        // Serialize enhanced error as JSON and add to metadata
        if let Ok(error_json) = serde_json::to_string(enhanced_error) {
            // gRPC metadata values must be ASCII, so we base64 encode if needed
            let metadata_value = if error_json.is_ascii() {
                error_json
            } else {
                STANDARD.encode(error_json)
            };

            if let Ok(ascii_value) = metadata_value.parse() {
                status.metadata_mut().insert("bingo-error-details", ascii_value);
            }
        }

        // Add simple metadata fields for easy client access
        if let Ok(diagnostic_id) = enhanced_error.diagnostic_id.as_ref().unwrap().parse() {
            status.metadata_mut().insert("bingo-diagnostic-id", diagnostic_id);
        }

        if let Ok(category) = enhanced_error.category.parse() {
            status.metadata_mut().insert("bingo-error-category", category);
        }

        if let Ok(severity) = enhanced_error.severity.parse() {
            status.metadata_mut().insert("bingo-error-severity", severity);
        }

        if let Some(request_id) = &enhanced_error.request_id {
            if let Ok(request_id_value) = request_id.parse() {
                status.metadata_mut().insert("bingo-request-id", request_id_value);
            }
        }
    }

    /// Log error for internal tracking
    fn log_error(
        &self,
        diagnostic: &ErrorDiagnostic,
        mapping: &GrpcErrorMapping,
        context: Option<&GrpcRequestContext>,
    ) {
        let log_context = context
            .map(|ctx| format!("method={} request_id={:?}", ctx.method, ctx.request_id))
            .unwrap_or_else(|| "context=unknown".to_string());

        match mapping.status_code {
            Code::InvalidArgument
            | Code::NotFound
            | Code::AlreadyExists
            | Code::PermissionDenied => {
                // Client errors - log as warnings
                warn!(
                    diagnostic_id = %diagnostic.diagnostic_id,
                    category = %diagnostic.category,
                    severity = ?diagnostic.severity,
                    %log_context,
                    "Client error in gRPC request: {}",
                    diagnostic.error
                );
            }
            Code::Internal | Code::DataLoss | Code::Unknown => {
                // Server errors - log as errors
                error!(
                    diagnostic_id = %diagnostic.diagnostic_id,
                    category = %diagnostic.category,
                    severity = ?diagnostic.severity,
                    %log_context,
                    "Server error in gRPC request: {}",
                    diagnostic.error
                );
            }
            _ => {
                // Other errors - log as debug
                debug!(
                    diagnostic_id = %diagnostic.diagnostic_id,
                    category = %diagnostic.category,
                    severity = ?diagnostic.severity,
                    %log_context,
                    "gRPC request error: {}",
                    diagnostic.error
                );
            }
        }
    }

    /// Create request context from gRPC request
    pub fn create_request_context(
        method: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> GrpcRequestContext {
        let mut context_metadata = HashMap::new();
        let mut client_info = None;
        let mut request_id = None;

        // Extract relevant metadata
        for key_value in metadata.iter() {
            match key_value {
                tonic::metadata::KeyAndValueRef::Ascii(key, value) => {
                    let key_str = key.as_str();
                    if let Ok(value_str) = value.to_str() {
                        match key_str {
                            "user-agent" => client_info = Some(value_str.to_string()),
                            "x-request-id" | "request-id" => {
                                request_id = Some(value_str.to_string())
                            }
                            _ => {
                                context_metadata.insert(key_str.to_string(), value_str.to_string());
                            }
                        }
                    }
                }
                tonic::metadata::KeyAndValueRef::Binary(key, _value) => {
                    // For binary metadata, just record the key
                    context_metadata.insert(key.as_str().to_string(), "<binary>".to_string());
                }
            }
        }

        // Generate request ID if not provided
        if request_id.is_none() {
            request_id = Some(Uuid::new_v4().to_string());
        }

        GrpcRequestContext {
            request_id,
            method: method.to_string(),
            client_info,
            metadata: context_metadata,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Update error mapping for a specific category
    pub fn update_error_mapping(&mut self, category: String, mapping: GrpcErrorMapping) {
        self.error_mappings.insert(category, mapping);
    }

    /// Get error analytics
    pub fn get_error_analytics(&self) -> Result<serde_json::Value, String> {
        let manager = self.diagnostics_manager.lock().map_err(|e| e.to_string())?;
        let analytics = manager.get_analytics();

        serde_json::to_value(analytics).map_err(|e| e.to_string())
    }
}

/// Helper trait for converting BingoResult to gRPC Status
pub trait BingoResultToGrpcStatus<T> {
    fn to_grpc_status(
        self,
        handler: &GrpcErrorHandler,
        context: Option<GrpcRequestContext>,
    ) -> Result<T, Box<Status>>;
}

impl<T> BingoResultToGrpcStatus<T> for Result<T, BingoError> {
    fn to_grpc_status(
        self,
        handler: &GrpcErrorHandler,
        context: Option<GrpcRequestContext>,
    ) -> Result<T, Box<Status>> {
        self.map_err(|err| Box::new(handler.handle_error(err, context)))
    }
}

/// Macro for easy error handling in gRPC services
#[macro_export]
macro_rules! handle_grpc_error {
    ($result:expr, $handler:expr) => {
        $result.to_grpc_status($handler, None)?
    };
    ($result:expr, $handler:expr, $context:expr) => {
        $result.to_grpc_status($handler, Some($context))?
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use bingo_core::{BingoError, DiagnosticsConfig, ErrorDiagnosticsManager};

    #[test]
    fn test_grpc_error_handler_creation() {
        let diagnostics_manager = Arc::new(Mutex::new(ErrorDiagnosticsManager::new(
            DiagnosticsConfig::default(),
        )));
        let handler = GrpcErrorHandler::new(diagnostics_manager, GrpcErrorConfig::default());

        assert!(!handler.error_mappings.is_empty());
    }

    #[test]
    fn test_error_conversion() {
        let diagnostics_manager = Arc::new(Mutex::new(ErrorDiagnosticsManager::new(
            DiagnosticsConfig::default(),
        )));
        let handler = GrpcErrorHandler::new(diagnostics_manager, GrpcErrorConfig::default());

        let bingo_error = BingoError::rule_validation("Test rule error");
        let status = handler.handle_error(bingo_error, None);

        assert_eq!(status.code(), Code::InvalidArgument);
        assert!(status.message().contains("Test rule error"));
    }

    #[test]
    fn test_request_context_creation() {
        let metadata = tonic::metadata::MetadataMap::new();
        let context = GrpcErrorHandler::create_request_context("test_method", &metadata);

        assert_eq!(context.method, "test_method");
        assert!(context.request_id.is_some());
    }
}
