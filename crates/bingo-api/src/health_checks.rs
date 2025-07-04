//! Enhanced health check system with dependency validation
//!
//! This module provides comprehensive health checks that validate not only
//! the service itself but also its external dependencies like Redis, databases,
//! and other critical services.

use crate::cache::UnifiedCacheProvider;
use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerRegistry, CircuitBreakerStats};
use crate::config::BingoConfig;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use utoipa::ToSchema;

/// Overall health status of the service
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Service is fully operational
    Healthy,
    /// Service is operational but with some degraded functionality
    Degraded,
    /// Service is not operational
    Unhealthy,
}

/// Health status of an individual dependency
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DependencyHealth {
    /// Name of the dependency
    pub name: String,
    /// Current status of the dependency
    pub status: HealthStatus,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Last check timestamp
    pub last_checked: DateTime<Utc>,
    /// Additional details about the dependency
    pub details: Option<String>,
    /// Circuit breaker statistics if applicable
    #[serde(skip_deserializing)]
    pub circuit_breaker: Option<CircuitBreakerStats>,
}

/// Comprehensive health response with dependency validation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EnhancedHealthResponse {
    /// Overall service status
    pub status: HealthStatus,
    /// Service version
    pub version: String,
    /// Service uptime in seconds
    pub uptime_seconds: u64,
    /// Individual dependency health checks
    pub dependencies: HashMap<String, DependencyHealth>,
    /// Service startup time
    pub startup_time: DateTime<Utc>,
    /// Current timestamp
    pub timestamp: DateTime<Utc>,
    /// System resource information
    pub system: SystemHealth,
    /// Summary of dependency statuses
    pub summary: HealthSummary,
}

/// System resource health information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemHealth {
    /// Available memory in bytes
    pub memory_available_bytes: u64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Number of active threads
    pub active_threads: usize,
    /// Process ID
    pub process_id: u32,
}

/// Summary of dependency health status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthSummary {
    /// Total number of dependencies
    pub total_dependencies: usize,
    /// Number of healthy dependencies
    pub healthy_dependencies: usize,
    /// Number of degraded dependencies
    pub degraded_dependencies: usize,
    /// Number of unhealthy dependencies
    pub unhealthy_dependencies: usize,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Timeout for individual dependency checks
    pub check_timeout: Duration,
    /// Include detailed system information
    pub include_system_info: bool,
    /// Include circuit breaker statistics
    pub include_circuit_breaker_stats: bool,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_timeout: Duration::from_secs(5),
            include_system_info: true,
            include_circuit_breaker_stats: true,
        }
    }
}

/// Health check service for validating service and dependency health
#[derive(Debug)]
pub struct HealthCheckService {
    config: HealthCheckConfig,
    startup_time: DateTime<Utc>,
    circuit_breaker_registry: Option<Arc<CircuitBreakerRegistry>>,
}

impl HealthCheckService {
    /// Create a new health check service
    pub fn new(config: HealthCheckConfig) -> Self {
        Self { config, startup_time: Utc::now(), circuit_breaker_registry: None }
    }

    /// Set the circuit breaker registry for dependency monitoring
    pub fn with_circuit_breaker_registry(mut self, registry: Arc<CircuitBreakerRegistry>) -> Self {
        self.circuit_breaker_registry = Some(registry);
        self
    }

    /// Perform comprehensive health check
    pub async fn check_health(
        &self,
        cache_provider: Option<&dyn UnifiedCacheProvider>,
        config: &BingoConfig,
    ) -> EnhancedHealthResponse {
        let start_time = Instant::now();
        let mut dependencies = HashMap::new();

        // Check cache dependency (Redis)
        if let Some(cache) = cache_provider {
            dependencies.insert("cache".to_string(), self.check_cache_health(cache).await);
        }

        // Check configuration validity
        dependencies.insert(
            "configuration".to_string(),
            self.check_configuration_health(config).await,
        );

        // Get system health information
        let system = if self.config.include_system_info {
            self.get_system_health().await
        } else {
            SystemHealth {
                memory_available_bytes: 0,
                memory_usage_percent: 0.0,
                active_threads: 0,
                process_id: 0,
            }
        };

        // Add circuit breaker information if available
        if let Some(registry) = &self.circuit_breaker_registry {
            if self.config.include_circuit_breaker_stats {
                self.add_circuit_breaker_stats(&mut dependencies, registry).await;
            }
        }

        // Calculate overall status
        let overall_status = self.calculate_overall_status(&dependencies);

        // Generate summary
        let summary = self.generate_summary(&dependencies);

        let uptime_seconds = (Utc::now() - self.startup_time).num_seconds() as u64;

        let response = EnhancedHealthResponse {
            status: overall_status,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds,
            dependencies,
            startup_time: self.startup_time,
            timestamp: Utc::now(),
            system,
            summary,
        };

        let check_duration = start_time.elapsed();
        info!(
            duration_ms = check_duration.as_millis(),
            status = ?response.status,
            dependencies = response.summary.total_dependencies,
            "Health check completed"
        );

        response
    }

