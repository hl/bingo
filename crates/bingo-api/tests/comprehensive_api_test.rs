//! Comprehensive API Integration Tests
//!
//! This module provides comprehensive integration tests that validate the complete
//! API functionality including error handling, security validation, caching,
//! and all endpoint interactions.

use axum::http::StatusCode;
use axum_test::TestServer;
use bingo_api::{create_app, types::*};
use serde_json::json;
use uuid::Uuid;

/// Create a test server instance
async fn create_test_server() -> TestServer {
    let app = create_app().await.expect("Failed to create app");
    TestServer::new(app).expect("Failed to create test server")
}

/// Test health endpoint functionality
#[tokio::test]
async fn test_health_endpoint() {
    let server = create_test_server().await;

    let response = server.get("/health").await;

    response.assert_status_ok();
    response.assert_json(&json!({
        "status": "healthy",
        "version": "1.0.0"
    }));

    // Validate response structure
    let health_response: HealthResponse = response.json();
    assert_eq!(health_response.status, "healthy");
    assert_eq!(health_response.version, "1.0.0");
    assert!(health_response.uptime_seconds > 0);
}

/// Test basic rule evaluation endpoint
#[tokio::test]
async fn test_basic_evaluation() {
    let server = create_test_server().await;

    let request_payload = json!({
        "facts": [
            {
                "id": "test-fact",
                "data": {
                    "hours_worked": 45.0,
                    "employee_id": 12345,
                    "status": "active"
                },
                "created_at": "2024-01-01T09:00:00Z"
            }
        ],
        "rules": [
            {
                "id": "overtime-rule",
                "name": "Overtime Detection",
                "description": "Detect overtime hours",
                "conditions": [
                    {
                        "type": "simple",
                        "field": "hours_worked",
                        "operator": "GreaterThan",
                        "value": 40.0
                    }
                ],
                "actions": [
                    {
                        "type": "set_field",
                        "field": "overtime",
                        "value": true
                    }
                ],
                "priority": 100,
                "enabled": true,
                "tags": ["test"],
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ],
        "response_format": "json"
    });

    let response = server.post("/evaluate").json(&request_payload).await;

    response.assert_status_ok();

    // Validate response structure
    let evaluate_response: EvaluateResponse = response.json();
    assert_eq!(evaluate_response.facts_processed, 1);
    assert_eq!(evaluate_response.rules_processed, 1);
    assert_eq!(evaluate_response.rules_fired, 1);
    assert!(!evaluate_response.results.unwrap().is_empty());
}

/// Test ruleset registration and caching
#[tokio::test]
async fn test_ruleset_caching_workflow() {
    let server = create_test_server().await;

    let ruleset_id = format!("test-ruleset-{}", Uuid::new_v4());

    // Step 1: Register a ruleset
    let register_payload = json!({
        "ruleset_id": ruleset_id,
        "rules": [
            {
                "id": "cache-test-rule",
                "name": "Cache Test Rule",
                "description": "Rule for testing caching functionality",
                "conditions": [
                    {
                        "type": "simple",
                        "field": "test_field",
                        "operator": "Equal",
                        "value": "test_value"
                    }
                ],
                "actions": [
                    {
                        "type": "set_field",
                        "field": "result",
                        "value": "cached_result"
                    }
                ],
                "priority": 100,
                "enabled": true,
                "tags": ["cache", "test"],
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ],
        "ttl_seconds": 300,
        "description": "Test ruleset for caching validation"
    });

    let register_response = server.post("/rulesets").json(&register_payload).await;
    register_response.assert_status(StatusCode::CREATED);

    let registration_result: RegisterRulesetResponse = register_response.json();
    assert_eq!(registration_result.ruleset_id, ruleset_id);
    assert!(registration_result.compiled);
    assert_eq!(registration_result.rule_count, 1);

    // Step 2: Use the cached ruleset for evaluation
    let evaluate_payload = json!({
        "ruleset_id": ruleset_id,
        "facts": [
            {
                "id": "cache-test-fact",
                "data": {
                    "test_field": "test_value"
                },
                "created_at": "2024-01-01T09:00:00Z"
            }
        ],
        "response_format": "json"
    });

    let evaluate_response = server.post("/evaluate").json(&evaluate_payload).await;
    evaluate_response.assert_status_ok();

    let evaluation_result: EvaluateResponse = evaluate_response.json();
    assert_eq!(evaluation_result.facts_processed, 1);
    assert_eq!(evaluation_result.rules_fired, 1);
}

/// Test security validation and error handling
#[tokio::test]
async fn test_security_validation() {
    let server = create_test_server().await;

    // Test with dangerous field name that should be rejected
    let malicious_payload = json!({
        "facts": [
            {
                "id": "test-fact",
                "data": {
                    "normal_field": "value"
                },
                "created_at": "2024-01-01T09:00:00Z"
            }
        ],
        "rules": [
            {
                "id": "malicious-rule",
                "name": "Security Test",
                "description": "Rule with dangerous field",
                "conditions": [
                    {
                        "type": "simple",
                        "field": "__proto__",
                        "operator": "Equal",
                        "value": "attack"
                    }
                ],
                "actions": [
                    {
                        "type": "set_field",
                        "field": "result",
                        "value": true
                    }
                ],
                "priority": 100,
                "enabled": true,
                "tags": ["security", "test"],
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ],
        "response_format": "json"
    });

    let response = server.post("/evaluate").json(&malicious_payload).await;
    response.assert_status(StatusCode::BAD_REQUEST);

    // Validate error response structure
    response.assert_json(&json!({
        "code": "SECURITY_VALIDATION_ERROR"
    }));
}

/// Test validation error handling
#[tokio::test]
async fn test_validation_error_handling() {
    let server = create_test_server().await;

    // Test with invalid rule (empty name)
    let invalid_payload = json!({
        "facts": [
            {
                "id": "test-fact",
                "data": {
                    "field": "value"
                },
                "created_at": "2024-01-01T09:00:00Z"
            }
        ],
        "rules": [
            {
                "id": "invalid-rule",
                "name": "",
                "description": "Rule with empty name",
                "conditions": [],
                "actions": [],
                "priority": 100,
                "enabled": true,
                "tags": ["validation", "test"],
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ],
        "response_format": "json"
    });

    let response = server.post("/evaluate").json(&invalid_payload).await;
    response.assert_status(StatusCode::BAD_REQUEST);

    response.assert_json(&json!({
        "code": "VALIDATION_ERROR"
    }));
}

/// Test cache statistics endpoint
#[tokio::test]
async fn test_cache_stats_endpoint() {
    let server = create_test_server().await;

    let response = server.get("/cache/stats").await;
    response.assert_status_ok();

    // Validate that response contains cache statistics
    let stats: serde_json::Value = response.json();
    assert!(stats.get("unified_cache").is_some());
}

/// Test engine statistics endpoint
#[tokio::test]
async fn test_engine_stats_endpoint() {
    let server = create_test_server().await;

    let response = server.get("/engine/stats").await;
    response.assert_status_ok();

    let stats: EngineStats = response.json();
    // In stateless mode, these should be 0
    assert_eq!(stats.total_facts, 0);
    assert_eq!(stats.total_rules, 0);
    assert_eq!(stats.network_nodes, 0);
}

/// Test security limits endpoint
#[tokio::test]
async fn test_security_limits_endpoint() {
    let server = create_test_server().await;

    let response = server.get("/security/limits").await;
    response.assert_status_ok();

    let limits: serde_json::Value = response.json();
    assert!(limits.get("security_limits").is_some());
    assert!(limits.get("enforcement").is_some());
}

/// Test metrics endpoint
#[tokio::test]
async fn test_metrics_endpoint() {
    let server = create_test_server().await;

    let response = server.get("/metrics").await;
    response.assert_status_ok();

    // Metrics should be in Prometheus format
    assert!(
        response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("text/plain")
    );
}

/// Test large fact processing and memory management
#[tokio::test]
async fn test_large_fact_processing() {
    let server = create_test_server().await;

    // Create a larger set of facts to test memory management
    let mut facts = Vec::new();
    for i in 0..1000 {
        facts.push(json!({
            "id": format!("fact-{}", i),
            "data": {
                "employee_id": i,
                "hours_worked": 40.0 + (i % 10) as f64,
                "department": "test"
            },
            "created_at": "2024-01-01T09:00:00Z"
        }));
    }

    let large_payload = json!({
        "facts": facts,
        "rules": [
            {
                "id": "bulk-test-rule",
                "name": "Bulk Test Rule",
                "description": "Rule for testing large fact processing",
                "conditions": [
                    {
                        "type": "simple",
                        "field": "hours_worked",
                        "operator": "GreaterThan",
                        "value": 45.0
                    }
                ],
                "actions": [
                    {
                        "type": "set_field",
                        "field": "overtime",
                        "value": true
                    }
                ],
                "priority": 100,
                "enabled": true,
                "tags": ["bulk", "test"],
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ],
        "response_format": "json",
        "streaming_config": {
            "incremental_processing": true,
            "fact_batch_size": 100,
            "memory_limit_mb": 512
        }
    });

    let response = server.post("/evaluate").json(&large_payload).await;
    response.assert_status_ok();

    let evaluation_result: EvaluateResponse = response.json();
    assert_eq!(evaluation_result.facts_processed, 1000);
    assert_eq!(evaluation_result.rules_processed, 1);
    // Should fire for facts with hours > 45 (which is every 6th fact starting from fact-5)
    assert!(evaluation_result.rules_fired > 0);
}

/// Test OpenAPI documentation endpoints
#[tokio::test]
async fn test_openapi_documentation() {
    let server = create_test_server().await;

    // Test Swagger UI
    let response = server.get("/docs").await;
    response.assert_status_ok();

    // Test OpenAPI JSON
    let response = server.get("/api-docs/openapi.json").await;
    response.assert_status_ok();

    // Test ReDoc
    let response = server.get("/redoc").await;
    response.assert_status_ok();

    // Test RapiDoc
    let response = server.get("/rapidoc").await;
    response.assert_status_ok();
}
