//! Bingo Rules Engine REST API with OpenAPI specification
//!
//! This module provides a RESTful API for the Bingo RETE rules engine
//! with automatic OpenAPI documentation generation and native JSON types.

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use bingo_core::{
    BingoEngine, Fact as CoreFact, FactData as CoreFactData, FactValue as CoreFactValue,
};
use chrono::{DateTime, Utc};
use fnv::FnvHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing::{error, info, instrument};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

pub mod ruleset_cache;
pub mod types;

use ruleset_cache::RulesetCache;
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
    ),
    components(
        schemas(
            ApiFact,
            ApiRule,
            ApiCondition,
            EvaluateRequest,
            EvaluateResponse,
            RegisterRulesetRequest,
            RegisterRulesetResponse,
            ApiRuleExecutionResult,
            ApiActionResult,
            ApiAction,
            EngineStats,
            HealthResponse,
            ApiError,
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
        description = "High-performance stateless RETE-based rules engine with per-request processing. Rules and facts must be provided together in each evaluation request for maximum concurrency and horizontal scaling.",
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
#[derive(Clone)]
pub struct AppState {
    start_time: DateTime<Utc>,
    ruleset_cache: Arc<RulesetCache>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self { start_time: Utc::now(), ruleset_cache: Arc::new(RulesetCache::default()) })
    }
}

/// Create the web application with OpenAPI documentation
#[instrument]
pub fn create_app() -> anyhow::Result<Router> {
    info!("Creating Bingo API application with OpenAPI support");

    let state = AppState::new()?;

    let app = Router::new()
        // Health check
        .route("/health", get(health_handler))
        // Optimized evaluation endpoint (supports both rules and ruleset_id)
        .route("/evaluate", post(evaluate_handler))
        // Ruleset compilation and caching
        .route("/rulesets", post(register_ruleset_handler))
        // Engine and cache statistics
        .route("/engine/stats", get(get_engine_stats_handler))
        .route("/cache/stats", get(get_cache_stats_handler))
        // OpenAPI documentation endpoints
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .with_state(state)
        .layer(RequestBodyLimitLayer::new(50 * 1024 * 1024)) // 50MB max request size
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    info!("API application created successfully with OpenAPI documentation");
    Ok(app)
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unhealthy", body = ApiError)
    )
)]
#[instrument(skip(state))]
async fn health_handler(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, (StatusCode, Json<ApiError>)> {
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
        (status = 400, description = "Invalid ruleset definition", body = ApiError),
        (status = 409, description = "Ruleset ID already exists", body = ApiError),
        (status = 500, description = "Internal server error during compilation", body = ApiError)
    ),
    tag = "rulesets"
)]
#[instrument(skip(state))]
async fn register_ruleset_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRulesetRequest>,
) -> Result<(StatusCode, Json<RegisterRulesetResponse>), (StatusCode, Json<ApiError>)> {
    let request_id = Uuid::new_v4().to_string();

    info!(
        request_id = %request_id,
        ruleset_id = %payload.ruleset_id,
        rules_count = payload.rules.len(),
        "🚀 CACHE: Registering and compiling ruleset"
    );

    // Validate request
    if let Err(validation_error) = payload.validate() {
        error!(
            request_id = %request_id,
            error = %validation_error,
            "Ruleset validation failed"
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("VALIDATION_ERROR", &validation_error).with_request_id(request_id)),
        ));
    }

    // Check if ruleset already exists (prevent overwrite)
    if state.ruleset_cache.get(&payload.ruleset_id).is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(
                ApiError::new(
                    "RULESET_EXISTS",
                    &format!("Ruleset '{}' already exists in cache", payload.ruleset_id),
                )
                .with_request_id(request_id),
            ),
        ));
    }

    let start = std::time::Instant::now();

    // Convert API rules to core rules
    let mut core_rules = Vec::new();
    for api_rule in &payload.rules {
        match convert_api_rule_to_core(api_rule) {
            Ok(core_rule) => core_rules.push(core_rule),
            Err(e) => {
                error!(
                    request_id = %request_id,
                    rule_id = %api_rule.id,
                    error = %e,
                    "Failed to convert API rule to core rule"
                );
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(
                        ApiError::new(
                            "RULE_CONVERSION_ERROR",
                            &format!("Failed to convert rule '{}': {}", api_rule.id, e),
                        )
                        .with_request_id(request_id),
                    ),
                ));
            }
        }
    }

    // Compile and cache the ruleset
    let ttl = payload.ttl_seconds.map(std::time::Duration::from_secs);
    let compiled_ruleset = state
        .ruleset_cache
        .compile_and_cache(
            payload.ruleset_id.clone(),
            core_rules,
            ttl,
            payload.description.clone(),
        )
        .map_err(|e| {
            error!(
                request_id = %request_id,
                error = %e,
                "Failed to compile ruleset"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    ApiError::new(
                        "COMPILATION_ERROR",
                        &format!("Failed to compile ruleset: {}", e),
                    )
                    .with_request_id(request_id.clone()),
                ),
            )
        })?;

    let compilation_time_ms = start.elapsed().as_millis() as u64;
    let ttl_seconds =
        (compiled_ruleset.expires_at - compiled_ruleset.compiled_at).num_seconds() as u64;

    let response = RegisterRulesetResponse {
        ruleset_id: payload.ruleset_id.clone(),
        ruleset_hash: compiled_ruleset.hash.clone(),
        compiled: true,
        rule_count: payload.rules.len(),
        compilation_time_ms,
        ttl_seconds,
        registered_at: compiled_ruleset.compiled_at,
    };

    info!(
        request_id = %request_id,
        ruleset_id = %payload.ruleset_id,
        rule_count = payload.rules.len(),
        compilation_time_ms = compilation_time_ms,
        hash = %compiled_ruleset.hash,
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
    let cache_stats = state.ruleset_cache.get_stats();

    let stats = serde_json::json!({
        "cache": {
            "total_entries": cache_stats.total_entries,
            "cache_hits": cache_stats.cache_hits,
            "cache_misses": cache_stats.cache_misses,
            "hit_rate": cache_stats.hit_rate,
            "expired_entries": cache_stats.expired_entries,
            "total_compilations": cache_stats.total_compilations,
            "average_compilation_time_ms": cache_stats.average_compilation_time_ms,
        }
    });

    info!("Cache statistics retrieved");
    Json(stats)
}

