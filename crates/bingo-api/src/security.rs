//! Security validation and hardening utilities
//!
//! This module provides security measures to prevent DoS attacks and
//! validate potentially dangerous input patterns.

use crate::{
    config::SecurityConfig,
    types::{ApiAction, ApiRule, EvaluateRequest},
};
use tracing::{debug, warn};

/// Security validation result
#[derive(Debug)]
pub enum SecurityValidationResult {
    /// The request passed all security checks
    Safe,
    /// The request was rejected for security reasons
    Rejected {
        /// Reason for the rejection
        reason: String,
    },
}

/// Security validator for API requests
pub struct SecurityValidator;

impl SecurityValidator {
    /// Validate an entire evaluate request for security issues
    pub fn validate_request(
        request: &EvaluateRequest,
        limits: &SecurityConfig,
    ) -> SecurityValidationResult {
        // Validate request size limits
        if let Some(rules) = &request.rules {
            if rules.len() > limits.max_rules_per_request {
                warn!(
                    rules_count = rules.len(),
                    max_allowed = limits.max_rules_per_request,
                    "Request rejected: too many rules"
                );
                return SecurityValidationResult::Rejected {
                    reason: format!(
                        "Too many rules: {} (max: {})",
                        rules.len(),
                        limits.max_rules_per_request
                    ),
                };
            }

            // Validate each rule for security issues
            for (index, rule) in rules.iter().enumerate() {
                if let SecurityValidationResult::Rejected { reason } =
                    Self::validate_rule(rule, limits)
                {
                    warn!(
                        rule_index = index,
                        rule_id = %rule.id,
                        reason = %reason,
                        "Rule rejected for security reasons"
                    );
                    return SecurityValidationResult::Rejected {
                        reason: format!("Rule '{}': {}", rule.id, reason),
                    };
                }
            }
        }

        if request.facts.len() > limits.max_facts_per_request {
            warn!(
                facts_count = request.facts.len(),
                max_allowed = limits.max_facts_per_request,
                "Request rejected: too many facts"
            );
            return SecurityValidationResult::Rejected {
                reason: format!(
                    "Too many facts: {} (max: {})",
                    request.facts.len(),
                    limits.max_facts_per_request
                ),
            };
        }

        // Validate streaming configuration for abuse potential
        if let Some(streaming_config) = &request.streaming_config {
            if let Some(chunk_size) = streaming_config.chunk_size {
                if chunk_size == 0 || chunk_size > limits.max_streaming_chunk_size {
                    return SecurityValidationResult::Rejected {
                        reason: format!(
                            "Invalid chunk size: must be between 1 and {}",
                            limits.max_streaming_chunk_size
                        ),
                    };
                }
            }
        }

        SecurityValidationResult::Safe
    }

    /// Validate a single rule for security issues
    fn validate_rule(rule: &ApiRule, limits: &SecurityConfig) -> SecurityValidationResult {
        // Check for excessively long rule names/descriptions that could cause memory issues
        if rule.name.len() > limits.max_rule_name_length {
            return SecurityValidationResult::Rejected {
                reason: format!(
                    "Rule name too long (max: {} characters)",
                    limits.max_rule_name_length
                ),
            };
        }

        if let Some(desc) = &rule.description {
            if desc.len() > limits.max_rule_description_length {
                return SecurityValidationResult::Rejected {
                    reason: format!(
                        "Rule description too long (max: {} characters)",
                        limits.max_rule_description_length
                    ),
                };
            }
        }

        // Validate condition count per rule
        if rule.conditions.len() > limits.max_conditions_per_rule {
            return SecurityValidationResult::Rejected {
                reason: format!(
                    "Too many conditions: {} (max: {})",
                    rule.conditions.len(),
                    limits.max_conditions_per_rule
                ),
            };
        }

        // Validate conditions for security issues
        for condition in &rule.conditions {
            if let SecurityValidationResult::Rejected { reason } =
                Self::validate_condition(condition, limits)
            {
                return SecurityValidationResult::Rejected { reason };
            }
        }

        // Validate actions for security issues
        for action in &rule.actions {
            if let SecurityValidationResult::Rejected { reason } =
                Self::validate_action(action, limits)
            {
                return SecurityValidationResult::Rejected { reason };
            }
        }

        SecurityValidationResult::Safe
    }

