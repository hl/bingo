//! Error Testing and Validation Framework
//!
//! This module provides comprehensive testing utilities for validating
//! error handling behavior and ensuring error messages are helpful.

use crate::error::{BingoError, BingoResult, ErrorSeverity};
use crate::error_diagnostics::{
    DiagnosticsConfig, ErrorDiagnostic, ErrorDiagnosticsManager, ErrorSuggestion,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Test suite for error handling validation
#[derive(Debug)]
pub struct ErrorTestSuite {
    /// Test scenarios
    scenarios: Vec<ErrorTestScenario>,
    /// Test configuration
    config: ErrorTestConfig,
    /// Results
    results: Vec<ErrorTestResult>,
}

/// Configuration for error testing
#[derive(Debug, Clone)]
pub struct ErrorTestConfig {
    /// Test error message quality
    pub test_message_quality: bool,
    /// Test suggestion usefulness
    pub test_suggestions: bool,
    /// Test documentation links
    pub test_documentation: bool,
    /// Test error categorization
    pub test_categorization: bool,
    /// Test severity assignment
    pub test_severity: bool,
    /// Minimum suggestion priority threshold
    pub min_suggestion_priority: f64,
}

/// Individual error test scenario
#[derive(Debug, Clone)]
pub struct ErrorTestScenario {
    /// Scenario ID
    pub id: String,
    /// Scenario description
    pub description: String,
    /// Error to test
    pub error: BingoError,
    /// Expected properties
    pub expected: ExpectedErrorProperties,
    /// Test type
    pub test_type: ErrorTestType,
}

/// Expected properties of an error
#[derive(Debug, Clone)]
pub struct ExpectedErrorProperties {
    /// Expected category
    pub category: String,
    /// Expected severity
    pub severity: ErrorSeverity,
    /// Expected message quality score (0.0 to 1.0)
    pub min_message_quality: f64,
    /// Expected number of suggestions
    pub min_suggestions: usize,
    /// Expected recoverability
    pub is_recoverable: bool,
    /// Expected documentation links
    pub min_documentation_links: usize,
}

/// Type of error test
#[derive(Debug, Clone)]
pub enum ErrorTestType {
    /// Test basic error properties
    Basic,
    /// Test error diagnostics generation
    Diagnostics,
    /// Test error suggestion quality
    Suggestions,
    /// Test error message clarity
    MessageClarity,
    /// Test error context extraction
    ContextExtraction,
    /// Test similar error detection
    SimilarErrorDetection,
}

/// Result of an error test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTestResult {
    /// Scenario ID
    pub scenario_id: String,
    /// Test passed
    pub passed: bool,
    /// Score (0.0 to 1.0)
    pub score: f64,
    /// Details about the test
    pub details: HashMap<String, serde_json::Value>,
    /// Issues found
    pub issues: Vec<String>,
    /// Suggestions for improvement
    pub improvements: Vec<String>,
}

/// Error message quality analyzer
#[derive(Debug)]
pub struct ErrorMessageAnalyzer {
    /// Quality criteria
    criteria: Vec<MessageQualityCriterion>,
}

/// Individual quality criterion for error messages
#[derive(Debug, Clone)]
pub struct MessageQualityCriterion {
    /// Criterion name
    pub name: String,
    /// Weight in overall score
    pub weight: f64,
    /// Evaluation function
    pub evaluate: fn(&str) -> f64,
}

/// Error suggestion validator
#[derive(Debug)]
pub struct ErrorSuggestionValidator {
    /// Validation rules
    rules: Vec<SuggestionValidationRule>,
}

/// Validation rule for error suggestions
#[derive(Debug, Clone)]
pub struct SuggestionValidationRule {
    /// Rule name
    pub name: String,
    /// Validation function
    pub validate: fn(&ErrorSuggestion) -> ValidationResult,
}

/// Result of suggestion validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Validation passed
    pub passed: bool,
    /// Score (0.0 to 1.0)
    pub score: f64,
    /// Issues found
    pub issues: Vec<String>,
}

impl Default for ErrorTestConfig {
    fn default() -> Self {
        Self {
            test_message_quality: true,
            test_suggestions: true,
            test_documentation: true,
            test_categorization: true,
            test_severity: true,
            min_suggestion_priority: 0.5,
        }
    }
}