/// ✅ STATELESS EVALUATION ENDPOINT! Evaluate rules + facts with predefined calculators
///
/// This endpoint processes rules and facts in a completely stateless manner. Each request must
/// include both rules and facts (both mandatory). A fresh engine instance is created per request
/// for maximum concurrency and horizontal scaling. No state is shared between requests.
#[utoipa::path(
    post,
    path = "/evaluate",
    request_body = EvaluateRequest,
    responses(
        (status = 200, description = "Rules evaluated successfully with stateless processing", body = EvaluateResponse),
        (status = 400, description = "Invalid request payload - rules and facts are mandatory", body = ApiError),
        (status = 500, description = "Internal server error during stateless evaluation", body = ApiError)
    ),
    tag = "evaluation"
)]
#[instrument(skip(state))]
async fn evaluate_handler(
    State(state): State<AppState>,
    Json(payload): Json<EvaluateRequest>,
) -> Result<Json<EvaluateResponse>, (StatusCode, Json<ApiError>)> {
    let request_id = Uuid::new_v4().to_string();

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

    // Validate that rules and facts are provided (mandatory fields)
    if let Err(validation_error) = payload.validate() {
        error!(
            request_id = %request_id,
            error = %validation_error,
            "Request validation failed"
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("VALIDATION_ERROR", &validation_error).with_request_id(request_id)),
        ));
    }

    let start = std::time::Instant::now();

    // Convert API facts to core facts (always needed)
    let core_facts: Vec<CoreFact> = payload.facts.iter().map(convert_api_fact_to_core).collect();

    // ✅ OPTIMIZED: Create engine with cached or fresh rules
    let mut engine = match (&payload.rules, &payload.ruleset_id) {
        (Some(api_rules), None) => {
            // Traditional mode: convert rules and create fresh engine
            info!(
                request_id = %request_id,
                rules_count = api_rules.len(),
                "Creating fresh engine with provided rules"
            );

            let mut core_rules = Vec::new();
            for api_rule in api_rules {
                match convert_api_rule_to_core(api_rule) {
                    Ok(core_rule) => core_rules.push(core_rule),
                    Err(e) => {
                        error!(
                            request_id = %request_id,
                            rule_id = %api_rule.id,
                            error = %e,
                            "Failed to convert API rule to core rule"
                        );
                        return Err((
                            StatusCode::BAD_REQUEST,
                            Json(
                                ApiError::new(
                                    "RULE_CONVERSION_ERROR",
                                    &format!("Failed to convert rule '{}': {}", api_rule.id, e),
                                )
                                .with_request_id(request_id),
                            ),
                        ));
                    }
                }
            }

            let mut engine = BingoEngine::with_capacity(payload.facts.len()).map_err(|e| {
                error!(
                    request_id = %request_id,
                    error = %e,
                    "Failed to create engine"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(
                        ApiError::new(
                            "ENGINE_CREATION_ERROR",
                            &format!("Failed to create engine: {}", e),
                        )
                        .with_request_id(request_id.clone()),
                    ),
                )
            })?;

            // Add rules to engine
            for rule in core_rules {
                engine.add_rule(rule).map_err(|e| {
                    error!(
                        request_id = %request_id,
                        error = %e,
                        "Failed to add rule to engine"
                    );
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(
                            ApiError::new("RULE_ADD_ERROR", &format!("Failed to add rule: {}", e))
                                .with_request_id(request_id.clone()),
                        ),
                    )
                })?;
            }

            engine
        }
        (None, Some(ruleset_id)) => {
            // ✅ CACHE HIT: Use pre-compiled ruleset
            info!(
                request_id = %request_id,
                ruleset_id = %ruleset_id,
                "Attempting to use cached ruleset"
            );

            if let Some(compiled_ruleset) = state.ruleset_cache.get(ruleset_id) {
                info!(
                    request_id = %request_id,
                    ruleset_id = %ruleset_id,
                    rule_count = compiled_ruleset.rules.len(),
                    usage_count = compiled_ruleset.usage_count,
                    "🚀 CACHE HIT: Using pre-compiled ruleset"
                );

                // Create engine from cached ruleset
                compiled_ruleset.create_engine_with_capacity(payload.facts.len()).map_err(|e| {
                    error!(
                        request_id = %request_id,
                        error = %e,
                        "Failed to create engine from cached ruleset"
                    );
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(
                            ApiError::new(
                                "CACHED_ENGINE_ERROR",
                                &format!("Failed to create engine from cached ruleset: {}", e),
                            )
                            .with_request_id(request_id.clone()),
                        ),
                    )
                })?
            } else {
                // Cache miss
                error!(
                    request_id = %request_id,
                    ruleset_id = %ruleset_id,
                    "Cached ruleset not found"
                );
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(
                        ApiError::new(
                            "RULESET_NOT_FOUND",
                            &format!("Ruleset '{}' not found in cache", ruleset_id),
                        )
                        .with_request_id(request_id),
                    ),
                ));
            }
        }
        _ => unreachable!("Validation should have caught this case"),
    };

    // Process facts through the engine
    let results = engine.process_facts(core_facts).map_err(|e| {
        error!(
            request_id = %request_id,
            error = %e,
            "Failed to evaluate rules and facts"
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                ApiError::new("EVALUATION_ERROR", &format!("Failed to evaluate: {}", e))
                    .with_request_id(request_id.clone()),
            ),
        )
    })?;

    let processing_time_ms = start.elapsed().as_millis() as u64;
    let engine_stats = engine.get_stats();

    // Convert core results to API results
    let api_results: Vec<ApiRuleExecutionResult> = results
        .iter()
        .map(|result| {
            let actions_executed = result
                .actions_executed
                .iter()
                .map(|action| match action {
                    bingo_core::ActionResult::FieldSet { field, value } => {
                        ApiActionResult::FieldSet {
                            field: field.clone(),
                            value: convert_fact_value_to_json(value),
                        }
                    }
                    bingo_core::ActionResult::CalculatorResult { calculator, result } => {
                        ApiActionResult::CalculatorResult {
                            calculator: calculator.clone(),
                            result: result.clone(),
                        }
                    }
                    bingo_core::ActionResult::Logged { message } => {
                        ApiActionResult::Logged { message: message.clone() }
                    }
                    bingo_core::ActionResult::LazyLogged { .. } => {
                        // Materialize the lazy message for API response
                        ApiActionResult::Logged {
                            message: action
                                .get_message()
                                .unwrap_or_else(|| "Unknown message".to_string()),
                        }
                    }
                })
                .collect();

            ApiRuleExecutionResult {
                rule_id: result.rule_id.to_string(),
                fact_id: result.fact_id.to_string(),
                actions_executed,
            }
        })
        .collect();

    let rules_processed = match &payload.rules {
        Some(rules) => rules.len(),
        None => engine_stats.rule_count,
    };

    let response = EvaluateResponse {
        request_id: request_id.clone(),
        results: api_results,
        rules_processed,
        facts_processed: payload.facts.len(),
        rules_fired: results.len(),
        processing_time_ms,
        stats: EngineStats {
            total_facts: engine_stats.fact_count,
            total_rules: engine_stats.rule_count,
            network_nodes: engine_stats.node_count,
            memory_usage_bytes: engine_stats.memory_usage_bytes,
        },
    };

    info!(
        request_id = %request_id,
        mode = mode,
        rules_fired = results.len(),
        processing_time_ms = processing_time_ms,
        "✅ OPTIMIZED API: Successfully evaluated with cached/fresh rules + facts"
    );

    Ok(Json(response))
}