    /// Validate a single condition for security issues
    fn validate_condition(
        condition: &crate::types::ApiCondition,
        limits: &SecurityConfig,
    ) -> SecurityValidationResult {
        use crate::types::ApiCondition;

        match condition {
            ApiCondition::Simple { field, value, .. } => {
                // Validate field name
                if field.len() > limits.max_calculator_input_key_length {
                    return SecurityValidationResult::Rejected {
                        reason: format!(
                            "Condition field name too long: {} characters (max: {})",
                            field.len(),
                            limits.max_calculator_input_key_length
                        ),
                    };
                }

                // Check for dangerous field names
                if Self::is_dangerous_field_name(field) {
                    return SecurityValidationResult::Rejected {
                        reason: format!(
                            "Condition field name '{}' is not allowed for security reasons",
                            field
                        ),
                    };
                }

                // Validate value size
                let value_str = value.to_string();
                if value_str.len() > limits.max_created_fact_field_value_length {
                    return SecurityValidationResult::Rejected {
                        reason: format!(
                            "Condition value too large: {} characters (max: {})",
                            value_str.len(),
                            limits.max_created_fact_field_value_length
                        ),
                    };
                }

                // Check for potential injection patterns in string values
                if let Some(str_val) = value.as_str() {
                    if Self::contains_injection_patterns(str_val) {
                        return SecurityValidationResult::Rejected {
                            reason: "Condition value contains potentially dangerous patterns"
                                .to_string(),
                        };
                    }
                }

                SecurityValidationResult::Safe
            }
            ApiCondition::Complex { conditions, .. } => {
                // Recursively validate nested conditions
                for nested_condition in conditions {
                    if let SecurityValidationResult::Rejected { reason } =
                        Self::validate_condition(nested_condition, limits)
                    {
                        return SecurityValidationResult::Rejected { reason };
                    }
                }
                SecurityValidationResult::Safe
            }
        }
    }

