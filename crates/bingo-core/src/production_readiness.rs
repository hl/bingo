//! Production Readiness Validation and Configuration
//!
//! This module provides comprehensive production readiness checks for the Bingo RETE Rules Engine,
//! ensuring all components are properly configured for production deployment.

use crate::error::BingoResult;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Production readiness configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionConfig {
    /// Service configuration
    pub service: ServiceConfig,
    /// Performance tuning settings
    pub performance: PerformanceConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Monitoring and observability settings
    pub monitoring: MonitoringConfig,
    /// Resource limits and quotas
    pub resources: ResourceConfig,
}

/// Service-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name for identification
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Environment (production, staging, development)
    pub environment: String,
    /// gRPC server configuration
    pub grpc_address: String,
    /// HTTP metrics endpoint
    pub metrics_address: Option<String>,
    /// Health check endpoint
    pub health_check_address: Option<String>,
}

/// Performance tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum concurrent connections
    pub max_connections: u32,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Keep-alive timeout in seconds
    pub keepalive_timeout_s: u64,
    /// Thread pool size for processing
    pub thread_pool_size: Option<usize>,
    /// Memory pool configuration
    pub memory_pools_enabled: bool,
    /// Cache configuration
    pub cache_enabled: bool,
    /// Cache TTL in seconds
    pub cache_ttl_s: u64,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// TLS enabled for all connections
    pub tls_enabled: bool,
    /// Mutual TLS authentication
    pub mtls_enabled: bool,
    /// TLS certificate path
    pub tls_cert_path: Option<String>,
    /// TLS private key path
    pub tls_key_path: Option<String>,
    /// Rate limiting enabled
    pub rate_limiting_enabled: bool,
    /// Rate limit requests per minute
    pub rate_limit_rpm: u32,
    /// Authentication required
    pub auth_required: bool,
    /// JWT validation
    pub jwt_validation_enabled: bool,
}

/// Monitoring and observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Structured logging enabled
    pub structured_logging: bool,
    /// Log level (error, warn, info, debug, trace)
    pub log_level: String,
    /// Metrics collection enabled
    pub metrics_enabled: bool,
    /// Metrics export interval in seconds
    pub metrics_interval_s: u64,
    /// Distributed tracing enabled
    pub tracing_enabled: bool,
    /// Jaeger endpoint for trace export
    pub jaeger_endpoint: Option<String>,
    /// Prometheus metrics endpoint
    pub prometheus_endpoint: Option<String>,
}

/// Resource limits and quotas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Maximum memory usage in MB
    pub max_memory_mb: u64,
    /// Maximum CPU percentage (100 = 1 core)
    pub max_cpu_percent: u32,
    /// Maximum rule count
    pub max_rules: u32,
    /// Maximum fact count
    pub max_facts: u64,
    /// Maximum request size in bytes
    pub max_request_size_bytes: u64,
    /// Maximum response size in bytes
    pub max_response_size_bytes: u64,
}

impl Default for ProductionConfig {
    fn default() -> Self {
        Self {
            service: ServiceConfig {
                service_name: "bingo-grpc".to_string(),
                service_version: "1.0.0".to_string(),
                environment: "production".to_string(),
                grpc_address: "0.0.0.0:50051".to_string(),
                metrics_address: Some("0.0.0.0:9090".to_string()),
                health_check_address: Some("0.0.0.0:8080".to_string()),
            },
            performance: PerformanceConfig {
                max_connections: 1000,
                request_timeout_ms: 30000,
                keepalive_timeout_s: 60,
                thread_pool_size: None, // Auto-detect
                memory_pools_enabled: true,
                cache_enabled: true,
                cache_ttl_s: 300,
            },
            security: SecurityConfig {
                tls_enabled: true,
                mtls_enabled: false,
                tls_cert_path: Some("/etc/ssl/certs/server.crt".to_string()),
                tls_key_path: Some("/etc/ssl/private/server.key".to_string()),
                rate_limiting_enabled: true,
                rate_limit_rpm: 10000,
                auth_required: true,
                jwt_validation_enabled: true,
            },
            monitoring: MonitoringConfig {
                structured_logging: true,
                log_level: "info".to_string(),
                metrics_enabled: true,
                metrics_interval_s: 15,
                tracing_enabled: true,
                jaeger_endpoint: Some("http://jaeger:14268/api/traces".to_string()),
                prometheus_endpoint: Some("http://prometheus:9090".to_string()),
            },
            resources: ResourceConfig {
                max_memory_mb: 4096,
                max_cpu_percent: 200, // 2 cores
                max_rules: 10000,
                max_facts: crate::constants::limits::MAX_FACTS,
                max_request_size_bytes: 10485760,  // 10MB
                max_response_size_bytes: 10485760, // 10MB
            },
        }
    }
}

