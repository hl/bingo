# Compliance Engine Guide

## Overview

The Bingo Rules Engine provides comprehensive compliance checking capabilities. This guide demonstrates how to implement a multi-stage compliance process, using student visa work restrictions as an example. The engine can process raw, event-level facts (like individual shifts) and perform the necessary calculations and aggregations to determine compliance.

## Glossary

| Term | Definition |
| --- | --- |
| **Work Week** | A fixed seven-day period used for aggregation (e.g., Monday-Sunday). |
| **Compliance Fact** | A fact representing a specific entity to be checked (e.g., an employee with their compliance configuration). |
| **Event Fact** | A fact representing a single event in time (e.g., a work shift). |

## Student Visa Compliance Example

### Scenario
An employee on a student visa has a **compliance rule** that they are not allowed to work more than a specified number of hours per week (e.g., 20 hours, defined as Monday-Sunday). The system must process all their individual shifts within a given period to determine if a violation has occurred by comparing the sum of their shift hours against their configured limit.

### Staged Compliance Process

The engine handles this by breaking the problem into stages using rule priorities:

1.  **Shift Calculation (Priority 200):** First, a high-priority rule calculates the duration of each individual shift fact.
2.  **Compliance Check (Priority 100):** Next, a lower-priority rule runs on the employee fact. It aggregates the `calculated_hours` from all the relevant shift facts and compares the total against the employee's configured `weekly_hours_limit`.

## Rule Definitions

The compliance checking process is executed through a series of rules with different priorities.

### Rule 1: Calculate Shift Hours
```rust
Rule {
    id: "calculate_shift_hours".to_string(),
    name: "Calculate Hours for Each Shift".to_string(),
    description: "Calculates the duration in hours for any fact representing a shift.".to_string(),
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
            input_mapping: HashMap::from([
                ("start_field".to_string(), "start_datetime".to_string()),
                ("end_field".to_string(), "finish_datetime".to_string()),
                ("unit".to_string(), "hours".to_string()),
            ]),
            output_field: "calculated_hours".to_string(),
        })),
    }],
    priority: 200,
    enabled: true,
    tags: vec!["compliance".to_string(), "time_calculation".to_string()],
    created_at: chrono::Utc::now().timestamp(),
    updated_at: chrono::Utc::now().timestamp(),
}
```

### Rule 2: Student Visa Compliance Check
```rust
Rule {
    id: "student_visa_compliance_check".to_string(),
    name: "Student Visa Weekly Hours Compliance".to_string(),
    description: "Aggregates shift hours and checks them against the employee's weekly limit.".to_string(),
    conditions: vec![
        Condition {
            condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                field: "entity_type".to_string(),
                operator: SimpleOperator::Equal as i32,
                value: Some(Value {
                    value: Some(value::Value::StringValue("employee".to_string())),
                }),
            })),
        },
        Condition {
            condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                field: "is_student_visa".to_string(),
                operator: SimpleOperator::Equal as i32,
                value: Some(Value {
                    value: Some(value::Value::BoolValue(true)),
                }),
            })),
        },
    ],
    actions: vec![Action {
        action_type: Some(action::ActionType::CallCalculator(CallCalculatorAction {
            calculator_name: "limit_validator".to_string(),
            input_mapping: HashMap::from([
                ("aggregate_field".to_string(), "calculated_hours".to_string()),
                ("filter_condition".to_string(), "entity_type == 'shift' && employee_id == current_fact.employee_id".to_string()),
                ("warning_threshold_field".to_string(), "weekly_hours_warning".to_string()),
                ("max_threshold_field".to_string(), "weekly_hours_limit".to_string()),
            ]),
            output_field: "compliance_result".to_string(),
        })),
    }],
    priority: 100,
    enabled: true,
    tags: vec!["compliance".to_string(), "student_visa".to_string()],
    created_at: chrono::Utc::now().timestamp(),
    updated_at: chrono::Utc::now().timestamp(),
}
```

### gRPC API Example

Here is a complete example using the gRPC API to process compliance checking.

