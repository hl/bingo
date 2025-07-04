#![deny(warnings)]
#![allow(
    missing_docs,
    unused_imports,
    unused_variables,
    dead_code,
    unused_assignments,
    unused_mut,
    unreachable_patterns
)]
//! Bingo Rules Engine REST API with OpenAPI specification
//!
//! This module provides a RESTful API for the Bingo RETE rules engine
//! with automatic OpenAPI documentation generation and native JSON types,
//! now with a pluggable caching backend for horizontal scalability.

use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
};

use utoipa::{
    OpenApi,
    openapi::info::{Contact, Info, License},
};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use chrono::{DateTime, Utc};
use anyhow::anyhow;
use fnv::FnvHasher;
use std::hash::Hasher;
use std::sync::{Arc, atomic::AtomicUsize};
use std::time::Duration;
use tower_http::{
    cors::CorsLayer, limit::RequestBodyLimitLayer, timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use bingo_core::{
    Action as CoreAction, ActionResult, ActionType as CoreActionType, BingoEngine,
    Condition as CoreCondition, Fact as CoreFact, FactData as CoreFactData,
    FactValue as CoreFactValue, LogicalOperator, Operator, Rule as CoreRule, RuleExecutionResult,
};

pub mod cache;
pub mod circuit_breaker;
pub mod config;
pub mod error;
pub mod health_checks;
pub mod incremental_processor;
pub mod metrics;
pub mod operational_hardening;
pub mod optimized_conversions;
pub mod security;
pub mod streaming;
pub mod tracing_setup;
pub mod types;

use cache::{CompiledAsset, UnifiedCacheProvider, UnifiedCacheStats};
use config::BingoConfig;
use error::{ApiError, ApiErrorResponse, ApiResult};
use health_checks::{
    DependencyHealth, EnhancedHealthResponse, HealthCheckConfig, HealthCheckService, HealthStatus,
    HealthSummary, SystemHealth,
};
use incremental_processor::{IncrementalProcessor, MemoryMonitor};
use metrics::{ApiMetrics, MetricsMiddleware};
use operational_hardening::{ConcurrencyLimiter, HardeningBuilder, RateLimiter, RequestMonitor};
use security::{SecurityValidationResult, SecurityValidator};
use streaming::{ApiResponse, StreamingResponseBuilder};
use tracing_setup::opentelemetry_tracing_layer;
use types::*;

/// OpenAPI specification
#[derive(OpenApi)]
#[openapi(
    paths(
        health_handler,
        evaluate_handler,
        get_engine_stats_handler,
        register_ruleset_handler,
        get_cache_stats_handler,
        get_security_limits_handler,
        metrics_handler,
    ),
    components(
        schemas(
            ApiFact,
            ApiRule,
            ApiCondition,
            EvaluateRequest,
            RegisterRulesetRequest,
            RegisterRulesetResponse,
            ApiRuleExecutionResult,
            ApiActionResult,
            ApiAction,
            EngineStats,
            HealthResponse,
            EnhancedHealthResponse,
            DependencyHealth,
            HealthStatus,
            HealthSummary,
            SystemHealth,
            ApiErrorResponse,
            ResponseFormat,
            StreamingConfig,
            StreamingMetadata,
            ApiSimpleOperator,
            ApiLogicalOperator,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "evaluation", description = "Stateless rule evaluation endpoints"),
        (name = "rulesets", description = "Ruleset compilation and caching"),
        (name = "engine", description = "Engine statistics and management"),
    ),
    info(
        title = "Bingo Rules Engine API",
        version = "1.0.0",
        description = "High-performance stateless RETE-based rules engine. Evaluation can be done by providing rules and facts directly in each request, or by pre-registering a ruleset and referencing it by ID for improved performance.",
        contact(
            name = "Bingo Rules Engine",
            email = "support@bingo-rules.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server"),
        (url = "https://api.bingo-rules.com", description = "Production server")
    )
)]
struct ApiDoc;

/// Application state for optimized stateless API
#[derive(Debug, Clone)]
pub struct AppState {
    start_time: DateTime<Utc>,
    cache: Arc<dyn UnifiedCacheProvider>,
    metrics: Arc<ApiMetrics>,
    metrics_middleware: MetricsMiddleware,
    config: BingoConfig,
    health_service: Arc<HealthCheckService>,
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        let metrics = Arc::new(
            ApiMetrics::new()
                .map_err(|e| anyhow::anyhow!("Failed to initialize metrics: {}", e))?,
        );
        let metrics_middleware = MetricsMiddleware::new(metrics.clone());