impl ErrorTestSuite {
    /// Create new error test suite
    pub fn new(config: ErrorTestConfig) -> Self {
        Self { scenarios: Vec::new(), config, results: Vec::new() }
    }

    /// Add test scenario
    pub fn add_scenario(&mut self, scenario: ErrorTestScenario) {
        self.scenarios.push(scenario);
    }

    /// Create default test scenarios
    pub fn with_default_scenarios(mut self) -> Self {
        // Rule validation error
        self.add_scenario(ErrorTestScenario {
            id: "rule_validation_basic".to_string(),
            description: "Basic rule validation error".to_string(),
            error: BingoError::rule_validation("Missing required field 'conditions'"),
            expected: ExpectedErrorProperties {
                category: "rule".to_string(),
                severity: ErrorSeverity::Medium,
                min_message_quality: 0.7,
                min_suggestions: 1,
                is_recoverable: true,
                min_documentation_links: 1,
            },
            test_type: ErrorTestType::Basic,
        });

        // Rule compilation error with context
        self.add_scenario(ErrorTestScenario {
            id: "rule_compilation_context".to_string(),
            description: "Rule compilation error with context".to_string(),
            error: BingoError::rule_compilation(
                123,
                "customer_tier_rule",
                "Invalid condition operator 'xyz'",
            ),
            expected: ExpectedErrorProperties {
                category: "rule".to_string(),
                severity: ErrorSeverity::Medium,
                min_message_quality: 0.8,
                min_suggestions: 2,
                is_recoverable: true,
                min_documentation_links: 1,
            },
            test_type: ErrorTestType::ContextExtraction,
        });

        // Condition parsing error
        self.add_scenario(ErrorTestScenario {
            id: "condition_parse_error".to_string(),
            description: "Condition parsing error".to_string(),
            error: BingoError::condition_parse(
                "amount",
                "greater_than",
                "invalid_number",
                "Cannot parse value as number",
            ),
            expected: ExpectedErrorProperties {
                category: "condition".to_string(),
                severity: ErrorSeverity::Medium,
                min_message_quality: 0.8,
                min_suggestions: 1,
                is_recoverable: true,
                min_documentation_links: 1,
            },
            test_type: ErrorTestType::Diagnostics,
        });

        // Fact store error
        self.add_scenario(ErrorTestScenario {
            id: "fact_store_corruption".to_string(),
            description: "Fact store corruption error".to_string(),
            error: BingoError::fact_store(
                "index_lookup",
                "Index corruption detected in fact store",
            ),
            expected: ExpectedErrorProperties {
                category: "fact_store".to_string(),
                severity: ErrorSeverity::High,
                min_message_quality: 0.6,
                min_suggestions: 1,
                is_recoverable: false,
                min_documentation_links: 1,
            },
            test_type: ErrorTestType::Basic,
        });

        // Performance timeout error
        self.add_scenario(ErrorTestScenario {
            id: "performance_timeout".to_string(),
            description: "Performance timeout error".to_string(),
            error: BingoError::performance_timeout(
                "rule_evaluation",
                5000,
                1000,
                "Rule evaluation exceeded timeout",
            ),
            expected: ExpectedErrorProperties {
                category: "performance".to_string(),
                severity: ErrorSeverity::High,
                min_message_quality: 0.7,
                min_suggestions: 2,
                is_recoverable: true,
                min_documentation_links: 1,
            },
            test_type: ErrorTestType::Suggestions,
        });

        // Memory allocation error
        self.add_scenario(ErrorTestScenario {
            id: "memory_exhaustion".to_string(),
            description: "Memory allocation error".to_string(),
            error: BingoError::memory_allocation(
                "fact_pool",
                1024000,
                512000,
                "Insufficient memory for fact allocation",
            ),
            expected: ExpectedErrorProperties {
                category: "memory".to_string(),
                severity: ErrorSeverity::Critical,
                min_message_quality: 0.6,
                min_suggestions: 1,
                is_recoverable: false,
                min_documentation_links: 1,
            },
            test_type: ErrorTestType::Basic,
        });

        // Calculator error
        self.add_scenario(ErrorTestScenario {
            id: "calculator_expression".to_string(),
            description: "Calculator expression error".to_string(),
            error: BingoError::calculator("amount * tax_rate", "Division by zero in expression"),
            expected: ExpectedErrorProperties {
                category: "calculator".to_string(),
                severity: ErrorSeverity::Medium,
                min_message_quality: 0.8,
                min_suggestions: 1,
                is_recoverable: true,
                min_documentation_links: 1,
            },
            test_type: ErrorTestType::MessageClarity,
        });

        // Configuration error
        self.add_scenario(ErrorTestScenario {
            id: "configuration_invalid".to_string(),
            description: "Configuration validation error".to_string(),
            error: BingoError::configuration(
                "max_rules",
                "positive_integer",
                "-5",
                "Configuration value must be positive",
            ),
            expected: ExpectedErrorProperties {
                category: "configuration".to_string(),
                severity: ErrorSeverity::Critical,
                min_message_quality: 0.7,
                min_suggestions: 1,
                is_recoverable: false,
                min_documentation_links: 1,
            },
            test_type: ErrorTestType::Basic,
        });

        self
    }

