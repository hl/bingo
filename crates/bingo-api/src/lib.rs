//! Bingo Rules Engine REST API with OpenAPI specification
//!
//! This module provides a RESTful API for the Bingo RETE rules engine
//! with automatic OpenAPI documentation generation and native JSON types.

use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing::{error, info, instrument, warn};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

use bingo_core::{
    BingoEngine, Fact as CoreFact, FactData as CoreFactData, FactValue as CoreFactValue,
};

pub mod types;
use types::*;

/// OpenAPI specification
#[derive(OpenApi)]
#[openapi(
    paths(
        health_handler,
        process_facts_handler,
        create_rule_handler,
        get_rule_handler,
        update_rule_handler,
        delete_rule_handler,
        list_rules_handler,
        get_engine_stats_handler,
    ),
    components(
        schemas(
            ApiFact,
            ApiRule,
            ApiCondition,
            ApiAction,
            ProcessFactsRequest,
            ProcessFactsResponse,
            CreateRuleRequest,
            CreateRuleResponse,
            ListRulesQuery,
            ListRulesResponse,
            EngineStats,
            HealthResponse,
            ApiError,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "facts", description = "Fact processing endpoints"),
        (name = "rules", description = "Rule management endpoints"),
        (name = "engine", description = "Engine statistics and management"),
    ),
    info(
        title = "Bingo Rules Engine API",
        version = "1.0.0",
        description = "High-performance RETE-based rules engine with calculator DSL support",
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

/// Application state containing the rules engine
#[derive(Clone)]
pub struct AppState {
    engine: Arc<RwLock<BingoEngine>>,
    rules: Arc<Mutex<HashMap<String, ApiRule>>>,
    start_time: DateTime<Utc>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            engine: Arc::new(RwLock::new(BingoEngine::new()?)),
            rules: Arc::new(Mutex::new(HashMap::new())),
            start_time: Utc::now(),
        })
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
        // Fact processing
        .route("/facts/process", post(process_facts_handler))
        // Rule management
        .route("/rules", post(create_rule_handler))
        .route("/rules", get(list_rules_handler))
        .route("/rules/:rule_id", get(get_rule_handler))
        .route("/rules/:rule_id", put(update_rule_handler))
        .route("/rules/:rule_id", delete(delete_rule_handler))
        // Engine statistics
        .route("/engine/stats", get(get_engine_stats_handler))
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
    let engine = state.engine.read().await;
    let engine_stats = engine.get_stats();

    let uptime_seconds = (Utc::now() - state.start_time).num_seconds() as u64;

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: "1.0.0".to_string(),
        uptime_seconds,
        engine_stats: EngineStats {
            total_facts: engine_stats.fact_count,
            total_rules: engine_stats.rule_count,
            network_nodes: engine_stats.node_count,
            memory_usage_bytes: engine_stats.memory_usage_bytes,
        },
        timestamp: Utc::now(),
    };

    info!("Health check successful");
    Ok(Json(response))
}