// Utility functions for converting between API and core types

/// Creates a stable u64 hash from a string ID.
fn stable_hash_id(id: &str) -> u64 {
    let mut hasher = FnvHasher::default();
    id.hash(&mut hasher);
    hasher.finish()
}

// Helper function to convert a single ApiCondition to core Condition
fn convert_api_condition_to_core(
    api_condition: &ApiCondition,
) -> anyhow::Result<bingo_core::Condition> {
    use bingo_core::{Condition, FactValue as CoreFactValue, Operator};

    let condition = match api_condition {
        ApiCondition::Simple { field, operator, value } => {
            let core_operator = match operator.as_str() {
                "equal" => Operator::Equal,
                "not_equal" => Operator::NotEqual,
                "greater_than" => Operator::GreaterThan,
                "less_than" => Operator::LessThan,
                "greater_than_or_equal" => Operator::GreaterThanOrEqual,
                "less_than_or_equal" => Operator::LessThanOrEqual,
                "contains" => Operator::Contains,
                _ => return Err(anyhow::anyhow!("Unknown operator: {}", operator)),
            };

            let core_value = match value {
                serde_json::Value::String(s) => CoreFactValue::String(s.clone()),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        CoreFactValue::Integer(i)
                    } else if let Some(f) = n.as_f64() {
                        CoreFactValue::Float(f)
                    } else {
                        return Err(anyhow::anyhow!("Invalid number value"));
                    }
                }
                serde_json::Value::Bool(b) => CoreFactValue::Boolean(*b),
                _ => return Err(anyhow::anyhow!("Unsupported value type")),
            };

            Condition::Simple { field: field.clone(), operator: core_operator, value: core_value }
        }
        ApiCondition::Complex { operator, conditions } => {
            let logical_op = match operator.as_str() {
                "and" => bingo_core::LogicalOperator::And,
                "or" => bingo_core::LogicalOperator::Or,
                _ => return Err(anyhow::anyhow!("Unknown logical operator: {}", operator)),
            };

            // Recursively convert sub-conditions
            let mut core_conditions = Vec::new();
            for sub_condition in conditions {
                let core_condition = convert_api_condition_to_core(sub_condition)?;
                core_conditions.push(core_condition);
            }

            Condition::Complex { operator: logical_op, conditions: core_conditions }
        }
    };

    Ok(condition)
}

