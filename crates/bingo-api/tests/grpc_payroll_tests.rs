//! gRPC Payroll Engine Tests
//!
//! Tests for payroll calculation scenarios using the gRPC streaming interface.
//! Based on scenarios from docs/payroll-engine.md

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

/// Helper to create payroll rules for basic payroll calculation
fn create_payroll_rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "1".to_string(),
            name: "Calculate Payable Hours for Each Shift".to_string(),
            description: "Calculates the duration in hours for any fact representing a shift."
                .to_string(),
            conditions: vec![Condition {
                condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                    field: "entity_type".to_string(),
                    operator: SimpleOperator::Equal as i32,
                    value: Some(Value {
                        value: Some(value::Value::StringValue("shift".to_string())),
                    }),
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
            tags: vec!["payroll".to_string(), "calculation".to_string()],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        },
        Rule {
            id: "2".to_string(),
            name: "Calculate Weekly Overtime".to_string(),
            description: "Aggregates base pay shift hours and creates an overtime fact if the weekly threshold is exceeded."
                .to_string(),
            conditions: vec![Condition {
                condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                    field: "entity_type".to_string(),
                    operator: SimpleOperator::Equal as i32,
                    value: Some(Value {
                        value: Some(value::Value::StringValue("employee_config".to_string())),
                    }),
                })),
            }],
            actions: vec![Action {
                action_type: Some(action::ActionType::CallCalculator(CallCalculatorAction {
                    calculator_name: "threshold_check".to_string(),
                    input_mapping: {
                        let mut map = HashMap::new();
                        map.insert("value".to_string(), "aggregate_base_hours".to_string());
                        map.insert("threshold".to_string(), "weekly_overtime_threshold".to_string());
                        map.insert("operator".to_string(), "GreaterThan".to_string());
                        map
                    },
                    output_field: "overtime_calculation".to_string(),
                })),
            }],
            priority: 100,
            enabled: true,
            tags: vec!["payroll".to_string(), "overtime".to_string()],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        },
        Rule {
            id: "3".to_string(),
            name: "Calculate Gross Pay".to_string(),
            description: "Calculates the gross pay for any fact with hours and a pay rate."
                .to_string(),
            conditions: vec![
                Condition {
                    condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                        field: "hours".to_string(),
                        operator: SimpleOperator::GreaterThan as i32,
                        value: Some(Value { value: Some(value::Value::NumberValue(0.0)) }),
                    })),
                },
                Condition {
                    condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                        field: "pay_rate".to_string(),
                        operator: SimpleOperator::GreaterThan as i32,
                        value: Some(Value { value: Some(value::Value::NumberValue(0.0)) }),
                    })),
                },
            ],
            actions: vec![Action {
                action_type: Some(action::ActionType::CallCalculator(CallCalculatorAction {
                    calculator_name: "multiply".to_string(),
                    input_mapping: {
                        let mut map = HashMap::new();
                        map.insert("multiplicand".to_string(), "hours".to_string());
                        map.insert("multiplier".to_string(), "pay_rate".to_string());
                        map
                    },
                    output_field: "gross_pay".to_string(),
                })),
            }],
            priority: 50,
            enabled: true,
            tags: vec!["payroll".to_string(), "gross_pay".to_string()],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        },
    ]
}

/// Helper to create payroll test facts
fn create_payroll_facts() -> Vec<Fact> {
    vec![
        Fact {
            id: "emp_config_001".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            data: {
                let mut map = HashMap::new();
                map.insert(
                    "entity_type".to_string(),
                    Value { value: Some(value::Value::StringValue("employee_config".to_string())) },
                );
                map.insert(
                    "employee_number".to_string(),
                    Value { value: Some(value::Value::StringValue("EMP001".to_string())) },
                );
                map.insert(
                    "weekly_overtime_threshold".to_string(),
                    Value { value: Some(value::Value::NumberValue(40.0)) },
                );
                map.insert(
                    "base_hourly_rate".to_string(),
                    Value { value: Some(value::Value::NumberValue(20.0)) },
                );
                map.insert(
                    "holiday_rate_multiplier".to_string(),
                    Value { value: Some(value::Value::NumberValue(1.5)) },
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
                    Value { value: Some(value::Value::StringValue("shift".to_string())) },
                );
                map.insert(
                    "start_datetime".to_string(),
                    Value {
                        value: Some(value::Value::StringValue(
                            "2025-01-01T08:00:00Z".to_string(),
                        )),
                    },
                );
                map.insert(
                    "finish_datetime".to_string(),
                    Value {
                        value: Some(value::Value::StringValue(
                            "2025-01-01T18:00:00Z".to_string(),
                        )),
                    },
                );
                map.insert(
                    "break_minutes".to_string(),
                    Value { value: Some(value::Value::NumberValue(60.0)) },
                );
                map.insert(
                    "pay_code".to_string(),
                    Value { value: Some(value::Value::StringValue("base_pay".to_string())) },
                );
                map.insert(
                    "employee_number".to_string(),
                    Value { value: Some(value::Value::StringValue("EMP001".to_string())) },
                );
                map.insert(
                    "pay_rate".to_string(),
                    Value { value: Some(value::Value::NumberValue(20.0)) },
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
                    Value { value: Some(value::Value::StringValue("shift".to_string())) },
                );
                map.insert(
                    "start_datetime".to_string(),
                    Value {
                        value: Some(value::Value::StringValue(
                            "2025-01-02T08:00:00Z".to_string(),
                        )),
                    },
                );
                map.insert(
                    "finish_datetime".to_string(),
                    Value {
                        value: Some(value::Value::StringValue(
                            "2025-01-02T18:00:00Z".to_string(),
                        )),
                    },
                );
                map.insert(
                    "break_minutes".to_string(),
                    Value { value: Some(value::Value::NumberValue(60.0)) },
                );
                map.insert(
                    "pay_code".to_string(),
                    Value { value: Some(value::Value::StringValue("base_pay".to_string())) },
                );
                map.insert(
                    "employee_number".to_string(),
                    Value { value: Some(value::Value::StringValue("EMP001".to_string())) },
                );
                map.insert(
                    "pay_rate".to_string(),
                    Value { value: Some(value::Value::NumberValue(20.0)) },
                );
                map
            },
        },
    ]
}

