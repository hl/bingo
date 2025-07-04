//! Circuit breaker patterns for external dependencies
//!
//! This module provides circuit breaker implementation to protect against
//! cascading failures when external services (like Redis) become unavailable.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};


/// Circuit breaker states following the classic pattern
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum CircuitBreakerState {
    /// Normal operation - requests pass through
    Closed,
    /// Failing fast - requests are rejected immediately
    Open,
    /// Testing if service has recovered - limited requests allowed
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,
    /// Time to wait before transitioning from Open to HalfOpen
    pub recovery_timeout: Duration,
    /// Number of successful calls needed in HalfOpen to close the circuit
    pub success_threshold: u32,
    /// Timeout for individual operations
    pub call_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            success_threshold: 3,
            call_timeout: Duration::from_secs(5),
        }
    }
}

/// Circuit breaker statistics for monitoring
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CircuitBreakerStats {
    pub state: CircuitBreakerState,
    pub failure_count: u32,
    pub success_count: u32,
    pub total_calls: u64,
    pub rejected_calls: u64,
    #[serde(skip_serializing, skip_deserializing)]
    pub last_failure_time: Option<Instant>,
    #[serde(skip_serializing, skip_deserializing)]
    pub last_state_change: Instant,
}


/// Internal state of the circuit breaker
#[derive(Debug)]
struct CircuitBreakerInner {
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
    total_calls: u64,
    rejected_calls: u64,
    last_failure_time: Option<Instant>,
    last_state_change: Instant,
    config: CircuitBreakerConfig,
}

impl CircuitBreakerInner {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            total_calls: 0,
            rejected_calls: 0,
            last_failure_time: None,
            last_state_change: Instant::now(),
            config,
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.config.recovery_timeout {
                        self.transition_to_half_open();
                        true
                    } else {
                        self.rejected_calls += 1;
                        false
                    }
                } else {
                    // No previous failure recorded, allow execution
                    true
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Allow limited number of calls to test recovery
                true
            }
        }
    }

    fn record_success(&mut self) {
        self.total_calls += 1;
        self.success_count += 1;

        match self.state {
            CircuitBreakerState::HalfOpen => {
                if self.success_count >= self.config.success_threshold {
                    self.transition_to_closed();
                }
            }
            CircuitBreakerState::Open => {
                // This shouldn't happen, but if it does, transition to half-open
                self.transition_to_half_open();
            }
            CircuitBreakerState::Closed => {
                // Reset failure count on successful operation
                self.failure_count = 0;
            }
        }
    }

    fn record_failure(&mut self) {
        self.total_calls += 1;
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.config.failure_threshold {
                    self.transition_to_open();
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Any failure in half-open immediately opens the circuit
                self.transition_to_open();
            }
            CircuitBreakerState::Open => {
                // Already open, just update failure time
            }
        }
    }

    fn transition_to_open(&mut self) {
        if self.state != CircuitBreakerState::Open {
            warn!(
                previous_state = ?self.state,
                failure_count = self.failure_count,
                "Circuit breaker transitioning to OPEN state"
            );
            self.state = CircuitBreakerState::Open;
            self.last_state_change = Instant::now();
            self.success_count = 0; // Reset success count
        }
    }

    fn transition_to_half_open(&mut self) {
        if self.state != CircuitBreakerState::HalfOpen {
            info!(
                previous_state = ?self.state,
                "Circuit breaker transitioning to HALF_OPEN state"
            );
            self.state = CircuitBreakerState::HalfOpen;
            self.last_state_change = Instant::now();
            self.success_count = 0; // Reset success count for testing
        }
    }

    fn transition_to_closed(&mut self) {
        if self.state != CircuitBreakerState::Closed {
            info!(
                previous_state = ?self.state,
                success_count = self.success_count,
                "Circuit breaker transitioning to CLOSED state"
            );
            self.state = CircuitBreakerState::Closed;
            self.last_state_change = Instant::now();
            self.failure_count = 0; // Reset failure count
            self.success_count = 0; // Reset success count
        }
    }

    fn get_stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.state.clone(),
            failure_count: self.failure_count,
            success_count: self.success_count,
            total_calls: self.total_calls,
            rejected_calls: self.rejected_calls,
            last_failure_time: self.last_failure_time,
            last_state_change: self.last_state_change,
        }
    }
}