    /// Validate a single action for security issues
    fn validate_action(action: &ApiAction, limits: &SecurityConfig) -> SecurityValidationResult {
        match action {
            ApiAction::SetField { field, value } => {
                // Validate field name length
                if field.len() > limits.max_calculator_input_key_length {
                    return SecurityValidationResult::Rejected {
                        reason: format!(
                            "Field name too long: {} characters (max: {})",
                            field.len(),
                            limits.max_calculator_input_key_length
                        ),
                    };
                }

                // Validate value size (approximate via JSON serialization)
                let value_str = value.to_string();
                if value_str.len() > limits.max_created_fact_field_value_length {
                    return SecurityValidationResult::Rejected {
                        reason: format!(
                            "Field value too large: {} characters (max: {})",
                            value_str.len(),
                            limits.max_created_fact_field_value_length
                        ),
                    };
                }

                // Check for potentially dangerous field names
                if Self::is_dangerous_field_name(field) {
                    return SecurityValidationResult::Rejected {
                        reason: format!(
                            "Field name '{}' is not allowed for security reasons",
                            field
                        ),
                    };
                }

                SecurityValidationResult::Safe
            }
            ApiAction::CallCalculator { input_mapping, calculator_name, .. } => {
                // Validate calculator name
                if calculator_name.len() > limits.max_calculator_name_length {
                    return SecurityValidationResult::Rejected {
                        reason: format!(
                            "Calculator name too long (max: {} characters)",
                            limits.max_calculator_name_length
                        ),
                    };
                }

                // Validate input mapping size
                if input_mapping.len() > limits.max_calculator_inputs {
                    return SecurityValidationResult::Rejected {
                        reason: format!(
                            "Too many calculator inputs: {} (max: {})",
                            input_mapping.len(),
                            limits.max_calculator_inputs
                        ),
                    };
                }

                // Validate input mapping keys and values
                for (key, value) in input_mapping {
                    if key.len() > limits.max_calculator_input_key_length
                        || value.len() > limits.max_calculator_input_value_length
                    {
                        return SecurityValidationResult::Rejected {
                            reason: format!(
                                "Calculator input key (max: {} chars) or value (max: {} chars) too long",
                                limits.max_calculator_input_key_length,
                                limits.max_calculator_input_value_length
                            ),
                        };
                    }
                }

                SecurityValidationResult::Safe
            }
            ApiAction::Log { message, .. } => {
                if message.len() > limits.max_log_message_length {
                    return SecurityValidationResult::Rejected {
                        reason: format!(
                            "Log message too long (max: {} characters)",
                            limits.max_log_message_length
                        ),
                    };
                }
                SecurityValidationResult::Safe
            }
            ApiAction::CreateFact { data } => {
                // Validate fact data size and content
                if data.len() > limits.max_created_fact_fields {
                    return SecurityValidationResult::Rejected {
                        reason: format!(
                            "Too many fields in created fact (max: {})",
                            limits.max_created_fact_fields
                        ),
                    };
                }

                for (key, value) in data {
                    if key.len() > limits.max_calculator_input_key_length {
                        return SecurityValidationResult::Rejected {
                            reason: format!(
                                "Fact field name too long (max: {} characters)",
                                limits.max_calculator_input_key_length
                            ),
                        };
                    }

                    // Validate JSON value size (approximate)
                    let value_str = value.to_string();
                    if value_str.len() > limits.max_created_fact_field_value_length {
                        return SecurityValidationResult::Rejected {
                            reason: format!(
                                "Fact field value too large (max: {} characters)",
                                limits.max_created_fact_field_value_length
                            ),
                        };
                    }
                }

                SecurityValidationResult::Safe
            }
            _ => SecurityValidationResult::Safe,
        }
    }

    /// Validate expression complexity to prevent DoS via complex calculations
    fn validate_expression_complexity(
        expression: &str,
        limits: &SecurityConfig,
    ) -> SecurityValidationResult {
        debug!(
            expression_length = expression.len(),
            "Validating expression complexity"
        );

        // Basic length check
        if expression.len() > limits.max_expression_length {
            return SecurityValidationResult::Rejected {
                reason: format!(
                    "Expression too long: {} characters (max: {})",
                    expression.len(),
                    limits.max_expression_length
                ),
            };
        }

        // Count complexity indicators
        let mut complexity_score = 0;
        let mut nesting_depth: i32 = 0;
        let mut max_depth = 0;

        for ch in expression.chars() {
            match ch {
                // Operators add to complexity
                '+' | '-' | '*' | '/' | '%' | '^' => complexity_score += 1,
                // Comparisons
                '=' | '<' | '>' | '!' => complexity_score += 1,
                // Function calls (approximate detection)
                '(' => {
                    complexity_score += 2;
                    nesting_depth += 1;
                    max_depth = max_depth.max(nesting_depth);
                }
                ')' => {
                    nesting_depth = nesting_depth.saturating_sub(1);
                }
                // Conditionals add significant complexity
                '?' | ':' => complexity_score += 5,
                _ => {}
            }
        }

        debug!(
            complexity_score = complexity_score,
            max_depth = max_depth,
            "Expression complexity analysis"
        );

        if complexity_score > limits.max_expression_complexity {
            return SecurityValidationResult::Rejected {
                reason: format!(
                    "Expression too complex: score {} (max: {})",
                    complexity_score, limits.max_expression_complexity
                ),
            };
        }

        if max_depth > limits.max_expression_depth as i32 {
            return SecurityValidationResult::Rejected {
                reason: format!(
                    "Expression nesting too deep: {} levels (max: {})",
                    max_depth, limits.max_expression_depth
                ),
            };
        }

        // Check for potential ReDoS patterns (simple heuristics)
        if expression.contains(".*.*") || expression.contains("(.+)+") {
            return SecurityValidationResult::Rejected {
                reason: "Expression contains potentially dangerous regex patterns".to_string(),
            };
        }

        SecurityValidationResult::Safe
    }