fn convert_api_rule_to_core(api_rule: &ApiRule) -> anyhow::Result<bingo_core::Rule> {
    use bingo_core::{Action, ActionType, FactValue as CoreFactValue, Rule};

    // Convert conditions
    let mut conditions = Vec::new();
    for api_condition in &api_rule.conditions {
        let condition = convert_api_condition_to_core(api_condition)?;
        conditions.push(condition);
    }

    // Convert actions
    let mut actions = Vec::new();
    for api_action in &api_rule.actions {
        let action = match api_action {
            ApiAction::Log { level: _, message } => {
                Action { action_type: ActionType::Log { message: message.clone() } }
            }
            ApiAction::SetField { field, value } => {
                let core_value = match value {
                    serde_json::Value::String(s) => CoreFactValue::String(s.clone()),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            CoreFactValue::Integer(i)
                        } else if let Some(f) = n.as_f64() {
                            CoreFactValue::Float(f)
                        } else {
                            return Err(anyhow::anyhow!("Invalid number value"));
                        }
                    }
                    serde_json::Value::Bool(b) => CoreFactValue::Boolean(*b),
                    _ => return Err(anyhow::anyhow!("Unsupported value type")),
                };

                Action {
                    action_type: ActionType::SetField { field: field.clone(), value: core_value },
                }
            }
            ApiAction::Formula { field, expression } => Action {
                action_type: ActionType::Formula {
                    target_field: field.clone(),
                    expression: expression.clone(),
                    source_calculator: None,
                },
            },
            ApiAction::CreateFact { data } => {
                // Convert JSON data to FactData
                let mut fields = std::collections::HashMap::new();
                for (key, value) in data {
                    let core_value = match value {
                        serde_json::Value::String(s) => CoreFactValue::String(s.clone()),
                        serde_json::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                CoreFactValue::Integer(i)
                            } else if let Some(f) = n.as_f64() {
                                CoreFactValue::Float(f)
                            } else {
                                return Err(anyhow::anyhow!(
                                    "Invalid number value in CreateFact data"
                                ));
                            }
                        }
                        serde_json::Value::Bool(b) => CoreFactValue::Boolean(*b),
                        _ => CoreFactValue::String(value.to_string()),
                    };
                    fields.insert(key.clone(), core_value);
                }

                Action {
                    action_type: ActionType::CreateFact { data: bingo_core::FactData { fields } },
                }
            }
            ApiAction::CallCalculator { calculator_name, input_mapping, output_field } => Action {
                action_type: ActionType::CallCalculator {
                    calculator_name: calculator_name.clone(),
                    input_mapping: input_mapping.clone(),
                    output_field: output_field.clone(),
                },
            },
        };
        actions.push(action);
    }

    // Use a stable hash of the string ID to create a consistent numeric ID for the core engine
    let numeric_id = stable_hash_id(&api_rule.id);

    Ok(Rule { id: numeric_id, name: api_rule.name.clone(), conditions, actions })
}

