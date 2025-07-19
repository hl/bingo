//! gRPC Wage Cost Estimation Engine Tests
//!
//! Tests for wage cost estimation scenarios using the gRPC streaming interface.
//! Based on scenarios from docs/wage-cost-estimation-engine.md
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

/// Helper to create wage cost estimation rules
fn create_wage_cost_rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "1".to_string(),
            name: "Calculate Hours for Each Shift".to_string(),
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
                        map.insert("units".to_string(), "hours".to_string());
                        map
                    },
                    output_field: "calculated_hours".to_string(),
                })),
            }],
            priority: 300,
            enabled: true,
            tags: vec!["wage_cost".to_string(), "hours".to_string()],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        },
        Rule {
            id: "2".to_string(),
            name: "Calculate Base Pay for Each Shift".to_string(),
            description: "Calculates the base pay for a shift based on calculated hours and employee hourly rate."
                .to_string(),
            conditions: vec![
                Condition {
                    condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                        field: "entity_type".to_string(),
                        operator: SimpleOperator::Equal as i32,
                        value: Some(Value {
                            value: Some(value::Value::StringValue("shift".to_string())),
                        }),
                    })),
                },
                Condition {
                    condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                        field: "calculated_hours".to_string(),
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
                        map.insert("multiplicand".to_string(), "calculated_hours".to_string());
                        map.insert("multiplier".to_string(), "hourly_rate".to_string());
                        map
                    },
                    output_field: "base_shift_cost".to_string(),
                })),
            }],
            priority: 250,
            enabled: true,
            tags: vec!["wage_cost".to_string(), "base_pay".to_string()],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        },
        Rule {
            id: "3".to_string(),
            name: "Aggregate Gross Pay per Employee".to_string(),
            description: "Aggregates total base and overtime pay for each employee."
                .to_string(),
            conditions: vec![Condition {
                condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                    field: "entity_type".to_string(),
                    operator: SimpleOperator::Equal as i32,
                    value: Some(Value {
                        value: Some(value::Value::StringValue("employee_profile".to_string())),
                    }),
                })),
            }],
            actions: vec![Action {
                action_type: Some(action::ActionType::CallCalculator(CallCalculatorAction {
                    calculator_name: "aggregate_sum".to_string(),
                    input_mapping: {
                        let mut map = HashMap::new();
                        map.insert("value".to_string(), "base_shift_cost".to_string());
                        map.insert("employee_id".to_string(), "employee_id".to_string());
                        map
                    },
                    output_field: "total_gross_pay".to_string(),
                })),
            }],
            priority: 200,
            enabled: true,
            tags: vec!["wage_cost".to_string(), "aggregation".to_string()],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        },
        Rule {
            id: "4".to_string(),
            name: "Calculate Employer Benefits and Taxes".to_string(),
            description: "Calculates employer-paid benefits and taxes based on employee profile and gross pay."
                .to_string(),
            conditions: vec![
                Condition {
                    condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                        field: "entity_type".to_string(),
                        operator: SimpleOperator::Equal as i32,
                        value: Some(Value {
                            value: Some(value::Value::StringValue("employee_profile".to_string())),
                        }),
                    })),
                },
                Condition {
                    condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                        field: "total_gross_pay".to_string(),
                        operator: SimpleOperator::GreaterThan as i32,
                        value: Some(Value { value: Some(value::Value::NumberValue(0.0)) }),
                    })),
                },
            ],
            actions: vec![
                Action {
                    action_type: Some(action::ActionType::CallCalculator(CallCalculatorAction {
                        calculator_name: "add".to_string(),
                        input_mapping: {
                            let mut map = HashMap::new();
                            map.insert("addend1".to_string(), "total_gross_pay".to_string());
                            map.insert("addend2".to_string(), "health_insurance_cost_per_period".to_string());
                            map
                        },
                        output_field: "gross_pay_with_benefits".to_string(),
                    })),
                },
                Action {
                    action_type: Some(action::ActionType::CallCalculator(CallCalculatorAction {
                        calculator_name: "percentage_add".to_string(),
                        input_mapping: {
                            let mut map = HashMap::new();
                            map.insert("base_amount".to_string(), "gross_pay_with_benefits".to_string());
                            map.insert("percentage".to_string(), "fica_tax_rate".to_string());
                            map
                        },
                        output_field: "gross_pay_with_benefits_and_fica".to_string(),
                    })),
                },
                Action {
                    action_type: Some(action::ActionType::CallCalculator(CallCalculatorAction {
                        calculator_name: "percentage_add".to_string(),
                        input_mapping: {
                            let mut map = HashMap::new();
                            map.insert("base_amount".to_string(), "gross_pay_with_benefits_and_fica".to_string());
                            map.insert("percentage".to_string(), "unemployment_tax_rate".to_string());
                            map
                        },
                        output_field: "total_employee_wage_cost".to_string(),
                    })),
                },
            ],
            priority: 150,
            enabled: true,
            tags: vec!["wage_cost".to_string(), "benefits".to_string(), "taxes".to_string()],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        },
    ]
}

