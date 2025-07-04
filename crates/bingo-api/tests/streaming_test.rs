//! Tests for streaming response functionality
//!
//! These tests validate that the streaming API works correctly for large result sets
//! and that memory usage remains controlled.

use axum::http::StatusCode;
use axum_test::TestServer;
use bingo_api::{create_app, types::*};
use serde_json::json;

/// Helper function to create a test server
async fn create_test_server() -> TestServer {
    let app = create_app().await.expect("Failed to create app");
    TestServer::new(app).expect("Failed to create test server")
}

/// Helper function to create test facts
fn create_test_facts(count: usize, prefix: &str) -> Vec<ApiFact> {
    (0..count)
        .map(|i| ApiFact {
            id: format!("{}-{}", prefix, i),
            data: json!({
                "entity_id": i,
                "amount": i as f64 * 10.5,
                "status": "active",
                "category": format!("cat_{}", i % 5)
            })
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
            created_at: chrono::Utc::now(),
        })
        .collect()
}

/// Helper function to create a test rule that will fire
fn create_firing_rule(id: &str, name: &str) -> ApiRule {
    ApiRule {
        id: id.to_string(),
        name: name.to_string(),
        description: Some(format!("Test rule {}", name)),
        conditions: vec![ApiCondition::Simple {
            field: "status".to_string(),
            operator: ApiSimpleOperator::Equal,
            value: json!("active"),
        }],
        actions: vec![ApiAction::Log {
            level: "info".to_string(),
            message: format!("Rule {} fired for entity {{entity_id}}", name),
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn test_auto_streaming_threshold() {
    let server = create_test_server().await;

    // Create enough facts to trigger auto-streaming (over 1000)
    let facts = create_test_facts(1200, "stream-test");
    let rules = vec![create_firing_rule("rule-1", "Auto Stream Test Rule")];

    let request = EvaluateRequest {
        facts,
        rules: Some(rules),
        ruleset_id: None,
        response_format: Some(ResponseFormat::Auto), // Auto-detect should trigger streaming
        streaming_config: None,
    };

    let response = server.post("/evaluate").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::OK);

    // Check that the response uses streaming format
    let headers = response.headers();
    assert_eq!(headers.get("content-type").unwrap(), "application/x-ndjson");
    assert!(headers.contains_key("x-streaming-format"));
    assert_eq!(headers.get("x-streaming-format").unwrap(), "ndjson");
}

#[tokio::test]
async fn test_explicit_streaming_request() {
    let server = create_test_server().await;

    // Use small dataset but explicitly request streaming
    let facts = create_test_facts(50, "explicit-stream-test");
    let rules = vec![create_firing_rule("rule-1", "Explicit Stream Test Rule")];

    let request = EvaluateRequest {
        facts,
        rules: Some(rules),
        ruleset_id: None,
        response_format: Some(ResponseFormat::Stream),
        streaming_config: Some(StreamingConfig {
            result_threshold: None,
            chunk_size: Some(10),
            include_progress: Some(true),
            incremental_processing: None,
            fact_batch_size: None,
            memory_limit_mb: None,
        }),
    };

    let response = server.post("/evaluate").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::OK);

    // Verify streaming headers
    let headers = response.headers();
    assert_eq!(headers.get("content-type").unwrap(), "application/x-ndjson");
    assert!(headers.contains_key("x-total-results"));
    assert!(headers.contains_key("x-chunk-size"));
    assert_eq!(headers.get("x-chunk-size").unwrap(), "10");
}

#[tokio::test]
async fn test_standard_response_format() {
    let server = create_test_server().await;

    // Small dataset with explicit standard format
    let facts = create_test_facts(10, "standard-test");
    let rules = vec![create_firing_rule("rule-1", "Standard Test Rule")];

    let request = EvaluateRequest {
        facts,
        rules: Some(rules),
        ruleset_id: None,
        response_format: Some(ResponseFormat::Standard),
        streaming_config: None,
    };

    let response = server.post("/evaluate").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::OK);

    // Should be standard JSON response
    let headers = response.headers();
    assert_eq!(headers.get("content-type").unwrap(), "application/json");
    assert!(!headers.contains_key("x-streaming-format"));

    let eval_response: EvaluateResponse = response.json();

    // Standard response should have results populated
    assert!(eval_response.results.is_some());
    assert!(eval_response.streaming.is_none());
    assert!(!eval_response.results.unwrap().is_empty());
}

#[tokio::test]
async fn test_memory_safety_override() {
    let server = create_test_server().await;

    // Create very large dataset to test memory safety override
    let facts = create_test_facts(12000, "memory-safety-test"); // Above MEMORY_SAFETY_THRESHOLD
    let rules = vec![create_firing_rule("rule-1", "Memory Safety Test Rule")];

    let request = EvaluateRequest {
        facts,
        rules: Some(rules),
        ruleset_id: None,
        response_format: Some(ResponseFormat::Standard), // Explicitly request standard...
        streaming_config: None,
    };

    let response = server.post("/evaluate").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::OK);

    // Memory safety should override and force streaming
    let headers = response.headers();
    assert_eq!(headers.get("content-type").unwrap(), "application/x-ndjson");
    assert!(headers.contains_key("x-streaming-format"));
}

#[tokio::test]
async fn test_streaming_with_custom_config() {
    let server = create_test_server().await;

    let facts = create_test_facts(300, "custom-config-test");
    let rules = vec![create_firing_rule("rule-1", "Custom Config Test Rule")];

    let request = EvaluateRequest {
        facts,
        rules: Some(rules),
        ruleset_id: None,
        response_format: Some(ResponseFormat::Auto),
        streaming_config: Some(StreamingConfig {
            result_threshold: Some(200), // Lower threshold than default
            chunk_size: Some(25),
            include_progress: Some(false),
            incremental_processing: None,
            fact_batch_size: None,
            memory_limit_mb: None,
        }),
    };

    let response = server.post("/evaluate").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::OK);

    // Should trigger streaming with custom threshold
    let headers = response.headers();
    assert_eq!(headers.get("content-type").unwrap(), "application/x-ndjson");
    assert_eq!(headers.get("x-chunk-size").unwrap(), "25");
}