    /// Run all test scenarios
    pub fn run_tests(&mut self) -> BingoResult<ErrorTestSummary> {
        let mut diagnostics_manager = ErrorDiagnosticsManager::new(DiagnosticsConfig::default());
        let message_analyzer = ErrorMessageAnalyzer::new();
        let suggestion_validator = ErrorSuggestionValidator::new();

        self.results.clear();

        for scenario in &self.scenarios {
            let result = self.run_scenario(
                scenario,
                &mut diagnostics_manager,
                &message_analyzer,
                &suggestion_validator,
            );
            self.results.push(result);
        }

        Ok(self.generate_summary())
    }

    /// Run individual test scenario
    fn run_scenario(
        &self,
        scenario: &ErrorTestScenario,
        diagnostics_manager: &mut ErrorDiagnosticsManager,
        message_analyzer: &ErrorMessageAnalyzer,
        suggestion_validator: &ErrorSuggestionValidator,
    ) -> ErrorTestResult {
        let mut details = HashMap::new();
        let mut issues = Vec::new();
        let mut improvements = Vec::new();
        let mut scores = Vec::new();

        // Test basic error properties
        if self.config.test_categorization {
            let category_score =
                self.test_categorization(&scenario.error, &scenario.expected, &mut issues);
            scores.push(category_score);
            details.insert(
                "category_score".to_string(),
                serde_json::Value::from(category_score),
            );
        }

        if self.config.test_severity {
            let severity_score =
                self.test_severity(&scenario.error, &scenario.expected, &mut issues);
            scores.push(severity_score);
            details.insert(
                "severity_score".to_string(),
                serde_json::Value::from(severity_score),
            );
        }

        // Create diagnostic and test it
        let diagnostic = diagnostics_manager.create_diagnostic(scenario.error.clone());

        if self.config.test_message_quality {
            let message_score = message_analyzer.analyze_message(&diagnostic.error);
            scores.push(message_score);
            details.insert(
                "message_quality_score".to_string(),
                serde_json::Value::from(message_score),
            );

            if message_score < scenario.expected.min_message_quality {
                issues.push(format!(
                    "Message quality score {:.2} below expected {:.2}",
                    message_score, scenario.expected.min_message_quality
                ));
                improvements.push("Improve error message clarity and specificity".to_string());
            }
        }

        if self.config.test_suggestions {
            let suggestion_score =
                self.test_suggestions(&diagnostic, &scenario.expected, &mut issues);
            scores.push(suggestion_score);
            details.insert(
                "suggestions_score".to_string(),
                serde_json::Value::from(suggestion_score),
            );

            // Validate individual suggestions
            for suggestion in &diagnostic.suggestions {
                let validation = suggestion_validator.validate_suggestion(suggestion);
                if !validation.passed {
                    issues.extend(validation.issues);
                }
            }
        }

        if self.config.test_documentation {
            let docs_score = self.test_documentation(&diagnostic, &scenario.expected, &mut issues);
            scores.push(docs_score);
            details.insert(
                "documentation_score".to_string(),
                serde_json::Value::from(docs_score),
            );
        }

        // Calculate overall score
        let overall_score = if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        let passed = overall_score >= 0.7 && issues.is_empty();

        ErrorTestResult {
            scenario_id: scenario.id.clone(),
            passed,
            score: overall_score,
            details,
            issues,
            improvements,
        }
    }