        // Load configuration from file with environment overrides
        let config = BingoConfig::load().apply_profile();
        info!(
            "Configuration loaded: environment={}, max_body_size={}MB",
            config.environment.env_type, config.limits.max_body_size_mb
        );

        // Initialize cache provider based on config
        let cache = init_cache_provider(&config.caching).await?;

        // Initialize health check service
        let health_config = HealthCheckConfig::default();
        let health_service = Arc::new(HealthCheckService::new(health_config));

        Ok(Self {
            start_time: Utc::now(),
            cache,
            metrics,
            metrics_middleware,
            config,
            health_service,
        })
    }
}

/// Factory function to initialize the correct cache provider based on configuration.
async fn init_cache_provider(
    config: &config::CachingConfig,
) -> anyhow::Result<Arc<dyn UnifiedCacheProvider>> {
    use crate::cache::in_memory_provider::InMemoryCacheProvider;
    #[cfg(feature = "redis-cache")]
    use crate::cache::redis_provider::RedisCacheProvider;

    match config.cache_type.as_str() {
        "redis" => {
            #[cfg(feature = "redis-cache")]
            {
                let redis_url = config.redis_url.as_deref().ok_or_else(|| {
                    anyhow::anyhow!("`redis_url` must be provided for `redis` cache type")
                })?;
                info!(url = %redis_url, "Initializing Redis cache provider.");

                let provider = RedisCacheProvider::new(
                    redis_url,
                    config.ruleset_cache_ttl_minutes, // Using ruleset_cache_ttl_minutes as unified TTL
                )
                .await
                .map_err(|e| anyhow::anyhow!("Failed to connect to Redis: {}", e))?;

                Ok(Arc::new(provider))
            }
            #[cfg(not(feature = "redis-cache"))]
            {
                anyhow::bail!(
                    "Redis cache requested but redis-cache feature not enabled. Use 'in_memory' cache type or enable redis-cache feature."
                )
            }
        }
        "in_memory" => {
            info!("Initializing in-memory cache provider.");
            let cache = InMemoryCacheProvider::new(1000, config.ruleset_cache_ttl_minutes); // Using ruleset_cache_ttl_minutes as unified TTL
            Ok(Arc::new(cache))
        }
        other => anyhow::bail!(
            "Unknown cache_type: '{}'. Use 'in_memory' or 'redis'.",
            other
        ),
    }
}