    /// Check cache health (Redis or in-memory)
    async fn check_cache_health(&self, cache: &dyn UnifiedCacheProvider) -> DependencyHealth {
        let start_time = Instant::now();
        let test_key = format!("health_check_{}", Utc::now().timestamp());

        // Try to get cache stats as a health check
        let (status, details) =
            match tokio::time::timeout(self.config.check_timeout, cache.get_stats()).await {
                Ok(_stats) => (
                    HealthStatus::Healthy,
                    Some("Cache operations successful".to_string()),
                ),
                Err(_) => (
                    HealthStatus::Unhealthy,
                    Some("Cache operation timed out".to_string()),
                ),
            };

        let response_time = start_time.elapsed();

        DependencyHealth {
            name: "cache".to_string(),
            status,
            response_time_ms: response_time.as_millis() as u64,
            last_checked: Utc::now(),
            details,
            circuit_breaker: None, // Will be added separately if available
        }
    }

    /// Check configuration health
    async fn check_configuration_health(&self, config: &BingoConfig) -> DependencyHealth {
        let start_time = Instant::now();

        // Validate configuration sanity
        let (status, details) = if config.security.max_rules_per_request > 0
            && config.security.max_facts_per_request > 0
            && config.limits.max_body_size_mb > 0
        {
            (
                HealthStatus::Healthy,
                Some("Configuration validation passed".to_string()),
            )
        } else {
            (
                HealthStatus::Unhealthy,
                Some("Invalid configuration values detected".to_string()),
            )
        };

        let response_time = start_time.elapsed();

        DependencyHealth {
            name: "configuration".to_string(),
            status,
            response_time_ms: response_time.as_millis() as u64,
            last_checked: Utc::now(),
            details,
            circuit_breaker: None,
        }
    }

    /// Get system health information
    async fn get_system_health(&self) -> SystemHealth {
        // Basic system information
        let memory_info = self.get_memory_info().await;
        let process_id = std::process::id();

        // Estimate active threads (this is approximate)
        let active_threads = std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1);