/// Circuit breaker for protecting external service calls
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    inner: Arc<Mutex<CircuitBreakerInner>>,
    name: String,
}

/// Circuit breaker error types
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker '{name}' is open - calls are being rejected")]
    CircuitOpen { name: String },
    #[error("Operation timed out after {timeout:?}")]
    Timeout { timeout: Duration },
    #[error("External service error: {source}")]
    ServiceError { source: anyhow::Error },

    #[error("Internal circuit breaker error: {message}")]
    Internal { message: String },
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        debug!(
            name = %name,
            failure_threshold = config.failure_threshold,
            recovery_timeout = ?config.recovery_timeout,
            "Creating circuit breaker"
        );

        Self { inner: Arc::new(Mutex::new(CircuitBreakerInner::new(config))), name }
    }

    /// Execute a closure with circuit breaker protection
    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, anyhow::Error>>,
    {
        // Check if we can execute
        let can_execute = {
            let mut inner = self
                .inner
                .lock()
                .map_err(|_| CircuitBreakerError::Internal {
                    message: "Mutex poisoned".to_string(),
                })?;
            inner.can_execute()
        };

        if !can_execute {
            return Err(CircuitBreakerError::CircuitOpen { name: self.name.clone() });
        }

        // Get timeout from config
        let timeout = {
            let inner = self
                .inner
                .lock()
                .map_err(|_| CircuitBreakerError::Internal { message: "Mutex poisoned".to_string() })?;
            inner.config.call_timeout
        };

        // Execute with timeout
        let result = tokio::time::timeout(timeout, operation()).await;

        match result {
            Ok(Ok(value)) => {
                // Success
                {
                    let mut inner = self
                        .inner
                        .lock()
                        .map_err(|_| CircuitBreakerError::Internal { message: "Mutex poisoned".to_string() })?;
                    inner.record_success();
                }
                debug!(name = %self.name, "Circuit breaker call succeeded");
                Ok(value)
            }
            Ok(Err(error)) => {
                // Service error
                {
                    let mut inner = self
                        .inner
                        .lock()
                        .map_err(|_| CircuitBreakerError::Internal { message: "Mutex poisoned".to_string() })?;
                    inner.record_failure();
                }
                warn!(
                    name = %self.name,
                    error = %error,
                    "Circuit breaker call failed"
                );
                Err(CircuitBreakerError::ServiceError { source: error })
            }
            Err(_) => {
                // Timeout
                {
                    let mut inner = self
                        .inner
                        .lock()
                        .map_err(|_| CircuitBreakerError::Internal { message: "Mutex poisoned".to_string() })?;
                    inner.record_failure();
                }
                warn!(
                    name = %self.name,
                    timeout = ?timeout,
                    "Circuit breaker call timed out"
                );
                Err(CircuitBreakerError::Timeout { timeout })
            }
        }
    }

    /// Get circuit breaker statistics
    pub fn stats(&self) -> CircuitBreakerStats {
        match self.inner.lock() {
            Ok(inner) => inner.get_stats(),
            Err(_) => CircuitBreakerStats {
                state: CircuitBreakerState::Open,
                failure_count: 0,
                success_count: 0,
                total_calls: 0,
                rejected_calls: 0,
                last_failure_time: None,
                last_state_change: Instant::now(),
            },
        }
    }

    /// Get circuit breaker name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Manually open the circuit (for testing or emergency situations)
    pub fn force_open(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.transition_to_open();
        }
        warn!(name = %self.name, "Circuit breaker manually forced to OPEN state");
    }

    /// Manually close the circuit (for testing or recovery situations)
    pub fn force_close(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.transition_to_closed();
        }
        info!(name = %self.name, "Circuit breaker manually forced to CLOSED state");
    }
}