/// Production readiness validation results
#[derive(Debug, Clone)]
pub struct ReadinessReport {
    /// Overall readiness status
    pub ready: bool,
    /// Service checks
    pub service_checks: Vec<CheckResult>,
    /// Performance checks
    pub performance_checks: Vec<CheckResult>,
    /// Security checks
    pub security_checks: Vec<CheckResult>,
    /// Monitoring checks
    pub monitoring_checks: Vec<CheckResult>,
    /// Resource checks
    pub resource_checks: Vec<CheckResult>,
    /// Summary statistics
    pub summary: ReadinessSummary,
}

/// Individual check result
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// Check name
    pub name: String,
    /// Check status
    pub status: CheckStatus,
    /// Optional message
    pub message: Option<String>,
    /// Severity level
    pub severity: CheckSeverity,
    /// Recommendations for fixes
    pub recommendations: Vec<String>,
}

/// Check status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum CheckStatus {
    /// Check passed
    Pass,
    /// Check failed but not critical
    Warning,
    /// Check failed and is critical
    Fail,
    /// Check could not be completed
    Unknown,
}

/// Check severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum CheckSeverity {
    /// Critical for production operation
    Critical,
    /// Important but not blocking
    High,
    /// Recommended improvement
    Medium,
    /// Nice to have
    Low,
}

/// Summary of readiness checks
#[derive(Debug, Clone)]
pub struct ReadinessSummary {
    /// Total checks performed
    pub total_checks: usize,
    /// Number of passing checks
    pub passed: usize,
    /// Number of warnings
    pub warnings: usize,
    /// Number of failures
    pub failures: usize,
    /// Number of unknown results
    pub unknown: usize,
    /// Overall readiness score (0.0 to 1.0)
    pub readiness_score: f64,
}

/// Production readiness validator
#[derive(Debug)]
pub struct ProductionReadinessValidator {
    /// Configuration to validate against
    config: ProductionConfig,
}

impl ProductionReadinessValidator {
    /// Create a new production readiness validator
    pub fn new(config: ProductionConfig) -> Self {
        Self { config }
    }

    /// Create validator with default production configuration
    pub fn with_default_config() -> Self {
        Self::new(ProductionConfig::default())
    }

    /// Perform comprehensive production readiness validation
    pub fn validate(&self) -> BingoResult<ReadinessReport> {
        info!("Starting production readiness validation");

        let mut service_checks = self.validate_service_config()?;
        let mut performance_checks = self.validate_performance_config()?;
        let mut security_checks = self.validate_security_config()?;
        let mut monitoring_checks = self.validate_monitoring_config()?;
        let mut resource_checks = self.validate_resource_config()?;

        // Collect all checks
        let mut all_checks = Vec::new();
        all_checks.append(&mut service_checks);
        all_checks.append(&mut performance_checks);
        all_checks.append(&mut security_checks);
        all_checks.append(&mut monitoring_checks);
        all_checks.append(&mut resource_checks);

        // Calculate summary
        let summary = self.calculate_summary(&all_checks);
        let ready = summary.failures == 0 && summary.readiness_score >= 0.8;

        let report = ReadinessReport {
            ready,
            service_checks: self.validate_service_config()?,
            performance_checks: self.validate_performance_config()?,
            security_checks: self.validate_security_config()?,
            monitoring_checks: self.validate_monitoring_config()?,
            resource_checks: self.validate_resource_config()?,
            summary,
        };

        info!(
            "Production readiness validation completed: ready={}, score={:.2}",
            report.ready, report.summary.readiness_score
        );

        Ok(report)
    }