/// Process facts through the rules engine
#[utoipa::path(
    post,
    path = "/facts/process",
    tag = "facts",
    request_body = ProcessFactsRequest,
    responses(
        (status = 200, description = "Facts processed successfully", body = ProcessFactsResponse),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 500, description = "Processing error", body = ApiError)
    )
)]
#[instrument(skip(state, payload))]
async fn process_facts_handler(
    State(state): State<AppState>,
    Json(payload): Json<ProcessFactsRequest>,
) -> Result<Json<ProcessFactsResponse>, (StatusCode, Json<ApiError>)> {
    let request_id = Uuid::new_v4().to_string();

    info!(
        request_id = %request_id,
        fact_count = payload.facts.len(),
        "Processing facts request"
    );

    let start = std::time::Instant::now();

    // Convert API facts to core facts
    let mut next_id = 0u64;
    let core_facts: Vec<CoreFact> = payload
        .facts
        .iter()
        .map(|api_fact| convert_api_fact_to_core(api_fact, &mut next_id))
        .collect();

    let mut engine = state.engine.write().await;
    let _engine_stats_before = engine.get_stats();

    let results = engine.process_facts(core_facts).map_err(|e| {
        error!(
            request_id = %request_id,
            error = %e,
            "Failed to process facts"
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                ApiError::new(
                    "PROCESSING_ERROR",
                    &format!("Failed to process facts: {}", e),
                )
                .with_request_id(request_id.clone()),
            ),
        )
    })?;

    let engine_stats_after = engine.get_stats();
    let processing_time_ms = start.elapsed().as_millis() as u64;

    // Convert core facts back to API facts
    let api_results: Vec<ApiFact> = results.iter().map(convert_core_fact_to_api).collect();

    let response = ProcessFactsResponse {
        request_id: request_id.clone(),
        results: api_results,
        facts_processed: payload.facts.len(),
        rules_evaluated: engine_stats_after.rule_count,
        rules_fired: results.len(),
        processing_time_ms,
        stats: EngineStats {
            total_facts: engine_stats_after.fact_count,
            total_rules: engine_stats_after.rule_count,
            network_nodes: engine_stats_after.node_count,
            memory_usage_bytes: engine_stats_after.memory_usage_bytes,
        },
    };

    info!(
        request_id = %request_id,
        result_count = results.len(),
        processing_time_ms,
        "Facts processed successfully"
    );

    Ok(Json(response))
}

/// Create a new rule
#[utoipa::path(
    post,
    path = "/rules",
    tag = "rules",
    request_body = CreateRuleRequest,
    responses(
        (status = 201, description = "Rule created successfully", body = CreateRuleResponse),
        (status = 400, description = "Invalid rule definition", body = ApiError),
        (status = 409, description = "Rule already exists", body = ApiError)
    )
)]
#[instrument(skip(state, request))]
async fn create_rule_handler(
    State(state): State<AppState>,
    Json(request): Json<CreateRuleRequest>,
) -> Result<(StatusCode, Json<CreateRuleResponse>), (StatusCode, Json<ApiError>)> {
    info!(rule_id = %request.rule.id, "Creating new rule");

    // Validate rule
    request.rule.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("VALIDATION_ERROR", &e)),
        )
    })?;

    let mut rules = state.rules.lock().await;

    // Check if rule already exists
    if rules.contains_key(&request.rule.id) {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiError::new(
                "RULE_EXISTS",
                &format!("Rule with ID '{}' already exists", request.rule.id),
            )),
        ));
    }

    let mut rule = request.rule;
    rule.created_at = Utc::now();
    rule.updated_at = Utc::now();

    // Convert to core rule and add to engine
    let core_rule = convert_api_rule_to_core(&rule).map_err(|e| {
        error!(rule_id = %rule.id, error = %e, "Failed to convert rule");
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError::new(
                "CONVERSION_ERROR",
                &format!("Failed to convert rule: {}", e),
            )),
        )
    })?;

    // Add to engine (requires write lock)
    let mut engine = state.engine.write().await;
    engine.add_rule(core_rule).map_err(|e| {
        error!(rule_id = %rule.id, error = %e, "Failed to add rule to engine");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::new(
                "ENGINE_ERROR",
                &format!("Failed to add rule to engine: {}", e),
            )),
        )
    })?;
    drop(engine); // Release write lock early

    rules.insert(rule.id.clone(), rule.clone());

    info!(rule_id = %rule.id, "Rule created successfully");

    Ok((
        StatusCode::CREATED,
        Json(CreateRuleResponse { rule, created: true }),
    ))
}