    /// Test error categorization
    fn test_categorization(
        &self,
        error: &BingoError,
        expected: &ExpectedErrorProperties,
        issues: &mut Vec<String>,
    ) -> f64 {
        let actual_category = error.category();
        if actual_category == expected.category {
            1.0
        } else {
            issues.push(format!(
                "Expected category '{}', got '{}'",
                expected.category, actual_category
            ));
            0.0
        }
    }

    /// Test error severity
    fn test_severity(
        &self,
        error: &BingoError,
        expected: &ExpectedErrorProperties,
        issues: &mut Vec<String>,
    ) -> f64 {
        let actual_severity = error.severity();
        if std::mem::discriminant(&actual_severity) == std::mem::discriminant(&expected.severity) {
            1.0
        } else {
            issues.push(format!(
                "Expected severity '{:?}', got '{:?}'",
                expected.severity, actual_severity
            ));
            0.0
        }
    }

    /// Test error suggestions
    fn test_suggestions(
        &self,
        diagnostic: &ErrorDiagnostic,
        expected: &ExpectedErrorProperties,
        issues: &mut Vec<String>,
    ) -> f64 {
        let suggestion_count = diagnostic.suggestions.len();
        let high_priority_count = diagnostic
            .suggestions
            .iter()
            .filter(|s| s.priority >= self.config.min_suggestion_priority)
            .count();

        let mut score = 0.0;

        if suggestion_count >= expected.min_suggestions {
            score += 0.5;
        } else {
            issues.push(format!(
                "Expected at least {} suggestions, got {}",
                expected.min_suggestions, suggestion_count
            ));
        }

        if high_priority_count > 0 {
            score += 0.5;
        } else {
            issues.push("No high-priority suggestions found".to_string());
        }

        score
    }

    /// Test documentation links
    fn test_documentation(
        &self,
        diagnostic: &ErrorDiagnostic,
        expected: &ExpectedErrorProperties,
        issues: &mut Vec<String>,
    ) -> f64 {
        let docs_count = diagnostic.documentation_links.len();
        if docs_count >= expected.min_documentation_links {
            1.0
        } else {
            issues.push(format!(
                "Expected at least {} documentation links, got {}",
                expected.min_documentation_links, docs_count
            ));
            docs_count as f64 / expected.min_documentation_links as f64
        }
    }

    /// Generate test summary
    fn generate_summary(&self) -> ErrorTestSummary {
        let total_tests = self.results.len();
        let passed_tests = self.results.iter().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;

        let average_score = if total_tests > 0 {
            self.results.iter().map(|r| r.score).sum::<f64>() / total_tests as f64
        } else {
            0.0
        };

        let category_scores = self.calculate_category_scores();
        let top_issues = self.get_top_issues();
        let improvement_recommendations = self.get_improvement_recommendations();

        ErrorTestSummary {
            total_tests,
            passed_tests,
            failed_tests,
            average_score,
            category_scores,
            top_issues,
            improvement_recommendations,
            test_results: self.results.clone(),
        }
    }

    /// Calculate scores by category
    fn calculate_category_scores(&self) -> HashMap<String, f64> {
        let mut category_scores = HashMap::new();
        let mut category_counts = HashMap::new();

        for result in &self.results {
            // Extract category from scenario ID (simplified)
            let category = result.scenario_id.split('_').next().unwrap_or("unknown").to_string();

            *category_scores.entry(category.clone()).or_insert(0.0) += result.score;
            *category_counts.entry(category).or_insert(0) += 1;
        }

        for (category, total_score) in category_scores.iter_mut() {
            if let Some(count) = category_counts.get(category) {
                *total_score /= *count as f64;
            }
        }

        category_scores
    }

    /// Get top issues across all tests
    fn get_top_issues(&self) -> Vec<String> {
        let mut issue_counts = HashMap::new();

        for result in &self.results {
            for issue in &result.issues {
                *issue_counts.entry(issue.clone()).or_insert(0) += 1;
            }
        }

        let mut sorted_issues: Vec<_> = issue_counts.into_iter().collect();
        sorted_issues.sort_by(|a, b| b.1.cmp(&a.1));

        sorted_issues.into_iter().take(5).map(|(issue, _)| issue).collect()
    }

