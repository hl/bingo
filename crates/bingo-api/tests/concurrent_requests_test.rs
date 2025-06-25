//! Integration tests for request handling performance
//!
//! These tests validate that the API performs well after the RwLock improvements
//! and that basic functionality works correctly.

use axum::http::StatusCode;
use axum_test::TestServer;
use bingo_api::{create_app, types::*};
use serde_json::json;
use std::time::{Duration, Instant};

/// Helper function to create a test server
async fn create_test_server() -> TestServer {
    let app = create_app().expect("Failed to create app");
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
                "status": if i % 2 == 0 { "active" } else { "pending" },
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

/// Helper function to create a test rule
fn create_test_rule(id: &str, name: &str) -> ApiRule {
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let unique_id = format!("{}-{}", id, timestamp);

    ApiRule {
        id: unique_id,
        name: name.to_string(),
        description: Some(format!("Test rule {}", name)),
        conditions: vec![ApiCondition::Simple {
            field: "status".to_string(),
            operator: "equal".to_string(),
            value: json!("active"),
        }],
        actions: vec![ApiAction::Log {
            level: "info".to_string(),
            message: format!("Rule {} fired", name),
        }],
        priority: Some(100),
        enabled: true,
        tags: vec!["test".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn test_health_check_performance() {
    let server = create_test_server().await;

    // Measure time for multiple health checks
    let start = Instant::now();

    for i in 0..20 {
        let response = server.get("/health").await;
        assert_eq!(
            response.status_code(),
            StatusCode::OK,
            "Health check {} failed",
            i
        );

        // Verify the response contains expected fields
        let health_response: HealthResponse = response.json();
        assert_eq!(health_response.status, "healthy");
        // uptime_seconds is always >= 0 by type definition
    }

    let total_duration = start.elapsed();
    let avg_duration = total_duration / 20;

    // Each health check should be very fast (using read locks)
    assert!(
        avg_duration < Duration::from_millis(50),
        "Average health check took too long: {:?}",
        avg_duration
    );

    println!(
        "20 health checks completed in {:?} (avg: {:?})",
        total_duration, avg_duration
    );
}

#[tokio::test]
async fn test_fact_processing_performance() {
    let server = create_test_server().await;

    // Test multiple fact processing requests sequentially
    let start = Instant::now();

    for i in 0..5 {
        let facts = create_test_facts(20, &format!("batch-{}", i));

        let request = ProcessFactsRequest { facts, rule_filter: None, execution_mode: None };

        let response = server.post("/facts/process").json(&request).await;

        assert_eq!(
            response.status_code(),
            StatusCode::OK,
            "Fact processing {} failed",
            i
        );

        // Verify the response structure
        let process_response: ProcessFactsResponse = response.json();
        assert_eq!(process_response.facts_processed, 20);
        assert!(!process_response.request_id.is_empty());
    }

    let total_duration = start.elapsed();
    let avg_duration = total_duration / 5;

    // Fact processing should complete reasonably quickly
    assert!(
        avg_duration < Duration::from_secs(2),
        "Average fact processing took too long: {:?}",
        avg_duration
    );

    println!(
        "5 fact processing requests completed in {:?} (avg: {:?})",
        total_duration, avg_duration
    );
}

#[tokio::test]
async fn test_rule_crud_operations() {
    let server = create_test_server().await;

    // Create multiple rules
    let start = Instant::now();
    let mut created_rule_ids = Vec::new();

    for i in 0..5 {
        let rule = create_test_rule(&format!("rule-{}", i), &format!("Test Rule {}", i));
        let rule_id = rule.id.clone();
        created_rule_ids.push(rule_id.clone());

        let request = CreateRuleRequest { rule };

        let response = server.post("/rules").json(&request).await;

        if response.status_code() != StatusCode::CREATED {
            let body = response.text();
            println!(
                "Rule creation {} failed with status: {}, body: {}",
                i,
                response.status_code(),
                body
            );
        }

        assert_eq!(
            response.status_code(),
            StatusCode::CREATED,
            "Rule creation {} failed",
            i
        );

        let create_response: CreateRuleResponse = response.json();
        assert!(create_response.created);
        assert_eq!(create_response.rule.id, rule_id);
    }

    let rule_creation_duration = start.elapsed();

    // List all rules
    let list_start = Instant::now();
    let response = server.get("/rules").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let list_response: ListRulesResponse = response.json();
    assert_eq!(list_response.count, 5);
    assert_eq!(list_response.total, 5);

    let list_duration = list_start.elapsed();

    // Get individual rules
    let get_start = Instant::now();
    for rule_id in created_rule_ids.iter() {
        let response = server.get(&format!("/rules/{}", rule_id)).await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let rule: ApiRule = response.json();
        assert_eq!(rule.id, *rule_id);
    }
    let get_duration = get_start.elapsed();

    println!(
        "Rule operations - Create: {:?}, List: {:?}, Get 5 rules: {:?}",
        rule_creation_duration, list_duration, get_duration
    );

    // All read operations should be fast with RwLock
    assert!(list_duration < Duration::from_millis(100));
    assert!(get_duration < Duration::from_millis(200));
}

#[tokio::test]
async fn test_engine_stats_performance() {
    let server = create_test_server().await;

    // Test engine stats endpoint performance
    let start = Instant::now();

    for i in 0..10 {
        let response = server.get("/engine/stats").await;
        assert_eq!(
            response.status_code(),
            StatusCode::OK,
            "Engine stats {} failed",
            i
        );

        let stats: EngineStats = response.json();
        // total_facts and total_rules are always >= 0 by type definition
        assert!(stats.memory_usage_bytes > 0);
    }

    let total_duration = start.elapsed();
    let avg_duration = total_duration / 10;

    // Engine stats should be very fast (read-only operation)
    assert!(
        avg_duration < Duration::from_millis(30),
        "Average engine stats took too long: {:?}",
        avg_duration
    );

    println!(
        "10 engine stats requests completed in {:?} (avg: {:?})",
        total_duration, avg_duration
    );
}

#[tokio::test]
async fn test_mixed_operations_performance() {
    let server = create_test_server().await;

    // Create some rules first
    for i in 0..3 {
        let rule = create_test_rule(
            &format!("mixed-rule-{}", i),
            &format!("Mixed Test Rule {}", i),
        );
        let request = CreateRuleRequest { rule };
        let response = server.post("/rules").json(&request).await;
        assert_eq!(response.status_code(), StatusCode::CREATED);
    }

    // Now perform mixed read and write operations
    let start = Instant::now();

    // Interleave read and write operations
    for i in 0..10 {
        if i % 3 == 0 {
            // Write operation: process facts
            let facts = create_test_facts(5, &format!("mixed-{}", i));
            let request = ProcessFactsRequest { facts, rule_filter: None, execution_mode: None };
            let response = server.post("/facts/process").json(&request).await;
            assert_eq!(response.status_code(), StatusCode::OK);
        } else if i % 3 == 1 {
            // Read operation: health check
            let response = server.get("/health").await;
            assert_eq!(response.status_code(), StatusCode::OK);
        } else {
            // Read operation: engine stats
            let response = server.get("/engine/stats").await;
            assert_eq!(response.status_code(), StatusCode::OK);
        }
    }

    let total_duration = start.elapsed();
    let avg_duration = total_duration / 10;

    // Mixed operations should still perform reasonably well
    assert!(
        avg_duration < Duration::from_millis(500),
        "Average mixed operation took too long: {:?}",
        avg_duration
    );

    println!(
        "10 mixed operations completed in {:?} (avg: {:?})",
        total_duration, avg_duration
    );
}

#[tokio::test]
async fn test_request_size_handling() {
    let server = create_test_server().await;

    // Create a reasonable-sized request (well under 50MB limit)
    let large_facts = create_test_facts(1000, "large-batch");

    let request =
        ProcessFactsRequest { facts: large_facts, rule_filter: None, execution_mode: None };

    // This should succeed as it's under the limit
    let start = Instant::now();
    let response = server.post("/facts/process").json(&request).await;
    let duration = start.elapsed();

    assert_eq!(response.status_code(), StatusCode::OK);

    let process_response: ProcessFactsResponse = response.json();
    assert_eq!(process_response.facts_processed, 1000);

    // Even large requests should complete in reasonable time
    assert!(
        duration < Duration::from_secs(10),
        "Large request took too long: {:?}",
        duration
    );

    println!("1000 facts processed in {:?}", duration);
}

#[tokio::test]
async fn test_api_correctness_after_concurrency_changes() {
    let server = create_test_server().await;

    // Verify that our concurrency changes didn't break basic API functionality

    // 1. Health check should work
    let response = server.get("/health").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    // 2. Engine stats should work
    let response = server.get("/engine/stats").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    // 3. Rule creation should work
    let rule = create_test_rule("correctness-test", "Correctness Test Rule");
    let rule_id = rule.id.clone();
    let request = CreateRuleRequest { rule };
    let response = server.post("/rules").json(&request).await;
    assert_eq!(response.status_code(), StatusCode::CREATED);

    // 4. Rule retrieval should work
    let response = server.get(&format!("/rules/{}", rule_id)).await;
    assert_eq!(response.status_code(), StatusCode::OK);

    // 5. Fact processing should work
    let facts = create_test_facts(10, "correctness");
    let request = ProcessFactsRequest { facts, rule_filter: None, execution_mode: None };
    let response = server.post("/facts/process").json(&request).await;
    assert_eq!(response.status_code(), StatusCode::OK);

    // 6. Rule listing should work
    let response = server.get("/rules").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    println!("All API endpoints working correctly after concurrency improvements");
}
