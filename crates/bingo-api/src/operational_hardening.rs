//! Operational hardening for production deployment
//!
//! This module provides additional security layers including rate limiting,
//! concurrency control, and request monitoring to prevent DoS attacks.

use crate::config;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tracing::{debug, instrument, warn};

/// Concurrency limiter using a semaphore
#[derive(Clone)]
pub struct ConcurrencyLimiter {
    semaphore: Arc<Semaphore>,
    max_permits: usize,
}

impl ConcurrencyLimiter {
    pub fn new(max_concurrent: usize) -> Self {
        Self { semaphore: Arc::new(Semaphore::new(max_concurrent)), max_permits: max_concurrent }
    }

    /// Middleware function for concurrency limiting
    #[instrument(skip(self, request, next))]
    pub async fn limit_concurrency(&self, request: Request, next: Next) -> Response {
        // Try to acquire a permit (non-blocking check first)
        let permit = match self.semaphore.try_acquire() {
            Ok(permit) => permit,
            Err(_) => {
                // No permits available - check if we should wait or reject
                let available = self.semaphore.available_permits();
                warn!(
                    available_permits = available,
                    max_permits = self.max_permits,
                    "Concurrency limit reached, rejecting request"
                );

                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    "Server is at maximum capacity. Please try again later.",
                )
                    .into_response();
            }
        };

        debug!(
            available_permits = self.semaphore.available_permits(),
            max_permits = self.max_permits,
            "Request acquired concurrency permit"
        );

        // Process the request
        let response = next.run(request).await;

        // Permit is automatically released when dropped
        drop(permit);

        response
    }
}

#[cfg(feature = "redis-cache")]
use redis::AsyncCommands;

/// In-memory client tracking for rate limiting fallback
#[derive(Debug, Clone)]
struct ClientInfo {
    request_count: u32,
    window_start: Instant,
}

impl ClientInfo {
    fn new() -> Self {
        Self { request_count: 1, window_start: Instant::now() }
    }

    fn is_expired(&self, window_duration: Duration) -> bool {
        self.window_start.elapsed() > window_duration
    }

    fn reset(&mut self) {
        self.request_count = 1;
        self.window_start = Instant::now();
    }

    fn increment(&mut self) {
        self.request_count += 1;
    }
}

#[derive(Clone)]
pub struct RateLimiter {
    #[cfg(feature = "redis-cache")]
    redis_conn: Option<redis::aio::MultiplexedConnection>,
    requests_per_minute: u32,
    window_duration: Duration,
    // In-memory fallback
    in_memory_clients: Arc<Mutex<HashMap<String, ClientInfo>>>,
}