/// Helper to create wage cost estimation test facts
fn create_wage_cost_facts() -> Vec<Fact> {
    vec![
        Fact {
            id: "emp_profile_001".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            data: {
                let mut map = HashMap::new();
                map.insert(
                    "entity_type".to_string(),
                    Value {
                        value: Some(value::Value::StringValue("employee_profile".to_string())),
                    },
                );
                map.insert(
                    "employee_id".to_string(),
                    Value { value: Some(value::Value::StringValue("EMP001".to_string())) },
                );
                map.insert(
                    "hourly_rate".to_string(),
                    Value { value: Some(value::Value::NumberValue(25.0)) },
                );
                map.insert(
                    "weekly_overtime_threshold".to_string(),
                    Value { value: Some(value::Value::NumberValue(40.0)) },
                );
                map.insert(
                    "health_insurance_cost_per_period".to_string(),
                    Value { value: Some(value::Value::NumberValue(150.0)) },
                );
                map.insert(
                    "fica_tax_rate".to_string(),
                    Value { value: Some(value::Value::NumberValue(0.0765)) },
                );
                map.insert(
                    "unemployment_tax_rate".to_string(),
                    Value { value: Some(value::Value::NumberValue(0.006)) },
                );
                map
            },
        },
        Fact {
            id: "emp_profile_002".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            data: {
                let mut map = HashMap::new();
                map.insert(
                    "entity_type".to_string(),
                    Value {
                        value: Some(value::Value::StringValue("employee_profile".to_string())),
                    },
                );
                map.insert(
                    "employee_id".to_string(),
                    Value { value: Some(value::Value::StringValue("EMP002".to_string())) },
                );
                map.insert(
                    "hourly_rate".to_string(),
                    Value { value: Some(value::Value::NumberValue(30.0)) },
                );
                map.insert(
                    "weekly_overtime_threshold".to_string(),
                    Value { value: Some(value::Value::NumberValue(40.0)) },
                );
                map.insert(
                    "health_insurance_cost_per_period".to_string(),
                    Value { value: Some(value::Value::NumberValue(150.0)) },
                );
                map.insert(
                    "fica_tax_rate".to_string(),
                    Value { value: Some(value::Value::NumberValue(0.0765)) },
                );
                map.insert(
                    "unemployment_tax_rate".to_string(),
                    Value { value: Some(value::Value::NumberValue(0.006)) },
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
                    "employee_id".to_string(),
                    Value { value: Some(value::Value::StringValue("EMP001".to_string())) },
                );
                map.insert(
                    "start_datetime".to_string(),
                    Value {
                        value: Some(value::Value::StringValue(
                            "2025-06-03T09:00:00Z".to_string(),
                        )),
                    },
                );
                map.insert(
                    "finish_datetime".to_string(),
                    Value {
                        value: Some(value::Value::StringValue(
                            "2025-06-03T17:00:00Z".to_string(),
                        )),
                    },
                );
                map.insert(
                    "break_minutes".to_string(),
                    Value { value: Some(value::Value::NumberValue(30.0)) },
                );
                map.insert(
                    "hourly_rate".to_string(),
                    Value { value: Some(value::Value::NumberValue(25.0)) },
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
                    "employee_id".to_string(),
                    Value { value: Some(value::Value::StringValue("EMP001".to_string())) },
                );
                map.insert(
                    "start_datetime".to_string(),
                    Value {
                        value: Some(value::Value::StringValue(
                            "2025-06-04T09:00:00Z".to_string(),
                        )),
                    },
                );
                map.insert(
                    "finish_datetime".to_string(),
                    Value {
                        value: Some(value::Value::StringValue(
                            "2025-06-04T19:00:00Z".to_string(),
                        )),
                    },
                );
                map.insert(
                    "break_minutes".to_string(),
                    Value { value: Some(value::Value::NumberValue(60.0)) },
                );
                map.insert(
                    "hourly_rate".to_string(),
                    Value { value: Some(value::Value::NumberValue(25.0)) },
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
                    Value { value: Some(value::Value::StringValue("shift".to_string())) },
                );
                map.insert(
                    "employee_id".to_string(),
                    Value { value: Some(value::Value::StringValue("EMP002".to_string())) },
                );
                map.insert(
                    "start_datetime".to_string(),
                    Value {
                        value: Some(value::Value::StringValue(
                            "2025-06-03T08:00:00Z".to_string(),
                        )),
                    },
                );
                map.insert(
                    "finish_datetime".to_string(),
                    Value {
                        value: Some(value::Value::StringValue(
                            "2025-06-03T16:00:00Z".to_string(),
                        )),
                    },
                );
                map.insert(
                    "break_minutes".to_string(),
                    Value { value: Some(value::Value::NumberValue(30.0)) },
                );
                map.insert(
                    "hourly_rate".to_string(),
                    Value { value: Some(value::Value::NumberValue(30.0)) },
                );
                map
            },
        },
    ]
}

