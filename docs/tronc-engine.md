# TRONC Engine Guide

## Overview

The Bingo Rules Engine can be configured to manage and distribute TRONC (Tronc-related Outgoings Not Counted) payments, which typically involve tips, gratuities, and service charges. This guide demonstrates how to implement a system for proportionally distributing a TRONC pool among eligible employees based on their hours worked within a specific period.

## Glossary

| Term | Definition |
| --- | --- |
| **TRONC** | A system for distributing tips, gratuities, and service charges to employees, often managed by a "troncmaster" to ensure fair and tax-efficient distribution. |
| **TRONC Pool** | The total amount of tips, gratuities, or service charges collected for a specific distribution period. |
| **Distribution Period** | The defined timeframe (e.g., a day, week, or month) over which TRONC is collected and subsequently distributed. |
| **Eligible Hours** | The hours worked by an employee that qualify them for a share of the TRONC pool. |

## Scenario: Proportional TRONC Distribution

### Scenario
A restaurant collects a daily service charge that needs to be distributed among its front-of-house staff. The distribution is based on the proportion of hours each eligible staff member worked during that day relative to the total eligible hours worked by all staff for that day.

### Staged Distribution Process

The engine handles this by breaking the problem into stages using rule priorities:

1.  **Deduct Administration Fee (Priority 300):** A high-priority rule deducts a specified administration fee from the total TRONC pool.
2.  **Calculate Total Eligible Hours (Priority 200):** Next, a rule aggregates the total hours worked by all eligible employees for a given distribution period, potentially weighted by role. This total is stored on the TRONC configuration fact.
3.  **Allocate TRONC to Employee Shifts (Priority 100):** Finally, a lower-priority rule runs on each individual employee shift fact. It uses the adjusted TRONC pool amount and the aggregated total eligible hours (from the TRONC configuration fact) to calculate each shift's proportional share of the TRONC.

## Input Data

### 1. TRONC Distribution Configuration
This fact defines the total TRONC pool amount, the administration fee, and the distribution period.

| id | entity_type | total_tronc_amount | administration_fee_percentage | distribution_date |
| --- | --- | --- | --- | --- |
| tronc_config_2025-06-28 | tronc_distribution_config | 500.00 | 0.05 | 2025-06-28 |

### 2. Role-based Weighting Configuration
This fact defines weighting factors for different employee roles, allowing certain roles to receive a larger share of TRONC per hour worked.

| id | entity_type | role | weight |
| --- | --- | --- | --- |
| role_weight_waiter | role_weight_config | waiter | 1.0 |
| role_weight_bartender | role_weight_config | bartender | 1.2 |

### 3. Employee Shifts
Raw shift data for employees, including hours worked that are eligible for TRONC distribution.

| id | entity_type | employee_id | hours_worked | shift_date | role |
| --- | --- | --- | --- | --- | --- |
| shift_001 | employee_shift | emp_A | 8.0 | 2025-06-28 | waiter |
| shift_002 | employee_shift | emp_B | 6.0 | 2025-06-28 | waiter |
| shift_003 | employee_shift | emp_C | 10.0 | 2025-06-28 | bartender |

## Rule Definitions

The TRONC distribution is executed through a series of rules with different priorities. This ensures that the total eligible hours are calculated before individual allocations are made.

### Rule 1: Deduct Administration Fee
```rust
Rule {
    id: "deduct_admin_fee".to_string(),
    name: "Deduct Administration Fee from TRONC Pool".to_string(),
    description: "Deducts a specified percentage as an administration fee from the total TRONC amount.".to_string(),
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
            input_mapping: HashMap::from([
                ("total_amount".to_string(), "total_tronc_amount".to_string()),
                ("percentage".to_string(), "administration_fee_percentage".to_string()),
            ]),
            output_field: "adjusted_tronc_amount".to_string(),
        })),
    }],
    priority: 300,
    enabled: true,
    tags: vec!["tronc".to_string(), "administration".to_string()],
    created_at: chrono::Utc::now().timestamp(),
    updated_at: chrono::Utc::now().timestamp(),
}
```

### Rule 2: Calculate Total Weighted Eligible Hours
```rust
Rule {
    id: "calculate_weighted_eligible_hours".to_string(),
    name: "Calculate Total Weighted Eligible Hours for TRONC Distribution".to_string(),
    description: "Aggregates total weighted hours worked by eligible employees for a given distribution period, considering role-based weights.".to_string(),
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
            input_mapping: HashMap::from([
                ("source_field".to_string(), "hours_worked".to_string()),
                ("filter_condition".to_string(), "entity_type == 'employee_shift' && shift_date == current_fact.distribution_date".to_string()),
                ("weight_field".to_string(), "role".to_string()),
                ("weight_lookup_type".to_string(), "role_weight_config".to_string()),
                ("weight_lookup_field".to_string(), "weight".to_string()),
            ]),
            output_field: "total_weighted_eligible_hours".to_string(),
        })),
    }],
    priority: 200,
    enabled: true,
    tags: vec!["tronc".to_string(), "calculation".to_string()],
    created_at: chrono::Utc::now().timestamp(),
    updated_at: chrono::Utc::now().timestamp(),
}
```