    /// Get improvement recommendations
    fn get_improvement_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        let failed_results: Vec<_> = self.results.iter().filter(|r| !r.passed).collect();

        if !failed_results.is_empty() {
            if failed_results.iter().any(|r| r.details.contains_key("message_quality_score")) {
                recommendations
                    .push("Improve error message clarity and user-friendliness".to_string());
            }

            if failed_results.iter().any(|r| r.details.contains_key("suggestions_score")) {
                recommendations
                    .push("Enhance error suggestion generation and relevance".to_string());
            }

            if failed_results.iter().any(|r| r.details.contains_key("documentation_score")) {
                recommendations.push("Add more comprehensive documentation links".to_string());
            }
        }

        recommendations
    }

    /// Get test results
    pub fn get_results(&self) -> &[ErrorTestResult] {
        &self.results
    }
}

/// Summary of error test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTestSummary {
    /// Total number of tests
    pub total_tests: usize,
    /// Number of passed tests
    pub passed_tests: usize,
    /// Number of failed tests
    pub failed_tests: usize,
    /// Average score across all tests
    pub average_score: f64,
    /// Scores by category
    pub category_scores: HashMap<String, f64>,
    /// Most common issues
    pub top_issues: Vec<String>,
    /// Improvement recommendations
    pub improvement_recommendations: Vec<String>,
    /// Detailed test results
    pub test_results: Vec<ErrorTestResult>,
}

impl Default for ErrorMessageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorMessageAnalyzer {
    /// Create new message analyzer
    pub fn new() -> Self {
        let mut analyzer = Self { criteria: Vec::new() };

        analyzer.add_default_criteria();
        analyzer
    }

    /// Add default quality criteria
    fn add_default_criteria(&mut self) {
        self.criteria.push(MessageQualityCriterion {
            name: "clarity".to_string(),
            weight: 0.3,
            evaluate: |message| {
                // Check for clear, specific language
                let has_specific_details = message.contains("field")
                    || message.contains("value")
                    || message.contains("rule");
                let avoids_jargon =
                    !message.contains("null pointer") && !message.contains("segfault");

                if has_specific_details && avoids_jargon {
                    1.0
                } else if has_specific_details || avoids_jargon {
                    0.5
                } else {
                    0.0
                }
            },
        });

        self.criteria.push(MessageQualityCriterion {
            name: "actionability".to_string(),
            weight: 0.3,
            evaluate: |message| {
                // Check if message suggests what user can do
                let suggests_action = message.contains("check")
                    || message.contains("ensure")
                    || message.contains("verify")
                    || message.contains("correct")
                    || message.contains("fix")
                    || message.contains("update");

                if suggests_action { 1.0 } else { 0.0 }
            },
        });

        self.criteria.push(MessageQualityCriterion {
            name: "context".to_string(),
            weight: 0.2,
            evaluate: |message| {
                // Check if message provides context
                let has_context = message.contains("in")
                    || message.contains("when")
                    || message.contains("for")
                    || message.contains("during");

                if has_context { 1.0 } else { 0.5 }
            },
        });

        self.criteria.push(MessageQualityCriterion {
            name: "tone".to_string(),
            weight: 0.2,
            evaluate: |message| {
                // Check for helpful, non-blaming tone
                let is_helpful = !message.contains("invalid") || message.contains("please");
                let is_non_blaming = !message.contains("you") || !message.contains("your mistake");

                if is_helpful && is_non_blaming {
                    1.0
                } else if is_helpful || is_non_blaming {
                    0.7
                } else {
                    0.3
                }
            },
        });
    }

    /// Analyze message quality
    pub fn analyze_message(&self, message: &str) -> f64 {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;

        for criterion in &self.criteria {
            let score = (criterion.evaluate)(message);
            total_score += score * criterion.weight;
            total_weight += criterion.weight;
        }

        if total_weight > 0.0 {
            total_score / total_weight
        } else {
            0.0
        }
    }
}