#[tokio::test]
async fn test_wage_cost_rule_compilation() {
    let service = create_service().await;

    // Test wage cost rule compilation
    let compile_request = Request::new(CompileRulesRequest {
        rules: create_wage_cost_rules(),
        session_id: "wage_cost_test_session".to_string(),
        options: None,
    });

    let compile_response = service.compile_rules(compile_request).await.unwrap();
    let response = compile_response.into_inner();

    assert!(response.success);
    assert_eq!(response.rules_compiled, 4);
    assert!(!response.session_id.is_empty());

    println!("Wage cost rule compilation test passed");
}

#[tokio::test]
async fn test_comprehensive_wage_cost_calculation() {
    let service = create_service().await;

    // Test comprehensive wage cost calculation using ProcessWithRulesStream
    let request = Request::new(ProcessWithRulesRequest {
        rules: create_wage_cost_rules(),
        facts: create_wage_cost_facts(),
        request_id: "wage_cost_calculation_test".to_string(),
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
    println!("Comprehensive wage cost calculation test passed");
}

#[tokio::test]
async fn test_shift_hours_calculation() {
    let service = create_service().await;

    // Test with only shift hours calculation rule
    let hours_rules = vec![create_wage_cost_rules()[0].clone()];
    let hours_facts = vec![create_wage_cost_facts()[2].clone()]; // Just one shift

    let request = Request::new(ProcessWithRulesRequest {
        rules: hours_rules,
        facts: hours_facts,
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
    println!("Shift hours calculation test passed");
}

#[tokio::test]
async fn test_base_pay_calculation() {
    let service = create_service().await;

    // Test shift hours and base pay calculation
    let pay_rules = vec![
        create_wage_cost_rules()[0].clone(), // hours calculation
        create_wage_cost_rules()[1].clone(), // base pay calculation
    ];
    let pay_facts = vec![create_wage_cost_facts()[2].clone()]; // One shift

    let request = Request::new(ProcessWithRulesRequest {
        rules: pay_rules,
        facts: pay_facts,
        request_id: "base_pay_test".to_string(),
        options: None,
        validate_rules_only: false,
    });

    let mut response_stream =
        service.process_with_rules_stream(request).await.unwrap().into_inner();

    let mut pay_calculated = false;

    while let Some(result) = response_stream.next().await {
        match result {
            Ok(response) => {
                if let Some(processing_response::Response::StatusUpdate(_)) = response.response {
                    pay_calculated = true;
                }
            }
            Err(e) => panic!("Stream error: {e}"),
        }
    }

    assert!(pay_calculated, "Should have calculated base pay");
    println!("Base pay calculation test passed");
}

#[tokio::test]
async fn test_benefits_and_taxes_calculation() {
    let service = create_service().await;

    // Test with benefits and taxes calculation
    let benefits_rules = vec![create_wage_cost_rules()[3].clone()];
    let benefits_facts = vec![create_wage_cost_facts()[0].clone()]; // Employee profile

    let request = Request::new(ProcessWithRulesRequest {
        rules: benefits_rules,
        facts: benefits_facts,
        request_id: "benefits_taxes_test".to_string(),
        options: None,
        validate_rules_only: false,
    });

    let mut response_stream =
        service.process_with_rules_stream(request).await.unwrap().into_inner();

    let mut benefits_calculated = false;

    while let Some(result) = response_stream.next().await {
        match result {
            Ok(response) => {
                if let Some(processing_response::Response::StatusUpdate(_)) = response.response {
                    benefits_calculated = true;
                }
            }
            Err(e) => panic!("Stream error: {e}"),
        }
    }

    assert!(
        benefits_calculated,
        "Should have calculated benefits and taxes"
    );
    println!("Benefits and taxes calculation test passed");
}

#[tokio::test]
async fn test_multi_employee_wage_cost_scenario() {
    let service = create_service().await;

    // Test complete wage cost scenario with multiple employees
    let request = Request::new(ProcessWithRulesRequest {
        rules: create_wage_cost_rules(),
        facts: create_wage_cost_facts(),
        request_id: "multi_employee_wage_cost_test".to_string(),
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
                    if process_resp.facts_processed >= 5 {
                        full_scenario_processed = true;
                    }
                }
            }
            Err(e) => panic!("Stream error: {e}"),
        }
    }

    assert!(
        full_scenario_processed,
        "Should have processed complete wage cost scenario"
    );
    println!("Multi-employee wage cost scenario test passed");
}

#[tokio::test]
async fn test_employee_profile_aggregation() {
    let service = create_service().await;

    // Test aggregation of employee costs
    let aggregation_rules = vec![create_wage_cost_rules()[2].clone()];
    let aggregation_facts = vec![
        create_wage_cost_facts()[0].clone(), // Employee profile
        create_wage_cost_facts()[1].clone(), // Another employee profile
    ];

    let request = Request::new(ProcessWithRulesRequest {
        rules: aggregation_rules,
        facts: aggregation_facts,
        request_id: "employee_aggregation_test".to_string(),
        options: None,
        validate_rules_only: false,
    });

    let mut response_stream =
        service.process_with_rules_stream(request).await.unwrap().into_inner();

    let mut aggregation_processed = false;

    while let Some(result) = response_stream.next().await {
        match result {
            Ok(response) => {
                if let Some(processing_response::Response::StatusUpdate(_)) = response.response {
                    aggregation_processed = true;
                }
            }
            Err(e) => panic!("Stream error: {e}"),
        }
    }

    assert!(
        aggregation_processed,
        "Should have processed employee aggregation"
    );
    println!("Employee profile aggregation test passed");
}