```rust
use rules_engine::{*, rules_engine_service_client::RulesEngineServiceClient};
use std::collections::HashMap;

#[tokio::main]
async fn compliance_check_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RulesEngineServiceClient::connect("http://127.0.0.1:50051").await?;

    let rules = vec![
        // Rule 1: Calculate shift hours
        Rule {
            id: "calculate_shift_hours".to_string(),
            name: "Calculate Hours for Each Shift".to_string(),
            description: "Calculates the duration in hours for any fact representing a shift.".to_string(),
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
                    input_mapping: HashMap::from([
                        ("start_field".to_string(), "start_datetime".to_string()),
                        ("end_field".to_string(), "finish_datetime".to_string()),
                        ("unit".to_string(), "hours".to_string()),
                    ]),
                    output_field: "calculated_hours".to_string(),
                })),
            }],
            priority: 200,
            enabled: true,
            tags: vec!["compliance".to_string(), "time_calculation".to_string()],
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        },
        // Rule 2: Compliance check
        Rule {
            id: "student_visa_compliance_check".to_string(),
            name: "Student Visa Weekly Hours Compliance".to_string(),
            description: "Aggregates shift hours and checks them against the employee's weekly limit.".to_string(),
            conditions: vec![
                Condition {
                    condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                        field: "entity_type".to_string(),
                        operator: SimpleOperator::Equal as i32,
                        value: Some(Value {
                            value: Some(value::Value::StringValue("employee".to_string())),
                        }),
                    })),
                },
                Condition {
                    condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                        field: "is_student_visa".to_string(),
                        operator: SimpleOperator::Equal as i32,
                        value: Some(Value {
                            value: Some(value::Value::BoolValue(true)),
                        }),
                    })),
                },
            ],
            actions: vec![Action {
                action_type: Some(action::ActionType::CallCalculator(CallCalculatorAction {
                    calculator_name: "limit_validator".to_string(),
                    input_mapping: HashMap::from([
                        ("aggregate_field".to_string(), "calculated_hours".to_string()),
                        ("filter_condition".to_string(), "entity_type == 'shift' && employee_id == current_fact.employee_id".to_string()),
                        ("warning_threshold_field".to_string(), "weekly_hours_warning".to_string()),
                        ("max_threshold_field".to_string(), "weekly_hours_limit".to_string()),
                    ]),
                    output_field: "compliance_result".to_string(),
                })),
            }],
            priority: 100,
            enabled: true,
            tags: vec!["compliance".to_string(), "student_visa".to_string()],
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        },
    ];

    let facts = vec![
        Fact {
            id: "emp_123".to_string(),
            data: HashMap::from([
                ("entity_type".to_string(), Value { value: Some(value::Value::StringValue("employee".to_string())) }),
                ("employee_id".to_string(), Value { value: Some(value::Value::StringValue("emp_123".to_string())) }),
                ("name".to_string(), Value { value: Some(value::Value::StringValue("Alice Johnson".to_string())) }),
                ("is_student_visa".to_string(), Value { value: Some(value::Value::BoolValue(true)) }),
                ("weekly_hours_limit".to_string(), Value { value: Some(value::Value::NumberValue(20.0)) }),
                ("weekly_hours_warning".to_string(), Value { value: Some(value::Value::NumberValue(18.0)) }),
            ]),
            created_at: chrono::Utc::now().timestamp(),
        },
        Fact {
            id: "shift_001".to_string(),
            data: HashMap::from([
                ("entity_type".to_string(), Value { value: Some(value::Value::StringValue("shift".to_string())) }),
                ("employee_id".to_string(), Value { value: Some(value::Value::StringValue("emp_123".to_string())) }),
                ("start_datetime".to_string(), Value { value: Some(value::Value::StringValue("2024-06-17T09:00:00Z".to_string())) }),
                ("finish_datetime".to_string(), Value { value: Some(value::Value::StringValue("2024-06-17T17:00:00Z".to_string())) }),
            ]),
            created_at: chrono::Utc::now().timestamp(),
        },
        Fact {
            id: "shift_002".to_string(),
            data: HashMap::from([
                ("entity_type".to_string(), Value { value: Some(value::Value::StringValue("shift".to_string())) }),
                ("employee_id".to_string(), Value { value: Some(value::Value::StringValue("emp_123".to_string())) }),
                ("start_datetime".to_string(), Value { value: Some(value::Value::StringValue("2024-06-18T10:00:00Z".to_string())) }),
                ("finish_datetime".to_string(), Value { value: Some(value::Value::StringValue("2024-06-18T18:00:00Z".to_string())) }),
            ]),
            created_at: chrono::Utc::now().timestamp(),
        },
        Fact {
            id: "shift_003".to_string(),
            data: HashMap::from([
                ("entity_type".to_string(), Value { value: Some(value::Value::StringValue("shift".to_string())) }),
                ("employee_id".to_string(), Value { value: Some(value::Value::StringValue("emp_123".to_string())) }),
                ("start_datetime".to_string(), Value { value: Some(value::Value::StringValue("2024-06-19T09:00:00Z".to_string())) }),
                ("finish_datetime".to_string(), Value { value: Some(value::Value::StringValue("2024-06-19T17:30:00Z".to_string())) }),
            ]),
            created_at: chrono::Utc::now().timestamp(),
        },
    ];

    // Use two-phase processing
    let compile_request = CompileRulesRequest {
        rules,
        session_id: "compliance_session".to_string(),
        options: None,
    };

    let compile_response = client.compile_rules(compile_request).await?.into_inner();
    println!("Rules compiled successfully! Session: {}", compile_response.session_id);

    // Stream facts and process results
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let request_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    let mut response_stream = client
        .process_facts_stream(tonic::Request::new(request_stream))
        .await?
        .into_inner();

    tx.send(ProcessFactsStreamRequest {
        request: Some(process_facts_stream_request::Request::SessionId(compile_response.session_id)),
    })?;

    for fact in facts {
        tx.send(ProcessFactsStreamRequest {
            request: Some(process_facts_stream_request::Request::FactBatch(fact)),
        })?;
    }

    while let Some(result) = response_stream.next().await {
        match result {
            Ok(execution_result) => {
                println!("Rule '{}' fired for fact '{}'", 
                    execution_result.rule_name, 
                    execution_result.matched_fact.unwrap().id
                );
                
                for action_result in execution_result.action_results {
                    if action_result.success {
                        println!("  Compliance check completed successfully");
                    } else {
                        println!("  Compliance check failed: {}", action_result.error_message);
                    }
                }
            }
            Err(e) => {
                eprintln!("Stream error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
```