/// Create the web application with OpenAPI documentation
#[instrument]
pub async fn create_app() -> anyhow::Result<Router> {
    info!("Creating Bingo API application with OpenAPI support");

    let state = AppState::new().await?;

    // Build operational hardening middleware from config
    let redis_url = state.config.caching.redis_url.clone();
    let (concurrency_limiter, rate_limiter, request_monitor, timeout_layer) =
        HardeningBuilder::new(state.config.hardening.clone(), redis_url).build().await?;

    info!(
        max_body_size_mb = state.config.limits.max_body_size_mb,
        max_concurrent_requests = state.config.hardening.max_concurrent_requests,
        requests_per_minute = state.config.hardening.requests_per_minute,
        request_timeout_seconds = state.config.hardening.request_timeout_seconds,
        "Operational hardening configured"
    );

    let mut app = Router::new()
        // Health check endpoints
        .route("/health", get(health_handler))
        .route("/health/detailed", get(enhanced_health_handler))
        // Optimized evaluation endpoint (supports both rules and ruleset_id)
        .route("/evaluate", post(evaluate_handler))
        // Ruleset compilation and caching
        .route("/rulesets", post(register_ruleset_handler))
        // Engine and cache statistics
        .route("/engine/stats", get(get_engine_stats_handler))
        .route("/cache/stats", get(get_cache_stats_handler))
        // Security information
        .route("/security/limits", get(get_security_limits_handler))
        // Metrics endpoint for Prometheus
        .route("/metrics", get(metrics_handler))
        // OpenAPI documentation endpoints
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .with_state(state.clone());

    // Conditionally apply hardening middleware
    if let Some(limiter) = concurrency_limiter {
        info!("Enabling concurrency limiter middleware.");
        app = app.layer(axum::middleware::from_fn_with_state(
            limiter,
            |State(limiter): State<ConcurrencyLimiter>, request, next| async move {
                limiter.limit_concurrency(request, next).await
            },
        ));
    }

    if let Some(limiter) = rate_limiter.clone() {
        info!("Enabling rate limiter middleware.");
        app = app.layer(axum::middleware::from_fn_with_state(
            limiter,
            |State(limiter): State<RateLimiter>, request, next| async move {
                limiter.limit_rate(request, next).await
            },
        ));
    }

    if let Some(monitor) = request_monitor {
        info!("Enabling request monitor middleware.");
        app = app.layer(axum::middleware::from_fn_with_state(
            monitor,
            |State(monitor): State<RequestMonitor>, request, next| async move {
                monitor.monitor_request(request, next).await
            },
        ));
    }

    app = app
        .layer(CorsLayer::permissive())
        .layer(axum::middleware::from_fn(opentelemetry_tracing_layer))
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(
            state.config.limits.max_body_size_mb * 1024 * 1024,
        ));

    if state.config.hardening.enable_timeout {
        info!("Enabling request timeout middleware.");
        app = app.layer(TimeoutLayer::new(Duration::from_secs(
            state.config.hardening.request_timeout_seconds,
        )));
    }

    // Start cleanup task for rate limiter if it's enabled
    if let Some(cleanup_rate_limiter) = rate_limiter {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Clean up every 5 minutes
            loop {
                interval.tick().await;
                cleanup_rate_limiter.cleanup_old_entries().await;
            }
        });
    }

    info!("API application created successfully with operational hardening");
    Ok(app)
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unhealthy", body = ApiErrorResponse)
    )
)]
#[instrument(skip(state))]
async fn health_handler(State(state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
    let uptime_seconds = (Utc::now() - state.start_time).num_seconds() as u64;

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: "1.0.0".to_string(),
        uptime_seconds,
        engine_stats: EngineStats {
            total_facts: 0,   // Stateless - no persistent facts
            total_rules: 0,   // Stateless - no persistent rules
            network_nodes: 0, // Stateless - no persistent network
            memory_usage_bytes: std::mem::size_of::<AppState>(),
        },
        timestamp: Utc::now(),
    };

    info!("Health check successful");
    Ok(Json(response))
}

/// Enhanced health check endpoint with dependency validation
#[utoipa::path(
    get,
    path = "/health/detailed",
    tag = "health",
    responses(
        (status = 200, description = "Detailed health check with dependency validation", body = EnhancedHealthResponse),
        (status = 503, description = "Service is unhealthy", body = ApiErrorResponse)
    )
)]
#[instrument(skip(state))]
async fn enhanced_health_handler(
    State(state): State<AppState>,
) -> Result<Json<EnhancedHealthResponse>, ApiError> {
    let response = state.health_service.check_health(Some(&*state.cache), &state.config).await;

    // Return appropriate HTTP status based on health
    match response.status {
        health_checks::HealthStatus::Healthy => {
            info!("Enhanced health check passed");
            Ok(Json(response))
        }
        health_checks::HealthStatus::Degraded => {
            warn!("Enhanced health check shows degraded status");
            Ok(Json(response))
        }
        health_checks::HealthStatus::Unhealthy => {
            error!("Enhanced health check failed");
            Err(ApiError::internal("Service is unhealthy"))
        }
    }
}

/// Get engine statistics
#[utoipa::path(
    get,
    path = "/engine/stats",
    tag = "engine",
    responses(
        (status = 200, description = "Statistics retrieved successfully", body = EngineStats)
    )
)]
#[instrument(skip(_state))]
async fn get_engine_stats_handler(State(_state): State<AppState>) -> Json<EngineStats> {
    let stats = EngineStats {
        total_facts: 0,   // Stateless - no persistent facts
        total_rules: 0,   // Stateless - no persistent rules
        network_nodes: 0, // Stateless - no persistent network
        memory_usage_bytes: std::mem::size_of::<AppState>(),
    };

    info!("Engine statistics retrieved (stateless mode)");
    Json(stats)
}

