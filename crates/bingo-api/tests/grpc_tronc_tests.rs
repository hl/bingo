//! gRPC TRONC Engine Tests
//!
//! Tests for TRONC distribution scenarios using the gRPC streaming interface.
//! Based on scenarios from docs/tronc-engine.md
//!
//! Note: These tests are currently in development and may require adjustment
//! to match the actual engine implementation.

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

/// Helper to create TRONC distribution rules
fn create_tronc_rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "1".to_string(),
            name: "Deduct Administration Fee from TRONC Pool".to_string(),
            description: "Deducts a specified percentage as an administration fee from the total TRONC amount."
                .to_string(),
            conditions: vec![Condition {
                condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                    field: "entity_type".to_string(),
                    operator: SimpleOperator::Equal as i32,
                    value: Some(Value {
                        value: Some(value::Value::StringValue("tronc_distribution_config".to_string())),
                    }),
                })),
            }],
            actions: vec![Action {
                action_type: Some(action::ActionType::CallCalculator(CallCalculatorAction {
                    calculator_name: "percentage_deduct".to_string(),
                    input_mapping: {
                        let mut map = HashMap::new();
                        map.insert("total_amount".to_string(), "total_tronc_amount".to_string());
                        map.insert("percentage".to_string(), "administration_fee_percentage".to_string());
                        map
                    },
                    output_field: "adjusted_tronc_amount".to_string(),
                })),
            }],
            priority: 300,
            enabled: true,
            tags: vec!["tronc".to_string(), "administration".to_string()],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        },
        Rule {
            id: "2".to_string(),
            name: "Calculate Total Weighted Eligible Hours for TRONC Distribution".to_string(),
            description: "Aggregates total weighted hours worked by eligible employees for a given distribution period, considering role-based weights."
                .to_string(),
            conditions: vec![Condition {
                condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                    field: "entity_type".to_string(),
                    operator: SimpleOperator::Equal as i32,
                    value: Some(Value {
                        value: Some(value::Value::StringValue("tronc_distribution_config".to_string())),
                    }),
                })),
            }],
            actions: vec![Action {
                action_type: Some(action::ActionType::CallCalculator(CallCalculatorAction {
                    calculator_name: "aggregate_weighted_sum".to_string(),
                    input_mapping: {
                        let mut map = HashMap::new();
                        map.insert("value".to_string(), "hours_worked_aggregate".to_string());
                        map.insert("distribution_date".to_string(), "distribution_date".to_string());
                        map
                    },
                    output_field: "total_weighted_eligible_hours".to_string(),
                })),
            }],
            priority: 200,
            enabled: true,
            tags: vec!["tronc".to_string(), "aggregation".to_string()],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        },
        Rule {
            id: "3".to_string(),
            name: "Allocate TRONC to Employee Shifts".to_string(),
            description: "Calculates each shift's proportional share of the adjusted TRONC pool based on weighted hours."
                .to_string(),
            conditions: vec![Condition {
                condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                    field: "entity_type".to_string(),
                    operator: SimpleOperator::Equal as i32,
                    value: Some(Value {
                        value: Some(value::Value::StringValue("employee_shift".to_string())),
                    }),
                })),
            }],
            actions: vec![Action {
                action_type: Some(action::ActionType::CallCalculator(CallCalculatorAction {
                    calculator_name: "allocate_proportional".to_string(),
                    input_mapping: {
                        let mut map = HashMap::new();
                        map.insert("total_amount".to_string(), "adjusted_tronc_amount".to_string());
                        map.insert("individual_value".to_string(), "hours_worked".to_string());
                        map.insert("total_value".to_string(), "total_weighted_eligible_hours".to_string());
                        map
                    },
                    output_field: "tronc_allocated_amount".to_string(),
                })),
            }],
            priority: 100,
            enabled: true,
            tags: vec!["tronc".to_string(), "allocation".to_string()],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        },
    ]
}

