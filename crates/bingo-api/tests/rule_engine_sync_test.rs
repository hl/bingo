//! Tests for rule engine synchronization
//!
//! These tests validate that rule updates and deletes are properly
//! synchronized between the API memory store and the engine.

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
async fn test_rule_update_synchronization() {
    let server = create_test_server().await;

    // First, create a rule
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let rule_id = format!("sync-test-rule-{}", timestamp);
    let original_rule = ApiRule {
        id: rule_id.clone(),
        name: "Original Rule".to_string(),
        description: Some("Original description".to_string()),
        conditions: vec![ApiCondition::Simple {
            field: "status".to_string(),
            operator: "equal".to_string(),
            value: json!("active"),
        }],
        actions: vec![ApiAction::Log {
            level: "info".to_string(),
            message: "Original rule fired".to_string(),
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string(), "sync".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let create_request = CreateRuleRequest { rule: original_rule };
    let create_response = server.post("/rules").json(&create_request).await;
    assert_eq!(create_response.status_code(), StatusCode::CREATED);

    let create_result: CreateRuleResponse = create_response.json();
    assert!(create_result.created);

    // Now update the rule
    let updated_rule = ApiRule {
        id: rule_id.clone(),
        name: "Updated Rule".to_string(),
        description: Some("Updated description".to_string()),
        conditions: vec![ApiCondition::Simple {
            field: "status".to_string(),
            operator: "equal".to_string(),
            value: json!("updated"),
        }],
        actions: vec![
            ApiAction::Log { level: "warn".to_string(), message: "Updated rule fired".to_string() },
            ApiAction::SetField { field: "processed".to_string(), value: json!(true) },
        ],
        priority: Some(200),
        enabled: true,
        tags: vec!["test".to_string(), "sync".to_string(), "updated".to_string()],
        created_at: create_result.rule.created_at, // Keep original created_at
        updated_at: chrono::Utc::now(),
    };

    let update_response = server.put(&format!("/rules/{}", rule_id)).json(&updated_rule).await;

    assert_eq!(update_response.status_code(), StatusCode::OK);

    let update_result: CreateRuleResponse = update_response.json();
    assert!(!update_result.created); // Should be false for updates
    assert_eq!(update_result.rule.name, "Updated Rule");
    assert_eq!(update_result.rule.actions.len(), 2);

    // Verify the rule can still be retrieved
    let get_response = server.get(&format!("/rules/{}", rule_id)).await;
    assert_eq!(get_response.status_code(), StatusCode::OK);

    let get_result: ApiRule = get_response.json();
    assert_eq!(get_result.name, "Updated Rule");
    assert_eq!(get_result.description.unwrap(), "Updated description");

    // Verify engine stats reflect the rule (should still be 1 rule, just updated)
    let stats_response = server.get("/engine/stats").await;
    assert_eq!(stats_response.status_code(), StatusCode::OK);

    let stats: EngineStats = stats_response.json();
    assert!(stats.total_rules >= 1); // At least our rule exists
}

#[tokio::test]
async fn test_rule_delete_synchronization() {
    let server = create_test_server().await;

    // First, create a rule
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let rule_id = format!("delete-test-rule-{}", timestamp);
    let rule = ApiRule {
        id: rule_id.clone(),
        name: "Rule to Delete".to_string(),
        description: Some("This rule will be deleted".to_string()),
        conditions: vec![ApiCondition::Simple {
            field: "type".to_string(),
            operator: "equal".to_string(),
            value: json!("deletable"),
        }],
        actions: vec![ApiAction::Log {
            level: "info".to_string(),
            message: "About to be deleted".to_string(),
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string(), "delete".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let create_request = CreateRuleRequest { rule };
    let create_response = server.post("/rules").json(&create_request).await;
    assert_eq!(create_response.status_code(), StatusCode::CREATED);

    // Get initial engine stats
    let initial_stats_response = server.get("/engine/stats").await;
    assert_eq!(initial_stats_response.status_code(), StatusCode::OK);
    let initial_stats: EngineStats = initial_stats_response.json();
    let initial_rule_count = initial_stats.total_rules;

    // Verify the rule exists
    let get_response = server.get(&format!("/rules/{}", rule_id)).await;
    assert_eq!(get_response.status_code(), StatusCode::OK);

    // Now delete the rule
    let delete_response = server.delete(&format!("/rules/{}", rule_id)).await;
    assert_eq!(delete_response.status_code(), StatusCode::NO_CONTENT);

    // Verify the rule no longer exists in API
    let get_after_delete_response = server.get(&format!("/rules/{}", rule_id)).await;
    assert_eq!(
        get_after_delete_response.status_code(),
        StatusCode::NOT_FOUND
    );

    // Verify engine stats reflect the deletion
    let final_stats_response = server.get("/engine/stats").await;
    assert_eq!(final_stats_response.status_code(), StatusCode::OK);

    let final_stats: EngineStats = final_stats_response.json();
    assert_eq!(final_stats.total_rules, initial_rule_count - 1);
}

#[tokio::test]
async fn test_rule_update_with_invalid_data() {
    let server = create_test_server().await;

    // First, create a rule
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let rule_id = format!("invalid-update-rule-{}", timestamp);
    let rule = ApiRule {
        id: rule_id.clone(),
        name: "Valid Rule".to_string(),
        description: Some("Valid description".to_string()),
        conditions: vec![ApiCondition::Simple {
            field: "status".to_string(),
            operator: "equal".to_string(),
            value: json!("active"),
        }],
        actions: vec![ApiAction::Log {
            level: "info".to_string(),
            message: "Valid rule fired".to_string(),
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let create_request = CreateRuleRequest { rule };
    let create_response = server.post("/rules").json(&create_request).await;
    assert_eq!(create_response.status_code(), StatusCode::CREATED);

    // Try to update with invalid data (empty conditions)
    let invalid_rule = ApiRule {
        id: rule_id.clone(),
        name: "Invalid Rule".to_string(),
        description: Some("This has no conditions".to_string()),
        conditions: vec![], // Invalid: empty conditions
        actions: vec![ApiAction::Log {
            level: "error".to_string(),
            message: "Should not work".to_string(),
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let update_response = server.put(&format!("/rules/{}", rule_id)).json(&invalid_rule).await;

    // Should fail with bad request
    assert_eq!(update_response.status_code(), StatusCode::BAD_REQUEST);

    // Verify the original rule is still intact
    let get_response = server.get(&format!("/rules/{}", rule_id)).await;
    assert_eq!(get_response.status_code(), StatusCode::OK);

    let get_result: ApiRule = get_response.json();
    assert_eq!(get_result.name, "Valid Rule"); // Should still be the original
    assert_eq!(get_result.conditions.len(), 1); // Should still have conditions
}

#[tokio::test]
async fn test_rule_delete_nonexistent() {
    let server = create_test_server().await;

    // Try to delete a rule that doesn't exist
    let nonexistent_rule_id = "nonexistent-rule-12345";
    let delete_response = server.delete(&format!("/rules/{}", nonexistent_rule_id)).await;

    // Should return 404 Not Found
    assert_eq!(delete_response.status_code(), StatusCode::NOT_FOUND);

    let error_response: ApiError = delete_response.json();
    assert!(error_response.message.contains("not found"));
}

#[tokio::test]
async fn test_rule_update_nonexistent() {
    let server = create_test_server().await;

    // Try to update a rule that doesn't exist
    let nonexistent_rule_id = "nonexistent-rule-67890";
    let rule = ApiRule {
        id: nonexistent_rule_id.to_string(),
        name: "Nonexistent Rule".to_string(),
        description: Some("This rule doesn't exist".to_string()),
        conditions: vec![ApiCondition::Simple {
            field: "status".to_string(),
            operator: "equal".to_string(),
            value: json!("active"),
        }],
        actions: vec![ApiAction::Log {
            level: "info".to_string(),
            message: "Should not work".to_string(),
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let update_response = server.put(&format!("/rules/{}", nonexistent_rule_id)).json(&rule).await;

    // Should return 404 Not Found
    assert_eq!(update_response.status_code(), StatusCode::NOT_FOUND);

    let error_response: ApiError = update_response.json();
    assert!(error_response.message.contains("not found"));
}