    /// Validate service configuration
    fn validate_service_config(&self) -> BingoResult<Vec<CheckResult>> {
        let mut checks = Vec::new();

        // Service name validation
        checks.push(if self.config.service.service_name.is_empty() {
            CheckResult {
                name: "Service Name".to_string(),
                status: CheckStatus::Fail,
                message: Some("Service name cannot be empty".to_string()),
                severity: CheckSeverity::Critical,
                recommendations: vec!["Set SERVICE_NAME environment variable".to_string()],
            }
        } else {
            CheckResult {
                name: "Service Name".to_string(),
                status: CheckStatus::Pass,
                message: Some(format!(
                    "Service name: {}",
                    self.config.service.service_name
                )),
                severity: CheckSeverity::Critical,
                recommendations: vec![],
            }
        });

        // Environment validation
        checks.push(if self.config.service.environment == "production" {
            CheckResult {
                name: "Environment".to_string(),
                status: CheckStatus::Pass,
                message: Some("Environment set to production".to_string()),
                severity: CheckSeverity::High,
                recommendations: vec![],
            }
        } else {
            CheckResult {
                name: "Environment".to_string(),
                status: CheckStatus::Warning,
                message: Some(format!("Environment: {}", self.config.service.environment)),
                severity: CheckSeverity::High,
                recommendations: vec![
                    "Set environment to 'production' for production deployment".to_string(),
                ],
            }
        });

        // gRPC address validation
        checks.push(if self.config.service.grpc_address.contains("0.0.0.0:") {
            CheckResult {
                name: "gRPC Address".to_string(),
                status: CheckStatus::Pass,
                message: Some(format!(
                    "gRPC listening on: {}",
                    self.config.service.grpc_address
                )),
                severity: CheckSeverity::Critical,
                recommendations: vec![],
            }
        } else {
            CheckResult {
                name: "gRPC Address".to_string(),
                status: CheckStatus::Fail,
                message: Some(
                    "gRPC address should bind to 0.0.0.0 for container deployment".to_string(),
                ),
                severity: CheckSeverity::Critical,
                recommendations: vec!["Set GRPC_LISTEN_ADDRESS=0.0.0.0:50051".to_string()],
            }
        });

        Ok(checks)
    }