impl RateLimiter {
    pub async fn new(redis_url: Option<String>, requests_per_minute: u32) -> anyhow::Result<Self> {
        #[cfg(feature = "redis-cache")]
        let redis_conn = if let Some(url) = redis_url {
            match redis::Client::open(url) {
                Ok(client) => match client.get_multiplexed_async_connection().await {
                    Ok(conn) => {
                        debug!("Redis rate limiter connected successfully");
                        Some(conn)
                    }
                    Err(e) => {
                        warn!(
                            "Redis connection failed, falling back to in-memory rate limiting: {}",
                            e
                        );
                        None
                    }
                },
                Err(e) => {
                    warn!(
                        "Redis client creation failed, falling back to in-memory rate limiting: {}",
                        e
                    );
                    None
                }
            }
        } else {
            debug!("No Redis URL provided, using in-memory rate limiting");
            None
        };

        Ok(Self {
            #[cfg(feature = "redis-cache")]
            redis_conn,
            requests_per_minute,
            window_duration: Duration::from_secs(60), // 1 minute window
            in_memory_clients: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Middleware function for rate limiting
    #[instrument(skip(self, request, next))]
    pub async fn limit_rate(&self, request: Request, next: Next) -> Response {
        let client_ip = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .split(',')
            .next()
            .unwrap_or("unknown")
            .trim()
            .to_string();

        // Try Redis first, fall back to in-memory on failure
        let is_rate_limited = self.check_rate_limit(&client_ip).await;

        if is_rate_limited {
            warn!(
                client_ip = %client_ip,
                requests_per_minute = self.requests_per_minute,
                "Rate limit exceeded for client"
            );

            return (
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", "60")],
                format!(
                    "Rate limit exceeded. Maximum {} requests per minute allowed.",
                    self.requests_per_minute
                ),
            )
                .into_response();
        }

        debug!(
            client_ip = %client_ip,
            "Request allowed by rate limiter"
        );

        next.run(request).await
    }

    /// Check rate limit using Redis or in-memory fallback
    async fn check_rate_limit(&self, client_ip: &str) -> bool {
        #[cfg(feature = "redis-cache")]
        if let Some(mut conn) = self.redis_conn.clone() {
            // Try Redis rate limiting
            match self.check_redis_rate_limit(client_ip, &mut conn).await {
                Ok(is_limited) => return is_limited,
                Err(e) => {
                    warn!(
                        "Redis rate limit check failed, falling back to in-memory: {}",
                        e
                    );
                }
            }
        }

        // Fall back to in-memory rate limiting
        self.check_in_memory_rate_limit(client_ip).await
    }

    #[cfg(feature = "redis-cache")]
    async fn check_redis_rate_limit(
        &self,
        client_ip: &str,
        conn: &mut redis::aio::MultiplexedConnection,
    ) -> Result<bool, redis::RedisError> {
        let key = format!("rate_limit:{}", client_ip);

        let (count, ttl): (u32, i64) =
            redis::pipe().atomic().incr(&key, 1).ttl(&key).query_async(conn).await?;

        if ttl == -1 {
            // Key just created, set expiration
            let _: () = conn.expire(&key, self.window_duration.as_secs() as i64).await?;
        }

        Ok(count > self.requests_per_minute)
    }

    async fn check_in_memory_rate_limit(&self, client_ip: &str) -> bool {
        let mut clients = self.in_memory_clients.lock().await;

        let is_limited = match clients.get_mut(client_ip) {
            Some(client_info) => {
                if client_info.is_expired(self.window_duration) {
                    // Reset expired window
                    client_info.reset();
                    false
                } else {
                    client_info.increment();
                    client_info.request_count > self.requests_per_minute
                }
            }
            None => {
                // New client
                clients.insert(client_ip.to_string(), ClientInfo::new());
                false
            }
        };

        // Clean up expired entries periodically (simple cleanup)
        if clients.len() > 1000 {
            clients.retain(|_, info| !info.is_expired(self.window_duration));
        }

        is_limited
    }

    /// Clean up old entries periodically
    pub async fn cleanup_old_entries(&self) {
        #[cfg(feature = "redis-cache")]
        if self.redis_conn.is_some() {
            debug!("Redis-backed rate limiter does not require explicit cleanup.");
            return;
        }

        // Clean up in-memory entries
        let mut clients = self.in_memory_clients.lock().await;
        let initial_size = clients.len();
        clients.retain(|_, info| !info.is_expired(self.window_duration));
        let final_size = clients.len();
        debug!(
            "Cleaned up {} expired rate limit entries",
            initial_size - final_size
        );
    }
}

#[cfg(not(feature = "redis-cache"))]
impl RateLimiter {
    pub async fn new(_redis_url: Option<String>, requests_per_minute: u32) -> anyhow::Result<Self> {
        debug!("Redis feature disabled, using in-memory rate limiting only");
        Ok(Self {
            requests_per_minute,
            window_duration: Duration::from_secs(60),
            in_memory_clients: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Middleware function for rate limiting (in-memory only)
    #[instrument(skip(self, request, next))]
    pub async fn limit_rate(&self, request: Request, next: Next) -> Response {
        let client_ip = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .split(',')
            .next()
            .unwrap_or("unknown")
            .trim()
            .to_string();

        let is_rate_limited = self.check_in_memory_rate_limit(&client_ip).await;

        if is_rate_limited {
            warn!(
                client_ip = %client_ip,
                requests_per_minute = self.requests_per_minute,
                "Rate limit exceeded for client"
            );

            return (
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", "60")],
                format!(
                    "Rate limit exceeded. Maximum {} requests per minute allowed.",
                    self.requests_per_minute
                ),
            )
                .into_response();
        }

        debug!(
            client_ip = %client_ip,
            "Request allowed by rate limiter"
        );

        next.run(request).await
    }

    async fn check_in_memory_rate_limit(&self, client_ip: &str) -> bool {
        let mut clients = self.in_memory_clients.lock().await;

        let is_limited = match clients.get_mut(client_ip) {
            Some(client_info) => {
                if client_info.is_expired(self.window_duration) {
                    client_info.reset();
                    false
                } else {
                    client_info.increment();
                    client_info.request_count > self.requests_per_minute
                }
            }
            None => {
                clients.insert(client_ip.to_string(), ClientInfo::new());
                false
            }
        };

        // Clean up expired entries periodically
        if clients.len() > 1000 {
            clients.retain(|_, info| !info.is_expired(self.window_duration));
        }

        is_limited
    }

    pub async fn cleanup_old_entries(&self) {
        let mut clients = self.in_memory_clients.lock().await;
        let initial_size = clients.len();
        clients.retain(|_, info| !info.is_expired(self.window_duration));
        let final_size = clients.len();
        debug!(
            "Cleaned up {} expired rate limit entries",
            initial_size - final_size
        );
    }
}

/// Request monitoring middleware
#[derive(Clone)]
pub struct RequestMonitor {
    enable_monitoring: bool,
}

impl RequestMonitor {
    pub fn new(enable_monitoring: bool) -> Self {
        Self { enable_monitoring }
    }

    /// Middleware function for request monitoring
    #[instrument(skip(self, request, next))]
    pub async fn monitor_request(&self, request: Request, next: Next) -> Response {
        if !self.enable_monitoring {
            return next.run(request).await;
        }

        let start = Instant::now();
        let method = request.method().clone();
        let path = request.uri().path().to_string();

        // Extract content length for monitoring
        let content_length = request
            .headers()
            .get("content-length")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        debug!(
            method = %method,
            path = %path,
            content_length = content_length,
            "Processing request"
        );

        let response = next.run(request).await;

        let duration = start.elapsed();
        let status = response.status();

        if self.enable_monitoring {
            debug!(
                method = %method,
                path = %path,
                status = %status,
                duration_ms = duration.as_millis(),
                content_length = content_length,
                "Request completed"
            );

            // Log slow requests
            if duration > Duration::from_secs(5) {
                warn!(
                    method = %method,
                    path = %path,
                    status = %status,
                    duration_ms = duration.as_millis(),
                    "Slow request detected"
                );
            }
        }

        response
    }
}

/// Builder for creating hardened middleware stack
pub struct HardeningBuilder {
    config: config::HardeningConfig,
    redis_url: Option<String>,
}

impl HardeningBuilder {
    pub fn new(config: config::HardeningConfig, redis_url: Option<String>) -> Self {
        Self { config, redis_url }
    }

    /// Build the hardening middleware components
    pub async fn build(
        self,
    ) -> anyhow::Result<(
        Option<ConcurrencyLimiter>,
        Option<RateLimiter>,
        Option<RequestMonitor>,
        Option<tower_http::timeout::TimeoutLayer>,
    )> {
        #[cfg(not(feature = "disable_concurrency_limiter"))]
        let concurrency_limiter = if self.config.enable_concurrency_limiter {
            Some(ConcurrencyLimiter::new(self.config.max_concurrent_requests))
        } else {
            None
        };
        #[cfg(feature = "disable_concurrency_limiter")]
        let concurrency_limiter = None;

        #[cfg(not(feature = "disable_rate_limiter"))]
        let rate_limiter = if self.config.enable_rate_limiter {
            Some(RateLimiter::new(self.redis_url.clone(), self.config.requests_per_minute).await?)
        } else {
            None
        };
        #[cfg(feature = "disable_rate_limiter")]
        let rate_limiter = None;

        #[cfg(not(feature = "disable_request_monitor"))]
        let request_monitor = if self.config.enable_request_monitor {
            Some(RequestMonitor::new(true))
        } else {
            None
        };
        #[cfg(feature = "disable_request_monitor")]
        let request_monitor = None;

        #[cfg(not(feature = "disable_timeout"))]
        let timeout_layer = if self.config.enable_timeout {
            Some(tower_http::timeout::TimeoutLayer::new(Duration::from_secs(
                self.config.request_timeout_seconds,
            )))
        } else {
            None
        };
        #[cfg(feature = "disable_timeout")]
        let timeout_layer = None;

        Ok((
            concurrency_limiter,
            rate_limiter,
            request_monitor,
            timeout_layer,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;

    #[test]
    fn test_hardening_config_defaults() {
        let config = config::HardeningConfig::default();
        assert!(config.enable_concurrency_limiter);
        assert!(config.enable_rate_limiter);
        assert!(config.enable_request_monitor);
        assert_eq!(config.max_concurrent_requests, 100);
        assert_eq!(config.requests_per_minute, 300);
    }

    #[cfg(not(any(
        feature = "disable_concurrency_limiter",
        feature = "disable_rate_limiter",
        feature = "disable_request_monitor"
    )))]
    #[tokio::test]
    async fn test_builder_enables_all() {
        let config = config::HardeningConfig::default();
        let (concurrency, rate, monitor, _timeout) =
            HardeningBuilder::new(config, None).build().await.unwrap();
        assert!(concurrency.is_some());
        assert!(rate.is_some());
        assert!(monitor.is_some());
    }

    #[tokio::test]
    async fn test_builder_disables_all() {
        let config = config::HardeningConfig {
            enable_concurrency_limiter: false,
            enable_rate_limiter: false,
            enable_request_monitor: false,
            ..Default::default()
        };
        let (concurrency, rate, monitor, _timeout) =
            HardeningBuilder::new(config, None).build().await.unwrap();
        assert!(concurrency.is_none());
        assert!(rate.is_none());
        assert!(monitor.is_none());
    }

    #[test]
    fn test_concurrency_limiter_creation() {
        let limiter = ConcurrencyLimiter::new(10);
        assert_eq!(limiter.max_permits, 10);
        assert_eq!(limiter.semaphore.available_permits(), 10);
    }

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(None, 10).await.expect("Failed to create rate limiter");

        // Basic functionality test - rate limiter created successfully
        assert_eq!(limiter.requests_per_minute, 10);
    }

    #[test]
    fn test_request_monitor_creation() {
        let monitor = RequestMonitor::new(true);
        assert!(monitor.enable_monitoring);

        let monitor_disabled = RequestMonitor::new(false);
        assert!(!monitor_disabled.enable_monitoring);
    }
}
