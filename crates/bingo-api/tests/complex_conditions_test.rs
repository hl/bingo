//! Tests for complex condition support
//!
//! These tests validate that the API correctly handles complex conditions
//! with logical operators (AND/OR) after Phase 2 implementation.

use axum::http::StatusCode;
use axum_test::TestServer;
use bingo_api::{create_app, types::*};
use serde_json::json;

/// Helper function to create a test server
async fn create_test_server() -> TestServer {
    let app = create_app().expect("Failed to create app");
    TestServer::new(app).expect("Failed to create test server")
}

#[tokio::test]
async fn test_complex_condition_and_logic() {
    let server = create_test_server().await;

    // Create a rule with complex AND condition
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let rule = ApiRule {
        id: format!("complex-and-rule-{}", timestamp),
        name: "Complex AND Rule".to_string(),
        description: Some("Rule with complex AND condition".to_string()),
        conditions: vec![ApiCondition::Complex {
            operator: "and".to_string(),
            conditions: vec![
                ApiCondition::Simple {
                    field: "status".to_string(),
                    operator: "equal".to_string(),
                    value: json!("active"),
                },
                ApiCondition::Simple {
                    field: "amount".to_string(),
                    operator: "greater_than".to_string(),
                    value: json!(100.0),
                },
            ],
        }],
        actions: vec![ApiAction::Log {
            level: "info".to_string(),
            message: "Complex AND condition fired".to_string(),
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string(), "complex".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let request = CreateRuleRequest { rule };

    let response = server.post("/rules").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let create_response: CreateRuleResponse = response.json();
    assert!(create_response.created);
    assert_eq!(create_response.rule.name, "Complex AND Rule");
}

#[tokio::test]
async fn test_complex_condition_or_logic() {
    let server = create_test_server().await;

    // Create a rule with complex OR condition
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let rule = ApiRule {
        id: format!("complex-or-rule-{}", timestamp),
        name: "Complex OR Rule".to_string(),
        description: Some("Rule with complex OR condition".to_string()),
        conditions: vec![ApiCondition::Complex {
            operator: "or".to_string(),
            conditions: vec![
                ApiCondition::Simple {
                    field: "priority".to_string(),
                    operator: "equal".to_string(),
                    value: json!("high"),
                },
                ApiCondition::Simple {
                    field: "amount".to_string(),
                    operator: "greater_than".to_string(),
                    value: json!(1000.0),
                },
            ],
        }],
        actions: vec![ApiAction::Log {
            level: "warn".to_string(),
            message: "Complex OR condition fired".to_string(),
        }],
        priority: Some(200),
        enabled: true,
        tags: vec!["test".to_string(), "complex".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let request = CreateRuleRequest { rule };

    let response = server.post("/rules").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let create_response: CreateRuleResponse = response.json();
    assert!(create_response.created);
    assert_eq!(create_response.rule.name, "Complex OR Rule");
}

#[tokio::test]
async fn test_nested_complex_conditions() {
    let server = create_test_server().await;

    // Create a rule with nested complex conditions (AND containing OR)
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let rule = ApiRule {
        id: format!("nested-complex-rule-{}", timestamp),
        name: "Nested Complex Rule".to_string(),
        description: Some("Rule with nested complex conditions".to_string()),
        conditions: vec![ApiCondition::Complex {
            operator: "and".to_string(),
            conditions: vec![
                ApiCondition::Simple {
                    field: "status".to_string(),
                    operator: "equal".to_string(),
                    value: json!("active"),
                },
                ApiCondition::Complex {
                    operator: "or".to_string(),
                    conditions: vec![
                        ApiCondition::Simple {
                            field: "category".to_string(),
                            operator: "equal".to_string(),
                            value: json!("premium"),
                        },
                        ApiCondition::Simple {
                            field: "amount".to_string(),
                            operator: "greater_than".to_string(),
                            value: json!(500.0),
                        },
                    ],
                },
            ],
        }],
        actions: vec![ApiAction::Log {
            level: "info".to_string(),
            message: "Nested complex condition fired".to_string(),
        }],
        priority: Some(150),
        enabled: true,
        tags: vec!["test".to_string(), "complex".to_string(), "nested".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let request = CreateRuleRequest { rule };

    let response = server.post("/rules").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let create_response: CreateRuleResponse = response.json();
    assert!(create_response.created);
    assert_eq!(create_response.rule.name, "Nested Complex Rule");
}

#[tokio::test]
async fn test_invalid_logical_operator() {
    let server = create_test_server().await;

    // Create a rule with invalid logical operator
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let rule = ApiRule {
        id: format!("invalid-op-rule-{}", timestamp),
        name: "Invalid Operator Rule".to_string(),
        description: Some("Rule with invalid logical operator".to_string()),
        conditions: vec![ApiCondition::Complex {
            operator: "invalid".to_string(), // Invalid operator
            conditions: vec![ApiCondition::Simple {
                field: "status".to_string(),
                operator: "equal".to_string(),
                value: json!("active"),
            }],
        }],
        actions: vec![ApiAction::Log {
            level: "info".to_string(),
            message: "Should not fire".to_string(),
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let request = CreateRuleRequest { rule };

    let response = server.post("/rules").json(&request).await;

    // Should fail with bad request due to invalid operator
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    let error_response: ApiError = response.json();
    println!("Error response: {:?}", error_response);
    assert!(
        error_response.message.contains("Unknown logical operator")
            || error_response.message.contains("invalid")
    );
}