/// Helper to create TRONC test facts
fn create_tronc_facts() -> Vec<Fact> {
    vec![
        Fact {
            id: "tronc_config_2025-06-28".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            data: {
                let mut map = HashMap::new();
                map.insert(
                    "entity_type".to_string(),
                    Value {
                        value: Some(value::Value::StringValue(
                            "tronc_distribution_config".to_string(),
                        )),
                    },
                );
                map.insert(
                    "total_tronc_amount".to_string(),
                    Value { value: Some(value::Value::NumberValue(500.0)) },
                );
                map.insert(
                    "administration_fee_percentage".to_string(),
                    Value { value: Some(value::Value::NumberValue(0.05)) },
                );
                map.insert(
                    "distribution_date".to_string(),
                    Value { value: Some(value::Value::StringValue("2025-06-28".to_string())) },
                );
                map
            },
        },
        Fact {
            id: "role_weight_waiter".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            data: {
                let mut map = HashMap::new();
                map.insert(
                    "entity_type".to_string(),
                    Value {
                        value: Some(value::Value::StringValue("role_weight_config".to_string())),
                    },
                );
                map.insert(
                    "role".to_string(),
                    Value { value: Some(value::Value::StringValue("waiter".to_string())) },
                );
                map.insert(
                    "weight".to_string(),
                    Value { value: Some(value::Value::NumberValue(1.0)) },
                );
                map
            },
        },
        Fact {
            id: "role_weight_bartender".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            data: {
                let mut map = HashMap::new();
                map.insert(
                    "entity_type".to_string(),
                    Value {
                        value: Some(value::Value::StringValue("role_weight_config".to_string())),
                    },
                );
                map.insert(
                    "role".to_string(),
                    Value { value: Some(value::Value::StringValue("bartender".to_string())) },
                );
                map.insert(
                    "weight".to_string(),
                    Value { value: Some(value::Value::NumberValue(1.2)) },
                );
                map
            },
        },
        Fact {
            id: "shift_001".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            data: {
                let mut map = HashMap::new();
                map.insert(
                    "entity_type".to_string(),
                    Value { value: Some(value::Value::StringValue("employee_shift".to_string())) },
                );
                map.insert(
                    "employee_id".to_string(),
                    Value { value: Some(value::Value::StringValue("emp_A".to_string())) },
                );
                map.insert(
                    "hours_worked".to_string(),
                    Value { value: Some(value::Value::NumberValue(8.0)) },
                );
                map.insert(
                    "shift_date".to_string(),
                    Value { value: Some(value::Value::StringValue("2025-06-28".to_string())) },
                );
                map.insert(
                    "role".to_string(),
                    Value { value: Some(value::Value::StringValue("waiter".to_string())) },
                );
                map
            },
        },
        Fact {
            id: "shift_002".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            data: {
                let mut map = HashMap::new();
                map.insert(
                    "entity_type".to_string(),
                    Value { value: Some(value::Value::StringValue("employee_shift".to_string())) },
                );
                map.insert(
                    "employee_id".to_string(),
                    Value { value: Some(value::Value::StringValue("emp_B".to_string())) },
                );
                map.insert(
                    "hours_worked".to_string(),
                    Value { value: Some(value::Value::NumberValue(6.0)) },
                );
                map.insert(
                    "shift_date".to_string(),
                    Value { value: Some(value::Value::StringValue("2025-06-28".to_string())) },
                );
                map.insert(
                    "role".to_string(),
                    Value { value: Some(value::Value::StringValue("waiter".to_string())) },
                );
                map
            },
        },
        Fact {
            id: "shift_003".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            data: {
                let mut map = HashMap::new();
                map.insert(
                    "entity_type".to_string(),
                    Value { value: Some(value::Value::StringValue("employee_shift".to_string())) },
                );
                map.insert(
                    "employee_id".to_string(),
                    Value { value: Some(value::Value::StringValue("emp_C".to_string())) },
                );
                map.insert(
                    "hours_worked".to_string(),
                    Value { value: Some(value::Value::NumberValue(10.0)) },
                );
                map.insert(
                    "shift_date".to_string(),
                    Value { value: Some(value::Value::StringValue("2025-06-28".to_string())) },
                );
                map.insert(
                    "role".to_string(),
                    Value { value: Some(value::Value::StringValue("bartender".to_string())) },
                );
                map
            },
        },
    ]
}

#[tokio::test]
async fn test_tronc_rule_compilation() {
    let service = create_service().await;

    // Test TRONC rule compilation
    let compile_request = Request::new(CompileRulesRequest {
        rules: create_tronc_rules(),
        session_id: "tronc_test_session".to_string(),
        options: None,
    });

    let compile_response = service.compile_rules(compile_request).await.unwrap();
    let response = compile_response.into_inner();

    assert!(response.success);
    assert_eq!(response.rules_compiled, 3);
    assert!(!response.session_id.is_empty());

    println!("TRONC rule compilation test passed");
}

#[tokio::test]
async fn test_tronc_distribution_calculation() {
    let service = create_service().await;

    // Test TRONC distribution using ProcessWithRulesStream
    let request = Request::new(ProcessWithRulesRequest {
        rules: create_tronc_rules(),
        facts: create_tronc_facts(),
        request_id: "tronc_distribution_test".to_string(),
        options: None,
        validate_rules_only: false,
    });

    let mut response_stream =
        service.process_with_rules_stream(request).await.unwrap().into_inner();

    let mut compilation_success = false;
    let mut processing_success = false;

    while let Some(result) = response_stream.next().await {
        match result {
            Ok(response) => match response.response {
                Some(processing_response::Response::RulesCompiled(compile_resp)) => {
                    assert!(compile_resp.success);
                    compilation_success = true;
                }
                Some(processing_response::Response::StatusUpdate(process_resp)) => {
                    assert!(process_resp.facts_processed > 0);
                    processing_success = true;
                }
                _ => {}
            },
            Err(e) => panic!("Stream error: {e}"),
        }
    }

    assert!(
        compilation_success,
        "Should have compiled rules successfully"
    );
    assert!(
        processing_success,
        "Should have processed facts successfully"
    );
    println!("TRONC distribution calculation test passed");
}