/// Register and compile a ruleset for caching
#[utoipa::path(
    post,
    path = "/rulesets",
    request_body = RegisterRulesetRequest,
    responses(
        (status = 201, description = "Ruleset compiled and cached successfully", body = RegisterRulesetResponse),
        (status = 400, description = "Invalid ruleset definition", body = ApiErrorResponse),
        (status = 409, description = "Ruleset ID already exists", body = ApiErrorResponse),
        (status = 500, description = "Internal server error during compilation", body = ApiErrorResponse)
    ),
    tag = "rulesets"
)]
#[instrument(skip(state))]
async fn register_ruleset_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRulesetRequest>,
) -> Result<(StatusCode, Json<RegisterRulesetResponse>), ApiError> {
    let request_id = Uuid::new_v4().to_string();

    info!(
        request_id = %request_id,
        ruleset_id = %payload.ruleset_id,
        rules_count = payload.rules.len(),
        "🚀 CACHE: Registering and compiling ruleset"
    );

    // Security validation to prevent DoS attacks via complex rulesets.
    // We create a temporary `EvaluateRequest` to reuse the existing validation logic.
    let temp_eval_request_for_validation = EvaluateRequest {
        rules: Some(payload.rules.clone()),
        facts: vec![], // Facts are not part of this request, only validate rules.
        ruleset_id: None,
        response_format: None,
        streaming_config: None,
    };

    match SecurityValidator::validate_request(
        &temp_eval_request_for_validation,
        &state.config.security,
    ) {
        SecurityValidationResult::Safe => {
            debug!(request_id = %request_id, "Security validation passed for ruleset registration");
        }
        SecurityValidationResult::Rejected { reason } => {
            error!(
                request_id = %request_id,
                reason = %reason,
                "Ruleset rejected for security reasons"
            );

            return Err(ApiError::security(reason));
        }
    }

    // Validate request
    if let Err(validation_error) = payload.validate() {
        error!(
            request_id = %request_id,
            error = %validation_error,
            "Ruleset validation failed"
        );
        return Err(ApiError::validation(validation_error));
    }

    // Check if ruleset already exists (prevent overwrite)
    if state.cache.get(&payload.ruleset_id).await.is_some() {
        return Err(ApiError::Conflict {
            message: format!("Ruleset '{}' already exists in cache", payload.ruleset_id),
        });
    }



    // Convert API rules to core rules
    let mut core_rules = Vec::new();
    for api_rule in &payload.rules {
        match CoreRule::try_from(api_rule) {
            Ok(core_rule) => core_rules.push(core_rule),
            Err(e) => {
                error!(
                    request_id = %request_id,
                    rule_id = %api_rule.id,
                    error = %e,
                    "Failed to convert API rule to core rule"
                );
                return Err(ApiError::validation(format!(
                    "Failed to convert rule '{}': {}",
                    api_rule.id, e
                )));
            }
        }
    }

    // Offload validation compilation to blocking thread so that the async future stays `Send`.
    let ttl_seconds = payload.ttl_seconds.unwrap_or(0);
    let ruleset_id_for_task = payload.ruleset_id.clone();
    let description_for_task = payload.description.clone();
    let core_rules_clone = core_rules.clone();

    let (compiled_asset, compilation_time_ms) = tokio::task::spawn_blocking(move || {
        let start_block = std::time::Instant::now();

        // Validate by compiling into the engine synchronously
        let mut engine = BingoEngine::new()
            .map_err(|e| anyhow::anyhow!("Failed to create validation engine: {}", e))?;
        engine
            .add_rules(core_rules_clone.clone())
            .map_err(|e| anyhow::anyhow!("Failed to compile ruleset: {}", e))?;

        let compiled_at = Utc::now();
        let expires_at = if ttl_seconds == 0 {
            compiled_at + Duration::from_secs(315_360_000)
        } else {
            compiled_at + Duration::from_secs(ttl_seconds)
        };

        let rule_count_val = core_rules_clone.len();
        let asset = Arc::new(CompiledAsset {
            id: ruleset_id_for_task.clone(),
            etag: Uuid::new_v4().to_string(),
            rules: core_rules_clone,
            rule_count: rule_count_val,
            description: description_for_task,
            compiled_at,
            expires_at,
            usage_count: Arc::new(AtomicUsize::new(0)),
        });

        Ok::<_, anyhow::Error>((asset, start_block.elapsed().as_millis() as u64))
    })
    .await
    .map_err(|e| ApiError::internal_with_source("Join error", e.to_string()))??;

    // Store in cache asynchronously
    state.cache.set(payload.ruleset_id.clone(), compiled_asset.clone()).await;

    // The response TTL should be 0 if it's set to never expire.
    let response_ttl_seconds = ttl_seconds;

    let response = RegisterRulesetResponse {
        ruleset_id: payload.ruleset_id.clone(),
        ruleset_hash: compiled_asset.etag.clone(),
        compiled: true,
        rule_count: payload.rules.len(),
        compilation_time_ms,
        ttl_seconds: response_ttl_seconds,
        registered_at: compiled_asset.compiled_at,
    };

    info!(
        request_id = %request_id,
        ruleset_id = %payload.ruleset_id,
        rule_count = payload.rules.len(),
        compilation_time_ms = compilation_time_ms,
        etag = %compiled_asset.etag,
        "✅ CACHE: Ruleset compiled and cached successfully"
    );

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get cache statistics
#[utoipa::path(
    get,
    path = "/cache/stats",
    tag = "engine",
    responses(
        (status = 200, description = "Cache statistics retrieved successfully", body = serde_json::Value)
    )
)]
#[instrument(skip(state))]
async fn get_cache_stats_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let cache_stats = state.cache.get_stats().await;

    let stats = serde_json::json!({
        "unified_cache": cache_stats,
        "engine_cache": cache_stats, // Legacy alias for backward-compat tests
    });

    info!("Unified cache statistics retrieved");
    Json(stats)
}