/// Circuit breaker registry for managing multiple circuit breakers
#[derive(Debug, Default)]
pub struct CircuitBreakerRegistry {
    breakers: Arc<Mutex<std::collections::HashMap<String, CircuitBreaker>>>,
}

impl CircuitBreakerRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a circuit breaker
    pub fn register(&self, breaker: CircuitBreaker) {
        let name = breaker.name().to_string();
        let mut breakers = self.breakers.lock().unwrap();
        breakers.insert(name.clone(), breaker);
        debug!(name = %name, "Circuit breaker registered");
    }

    /// Get a circuit breaker by name
    pub fn get(&self, name: &str) -> Option<CircuitBreaker> {
        let breakers = self.breakers.lock().unwrap();
        breakers.get(name).cloned()
    }

    /// Get all circuit breakers
    pub fn all(&self) -> Vec<CircuitBreaker> {
        let breakers = self.breakers.lock().unwrap();
        breakers.values().cloned().collect()
    }

    /// Get statistics for all circuit breakers
    pub fn all_stats(&self) -> std::collections::HashMap<String, CircuitBreakerStats> {
        let breakers = self.breakers.lock().unwrap();
        breakers.iter().map(|(name, breaker)| (name.clone(), breaker.stats())).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_millis(100),
            success_threshold: 2,
            call_timeout: Duration::from_millis(100),
        };
        let breaker = CircuitBreaker::new("test".to_string(), config);

        // Successful operation should work
        let result = breaker.call(|| async { Ok::<i32, anyhow::Error>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        let stats = breaker.stats();
        assert_eq!(stats.state, CircuitBreakerState::Closed);
        assert_eq!(stats.total_calls, 1);
        assert_eq!(stats.failure_count, 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_millis(100),
            success_threshold: 2,
            call_timeout: Duration::from_millis(100),
        };
        let breaker = CircuitBreaker::new("test".to_string(), config);

        // First failure
        let result = breaker
            .call(|| async { Err::<i32, anyhow::Error>(anyhow::anyhow!("error")) })
            .await;
        assert!(result.is_err());

        // Second failure - should open the circuit
        let result = breaker
            .call(|| async { Err::<i32, anyhow::Error>(anyhow::anyhow!("error")) })
            .await;
        assert!(result.is_err());

        let stats = breaker.stats();
        assert_eq!(stats.state, CircuitBreakerState::Open);
        assert_eq!(stats.failure_count, 2);
    }

    #[tokio::test]
    async fn test_circuit_breaker_rejects_when_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_secs(10), // Long timeout
            success_threshold: 2,
            call_timeout: Duration::from_millis(100),
        };
        let breaker = CircuitBreaker::new("test".to_string(), config);

        // Cause failure to open circuit
        let _result = breaker
            .call(|| async { Err::<i32, anyhow::Error>(anyhow::anyhow!("error")) })
            .await;

        // Next call should be rejected
        let result = breaker.call(|| async { Ok::<i32, anyhow::Error>(42) }).await;
        assert!(matches!(
            result,
            Err(CircuitBreakerError::CircuitOpen { .. })
        ));

        let stats = breaker.stats();
        assert_eq!(stats.rejected_calls, 1);
    }

    #[test]
    fn test_circuit_breaker_registry() {
        let registry = CircuitBreakerRegistry::new();
        let config = CircuitBreakerConfig::default();
        let breaker = CircuitBreaker::new("test".to_string(), config);

        registry.register(breaker.clone());

        let retrieved = registry.get("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test");

        let all_breakers = registry.all();
        assert_eq!(all_breakers.len(), 1);
    }
}