### Rule 3: Allocate TRONC to Employee Shifts
```rust
Rule {
    id: "allocate_tronc_to_shift".to_string(),
    name: "Allocate TRONC to Employee Shifts".to_string(),
    description: "Calculates each shift's proportional share of the adjusted TRONC pool based on weighted hours.".to_string(),
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
            input_mapping: HashMap::from([
                ("total_amount_fact_id".to_string(), "tronc_config_2025-06-28".to_string()),
                ("total_amount_field".to_string(), "adjusted_tronc_amount".to_string()),
                ("individual_hours_field".to_string(), "hours_worked".to_string()),
                ("weight_field".to_string(), "role".to_string()),
                ("weight_lookup_type".to_string(), "role_weight_config".to_string()),
                ("total_hours_fact_id".to_string(), "tronc_config_2025-06-28".to_string()),
                ("total_hours_field".to_string(), "total_weighted_eligible_hours".to_string()),
            ]),
            output_field: "tronc_allocated_amount".to_string(),
        })),
    }],
    priority: 100,
    enabled: true,
    tags: vec!["tronc".to_string(), "allocation".to_string()],
    created_at: chrono::Utc::now().timestamp(),
    updated_at: chrono::Utc::now().timestamp(),
}
```

*Note: The calculator actions use input mappings to reference aggregated data and fact lookups, enabling complex, multi-stage calculations across the fact network.*

## gRPC API Example

Here is a complete example using the gRPC API to process TRONC distribution.

### Rust Client Example