    /// Validate performance configuration
    fn validate_performance_config(&self) -> BingoResult<Vec<CheckResult>> {
        let mut checks = Vec::new();

        // Connection limits
        checks.push(if self.config.performance.max_connections >= 100 {
            CheckResult {
                name: "Connection Limits".to_string(),
                status: CheckStatus::Pass,
                message: Some(format!(
                    "Max connections: {}",
                    self.config.performance.max_connections
                )),
                severity: CheckSeverity::High,
                recommendations: vec![],
            }
        } else {
            CheckResult {
                name: "Connection Limits".to_string(),
                status: CheckStatus::Warning,
                message: Some("Connection limit may be too low for production".to_string()),
                severity: CheckSeverity::Medium,
                recommendations: vec![
                    "Consider increasing max_connections to at least 1000".to_string(),
                ],
            }
        });

        // Memory pools
        checks.push(if self.config.performance.memory_pools_enabled {
            CheckResult {
                name: "Memory Pools".to_string(),
                status: CheckStatus::Pass,
                message: Some("Memory pools enabled for optimization".to_string()),
                severity: CheckSeverity::High,
                recommendations: vec![],
            }
        } else {
            CheckResult {
                name: "Memory Pools".to_string(),
                status: CheckStatus::Warning,
                message: Some("Memory pools disabled - reduced performance".to_string()),
                severity: CheckSeverity::Medium,
                recommendations: vec!["Enable memory pools for better performance".to_string()],
            }
        });

        // Caching
        checks.push(if self.config.performance.cache_enabled {
            CheckResult {
                name: "Caching".to_string(),
                status: CheckStatus::Pass,
                message: Some(format!(
                    "Caching enabled with {}s TTL",
                    self.config.performance.cache_ttl_s
                )),
                severity: CheckSeverity::High,
                recommendations: vec![],
            }
        } else {
            CheckResult {
                name: "Caching".to_string(),
                status: CheckStatus::Warning,
                message: Some("Caching disabled - reduced performance".to_string()),
                severity: CheckSeverity::Medium,
                recommendations: vec!["Enable caching for better performance".to_string()],
            }
        });

        Ok(checks)
    }

    /// Validate security configuration
    fn validate_security_config(&self) -> BingoResult<Vec<CheckResult>> {
        let mut checks = Vec::new();

        // TLS validation
        checks.push(if self.config.security.tls_enabled {
            CheckResult {
                name: "TLS Encryption".to_string(),
                status: CheckStatus::Pass,
                message: Some("TLS encryption enabled".to_string()),
                severity: CheckSeverity::Critical,
                recommendations: vec![],
            }
        } else {
            CheckResult {
                name: "TLS Encryption".to_string(),
                status: CheckStatus::Fail,
                message: Some("TLS encryption disabled - security risk".to_string()),
                severity: CheckSeverity::Critical,
                recommendations: vec![
                    "Enable TLS encryption for production deployment".to_string(),
                    "Provide valid TLS certificates".to_string(),
                ],
            }
        });

        // Authentication validation
        checks.push(if self.config.security.auth_required {
            CheckResult {
                name: "Authentication".to_string(),
                status: CheckStatus::Pass,
                message: Some("Authentication required".to_string()),
                severity: CheckSeverity::Critical,
                recommendations: vec![],
            }
        } else {
            CheckResult {
                name: "Authentication".to_string(),
                status: CheckStatus::Fail,
                message: Some("Authentication disabled - security risk".to_string()),
                severity: CheckSeverity::Critical,
                recommendations: vec![
                    "Enable authentication for production deployment".to_string(),
                ],
            }
        });

        // Rate limiting validation
        checks.push(if self.config.security.rate_limiting_enabled {
            CheckResult {
                name: "Rate Limiting".to_string(),
                status: CheckStatus::Pass,
                message: Some(format!(
                    "Rate limiting: {} RPM",
                    self.config.security.rate_limit_rpm
                )),
                severity: CheckSeverity::High,
                recommendations: vec![],
            }
        } else {
            CheckResult {
                name: "Rate Limiting".to_string(),
                status: CheckStatus::Warning,
                message: Some("Rate limiting disabled - DoS risk".to_string()),
                severity: CheckSeverity::High,
                recommendations: vec!["Enable rate limiting to prevent abuse".to_string()],
            }
        });

        Ok(checks)
    }

