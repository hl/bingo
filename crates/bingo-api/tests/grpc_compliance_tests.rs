//! gRPC Compliance Engine Tests
//!
//! Tests for compliance checking scenarios using the gRPC streaming interface.
//! Based on scenarios from docs/compliance-engine.md

use bingo_api::AppState;
use bingo_api::generated::rules_engine_service_server::RulesEngineService;
use bingo_api::generated::*;
use bingo_api::grpc::service::RulesEngineServiceImpl;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_stream::StreamExt;
use tonic::Request;

/// Helper to create test service instance
async fn create_service() -> RulesEngineServiceImpl {
    let app_state = Arc::new(AppState::new().await.unwrap());
    RulesEngineServiceImpl::new(app_state)
}

/// Helper to create basic compliance rules
fn create_basic_compliance_rules() -> Vec<Rule> {
    vec![Rule {
        id: "1".to_string(),
        name: "Calculate Hours for Each Shift".to_string(),
        description: "Calculates the duration in hours for any fact representing a shift."
            .to_string(),
        conditions: vec![Condition {
            condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                field: "entity_type".to_string(),
                operator: SimpleOperator::Equal as i32,
                value: Some(Value { value: Some(value::Value::StringValue("shift".to_string())) }),
            })),
        }],
        actions: vec![Action {
            action_type: Some(action::ActionType::CallCalculator(CallCalculatorAction {
                calculator_name: "time_between_datetime".to_string(),
                input_mapping: {
                    let mut map = HashMap::new();
                    map.insert("start_field".to_string(), "start_datetime".to_string());
                    map.insert("end_field".to_string(), "finish_datetime".to_string());
                    map.insert("unit".to_string(), "hours".to_string());
                    map
                },
                output_field: "calculated_hours".to_string(),
            })),
        }],
        priority: 200,
        enabled: true,
        tags: vec!["compliance".to_string(), "calculation".to_string()],
        created_at: Utc::now().timestamp(),
        updated_at: Utc::now().timestamp(),
    }]
}

#[tokio::test]
async fn test_basic_rule_compilation() {
    let service = create_service().await;

    // Test rule compilation
    let compile_request = Request::new(CompileRulesRequest {
        rules: create_basic_compliance_rules(),
        session_id: "basic_test_session".to_string(),
        options: None,
    });

    let compile_response = service.compile_rules(compile_request).await.unwrap();
    let response = compile_response.into_inner();

    assert!(response.success);
    assert_eq!(response.rules_compiled, 1);
    assert!(!response.session_id.is_empty());

    println!("Basic rule compilation test passed");
}

#[tokio::test]
async fn test_health_check() {
    let service = create_service().await;

    let request = Request::new(());
    let response = service.health_check(request).await.unwrap();

    let health_response = response.into_inner();
    assert_eq!(health_response.status, "healthy");
    assert!(!health_response.version.is_empty());

    println!("Health check test passed");
}

#[tokio::test]
async fn test_process_with_rules_validation_only() {
    let service = create_service().await;

    // Test using ProcessWithRulesStream for validation only
    let request = Request::new(ProcessWithRulesRequest {
        rules: create_basic_compliance_rules(),
        facts: vec![], // No facts for validation-only test
        request_id: "validation_test".to_string(),
        options: None,
        validate_rules_only: true,
    });

    let mut response_stream =
        service.process_with_rules_stream(request).await.unwrap().into_inner();

    // Should get at least a compilation response
    let mut compilation_success = false;
    while let Some(result) = response_stream.next().await {
        match result {
            Ok(response) => {
                if let Some(processing_response::Response::RulesCompiled(compile_resp)) =
                    response.response
                {
                    assert!(compile_resp.success);
                    compilation_success = true;
                }
            }
            Err(e) => panic!("Stream error: {e}"),
        }
    }

    assert!(
        compilation_success,
        "Should have received successful compilation response"
    );
    println!("Process with rules validation test passed");
}