```rust
use rules_engine::{*, rules_engine_service_client::RulesEngineServiceClient};
use std::collections::HashMap;

#[tokio::main]
async fn tronc_distribution_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RulesEngineServiceClient::connect("http://127.0.0.1:50051").await?;

    // Create the rules for TRONC distribution
    let rules = vec![
        // Rule 1: Deduct Administration Fee (Priority 300)
        Rule {
            id: "deduct_admin_fee".to_string(),
            name: "Deduct Administration Fee from TRONC Pool".to_string(),
            description: "Deducts a specified percentage as an administration fee from the total TRONC amount.".to_string(),
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
                    input_mapping: HashMap::from([
                        ("total_amount".to_string(), "total_tronc_amount".to_string()),
                        ("percentage".to_string(), "administration_fee_percentage".to_string()),
                    ]),
                    output_field: "adjusted_tronc_amount".to_string(),
                })),
            }],
            priority: 300,
            enabled: true,
            tags: vec!["tronc".to_string(), "administration".to_string()],
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        },
        // Rule 2: Calculate Total Weighted Eligible Hours (Priority 200)
        Rule {
            id: "calculate_weighted_eligible_hours".to_string(),
            name: "Calculate Total Weighted Eligible Hours for TRONC Distribution".to_string(),
            description: "Aggregates total weighted hours worked by eligible employees for a given distribution period.".to_string(),
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
                    input_mapping: HashMap::from([
                        ("source_field".to_string(), "hours_worked".to_string()),
                        ("filter_condition".to_string(), "entity_type == 'employee_shift' && shift_date == current_fact.distribution_date".to_string()),
                        ("weight_field".to_string(), "role".to_string()),
                        ("weight_lookup_type".to_string(), "role_weight_config".to_string()),
                        ("weight_lookup_field".to_string(), "weight".to_string()),
                    ]),
                    output_field: "total_weighted_eligible_hours".to_string(),
                })),
            }],
            priority: 200,
            enabled: true,
            tags: vec!["tronc".to_string(), "calculation".to_string()],
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        },
        // Rule 3: Allocate TRONC to Employee Shifts (Priority 100)
        Rule {
            id: "allocate_tronc_to_shift".to_string(),
            name: "Allocate TRONC to Employee Shifts".to_string(),
            description: "Calculates each shift's proportional share of the adjusted TRONC pool based on weighted hours.".to_string(),
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
                    input_mapping: HashMap::from([
                        ("total_amount_fact_id".to_string(), "tronc_config_2025-06-28".to_string()),
                        ("total_amount_field".to_string(), "adjusted_tronc_amount".to_string()),
                        ("individual_hours_field".to_string(), "hours_worked".to_string()),
                        ("weight_field".to_string(), "role".to_string()),
                        ("weight_lookup_type".to_string(), "role_weight_config".to_string()),
                        ("total_hours_fact_id".to_string(), "tronc_config_2025-06-28".to_string()),
                        ("total_hours_field".to_string(), "total_weighted_eligible_hours".to_string()),
                    ]),
                    output_field: "tronc_allocated_amount".to_string(),
                })),
            }],
            priority: 100,
            enabled: true,
            tags: vec!["tronc".to_string(), "allocation".to_string()],
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        },
    ];

    // Create facts for the TRONC distribution scenario
    let facts = vec![
        Fact {
            id: "tronc_config_2025-06-28".to_string(),
            data: HashMap::from([
                ("entity_type".to_string(), Value { value: Some(value::Value::StringValue("tronc_distribution_config".to_string())) }),
                ("total_tronc_amount".to_string(), Value { value: Some(value::Value::NumberValue(500.0)) }),
                ("administration_fee_percentage".to_string(), Value { value: Some(value::Value::NumberValue(0.05)) }),
                ("distribution_date".to_string(), Value { value: Some(value::Value::StringValue("2025-06-28".to_string())) }),
            ]),
            created_at: chrono::Utc::now().timestamp(),
        },
        Fact {
            id: "role_weight_waiter".to_string(),
            data: HashMap::from([
                ("entity_type".to_string(), Value { value: Some(value::Value::StringValue("role_weight_config".to_string())) }),
                ("role".to_string(), Value { value: Some(value::Value::StringValue("waiter".to_string())) }),
                ("weight".to_string(), Value { value: Some(value::Value::NumberValue(1.0)) }),
            ]),
            created_at: chrono::Utc::now().timestamp(),
        },
        Fact {
            id: "role_weight_bartender".to_string(),
            data: HashMap::from([
                ("entity_type".to_string(), Value { value: Some(value::Value::StringValue("role_weight_config".to_string())) }),
                ("role".to_string(), Value { value: Some(value::Value::StringValue("bartender".to_string())) }),
                ("weight".to_string(), Value { value: Some(value::Value::NumberValue(1.2)) }),
            ]),
            created_at: chrono::Utc::now().timestamp(),
        },
        Fact {
            id: "shift_001".to_string(),
            data: HashMap::from([
                ("entity_type".to_string(), Value { value: Some(value::Value::StringValue("employee_shift".to_string())) }),
                ("employee_id".to_string(), Value { value: Some(value::Value::StringValue("emp_A".to_string())) }),
                ("hours_worked".to_string(), Value { value: Some(value::Value::NumberValue(8.0)) }),
                ("shift_date".to_string(), Value { value: Some(value::Value::StringValue("2025-06-28".to_string())) }),
                ("role".to_string(), Value { value: Some(value::Value::StringValue("waiter".to_string())) }),
            ]),
            created_at: chrono::Utc::now().timestamp(),
        },
        Fact {
            id: "shift_002".to_string(),
            data: HashMap::from([
                ("entity_type".to_string(), Value { value: Some(value::Value::StringValue("employee_shift".to_string())) }),
                ("employee_id".to_string(), Value { value: Some(value::Value::StringValue("emp_B".to_string())) }),
                ("hours_worked".to_string(), Value { value: Some(value::Value::NumberValue(6.0)) }),
                ("shift_date".to_string(), Value { value: Some(value::Value::StringValue("2025-06-28".to_string())) }),
                ("role".to_string(), Value { value: Some(value::Value::StringValue("waiter".to_string())) }),
            ]),
            created_at: chrono::Utc::now().timestamp(),
        },
        Fact {
            id: "shift_003".to_string(),
            data: HashMap::from([
                ("entity_type".to_string(), Value { value: Some(value::Value::StringValue("employee_shift".to_string())) }),
                ("employee_id".to_string(), Value { value: Some(value::Value::StringValue("emp_C".to_string())) }),
                ("hours_worked".to_string(), Value { value: Some(value::Value::NumberValue(10.0)) }),
                ("shift_date".to_string(), Value { value: Some(value::Value::StringValue("2025-06-28".to_string())) }),
                ("role".to_string(), Value { value: Some(value::Value::StringValue("bartender".to_string())) }),
            ]),
            created_at: chrono::Utc::now().timestamp(),
        },
    ];

    // Use two-phase processing for optimal performance
    let compile_request = CompileRulesRequest {
        rules,
        session_id: "tronc_distribution_session".to_string(),
        options: None,
    };

    let compile_response = client.compile_rules(compile_request).await?.into_inner();
    println!("Rules compiled successfully! Session: {}", compile_response.session_id);

    // Stream facts through compiled rules
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let request_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    let mut response_stream = client
        .process_facts_stream(tonic::Request::new(request_stream))
        .await?
        .into_inner();

    // Send session ID
    tx.send(ProcessFactsStreamRequest {
        request: Some(process_facts_stream_request::Request::SessionId(compile_response.session_id)),
    })?;

    // Send facts
    for fact in facts {
        tx.send(ProcessFactsStreamRequest {
            request: Some(process_facts_stream_request::Request::FactBatch(fact)),
        })?;
    }

    // Process results
    while let Some(result) = response_stream.next().await {
        match result {
            Ok(execution_result) => {
                println!("Rule '{}' fired for fact '{}'", 
                    execution_result.rule_name, 
                    execution_result.matched_fact.unwrap().id
                );
                
                for action_result in execution_result.action_results {
                    if action_result.success {
                        println!("  Action executed successfully");
                    } else {
                        println!("  Action failed: {}", action_result.error_message);
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

### Expected Results

Assuming `adjusted_tronc_amount` is calculated as 475.00 (500 - 500 * 0.05) and `total_weighted_eligible_hours` is 26.0 (8*1.0 + 6*1.0 + 10*1.2):

*   `shift_001`: (8.0 / 26.0) * 475.00 = 146.15
*   `shift_002`: (6.0 / 26.0) * 475.00 = 109.62
*   `shift_003`: (12.0 / 26.0) * 475.00 = 219.23

The gRPC streaming API will return a series of `RuleExecutionResult` messages as rules fire:

```
Rule 'Deduct Administration Fee from TRONC Pool' fired for fact 'tronc_config_2025-06-28'
  Action executed successfully
  