    /// Validate monitoring configuration
    fn validate_monitoring_config(&self) -> BingoResult<Vec<CheckResult>> {
        let mut checks = Vec::new();

        // Structured logging
        checks.push(if self.config.monitoring.structured_logging {
            CheckResult {
                name: "Structured Logging".to_string(),
                status: CheckStatus::Pass,
                message: Some(format!(
                    "Structured logging at {} level",
                    self.config.monitoring.log_level
                )),
                severity: CheckSeverity::High,
                recommendations: vec![],
            }
        } else {
            CheckResult {
                name: "Structured Logging".to_string(),
                status: CheckStatus::Warning,
                message: Some("Structured logging disabled".to_string()),
                severity: CheckSeverity::Medium,
                recommendations: vec![
                    "Enable structured logging for better observability".to_string(),
                ],
            }
        });

        // Metrics collection
        checks.push(if self.config.monitoring.metrics_enabled {
            CheckResult {
                name: "Metrics Collection".to_string(),
                status: CheckStatus::Pass,
                message: Some("Metrics collection enabled".to_string()),
                severity: CheckSeverity::High,
                recommendations: vec![],
            }
        } else {
            CheckResult {
                name: "Metrics Collection".to_string(),
                status: CheckStatus::Warning,
                message: Some("Metrics collection disabled".to_string()),
                severity: CheckSeverity::High,
                recommendations: vec!["Enable metrics collection for monitoring".to_string()],
            }
        });

        // Distributed tracing
        checks.push(if self.config.monitoring.tracing_enabled {
            CheckResult {
                name: "Distributed Tracing".to_string(),
                status: CheckStatus::Pass,
                message: Some("Distributed tracing enabled".to_string()),
                severity: CheckSeverity::Medium,
                recommendations: vec![],
            }
        } else {
            CheckResult {
                name: "Distributed Tracing".to_string(),
                status: CheckStatus::Warning,
                message: Some("Distributed tracing disabled".to_string()),
                severity: CheckSeverity::Medium,
                recommendations: vec!["Enable distributed tracing for debugging".to_string()],
            }
        });

        Ok(checks)
    }

    /// Validate resource configuration
    fn validate_resource_config(&self) -> BingoResult<Vec<CheckResult>> {
        let mut checks = Vec::new();

        // Memory limits
        checks.push(if self.config.resources.max_memory_mb >= 1024 {
            CheckResult {
                name: "Memory Limits".to_string(),
                status: CheckStatus::Pass,
                message: Some(format!(
                    "Memory limit: {}MB",
                    self.config.resources.max_memory_mb
                )),
                severity: CheckSeverity::High,
                recommendations: vec![],
            }
        } else {
            CheckResult {
                name: "Memory Limits".to_string(),
                status: CheckStatus::Warning,
                message: Some("Memory limit may be too low for production".to_string()),
                severity: CheckSeverity::Medium,
                recommendations: vec![
                    "Consider increasing memory limit to at least 2GB".to_string(),
                ],
            }
        });

        // Rule limits
        checks.push(if self.config.resources.max_rules >= 1000 {
            CheckResult {
                name: "Rule Capacity".to_string(),
                status: CheckStatus::Pass,
                message: Some(format!("Max rules: {}", self.config.resources.max_rules)),
                severity: CheckSeverity::Medium,
                recommendations: vec![],
            }
        } else {
            CheckResult {
                name: "Rule Capacity".to_string(),
                status: CheckStatus::Warning,
                message: Some("Rule limit may be too low".to_string()),
                severity: CheckSeverity::Low,
                recommendations: vec![
                    "Consider increasing rule limit for enterprise use".to_string(),
                ],
            }
        });

        // Fact limits
        checks.push(if self.config.resources.max_facts >= 100000 {
            CheckResult {
                name: "Fact Capacity".to_string(),
                status: CheckStatus::Pass,
                message: Some(format!("Max facts: {}", self.config.resources.max_facts)),
                severity: CheckSeverity::Medium,
                recommendations: vec![],
            }
        } else {
            CheckResult {
                name: "Fact Capacity".to_string(),
                status: CheckStatus::Warning,
                message: Some("Fact limit may be too low".to_string()),
                severity: CheckSeverity::Low,
                recommendations: vec![
                    "Consider increasing fact limit for large datasets".to_string(),
                ],
            }
        });

        Ok(checks)
    }

