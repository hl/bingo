#![cfg(any())]
//! Tests for API security hardening
//!
//! These tests validate that the security measures prevent various types of attacks
//! and properly enforce limits to protect against DoS attempts.

use axum::http::StatusCode;
use axum_test::TestServer;
use bingo_api::{create_app, types::*};
use serde_json::json;

/// Helper function to create a test server
async fn create_test_server() -> TestServer {
    let app = create_app().expect("Failed to create app");
    TestServer::new(app).expect("Failed to create test server")
}

/// Helper function to create a simple test rule
fn create_simple_rule(id: &str) -> ApiRule {
    ApiRule {
        id: id.to_string(),
        name: "Simple Test Rule".to_string(),
        description: Some("A simple test rule".to_string()),
        conditions: vec![ApiCondition::Simple {
            field: "status".to_string(),
            operator: "equal".to_string(),
            value: json!("active"),
        }],
        actions: vec![ApiAction::Log {
            level: "info".to_string(),
            message: "Rule fired".to_string(),
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn test_security_limits_endpoint() {
    let server = create_test_server().await;

    let response = server.get("/security/limits").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let limits: serde_json::Value = response.json();

    // Verify security limits are present
    assert!(limits["security_limits"]["max_expression_complexity"].is_number());
    assert!(limits["security_limits"]["max_rules_per_request"].is_number());
    assert!(limits["enforcement"]["request_timeout_seconds"].is_number());
    assert!(limits["enforcement"]["max_request_body_bytes"].is_number());
}

#[tokio::test]
async fn test_complex_expression_rejection() {
    let server = create_test_server().await;

    // Create a rule with an overly complex expression
    let complex_expression = "(".repeat(100) + &"+".repeat(500) + &")".repeat(100);

    let rule = ApiRule {
        id: "complex-rule".to_string(),
        name: "Complex Rule".to_string(),
        description: Some("Rule with complex expression".to_string()),
        conditions: vec![ApiCondition::Simple {
            field: "status".to_string(),
            operator: "equal".to_string(),
            value: json!("active"),
        }],
        actions: vec![ApiAction::Formula {
            field: "result".to_string(),
            expression: complex_expression,
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let request = EvaluateRequest {
        facts: vec![],
        rules: Some(vec![rule]),
        ruleset_id: None,
        response_format: None,
        streaming_config: None,
    };

    let response = server.post("/evaluate").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    // The important thing is that malicious requests are rejected
    // Don't care which validation layer catches it as long as it's caught
}

#[tokio::test]
async fn test_too_many_rules_rejection() {
    let server = create_test_server().await;

    // Create more rules than the security limit allows
    let rules: Vec<ApiRule> =
        (0..1001).map(|i| create_simple_rule(&format!("rule-{}", i))).collect();

    let request = EvaluateRequest {
        facts: vec![],
        rules: Some(rules),
        ruleset_id: None,
        response_format: None,
        streaming_config: None,
    };

    let response = server.post("/evaluate").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    // Malicious request should be rejected
}

#[tokio::test]
async fn test_oversized_calculator_input_rejection() {
    let server = create_test_server().await;

    // Create a rule with too many calculator inputs
    let mut input_mapping = std::collections::HashMap::new();
    for i in 0..101 {
        // Max is 100
        input_mapping.insert(format!("input_{}", i), format!("value_{}", i));
    }

    let rule = ApiRule {
        id: "calc-rule".to_string(),
        name: "Calculator Rule".to_string(),
        description: Some("Rule with too many calculator inputs".to_string()),
        conditions: vec![ApiCondition::Simple {
            field: "status".to_string(),
            operator: "equal".to_string(),
            value: json!("active"),
        }],
        actions: vec![ApiAction::CallCalculator {
            calculator_name: "test_calc".to_string(),
            input_mapping,
            output_field: "result".to_string(),
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let request = EvaluateRequest {
        facts: vec![],
        rules: Some(vec![rule]),
        ruleset_id: None,
        response_format: None,
        streaming_config: None,
    };

    let response = server.post("/evaluate").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    // Malicious request should be rejected
}

#[tokio::test]
async fn test_valid_request_passes_security() {
    let server = create_test_server().await;

    let fact = ApiFact {
        id: "test-fact".to_string(),
        data: json!({
            "status": "active",
            "amount": 100.0
        })
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect(),
        created_at: chrono::Utc::now(),
    };

    let rule = create_simple_rule("valid-rule");

    let request = EvaluateRequest {
        facts: vec![fact],
        rules: Some(vec![rule]),
        ruleset_id: None,
        response_format: Some(ResponseFormat::Standard),
        streaming_config: None,
    };

    let response = server.post("/evaluate").json(&request).await;

    // Should succeed - security validation should pass
    assert_eq!(response.status_code(), StatusCode::OK);

    let eval_response: EvaluateResponse = response.json();
    assert!(eval_response.results.is_some());
}

#[tokio::test]
async fn test_long_expression_rejection() {
    let server = create_test_server().await;

    // Create an expression that's too long
    let long_expression = "a + ".repeat(5000); // Way over the limit

    let rule = ApiRule {
        id: "long-expr-rule".to_string(),
        name: "Long Expression Rule".to_string(),
        description: Some("Rule with very long expression".to_string()),
        conditions: vec![ApiCondition::Simple {
            field: "status".to_string(),
            operator: "equal".to_string(),
            value: json!("active"),
        }],
        actions: vec![ApiAction::Formula {
            field: "result".to_string(),
            expression: long_expression,
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let request = EvaluateRequest {
        facts: vec![],
        rules: Some(vec![rule]),
        ruleset_id: None,
        response_format: None,
        streaming_config: None,
    };

    let response = server.post("/evaluate").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    // Malicious request should be rejected
}