impl Default for ErrorSuggestionValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorSuggestionValidator {
    /// Create new suggestion validator
    pub fn new() -> Self {
        let mut validator = Self { rules: Vec::new() };

        validator.add_default_rules();
        validator
    }

    /// Add default validation rules
    fn add_default_rules(&mut self) {
        self.rules.push(SuggestionValidationRule {
            name: "has_clear_title".to_string(),
            validate: |suggestion| {
                let title_length = suggestion.title.len();
                ValidationResult {
                    passed: (5..=100).contains(&title_length),
                    score: if (5..=100).contains(&title_length) {
                        1.0
                    } else {
                        0.0
                    },
                    issues: if title_length < 5 {
                        vec!["Suggestion title too short".to_string()]
                    } else if title_length > 100 {
                        vec!["Suggestion title too long".to_string()]
                    } else {
                        vec![]
                    },
                }
            },
        });

        self.rules.push(SuggestionValidationRule {
            name: "has_actionable_steps".to_string(),
            validate: |suggestion| {
                let has_steps = !suggestion.steps.is_empty();
                ValidationResult {
                    passed: has_steps,
                    score: if has_steps { 1.0 } else { 0.0 },
                    issues: if !has_steps {
                        vec!["Suggestion has no actionable steps".to_string()]
                    } else {
                        vec![]
                    },
                }
            },
        });

        self.rules.push(SuggestionValidationRule {
            name: "reasonable_priority".to_string(),
            validate: |suggestion| {
                let priority_ok = suggestion.priority >= 0.0 && suggestion.priority <= 1.0;
                ValidationResult {
                    passed: priority_ok,
                    score: if priority_ok { 1.0 } else { 0.0 },
                    issues: if !priority_ok {
                        vec!["Suggestion priority out of range [0.0, 1.0]".to_string()]
                    } else {
                        vec![]
                    },
                }
            },
        });
    }

    /// Validate suggestion
    pub fn validate_suggestion(&self, suggestion: &ErrorSuggestion) -> ValidationResult {
        let mut total_score = 0.0;
        let mut all_issues = Vec::new();
        let mut passed_count = 0;

        for rule in &self.rules {
            let result = (rule.validate)(suggestion);
            total_score += result.score;
            all_issues.extend(result.issues);
            if result.passed {
                passed_count += 1;
            }
        }

        let overall_passed = passed_count == self.rules.len();
        let average_score = if !self.rules.is_empty() {
            total_score / self.rules.len() as f64
        } else {
            0.0
        };

        ValidationResult { passed: overall_passed, score: average_score, issues: all_issues }
    }
}

/// Run comprehensive error handling tests
pub fn run_error_tests() -> BingoResult<ErrorTestSummary> {
    let config = ErrorTestConfig::default();
    let mut test_suite = ErrorTestSuite::new(config).with_default_scenarios();
    test_suite.run_tests()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_test_suite_creation() {
        let config = ErrorTestConfig::default();
        let test_suite = ErrorTestSuite::new(config);
        assert!(test_suite.scenarios.is_empty());
    }

    #[test]
    fn test_default_scenarios() {
        let test_suite = ErrorTestSuite::new(ErrorTestConfig::default()).with_default_scenarios();
        assert!(!test_suite.scenarios.is_empty());
    }

    #[test]
    fn test_message_analyzer() {
        let analyzer = ErrorMessageAnalyzer::new();

        let good_message = "Invalid field 'amount' in rule condition. Please check that the field exists and has the correct type.";
        let poor_message = "Error occurred";

        let good_score = analyzer.analyze_message(good_message);
        let poor_score = analyzer.analyze_message(poor_message);

        assert!(good_score > poor_score);
        assert!(good_score > 0.5);
    }

    #[test]
    fn test_suggestion_validator() {
        let validator = ErrorSuggestionValidator::new();

        let good_suggestion = ErrorSuggestion {
            id: "test".to_string(),
            priority: 0.8,
            title: "Fix rule syntax".to_string(),
            description: "Correct the syntax error".to_string(),
            steps: vec![],
            code_examples: vec![],
            effort: crate::error_diagnostics::EffortLevel::Quick,
            risk: crate::error_diagnostics::RiskLevel::VeryLow,
        };

        let result = validator.validate_suggestion(&good_suggestion);
        assert!(result.score > 0.0);
    }
}