    /// Calculate summary statistics
    fn calculate_summary(&self, checks: &[CheckResult]) -> ReadinessSummary {
        let total_checks = checks.len();
        let passed = checks.iter().filter(|c| c.status == CheckStatus::Pass).count();
        let warnings = checks.iter().filter(|c| c.status == CheckStatus::Warning).count();
        let failures = checks.iter().filter(|c| c.status == CheckStatus::Fail).count();
        let unknown = checks.iter().filter(|c| c.status == CheckStatus::Unknown).count();

        // Calculate readiness score based on weighted checks
        let mut score = 0.0;
        let mut total_weight = 0.0;

        for check in checks {
            let weight = match check.severity {
                CheckSeverity::Critical => 4.0,
                CheckSeverity::High => 3.0,
                CheckSeverity::Medium => 2.0,
                CheckSeverity::Low => 1.0,
            };

            let check_score = match check.status {
                CheckStatus::Pass => 1.0,
                CheckStatus::Warning => 0.5,
                CheckStatus::Fail => 0.0,
                CheckStatus::Unknown => 0.25,
            };

            score += weight * check_score;
            total_weight += weight;
        }

        let readiness_score = if total_weight > 0.0 {
            score / total_weight
        } else {
            0.0
        };

        ReadinessSummary { total_checks, passed, warnings, failures, unknown, readiness_score }
    }

    /// Generate production readiness report in markdown format
    pub fn generate_report(&self, report: &ReadinessReport) -> String {
        let mut markdown = String::new();

        markdown.push_str("# Production Readiness Report\n\n");
        markdown.push_str(&format!(
            "**Overall Status:** {}\n\n",
            if report.ready {
                "✅ READY"
            } else {
                "❌ NOT READY"
            }
        ));
        markdown.push_str(&format!(
            "**Readiness Score:** {:.2}%\n\n",
            report.summary.readiness_score * 100.0
        ));

        // Summary
        markdown.push_str("## Summary\n\n");
        markdown.push_str(&format!(
            "- **Total Checks:** {}\n",
            report.summary.total_checks
        ));
        markdown.push_str(&format!("- **Passed:** {} ✅\n", report.summary.passed));
        markdown.push_str(&format!("- **Warnings:** {} ⚠️\n", report.summary.warnings));
        markdown.push_str(&format!("- **Failures:** {} ❌\n", report.summary.failures));
        markdown.push_str(&format!("- **Unknown:** {} ❓\n\n", report.summary.unknown));

        // Detailed results
        self.add_section_to_report(
            &mut markdown,
            "Service Configuration",
            &report.service_checks,
        );
        self.add_section_to_report(
            &mut markdown,
            "Performance Configuration",
            &report.performance_checks,
        );
        self.add_section_to_report(
            &mut markdown,
            "Security Configuration",
            &report.security_checks,
        );
        self.add_section_to_report(
            &mut markdown,
            "Monitoring Configuration",
            &report.monitoring_checks,
        );
        self.add_section_to_report(
            &mut markdown,
            "Resource Configuration",
            &report.resource_checks,
        );

        markdown
    }

    /// Add a section to the markdown report
    fn add_section_to_report(&self, markdown: &mut String, title: &str, checks: &[CheckResult]) {
        markdown.push_str(&format!("## {title}\n\n"));

        for check in checks {
            let status_icon = match check.status {
                CheckStatus::Pass => "✅",
                CheckStatus::Warning => "⚠️",
                CheckStatus::Fail => "❌",
                CheckStatus::Unknown => "❓",
            };

            markdown.push_str(&format!(
                "### {} {} - {}\n\n",
                status_icon,
                check.name,
                check.severity_display()
            ));

            if let Some(message) = &check.message {
                markdown.push_str(&format!("**Status:** {message}\n\n"));
            }

            if !check.recommendations.is_empty() {
                markdown.push_str("**Recommendations:**\n");
                for rec in &check.recommendations {
                    markdown.push_str(&format!("- {rec}\n"));
                }
                markdown.push('\n');
            }
        }
    }
}