    /// Check if a field name might be dangerous for security reasons
    fn is_dangerous_field_name(field_name: &str) -> bool {
        // Convert to lowercase for case-insensitive checking
        let field_lower = field_name.to_lowercase();

        // Block potentially dangerous field names that could cause issues:
        // 1. System/internal fields that could interfere with engine operation
        // 2. SQL injection attempts
        // 3. JavaScript injection attempts
        // 4. Path traversal attempts
        // 5. Common attack vectors

        let dangerous_patterns = [
            // System/internal fields
            "__proto__",
            "constructor",
            "prototype",
            "_internal",
            "__internal",
            "system",
            "admin",
            "root",
            "config",
            "settings",
            // SQL injection patterns
            "select",
            "union",
            "insert",
            "update",
            "delete",
            "drop",
            "create",
            "alter",
            "exec",
            "execute",
            "sp_",
            "xp_",
            // JavaScript/XSS patterns
            "script",
            "javascript",
            "eval",
            "function",
            "onload",
            "onerror",
            "alert",
            "document",
            "window",
            "location",
            // Path traversal
            "..",
            "/",
            "\\",
            "../",
            "..\\",
            // Other attack vectors
            "null",
            "undefined",
            "infinity",
            "nan",
        ];

        // Check for exact matches or if field contains dangerous patterns
        for pattern in &dangerous_patterns {
            if field_lower == *pattern || field_lower.contains(pattern) {
                return true;
            }
        }

        // Check for suspicious patterns
        if field_name.starts_with('_') && field_name.len() > 2 {
            // Fields starting with underscore might be internal
            return true;
        }

        // Check for non-printable characters or control characters
        if field_name.chars().any(|c| c.is_control() || !c.is_ascii()) {
            return true;
        }

        // Check for excessive special characters (potential encoding attacks)
        let special_char_count = field_name
            .chars()
            .filter(|c| !c.is_alphanumeric() && *c != '_' && *c != '-')
            .count();
        if special_char_count > 2 {
            return true;
        }

        false
    }