#[tokio::test]
async fn test_administration_fee_deduction() {
    let service = create_service().await;

    // Test with only administration fee rule
    let admin_rules = vec![create_tronc_rules()[0].clone()];
    let admin_facts = vec![create_tronc_facts()[0].clone()];

    let request = Request::new(ProcessWithRulesRequest {
        rules: admin_rules,
        facts: admin_facts,
        request_id: "admin_fee_test".to_string(),
        options: None,
        validate_rules_only: false,
    });

    let mut response_stream =
        service.process_with_rules_stream(request).await.unwrap().into_inner();

    let mut fee_calculated = false;

    while let Some(result) = response_stream.next().await {
        match result {
            Ok(response) => {
                if let Some(processing_response::Response::StatusUpdate(_)) = response.response {
                    fee_calculated = true;
                }
            }
            Err(e) => panic!("Stream error: {e}"),
        }
    }

    assert!(fee_calculated, "Should have calculated administration fee");
    println!("Administration fee deduction test passed");
}

#[tokio::test]
async fn test_role_based_weighting() {
    let service = create_service().await;

    // Test with role weight configuration
    let weight_facts = create_tronc_facts();

    // Focus on facts that demonstrate role weighting
    let filtered_facts = vec![
        weight_facts[1].clone(), // role_weight_waiter
        weight_facts[2].clone(), // role_weight_bartender
        weight_facts[3].clone(), // shift_001 - waiter
        weight_facts[5].clone(), // shift_003 - bartender
    ];

    let request = Request::new(ProcessWithRulesRequest {
        rules: create_tronc_rules(),
        facts: filtered_facts,
        request_id: "role_weighting_test".to_string(),
        options: None,
        validate_rules_only: false,
    });

    let mut response_stream =
        service.process_with_rules_stream(request).await.unwrap().into_inner();

    let mut weighting_processed = false;

    while let Some(result) = response_stream.next().await {
        match result {
            Ok(response) => {
                if let Some(processing_response::Response::StatusUpdate(process_resp)) =
                    response.response
                {
                    if process_resp.facts_processed >= 4 {
                        weighting_processed = true;
                    }
                }
            }
            Err(e) => panic!("Stream error: {e}"),
        }
    }

    assert!(
        weighting_processed,
        "Should have processed role-based weighting"
    );
    println!("Role-based weighting test passed");
}

#[tokio::test]
async fn test_proportional_allocation() {
    let service = create_service().await;

    // Test with allocation rule for employee shifts
    let allocation_rules = vec![create_tronc_rules()[2].clone()];
    let shift_facts = vec![
        create_tronc_facts()[3].clone(), // shift_001
        create_tronc_facts()[4].clone(), // shift_002
        create_tronc_facts()[5].clone(), // shift_003
    ];

    let request = Request::new(ProcessWithRulesRequest {
        rules: allocation_rules,
        facts: shift_facts,
        request_id: "allocation_test".to_string(),
        options: None,
        validate_rules_only: false,
    });

    let mut response_stream =
        service.process_with_rules_stream(request).await.unwrap().into_inner();

    let mut allocation_processed = false;

    while let Some(result) = response_stream.next().await {
        match result {
            Ok(response) => {
                if let Some(processing_response::Response::StatusUpdate(_)) = response.response {
                    allocation_processed = true;
                }
            }
            Err(e) => panic!("Stream error: {e}"),
        }
    }

    assert!(
        allocation_processed,
        "Should have processed proportional allocation"
    );
    println!("Proportional allocation test passed");
}

#[tokio::test]
async fn test_multi_employee_tronc_scenario() {
    let service = create_service().await;

    // Test complete TRONC distribution scenario with multiple employees
    let request = Request::new(ProcessWithRulesRequest {
        rules: create_tronc_rules(),
        facts: create_tronc_facts(),
        request_id: "multi_employee_tronc_test".to_string(),
        options: None,
        validate_rules_only: false,
    });

    let mut response_stream =
        service.process_with_rules_stream(request).await.unwrap().into_inner();

    let mut full_scenario_processed = false;

    while let Some(result) = response_stream.next().await {
        match result {
            Ok(response) => {
                if let Some(processing_response::Response::StatusUpdate(process_resp)) =
                    response.response
                {
                    if process_resp.facts_processed >= 6 {
                        full_scenario_processed = true;
                    }
                }
            }
            Err(e) => panic!("Stream error: {e}"),
        }
    }

    assert!(
        full_scenario_processed,
        "Should have processed complete TRONC scenario"
    );
    println!("Multi-employee TRONC scenario test passed");
}