/// Get a specific rule by ID
#[utoipa::path(
    get,
    path = "/rules/{rule_id}",
    tag = "rules",
    params(
        ("rule_id" = String, Path, description = "Rule identifier")
    ),
    responses(
        (status = 200, description = "Rule found", body = ApiRule),
        (status = 404, description = "Rule not found", body = ApiError)
    )
)]
#[instrument(skip(state))]
async fn get_rule_handler(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> Result<Json<ApiRule>, (StatusCode, Json<ApiError>)> {
    let rules = state.rules.lock().await;

    match rules.get(&rule_id) {
        Some(rule) => {
            info!(rule_id = %rule_id, "Rule retrieved successfully");
            Ok(Json(rule.clone()))
        }
        None => {
            warn!(rule_id = %rule_id, "Rule not found");
            Err((
                StatusCode::NOT_FOUND,
                Json(ApiError::new(
                    "RULE_NOT_FOUND",
                    &format!("Rule with ID '{}' not found", rule_id),
                )),
            ))
        }
    }
}

/// Update an existing rule
#[utoipa::path(
    put,
    path = "/rules/{rule_id}",
    tag = "rules",
    params(
        ("rule_id" = String, Path, description = "Rule identifier")
    ),
    request_body = ApiRule,
    responses(
        (status = 200, description = "Rule updated successfully", body = CreateRuleResponse),
        (status = 400, description = "Invalid rule definition", body = ApiError),
        (status = 404, description = "Rule not found", body = ApiError)
    )
)]
#[instrument(skip(state, rule_update))]
async fn update_rule_handler(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
    Json(mut rule_update): Json<ApiRule>,
) -> Result<Json<CreateRuleResponse>, (StatusCode, Json<ApiError>)> {
    // Validate rule
    rule_update.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("VALIDATION_ERROR", &e)),
        )
    })?;

    let mut rules = state.rules.lock().await;

    match rules.get(&rule_id) {
        Some(existing_rule) => {
            rule_update.id = rule_id.clone();
            rule_update.created_at = existing_rule.created_at;
            rule_update.updated_at = Utc::now();

            // Convert to core rule and update in engine
            let core_rule = convert_api_rule_to_core(&rule_update).map_err(|e| {
                error!(rule_id = %rule_id, error = %e, "Failed to convert updated rule");
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError::new(
                        "CONVERSION_ERROR",
                        &format!("Failed to convert updated rule: {}", e),
                    )),
                )
            })?;

            // Update in engine (requires write lock)
            let mut engine = state.engine.write().await;
            engine.update_rule(core_rule).map_err(|e| {
                error!(rule_id = %rule_id, error = %e, "Failed to update rule in engine");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError::new(
                        "ENGINE_ERROR",
                        &format!("Failed to update rule in engine: {}", e),
                    )),
                )
            })?;
            drop(engine); // Release write lock early

            rules.insert(rule_id.clone(), rule_update.clone());

            info!(rule_id = %rule_id, "Rule updated successfully in memory and engine");

            Ok(Json(CreateRuleResponse {
                rule: rule_update,
                created: false,
            }))
        }
        None => {
            warn!(rule_id = %rule_id, "Cannot update: rule not found");
            Err((
                StatusCode::NOT_FOUND,
                Json(ApiError::new(
                    "RULE_NOT_FOUND",
                    &format!("Rule with ID '{}' not found", rule_id),
                )),
            ))
        }
    }
}

/// Delete a rule
#[utoipa::path(
    delete,
    path = "/rules/{rule_id}",
    tag = "rules",
    params(
        ("rule_id" = String, Path, description = "Rule identifier")
    ),
    responses(
        (status = 204, description = "Rule deleted successfully"),
        (status = 404, description = "Rule not found", body = ApiError)
    )
)]
#[instrument(skip(state))]
async fn delete_rule_handler(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let mut rules = state.rules.lock().await;

    match rules.remove(&rule_id) {
        Some(_) => {
            // Also remove from engine (requires write lock)
            let mut engine = state.engine.write().await;
            let rule_id_numeric = rule_id.parse::<u64>().unwrap_or(0);
            engine.remove_rule(rule_id_numeric).map_err(|e| {
                error!(rule_id = %rule_id, error = %e, "Failed to remove rule from engine");
                // Re-insert the rule in memory since engine removal failed
                // This would require getting the rule data back, but for now we log the error
                // In production, this would need proper rollback
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError::new(
                        "ENGINE_ERROR",
                        &format!("Failed to remove rule from engine: {}", e),
                    )),
                )
            })?;
            drop(engine); // Release write lock early

            info!(rule_id = %rule_id, "Rule deleted successfully from memory and engine");
            Ok(StatusCode::NO_CONTENT)
        }
        None => {
            warn!(rule_id = %rule_id, "Cannot delete: rule not found");
            Err((
                StatusCode::NOT_FOUND,
                Json(ApiError::new(
                    "RULE_NOT_FOUND",
                    &format!("Rule with ID '{}' not found", rule_id),
                )),
            ))
        }
    }
}