    /// Check if a string contains potential injection patterns
    fn contains_injection_patterns(input: &str) -> bool {
        let input_lower = input.to_lowercase();

        // Common injection patterns to detect
        let injection_patterns = [
            // SQL injection
            "union select",
            "'; drop",
            "; drop",
            "' or '1'='1",
            "\" or \"1\"=\"1",
            "' or 1=1",
            "\" or 1=1",
            "'; exec",
            "\"; exec",
            "xp_cmdshell",
            // NoSQL injection
            "$where",
            "$ne",
            "$gt",
            "$regex",
            "$nin",
            "$or",
            "$and",
            // JavaScript injection
            "<script",
            "</script>",
            "javascript:",
            "eval(",
            "function(",
            "alert(",
            "document.",
            "window.",
            "location.",
            // Command injection
            "; cat",
            "| cat",
            "; rm",
            "; ls",
            "| ls",
            "; curl",
            "| curl",
            "; wget",
            "| wget",
            "; nc",
            "| nc",
            "; bash",
            "| bash",
            // LDAP injection
            ")(cn=",
            ")(&(",
            ")(uid=",
            ")(!(",
            // XPath injection
            "' or text()=",
            "\" or text()=",
            "' or @*=",
            "\" or @*=",
        ];

        // Check for injection patterns
        for pattern in &injection_patterns {
            if input_lower.contains(pattern) {
                return true;
            }
        }

        // Check for suspicious character sequences
        if input.contains('\0') {
            return true; // Null byte injection
        }

        // Check for excessive escape sequences (potential encoding attacks)
        let escape_count = input.matches('\\').count();
        if escape_count > 3 {
            return true;
        }

        // Check for potential template injection
        if input.contains("{{") && input.contains("}}") {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::SecurityConfig,
        types::{ApiCondition, ResponseFormat},
    };
    use chrono::Utc;
    use serde_json::json;

    fn create_simple_rule() -> ApiRule {
        ApiRule {
            id: "test-rule".to_string(),
            name: "Test Rule".to_string(),
            description: Some("A test rule".to_string()),
            conditions: vec![ApiCondition::Simple {
                field: "test".to_string(),
                operator: crate::types::ApiSimpleOperator::Equal,
                value: json!("value"),
            }],
            actions: vec![ApiAction::Log {
                level: "info".to_string(),
                message: "Test message".to_string(),
            }],
            priority: Some(100),
            enabled: true,
            tags: vec!["test".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_simple_expression_validation() {
        let limits = SecurityConfig::default();
        let result = SecurityValidator::validate_expression_complexity("amount * 0.15", &limits);
        matches!(result, SecurityValidationResult::Safe);
    }

    #[test]
    fn test_complex_expression_rejection() {
        let limits = SecurityConfig::default();
        // Create an overly complex expression
        let complex_expr = "(".repeat(100) + &"+".repeat(2000) + &")".repeat(100);
        let result = SecurityValidator::validate_expression_complexity(&complex_expr, &limits);
        matches!(result, SecurityValidationResult::Rejected { .. });
    }

    #[test]
    fn test_deep_nesting_rejection() {
        let limits = SecurityConfig::default();
        let deep_expr = "(".repeat(100) + "1" + &")".repeat(100);
        let result = SecurityValidator::validate_expression_complexity(&deep_expr, &limits);
        matches!(result, SecurityValidationResult::Rejected { .. });
    }

    #[test]
    fn test_request_validation_safe() {
        let request = EvaluateRequest {
            facts: vec![],
            rules: Some(vec![create_simple_rule()]),
            ruleset_id: None,
            response_format: Some(ResponseFormat::Standard),
            streaming_config: None,
        };

        let limits = SecurityConfig::default();
        let result = SecurityValidator::validate_request(&request, &limits);
        matches!(result, SecurityValidationResult::Safe);
    }

    #[test]
    fn test_too_many_rules_rejection() {
        let limits = SecurityConfig::default();
        let rules: Vec<ApiRule> = (0..=limits.max_rules_per_request)
            .map(|i| {
                let mut rule = create_simple_rule();
                rule.id = format!("rule-{}", i);
                rule
            })
            .collect();

        let request = EvaluateRequest {
            facts: vec![],
            rules: Some(rules),
            ruleset_id: None,
            response_format: Some(ResponseFormat::Standard),
            streaming_config: None,
        };

        let result = SecurityValidator::validate_request(&request, &limits);
        matches!(result, SecurityValidationResult::Rejected { .. });
    }

    #[test]
    fn test_dangerous_field_names() {
        assert!(SecurityValidator::is_dangerous_field_name("__proto__"));
        assert!(SecurityValidator::is_dangerous_field_name("constructor"));
        assert!(SecurityValidator::is_dangerous_field_name("SELECT"));
        assert!(SecurityValidator::is_dangerous_field_name(
            "javascript:alert"
        ));
        assert!(SecurityValidator::is_dangerous_field_name("../etc/passwd"));
        assert!(SecurityValidator::is_dangerous_field_name(
            "_internal_field"
        ));
        assert!(SecurityValidator::is_dangerous_field_name("field\x00name")); // null byte
        assert!(SecurityValidator::is_dangerous_field_name("field%%%name")); // too many special chars

        // These should be safe
        assert!(!SecurityValidator::is_dangerous_field_name("customer_id"));
        assert!(!SecurityValidator::is_dangerous_field_name("total-amount"));
        assert!(!SecurityValidator::is_dangerous_field_name("a"));
        assert!(!SecurityValidator::is_dangerous_field_name("_")); // single underscore is ok
    }

    #[test]
    fn test_set_field_action_validation() {
        let limits = SecurityConfig::default();

        // Valid SetField action
        let safe_action =
            ApiAction::SetField { field: "customer_discount".to_string(), value: json!(15.5) };
        let result = SecurityValidator::validate_action(&safe_action, &limits);
        matches!(result, SecurityValidationResult::Safe);

        // Dangerous field name
        let dangerous_action =
            ApiAction::SetField { field: "__proto__".to_string(), value: json!(15.5) };
        let result = SecurityValidator::validate_action(&dangerous_action, &limits);
        matches!(result, SecurityValidationResult::Rejected { .. });

        // Field name too long
        let long_field_action = ApiAction::SetField { field: "a".repeat(1000), value: json!(15.5) };
        let result = SecurityValidator::validate_action(&long_field_action, &limits);
        matches!(result, SecurityValidationResult::Rejected { .. });
    }

    #[test]
    fn test_injection_pattern_detection() {
        // SQL injection patterns
        assert!(SecurityValidator::contains_injection_patterns(
            "' or '1'='1"
        ));
        assert!(SecurityValidator::contains_injection_patterns(
            "'; DROP TABLE users; --"
        ));
        assert!(SecurityValidator::contains_injection_patterns(
            "UNION SELECT password FROM users"
        ));

        // NoSQL injection patterns
        assert!(SecurityValidator::contains_injection_patterns(
            "$where: function() { return true; }"
        ));
        assert!(SecurityValidator::contains_injection_patterns(
            "{'$ne': null}"
        ));

        // JavaScript injection patterns
        assert!(SecurityValidator::contains_injection_patterns(
            "<script>alert('xss')</script>"
        ));
        assert!(SecurityValidator::contains_injection_patterns(
            "javascript:alert(1)"
        ));

        // Command injection patterns
        assert!(SecurityValidator::contains_injection_patterns("; rm -rf /"));
        assert!(SecurityValidator::contains_injection_patterns(
            "| cat /etc/passwd"
        ));

        // Template injection
        assert!(SecurityValidator::contains_injection_patterns("{{7*7}}"));

        // Null byte injection
        assert!(SecurityValidator::contains_injection_patterns("test\0.txt"));

        // Safe values should pass
        assert!(!SecurityValidator::contains_injection_patterns(
            "normal_value"
        ));
        assert!(!SecurityValidator::contains_injection_patterns(
            "customer_id_123"
        ));
        assert!(!SecurityValidator::contains_injection_patterns(
            "email@example.com"
        ));
    }

    #[test]
    fn test_condition_validation() {
        use crate::types::{ApiCondition, ApiSimpleOperator};

        let limits = SecurityConfig::default();

        // Safe condition
        let safe_condition = ApiCondition::Simple {
            field: "customer_id".to_string(),
            operator: ApiSimpleOperator::Equal,
            value: json!("12345"),
        };
        let result = SecurityValidator::validate_condition(&safe_condition, &limits);
        matches!(result, SecurityValidationResult::Safe);

        // Dangerous field name in condition
        let dangerous_condition = ApiCondition::Simple {
            field: "__proto__".to_string(),
            operator: ApiSimpleOperator::Equal,
            value: json!("value"),
        };
        let result = SecurityValidator::validate_condition(&dangerous_condition, &limits);
        matches!(result, SecurityValidationResult::Rejected { .. });

        // Injection pattern in value
        let injection_condition = ApiCondition::Simple {
            field: "user_input".to_string(),
            operator: ApiSimpleOperator::Equal,
            value: json!("' or '1'='1"),
        };
        let result = SecurityValidator::validate_condition(&injection_condition, &limits);
        matches!(result, SecurityValidationResult::Rejected { .. });
    }
}