        SystemHealth {
            memory_available_bytes: memory_info.0,
            memory_usage_percent: memory_info.1,
            active_threads,
            process_id,
        }
    }

    /// Get memory information (available bytes, usage percentage)
    async fn get_memory_info(&self) -> (u64, f64) {
        // Try to get actual memory usage, fallback to estimates
        match bingo_core::memory::get_memory_usage() {
            Ok(rss_bytes) => {
                // Estimate total system memory (this is a rough approximation)
                let estimated_total = 8 * 1024 * 1024 * 1024; // 8GB estimate
                let usage_percent = (rss_bytes as f64 / estimated_total as f64) * 100.0;
                (estimated_total - rss_bytes as u64, usage_percent.min(100.0))
            }
            Err(_) => (0, 0.0),
        }
    }

    /// Add circuit breaker statistics to dependencies
    async fn add_circuit_breaker_stats(
        &self,
        dependencies: &mut HashMap<String, DependencyHealth>,
        registry: &CircuitBreakerRegistry,
    ) {
        let all_stats = registry.all_stats();

        for (name, stats) in all_stats {
            // Convert circuit breaker state to health status
            let health_status = match stats.state {
                crate::circuit_breaker::CircuitBreakerState::Closed => HealthStatus::Healthy,
                crate::circuit_breaker::CircuitBreakerState::HalfOpen => HealthStatus::Degraded,
                crate::circuit_breaker::CircuitBreakerState::Open => HealthStatus::Unhealthy,
            };

            let detail_message = format!(
                "Circuit breaker state: {:?}, failures: {}, total calls: {}",
                stats.state, stats.failure_count, stats.total_calls
            );

            // Update existing dependency or create new one
            if let Some(existing) = dependencies.get_mut(&name) {
                existing.circuit_breaker = Some(stats.clone());
                // Downgrade health status if circuit breaker is not healthy
                if health_status == HealthStatus::Unhealthy
                    || health_status == HealthStatus::Degraded
                {
                    existing.status = health_status;
                    existing.details = Some(detail_message);
                }
            } else {
                // Create new dependency entry for circuit breaker
                dependencies.insert(
                    name.clone(),
                    DependencyHealth {
                        name: name.clone(),
                        status: health_status,
                        response_time_ms: 0,
                        last_checked: Utc::now(),
                        details: Some(detail_message),
                        circuit_breaker: Some(stats),
                    },
                );
            }
        }
    }

    /// Calculate overall service status based on dependency health
    fn calculate_overall_status(
        &self,
        dependencies: &HashMap<String, DependencyHealth>,
    ) -> HealthStatus {
        if dependencies.is_empty() {
            return HealthStatus::Healthy;
        }

        let unhealthy_count = dependencies
            .values()
            .filter(|dep| dep.status == HealthStatus::Unhealthy)
            .count();

        let degraded_count =
            dependencies.values().filter(|dep| dep.status == HealthStatus::Degraded).count();

        // If any critical dependency is unhealthy, service is unhealthy
        if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Generate summary of dependency health
    fn generate_summary(&self, dependencies: &HashMap<String, DependencyHealth>) -> HealthSummary {
        let total = dependencies.len();
        let healthy =
            dependencies.values().filter(|dep| dep.status == HealthStatus::Healthy).count();
        let degraded =
            dependencies.values().filter(|dep| dep.status == HealthStatus::Degraded).count();
        let unhealthy = dependencies
            .values()
            .filter(|dep| dep.status == HealthStatus::Unhealthy)
            .count();

        HealthSummary {
            total_dependencies: total,
            healthy_dependencies: healthy,
            degraded_dependencies: degraded,
            unhealthy_dependencies: unhealthy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_calculation() {
        let service = HealthCheckService::new(HealthCheckConfig::default());

        // Test empty dependencies
        let empty_deps = HashMap::new();
        assert_eq!(
            service.calculate_overall_status(&empty_deps),
            HealthStatus::Healthy
        );

        // Test all healthy
        let mut healthy_deps = HashMap::new();
        healthy_deps.insert(
            "test".to_string(),
            DependencyHealth {
                name: "test".to_string(),
                status: HealthStatus::Healthy,
                response_time_ms: 10,
                last_checked: Utc::now(),
                details: None,
                circuit_breaker: None,
            },
        );
        assert_eq!(
            service.calculate_overall_status(&healthy_deps),
            HealthStatus::Healthy
        );

        // Test with degraded dependency
        healthy_deps.get_mut("test").unwrap().status = HealthStatus::Degraded;
        assert_eq!(
            service.calculate_overall_status(&healthy_deps),
            HealthStatus::Degraded
        );

        // Test with unhealthy dependency
        healthy_deps.get_mut("test").unwrap().status = HealthStatus::Unhealthy;
        assert_eq!(
            service.calculate_overall_status(&healthy_deps),
            HealthStatus::Unhealthy
        );
    }

    #[test]
    fn test_summary_generation() {
        let service = HealthCheckService::new(HealthCheckConfig::default());

        let mut deps = HashMap::new();
        deps.insert(
            "healthy".to_string(),
            DependencyHealth {
                name: "healthy".to_string(),
                status: HealthStatus::Healthy,
                response_time_ms: 10,
                last_checked: Utc::now(),
                details: None,
                circuit_breaker: None,
            },
        );
        deps.insert(
            "degraded".to_string(),
            DependencyHealth {
                name: "degraded".to_string(),
                status: HealthStatus::Degraded,
                response_time_ms: 50,
                last_checked: Utc::now(),
                details: None,
                circuit_breaker: None,
            },
        );

        let summary = service.generate_summary(&deps);
        assert_eq!(summary.total_dependencies, 2);
        assert_eq!(summary.healthy_dependencies, 1);
        assert_eq!(summary.degraded_dependencies, 1);
        assert_eq!(summary.unhealthy_dependencies, 0);
    }
}