/// Get security limits and validation thresholds
#[utoipa::path(
    get,
    path = "/security/limits",
    tag = "engine",
    responses(
        (status = 200, description = "Security limits retrieved successfully", body = serde_json::Value)
    )
)]
#[instrument]
async fn get_security_limits_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let response = serde_json::json!({
        "security_limits": state.config.security,
        "description": "Security thresholds and limits enforced by the API",
        "enforcement": {
            "request_timeout_seconds": state.config.hardening.request_timeout_seconds,
            "max_request_body_bytes": state.config.limits.max_body_size_mb * 1024 * 1024,
        }
    });

    info!("Security limits retrieved");
    Json(response)
}

/// Prometheus metrics endpoint
#[utoipa::path(
    get,
    path = "/metrics",
    tag = "engine",
    responses(
        (status = 200, description = "Prometheus metrics in text format", body = String)
    )
)]
#[instrument(skip(state))]
async fn metrics_handler(
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    match state.metrics.export_prometheus() {
        Ok(metrics_text) => {
            debug!("Prometheus metrics exported successfully");
            Ok((
                StatusCode::OK,
                [("content-type", "text/plain; version=0.0.4")],
                metrics_text,
            ))
        }
        Err(e) => {
            error!(error = %e, "Failed to export Prometheus metrics");
            Err(ApiError::internal_with_source(
                "Failed to export metrics",
                e.to_string(),
            ))
        }
    }
}