#[tokio::test]
async fn test_payroll_rule_compilation() {
    let service = create_service().await;

    // Test payroll rule compilation
    let compile_request = Request::new(CompileRulesRequest {
        rules: create_payroll_rules(),
        session_id: "payroll_test_session".to_string(),
        options: None,
    });

    let compile_response = service.compile_rules(compile_request).await.unwrap();
    let response = compile_response.into_inner();

    assert!(response.success);
    assert_eq!(response.rules_compiled, 3);
    assert!(!response.session_id.is_empty());

    println!("Payroll rule compilation test passed");
}

#[tokio::test]
async fn test_basic_payroll_calculation() {
    let service = create_service().await;

    // Test basic payroll calculation using ProcessWithRulesStream
    let request = Request::new(ProcessWithRulesRequest {
        rules: create_payroll_rules(),
        facts: create_payroll_facts(),
        request_id: "payroll_calculation_test".to_string(),
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
    println!("Basic payroll calculation test passed");
}

#[tokio::test]
async fn test_payroll_shift_hours_validation() {
    let service = create_service().await;

    // Test with only shift hours calculation rule
    let shift_rules = vec![create_payroll_rules()[0].clone()];

    let request = Request::new(ProcessWithRulesRequest {
        rules: shift_rules,
        facts: vec![create_payroll_facts()[1].clone()], // Just one shift
        request_id: "shift_hours_test".to_string(),
        options: None,
        validate_rules_only: false,
    });

    let mut response_stream =
        service.process_with_rules_stream(request).await.unwrap().into_inner();

    let mut hours_calculated = false;

    while let Some(result) = response_stream.next().await {
        match result {
            Ok(response) => {
                if let Some(processing_response::Response::StatusUpdate(_)) = response.response {
                    hours_calculated = true;
                }
            }
            Err(e) => panic!("Stream error: {e}"),
        }
    }

    assert!(hours_calculated, "Should have calculated shift hours");
    println!("Payroll shift hours validation test passed");
}

#[tokio::test]
async fn test_payroll_scenario() {
    let service = create_service().await;

    // Create scenario with employee working over 40 hours
    let mut overtime_facts = create_payroll_facts();

    // Add more shifts to trigger overtime
    for i in 3..=6 {
        let shift = Fact {
            id: format!("shift_{i:03}"),
            created_at: chrono::Utc::now().timestamp(),
            data: {
                let mut map = HashMap::new();
                map.insert(
                    "entity_type".to_string(),
                    Value { value: Some(value::Value::StringValue("shift".to_string())) },
                );
                map.insert(
                    "start_datetime".to_string(),
                    Value {
                        value: Some(value::Value::StringValue(format!("2025-01-0{i}T08:00:00Z"))),
                    },
                );
                map.insert(
                    "finish_datetime".to_string(),
                    Value {
                        value: Some(value::Value::StringValue(format!("2025-01-0{i}T18:00:00Z"))),
                    },
                );
                map.insert(
                    "break_minutes".to_string(),
                    Value { value: Some(value::Value::NumberValue(60.0)) },
                );
                map.insert(
                    "pay_code".to_string(),
                    Value { value: Some(value::Value::StringValue("base_pay".to_string())) },
                );
                map.insert(
                    "employee_number".to_string(),
                    Value { value: Some(value::Value::StringValue("EMP001".to_string())) },
                );
                map.insert(
                    "pay_rate".to_string(),
                    Value { value: Some(value::Value::NumberValue(20.0)) },
                );
                map
            },
        };
        overtime_facts.push(shift);
    }

    let request = Request::new(ProcessWithRulesRequest {
        rules: create_payroll_rules(),
        facts: overtime_facts,
        request_id: "overtime_test".to_string(),
        options: None,
        validate_rules_only: false,
    });

    let mut response_stream =
        service.process_with_rules_stream(request).await.unwrap().into_inner();

    let mut overtime_processed = false;

    while let Some(result) = response_stream.next().await {
        match result {
            Ok(response) => {
                if let Some(processing_response::Response::StatusUpdate(process_resp)) =
                    response.response
                {
                    if process_resp.facts_processed >= 5 {
                        overtime_processed = true;
                    }
                }
            }
            Err(e) => panic!("Stream error: {e}"),
        }
    }

    assert!(
        overtime_processed,
        "Should have processed overtime scenario"
    );
    println!("Payroll overtime scenario test passed");
}