impl CheckSeverity {
    /// Get display string for severity
    fn display(&self) -> &'static str {
        match self {
            CheckSeverity::Critical => "CRITICAL",
            CheckSeverity::High => "HIGH",
            CheckSeverity::Medium => "MEDIUM",
            CheckSeverity::Low => "LOW",
        }
    }
}

impl CheckResult {
    /// Get severity display string
    fn severity_display(&self) -> &'static str {
        self.severity.display()
    }
}

/// Load production configuration from environment variables
pub fn load_config_from_env() -> ProductionConfig {
    let mut config = ProductionConfig::default();

    // Service configuration
    if let Ok(name) = std::env::var("SERVICE_NAME") {
        config.service.service_name = name;
    }
    if let Ok(version) = std::env::var("SERVICE_VERSION") {
        config.service.service_version = version;
    }
    if let Ok(env) = std::env::var("BINGO_ENVIRONMENT") {
        config.service.environment = env;
    }
    if let Ok(addr) = std::env::var("GRPC_LISTEN_ADDRESS") {
        config.service.grpc_address = addr;
    }

    // Performance configuration
    if let Ok(connections) = std::env::var("MAX_CONNECTIONS") {
        if let Ok(val) = connections.parse() {
            config.performance.max_connections = val;
        }
    }
    if let Ok(timeout) = std::env::var("REQUEST_TIMEOUT_MS") {
        if let Ok(val) = timeout.parse() {
            config.performance.request_timeout_ms = val;
        }
    }

    // Security configuration
    if let Ok(tls) = std::env::var("TLS_ENABLED") {
        config.security.tls_enabled = tls.to_lowercase() == "true";
    }
    if let Ok(auth) = std::env::var("AUTH_REQUIRED") {
        config.security.auth_required = auth.to_lowercase() == "true";
    }
    if let Ok(rate_limit) = std::env::var("RATE_LIMIT_RPM") {
        if let Ok(val) = rate_limit.parse() {
            config.security.rate_limit_rpm = val;
        }
    }

    // Monitoring configuration
    if let Ok(log_level) = std::env::var("RUST_LOG") {
        // Extract base log level from RUST_LOG format
        let level = log_level.split('=').next_back().unwrap_or("info");
        config.monitoring.log_level = level.to_string();
    }
    if let Ok(metrics) = std::env::var("METRICS_ENABLED") {
        config.monitoring.metrics_enabled = metrics.to_lowercase() == "true";
    }
    if let Ok(tracing) = std::env::var("TRACING_ENABLED") {
        config.monitoring.tracing_enabled = tracing.to_lowercase() == "true";
    }

    config
}

/// Run production readiness check and return report
pub fn check_production_readiness() -> BingoResult<ReadinessReport> {
    let config = load_config_from_env();
    let validator = ProductionReadinessValidator::new(config);
    validator.validate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_config_default() {
        let config = ProductionConfig::default();
        assert_eq!(config.service.service_name, "bingo-grpc");
        assert_eq!(config.service.environment, "production");
        assert!(config.security.tls_enabled);
        assert!(config.monitoring.metrics_enabled);
    }

    #[test]
    fn test_readiness_validator() {
        let config = ProductionConfig::default();
        let validator = ProductionReadinessValidator::new(config);
        let report = validator.validate().expect("Validation should succeed");

        assert!(report.summary.total_checks > 0);
        assert!(report.summary.readiness_score >= 0.0);
        assert!(report.summary.readiness_score <= 1.0);
    }

    #[test]
    fn test_report_generation() {
        let config = ProductionConfig::default();
        let validator = ProductionReadinessValidator::new(config);
        let report = validator.validate().expect("Validation should succeed");
        let markdown = validator.generate_report(&report);

        assert!(markdown.contains("# Production Readiness Report"));
        assert!(markdown.contains("## Summary"));
        assert!(markdown.contains("Service Configuration"));
    }
}