*Note: The calculator actions use input mappings to handle aggregations and compliance threshold validation across the fact network.*

### Expected Results

The gRPC streaming API will return a series of `RuleExecutionResult` messages as rules fire:

```
Rule 'Calculate Hours for Each Shift' fired for fact 'shift_001'
  Compliance check completed successfully
  
Rule 'Calculate Hours for Each Shift' fired for fact 'shift_002'
  Compliance check completed successfully
  
Rule 'Calculate Hours for Each Shift' fired for fact 'shift_003'
  Compliance check completed successfully
  
Rule 'Student Visa Weekly Hours Compliance' fired for fact 'emp_123'
  Compliance check completed successfully
```

If a compliance violation is detected, the action result will contain:
- Severity level (warning, breach)
- Detailed status message
- Actual value vs. threshold
- Utilization percentage

For example, if the employee worked 24.5 hours against a 20-hour limit:
- Severity: "breach"
- Status: "Value 24.5 has breached maximum threshold 20"
- Utilization: 122.5%

## Available Predefined Calculators

### 1. time_between_datetime
**Purpose:** Calculate duration between two datetime values in specified units.
**Input fields:** `start_field`, `end_field`, `unit` (optional, e.g., "hours", "minutes", defaults to "hours").
**Output:** Duration as a float.

### 2. threshold_checker
**Purpose:** Simple threshold compliance validation.
**Input fields:** `value`, `threshold`, `operator`.
**Output:** Object with `passes`, `status`, `violation_amount`.

### 3. limit_validator
**Purpose:** Multi-tier validation with warning/critical/breach levels.
**Input fields:** `value`, `warning_threshold`, `critical_threshold`, `max_threshold`.
**Output:** Object with `severity`, `status`, `utilization_percent`.

## API Testing

### Health Check
```bash
grpcurl -plaintext localhost:50051 grpc.health.v1.Health/Check
```

### Compliance Evaluation
```bash
grpcurl -plaintext -d '{"rules": [...], "facts": [...]}' \
  localhost:50051 bingo.v1.EngineService/Evaluate
```

## Performance Characteristics

The Bingo compliance engine delivers exceptional performance:

- **100K facts**: ~635ms processing time
- **1M facts**: ~6.59s processing time
- **Memory efficient**: <3GB for enterprise-scale workloads

## Best Practices

1.  **De-normalized Facts**: Provide facts in a de-normalized format. Instead of nesting shifts inside an employee object, provide them as separate, top-level facts with a common `employee_id` for easier processing.
2.  **Rule Priorities**: Use `priority` to control the order of execution, ensuring calculations and enrichments happen before validation rules.
3.  **Batching**: Submit all related facts (e.g., an employee and all their shifts for the period) in a single request to allow the engine to perform aggregations correctly.
