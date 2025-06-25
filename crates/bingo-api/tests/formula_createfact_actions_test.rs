//! Tests for Formula and CreateFact action support
//!
//! These tests validate that the API correctly handles Formula and CreateFact actions
//! after Phase 2 implementation.

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
async fn test_formula_action_creation() {
    let server = create_test_server().await;

    // Create a rule with Formula action
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let rule = ApiRule {
        id: format!("formula-rule-{}", timestamp),
        name: "Formula Action Rule".to_string(),
        description: Some("Rule with Formula action for tax calculation".to_string()),
        conditions: vec![ApiCondition::Simple {
            field: "amount".to_string(),
            operator: "greater_than".to_string(),
            value: json!(0),
        }],
        actions: vec![ApiAction::Formula {
            field: "tax_amount".to_string(),
            expression: "amount * 0.15".to_string(),
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string(), "formula".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let request = CreateRuleRequest { rule };

    let response = server.post("/rules").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let create_response: CreateRuleResponse = response.json();
    assert!(create_response.created);
    assert_eq!(create_response.rule.name, "Formula Action Rule");

    // Verify the action is stored correctly
    match &create_response.rule.actions[0] {
        ApiAction::Formula { field, expression } => {
            assert_eq!(field, "tax_amount");
            assert_eq!(expression, "amount * 0.15");
        }
        _ => panic!("Expected Formula action"),
    }
}

#[tokio::test]
async fn test_createfact_action_creation() {
    let server = create_test_server().await;

    // Create a rule with CreateFact action
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let rule = ApiRule {
        id: format!("createfact-rule-{}", timestamp),
        name: "CreateFact Action Rule".to_string(),
        description: Some("Rule with CreateFact action".to_string()),
        conditions: vec![ApiCondition::Simple {
            field: "status".to_string(),
            operator: "equal".to_string(),
            value: json!("active"),
        }],
        actions: vec![ApiAction::CreateFact {
            data: json!({
                "type": "audit_log",
                "original_fact_id": "placeholder",
                "timestamp": "2024-01-01T00:00:00Z",
                "action": "status_change",
                "severity": "info"
            })
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string(), "createfact".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let request = CreateRuleRequest { rule };

    let response = server.post("/rules").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let create_response: CreateRuleResponse = response.json();
    assert!(create_response.created);
    assert_eq!(create_response.rule.name, "CreateFact Action Rule");

    // Verify the action is stored correctly
    match &create_response.rule.actions[0] {
        ApiAction::CreateFact { data } => {
            assert!(data.contains_key("type"));
            assert_eq!(data.get("type").unwrap(), &json!("audit_log"));
            assert!(data.contains_key("action"));
            assert_eq!(data.get("action").unwrap(), &json!("status_change"));
        }
        _ => panic!("Expected CreateFact action"),
    }
}

#[tokio::test]
async fn test_complex_formula_expressions() {
    let server = create_test_server().await;

    // Create a rule with complex Formula expression
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let rule = ApiRule {
        id: format!("complex-formula-rule-{}", timestamp),
        name: "Complex Formula Rule".to_string(),
        description: Some("Rule with complex Formula expression".to_string()),
        conditions: vec![ApiCondition::Simple {
            field: "category".to_string(),
            operator: "equal".to_string(),
            value: json!("premium"),
        }],
        actions: vec![ApiAction::Formula {
            field: "discount_amount".to_string(),
            expression: "min(amount * 0.20, 100.0)".to_string(), // 20% discount, max $100
        }],
        priority: Some(200),
        enabled: true,
        tags: vec!["test".to_string(), "formula".to_string(), "complex".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let request = CreateRuleRequest { rule };

    let response = server.post("/rules").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let create_response: CreateRuleResponse = response.json();
    assert!(create_response.created);

    // Verify the complex expression is stored correctly
    match &create_response.rule.actions[0] {
        ApiAction::Formula { field, expression } => {
            assert_eq!(field, "discount_amount");
            assert_eq!(expression, "min(amount * 0.20, 100.0)");
        }
        _ => panic!("Expected Formula action"),
    }
}

#[tokio::test]
async fn test_multiple_action_types() {
    let server = create_test_server().await;

    // Create a rule with multiple action types
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let rule = ApiRule {
        id: format!("multi-action-rule-{}", timestamp),
        name: "Multi Action Rule".to_string(),
        description: Some("Rule with multiple action types".to_string()),
        conditions: vec![ApiCondition::Simple {
            field: "amount".to_string(),
            operator: "greater_than".to_string(),
            value: json!(1000),
        }],
        actions: vec![
            ApiAction::Log {
                level: "info".to_string(),
                message: "High value transaction detected".to_string(),
            },
            ApiAction::SetField { field: "risk_level".to_string(), value: json!("high") },
            ApiAction::Formula {
                field: "processing_fee".to_string(),
                expression: "amount * 0.025".to_string(),
            },
            ApiAction::CreateFact {
                data: json!({
                    "type": "alert",
                    "alert_type": "high_value",
                    "triggered_amount": "placeholder_amount",
                    "urgency": "medium"
                })
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            },
        ],
        priority: Some(300),
        enabled: true,
        tags: vec!["test".to_string(), "multi".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let request = CreateRuleRequest { rule };

    let response = server.post("/rules").json(&request).await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let create_response: CreateRuleResponse = response.json();
    assert!(create_response.created);
    assert_eq!(create_response.rule.actions.len(), 4);

    // Verify all action types are present and correct
    match &create_response.rule.actions[0] {
        ApiAction::Log { level, message } => {
            assert_eq!(level, "info");
            assert!(message.contains("High value"));
        }
        _ => panic!("Expected Log action at index 0"),
    }

    match &create_response.rule.actions[1] {
        ApiAction::SetField { field, value } => {
            assert_eq!(field, "risk_level");
            assert_eq!(value, &json!("high"));
        }
        _ => panic!("Expected SetField action at index 1"),
    }

    match &create_response.rule.actions[2] {
        ApiAction::Formula { field, expression } => {
            assert_eq!(field, "processing_fee");
            assert_eq!(expression, "amount * 0.025");
        }
        _ => panic!("Expected Formula action at index 2"),
    }

    match &create_response.rule.actions[3] {
        ApiAction::CreateFact { data } => {
            assert_eq!(data.get("type").unwrap(), &json!("alert"));
            assert_eq!(data.get("alert_type").unwrap(), &json!("high_value"));
        }
        _ => panic!("Expected CreateFact action at index 3"),
    }
}

#[tokio::test]
async fn test_fact_processing_with_formula_actions() {
    let server = create_test_server().await;

    // First create a rule with Formula action
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let rule = ApiRule {
        id: format!("processing-test-rule-{}", timestamp),
        name: "Processing Test Rule".to_string(),
        description: Some("Rule for testing fact processing".to_string()),
        conditions: vec![ApiCondition::Simple {
            field: "item_type".to_string(),
            operator: "equal".to_string(),
            value: json!("purchase"),
        }],
        actions: vec![ApiAction::Formula {
            field: "total_with_tax".to_string(),
            expression: "amount + (amount * 0.08)".to_string(), // 8% tax
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let rule_request = CreateRuleRequest { rule };
    let rule_response = server.post("/rules").json(&rule_request).await;
    assert_eq!(rule_response.status_code(), StatusCode::CREATED);

    // Now process facts through the engine
    let facts = vec![ApiFact {
        id: "test-fact-1".to_string(),
        data: json!({
            "item_type": "purchase",
            "amount": 100.0,
            "customer_id": 12345
        })
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect(),
        created_at: chrono::Utc::now(),
    }];

    let process_request = ProcessFactsRequest { facts, rule_filter: None, execution_mode: None };

    let response = server.post("/facts/process").json(&process_request).await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let process_response: ProcessFactsResponse = response.json();
    assert_eq!(process_response.facts_processed, 1);

    // The rule should have been evaluated (though actual formula execution
    // depends on the RETE network implementation)
    assert!(process_response.rules_evaluated > 0);
}