Rule 'Calculate Total Weighted Eligible Hours for TRONC Distribution' fired for fact 'tronc_config_2025-06-28'
  Action executed successfully
  
Rule 'Allocate TRONC to Employee Shifts' fired for fact 'shift_001'
  Action executed successfully
  
Rule 'Allocate TRONC to Employee Shifts' fired for fact 'shift_002'
  Action executed successfully
  
Rule 'Allocate TRONC to Employee Shifts' fired for fact 'shift_003'
  Action executed successfully
```

Each execution result contains:
- The rule that fired
- The fact that matched the conditions
- Action results with calculated values
- Execution timing metadata

After processing, facts will be updated with calculated fields like `adjusted_tronc_amount`, `total_weighted_eligible_hours`, and `tronc_allocated_amount` for each shift.

## Available Predefined Calculators

The engine includes a rich ecosystem of calculators to perform common business logic, including advanced calculators for distribution models like TRONC.

### 1. percentage_deduct
**Purpose:** Deducts a percentage from a total amount.
**Input fields:** `total_amount`, `percentage` (as a decimal, e.g., 0.05 for 5%).
**Output:** `adjusted_amount` (float).

### 2. limit_validate
**Purpose:** Multi-tier validation with warning/critical/breach levels.
**Input fields:** `value`, `warning_threshold`, `critical_threshold`, `max_threshold`.
**Output:** Object with `severity`, `status`, `utilization_percent`.

### Conceptual Calculators
The following calculators are part of the engine's powerful data access DSL and represent more complex, domain-specific operations.
*   **aggregate_weighted_sum**: Aggregates a numeric field across multiple facts, applying a weight.
*   **allocate_proportional**: Calculates a proportional share of a total amount.

*Note: The calculator ecosystem is extensible, allowing for custom business logic to be added.*

## Performance Characteristics

The Bingo engine delivers exceptional enterprise-scale performance. The following benchmarks were run on a standard development machine.

| Scale | Processing Time | Facts/Second | Memory Usage |
|---|---|---|---|
| 1M facts | 1.04s | 962K/s | <1GB |
| 500K facts | 0.44s | 1.1M/s | <500MB |
| 200K facts | 0.21s | 952K/s | <200MB |
| 100K facts | 0.11s | 909K/s | <100MB |

## Best Practices

1.  **De-normalized Facts**: Provide facts in a de-normalized format. Instead of nesting shifts within a TRONC configuration object, provide them as separate, top-level facts with common identifiers (e.g., `shift_date`) for easier processing and aggregation.
2.  **Rule Priorities**: Use `priority` to control the order of execution, ensuring aggregations (like total eligible hours) happen before individual allocations.
3.  **Batching**: Submit all related facts (e.g., the TRONC configuration and all relevant employee shifts for the period) in a single request to allow the engine to perform aggregations and distributions correctly.
4.  **Fact ID Consistency**: Ensure consistent `fact_id` for the `tronc_distribution_config` fact across requests for a given distribution period if you intend to reference it via `fact_lookup`.