/// List rules with optional filtering
#[utoipa::path(
    get,
    path = "/rules",
    tag = "rules",
    params(ListRulesQuery),
    responses(
        (status = 200, description = "Rules retrieved successfully", body = ListRulesResponse)
    )
)]
#[instrument(skip(state))]
async fn list_rules_handler(
    State(state): State<AppState>,
    Query(query): Query<ListRulesQuery>,
) -> Json<ListRulesResponse> {
    let rules = state.rules.lock().await;
    let mut filtered_rules: Vec<ApiRule> = rules.values().cloned().collect();

    // Apply filters
    if let Some(ref tags) = query.tags {
        filtered_rules.retain(|rule| tags.iter().any(|tag| rule.tags.contains(tag)));
    }

    if let Some(enabled) = query.enabled {
        filtered_rules.retain(|rule| rule.enabled == enabled);
    }

    if let Some(ref search) = query.search {
        let search_lower = search.to_lowercase();
        filtered_rules.retain(|rule| {
            rule.name.to_lowercase().contains(&search_lower)
                || rule
                    .description
                    .as_ref()
                    .map_or(false, |desc| desc.to_lowercase().contains(&search_lower))
        });
    }

    let total = filtered_rules.len();

    // Apply pagination
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50).min(1000); // Cap at 1000

    if offset < filtered_rules.len() {
        let end = (offset + limit).min(filtered_rules.len());
        filtered_rules = filtered_rules[offset..end].to_vec();
    } else {
        filtered_rules.clear();
    }

    info!(
        total_rules = total,
        returned_rules = filtered_rules.len(),
        "Rules listed successfully"
    );

    let count = filtered_rules.len();
    Json(ListRulesResponse { rules: filtered_rules, total, count })
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
#[instrument(skip(state))]
async fn get_engine_stats_handler(State(state): State<AppState>) -> Json<EngineStats> {
    let engine = state.engine.read().await;
    let core_stats = engine.get_stats();

    let stats = EngineStats {
        total_facts: core_stats.fact_count,
        total_rules: core_stats.rule_count,
        network_nodes: core_stats.node_count,
        memory_usage_bytes: core_stats.memory_usage_bytes,
    };

    info!("Engine statistics retrieved");
    Json(stats)
}

// Utility functions for converting between API and core types

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

    Ok(Rule {
        id: api_rule.id.parse::<u64>().unwrap_or(0),
        name: api_rule.name.clone(),
        conditions,
        actions,
    })
}

fn convert_api_fact_to_core(api_fact: &ApiFact, next_id: &mut u64) -> CoreFact {
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

    // Preserve user-provided ID if present, otherwise generate new one
    let fact_id = if let Ok(user_id) = api_fact.id.parse::<u64>() {
        user_id // Use user-provided ID
    } else {
        let generated = *next_id;
        *next_id += 1;
        generated // Generate new ID
    };

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
                arr.iter().map(|v| convert_fact_value_to_json(v)).collect();
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

fn convert_core_fact_to_api(core_fact: &CoreFact) -> ApiFact {
    let mut data = HashMap::new();

    for (key, value) in &core_fact.data.fields {
        let json_value = convert_fact_value_to_json(value);
        data.insert(key.clone(), json_value);
    }

    ApiFact { id: Uuid::new_v4().to_string(), data, created_at: Utc::now() }
}