fn convert_api_fact_to_core(api_fact: &ApiFact) -> CoreFact {
    let mut fields = std::collections::HashMap::new();

    for (key, value) in &api_fact.data {
        let core_value = match value {
            serde_json::Value::String(s) => CoreFactValue::String(s.clone()),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    CoreFactValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    CoreFactValue::Float(f)
                } else {
                    CoreFactValue::String(n.to_string())
                }
            }
            serde_json::Value::Bool(b) => CoreFactValue::Boolean(*b),
            _ => CoreFactValue::String(value.to_string()),
        };

        fields.insert(key.clone(), core_value);
    }

    // The core engine will assign the final u64 ID. We use the stable hash
    // of the user-provided string ID to ensure that if the same fact is sent
    // multiple times, the engine can recognize it.
    let fact_id = stable_hash_id(&api_fact.id);

    CoreFact { id: fact_id, data: CoreFactData { fields } }
}

fn convert_fact_value_to_json(value: &CoreFactValue) -> serde_json::Value {
    match value {
        CoreFactValue::String(s) => serde_json::Value::String(s.clone()),
        CoreFactValue::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
        CoreFactValue::Float(f) => serde_json::Value::Number(
            serde_json::Number::from_f64(*f).unwrap_or_else(|| serde_json::Number::from(0)),
        ),
        CoreFactValue::Boolean(b) => serde_json::Value::Bool(*b),
        CoreFactValue::Array(arr) => {
            let values: Vec<serde_json::Value> =
                arr.iter().map(convert_fact_value_to_json).collect();
            serde_json::Value::Array(values)
        }
        CoreFactValue::Object(obj) => {
            let mut map = serde_json::Map::new();
            for (k, v) in obj {
                map.insert(k.clone(), convert_fact_value_to_json(v));
            }
            serde_json::Value::Object(map)
        }
        CoreFactValue::Date(dt) => serde_json::Value::String(dt.to_rfc3339()),
        CoreFactValue::Null => serde_json::Value::Null,
    }
}