/// ✅ STATELESS EVALUATION ENDPOINT! Evaluate facts against a set of rules.
///
/// This endpoint supports two primary modes for maximum flexibility and performance:
/// 1.  **Ad-hoc Evaluation**: Provide the `rules` and `facts` directly in the request body. The engine
///     will compile the rules on-the-fly. This mode benefits from a short-lived cache; if the same
///     set of ad-hoc rules is sent repeatedly, the compiled version will be reused.
/// 2.  **Cached Ruleset Evaluation**: Provide a `ruleset_id` (obtained from the `/rulesets` endpoint)
///     and the `facts`. This is the most performant method for static rule sets, as it completely
///     avoids rule compilation overhead.
///
/// In both modes, a fresh engine instance is created per request to process the facts, ensuring
/// complete statelessness and enabling unlimited horizontal scaling.
#[utoipa::path(
    post,
    path = "/evaluate",
    request_body = EvaluateRequest,
    responses(
        (status = 200, description = "Facts evaluated successfully against the provided rules or ruleset", body = EvaluateResponse),
        (status = 400, description = "Invalid request payload - either `rules` or `ruleset_id` must be provided along with `facts`", body = ApiErrorResponse),
        (status = 500, description = "Internal server error during stateless evaluation", body = ApiErrorResponse)
    ),
    tag = "evaluation"
)]
#[instrument(skip(state))]
async fn evaluate_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<EvaluateRequest>,
) -> Result<ApiResponse, ApiError> {
    let request_id = Uuid::new_v4().to_string();

    // Start request tracking for metrics
    let request_tracker = state.metrics_middleware.start_request();

    // Determine mode based on payload
    let mode = match (&payload.rules, &payload.ruleset_id) {
        (Some(_), None) => "rules",
        (None, Some(_)) => "cached_ruleset",
        _ => "unknown",
    };

    info!(
        request_id = %request_id,
        mode = mode,
        facts_count = payload.facts.len(),
        "🚀 OPTIMIZED API: Evaluating with cached/fresh rules + facts"
    );

    // Check for conditional requests (ETag support)
    if let Some(if_none_match) = headers.get("if-none-match") {
        if let Ok(etag_value) = if_none_match.to_str() {
            if let Some(cached_asset) = state.cache.check_etag(etag_value).await {
                info!(
                    request_id = %request_id,
                    etag = %etag_value,
                    cached_ruleset_id = %cached_asset,
                    "🎯 ETag match - returning 304 Not Modified"
                );

                // Return 304 Not Modified without processing
                request_tracker.finish("POST", "/evaluate", 304);
                return Ok(ApiResponse::NotModified);
            }
        }
    }

    // Validate that rules and facts are provided (mandatory fields)
    if let Err(validation_error) = payload.validate() {
        error!(
            request_id = %request_id,
            error = %validation_error,
            "Request validation failed"
        );
        return Err(ApiError::validation(validation_error));
    }

    // Security validation to prevent DoS attacks
    match SecurityValidator::validate_request(&payload, &state.config.security) {
        SecurityValidationResult::Safe => {
            debug!(request_id = %request_id, "Security validation passed");
        }
        SecurityValidationResult::Rejected { reason } => {
            // Record security violation
            request_tracker.record_security_violation("request_validation");

            error!(
                request_id = %request_id,
                reason = %reason,
                "Request rejected for security reasons"
            );

            // Finish tracking with error status
            request_tracker.finish("POST", "/evaluate", 400);

            return Err(ApiError::security(reason));
        }
    }

    let start = std::time::Instant::now();

    // ✅ OPTIMIZED: Track cache information for response headers (lazy allocation)
    let mut cache_etag: Option<String> = None;
    let mut cache_headers: Option<Vec<(String, String)>> = None;

    // ✅ OPTIMIZED: Convert API facts to core facts with pre-allocated capacity
    let mut core_facts = Vec::with_capacity(payload.facts.len());
    for api_fact in &payload.facts {
        core_facts.push(CoreFact::from(api_fact));
    }

    // ✅ OPTIMIZED: Create engine with cached or fresh rules
    let mut engine = match (&payload.rules, &payload.ruleset_id) {
        (Some(api_rules), None) => {
            // ✅ OPTIMIZED: Calculate hash without cloning entire rules
            let rules_hash = {
                let mut hasher = FnvHasher::default();
                // Sort rule IDs only (lightweight) instead of full rule objects
                let mut rule_ids: Vec<&str> = api_rules.iter().map(|r| r.id.as_str()).collect();
                rule_ids.sort_unstable();

                // Hash rule IDs and critical fields without JSON serialization
                for rule_id in rule_ids {
                    hasher.write(rule_id.as_bytes());
                    // Include rule hash from the original rule in the same order
                    if let Some(rule) = api_rules.iter().find(|r| r.id.as_str() == rule_id) {
                        hasher.write(rule.name.as_bytes());
                        hasher.write(&[rule.enabled as u8]);
                        hasher.write(&rule.priority.unwrap_or(0).to_le_bytes());
                    }
                }
                format!("{:x}", hasher.finish())
            };

            // Try to get a pre-compiled asset from the cache using the hash
            if let Some(compiled_asset) = state.cache.get(&rules_hash).await {
                request_tracker.record_cache_activity("engine_by_hash", true);
                info!(request_id = %request_id, rules_hash = %rules_hash, "🚀 CACHE HIT (by hash): Using pre-validated asset");
                cache_etag = Some(compiled_asset.etag.clone());
                cache_headers = Some(compiled_asset.get_cache_headers());
                compiled_asset.create_engine_with_capacity(payload.facts.len()).map_err(|e| {
                     error!(request_id = %request_id, error = %e, "Failed to create engine from cached asset");
                    ApiError::cache("ruleset_cache", format!("Failed to create engine from cached asset: {}", e))
                })?
            } else {
                // Cache miss: compile, cache, and then create the engine
                request_tracker.record_cache_activity("engine_by_hash", false);
                info!(
                    request_id = %request_id, rules_hash = %rules_hash,
                    "🔄 CACHE MISS (by hash): Compiling and caching ad-hoc ruleset"
                );

                let mut core_rules = Vec::new();
                for api_rule in api_rules {
                    match CoreRule::try_from(api_rule) {
                        Ok(core_rule) => core_rules.push(core_rule),
                        Err(e) => {
                            error!(
                                request_id = %request_id,
                                rule_id = %api_rule.id,
                                error = %e,
                                "Failed to convert API rule to core rule"
                            );
                            return Err(ApiError::validation(format!(
                                "Failed to convert rule '{}': {}",
                                api_rule.id, e
                            )));
                        }
                    }
                }

                // Cache the newly compiled ruleset for future ad-hoc requests
                let ttl = Duration::from_secs(state.config.caching.adhoc_ruleset_ttl_seconds);
                let compiled_at = Utc::now();

                let compiled_asset = Arc::new(CompiledAsset {
                    id: rules_hash.clone(),
                    etag: Uuid::new_v4().to_string(),
                    rules: core_rules,
                    rule_count: api_rules.len(),
                    description: Some(format!("Ad-hoc ruleset with {} rules", api_rules.len())),
                    compiled_at,
                    expires_at: compiled_at + ttl,
                    usage_count: Arc::new(AtomicUsize::new(1)),
                });

                state.cache.set(rules_hash, compiled_asset.clone()).await;

                cache_etag = Some(compiled_asset.etag.clone());
                cache_headers = Some(compiled_asset.get_cache_headers());

                // Create the engine instance from the newly cached template
                compiled_asset.create_engine_with_capacity(payload.facts.len()).map_err(|e| {
                    error!(request_id = %request_id, error = %e, "Failed to create engine from newly cached asset");
                    ApiError::cache("ruleset_cache", format!("Failed to create engine from newly cached asset: {}", e))
                })?
            }
        }
        (None, Some(ruleset_id)) => {
            // 🚀 CACHE: Use pre-compiled asset
            info!(
                request_id = %request_id,
                ruleset_id = %ruleset_id,
                "Attempting to use cached asset"
            );

            if let Some(compiled_asset) = state.cache.get(ruleset_id).await {
                // Record cache hit
                request_tracker.record_cache_activity("engine", true);

                // Capture cache information for response headers
                cache_etag = Some(compiled_asset.etag.clone());
                cache_headers = Some(compiled_asset.get_cache_headers());

                info!(
                    request_id = %request_id,
                    ruleset_id = %ruleset_id,
                    rule_count = compiled_asset.rule_count,
                    usage_count = compiled_asset.usage_count.load(std::sync::atomic::Ordering::Relaxed),
                    etag = %compiled_asset.etag,
                    "🚀 CACHE HIT: Using pre-validated asset"
                );

                // Create engine from cached template (much faster than rule parsing)
                compiled_asset.create_engine_with_capacity(payload.facts.len()).map_err(|e| {
                    error!(
                        request_id = %request_id,
                        error = %e,
                        "Failed to create engine from cached asset"
                    );
                    ApiError::cache(
                        "ruleset_cache",
                        format!("Failed to create engine from cached asset: {}", e),
                    )
                })?
            } else {
                // Cache miss is a definitive miss. No fallback.
                request_tracker.record_cache_activity("engine", false);

                error!(
                    request_id = %request_id,
                    ruleset_id = %ruleset_id,
                    "Cached asset not found in cache"
                );

                // Finish tracking with error status
                request_tracker.finish("POST", "/evaluate", 404);

                return Err(ApiError::not_found(format!(
                    "Ruleset '{}' not found in cache",
                    ruleset_id
                )));
            }
        }
        _ => unreachable!("Validation should have caught this case"),
    };

    // Check if incremental processing should be used for memory efficiency
    let current_memory_mb = MemoryMonitor::current_memory_mb();
    state.metrics.memory_usage_mb.set(current_memory_mb as i64);

    let should_use_incremental = IncrementalProcessor::should_use_incremental(
        core_facts.len(),
        &payload.streaming_config,
        current_memory_mb,
        state.config.incremental_processing.default_memory_limit_mb,
    );

    if should_use_incremental {
        info!(
            request_id = %request_id,
            fact_count = core_facts.len(),
            current_memory_mb = current_memory_mb,
            "🔄 Using incremental processing for memory efficiency"
        );

        let rules_processed = match (&payload.rules, &payload.ruleset_id) {
            (Some(rules), None) => rules.len(),
            (None, Some(_)) => engine.rule_count(),
            _ => unreachable!("Validation should have caught this case"),
        };

        // Use incremental processor
        let incremental_processor = IncrementalProcessor::new(
            request_id.clone(),
            &payload.streaming_config,
            state.config.incremental_processing.default_fact_batch_size,
            state.config.incremental_processing.default_memory_limit_mb,
            state.metrics.clone(),
        );

        let response =
            incremental_processor.process_incrementally(engine, core_facts, rules_processed);

        // Finish request tracking
        request_tracker.finish("POST", "/evaluate", 200);

        // ✅ OPTIMIZED: Return incremental streaming response with cache headers if available
        return Ok(match cache_headers {
            None => ApiResponse::Streaming(response),
            Some(headers) if headers.is_empty() => ApiResponse::Streaming(response),
            Some(headers) => ApiResponse::StreamingWithHeaders(response, headers),
        });
    }

    // Process facts through the engine (traditional mode)
    let results = engine.process_facts(core_facts).map_err(|e| {
        error!(
            request_id = %request_id,
            error = %e,
            "Failed to evaluate rules and facts"
        );
        ApiError::engine(format!("Failed to evaluate: {}", e))
    })?;

    let processing_time_ms = start.elapsed().as_millis() as u64;

    // Convert core results to API results
    let api_results: Vec<ApiRuleExecutionResult> =
        results.iter().map(|result| result.into()).collect();

    let rules_processed = match &payload.rules {
        Some(rules) => rules.len(),
        None => 0, // No engine stats available in simplified process_facts result
    };

    // Build streaming response builder
    let response_builder = StreamingResponseBuilder {
        request_id: request_id.clone(),
        results: api_results,
        rules_processed,
        facts_processed: payload.facts.len(),
        processing_time_ms,
        stats: EngineStats {
            total_facts: payload.facts.len(),
            total_rules: rules_processed,
            network_nodes: 0,      // No stats available from simplified API
            memory_usage_bytes: 0, // No stats available from simplified API
        },
    };

    let result_count = response_builder.results.len();

    // Determine if streaming should be used
    let should_stream = StreamingResponseBuilder::should_stream(
        result_count,
        &payload.response_format,
        &payload.streaming_config,
        state.config.security.memory_safety_threshold,
    );

    // Record evaluation metrics with streaming info
    let evaluation_duration = start.elapsed();
    request_tracker.record_evaluation(
        rules_processed,
        payload.facts.len(),
        result_count, // rules_fired
        evaluation_duration,
        should_stream,
    );

    info!(
        request_id = %request_id,
        mode = mode,
        rules_fired = result_count,
        processing_time_ms = processing_time_ms,
        should_stream = should_stream,
        "✅ OPTIMIZED API: Successfully evaluated with cached/fresh rules + facts"
    );

    // Build appropriate response type
    let response = if should_stream {
        info!(
            request_id = %request_id,
            result_count = result_count,
            etag = ?cache_etag,
            "🚀 STREAMING: Using NDJSON streaming for large result set"
        );
        match cache_headers {
            None => ApiResponse::Streaming(
                response_builder.build_streaming_response(payload.streaming_config),
            ),
            Some(headers) if headers.is_empty() => ApiResponse::Streaming(
                response_builder.build_streaming_response(payload.streaming_config),
            ),
            Some(headers) => ApiResponse::StreamingWithHeaders(
                response_builder.build_streaming_response(payload.streaming_config),
                headers,
            ),
        }
    } else {
        info!(
            request_id = %request_id,
            result_count = result_count,
            etag = ?cache_etag,
            "📄 STANDARD: Using standard JSON response"
        );
        match cache_headers {
            None => ApiResponse::Standard(response_builder.build_standard_response()),
            Some(headers) if headers.is_empty() => {
                ApiResponse::Standard(response_builder.build_standard_response())
            }
            Some(headers) => ApiResponse::StandardWithHeaders(
                response_builder.build_standard_response(),
                headers,
            ),
        }
    };

    // Finish request tracking with success status
    request_tracker.finish("POST", "/evaluate", 200);

    Ok(response)
}
