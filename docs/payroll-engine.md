# Payroll Engine

A payroll engine is responsible for transforming employee time tracking data into gross pay components. It operates on a defined **Pay Period** and applies a series of rules to calculate payable hours for different pay codes, including base pay, overtime, and holiday pay.

## Glossary

| Term           | Definition                                                                                             |
| -------------- | ------------------------------------------------------------------------------------------------------ |
| **Pay Period** | The specific date range for which a payroll calculation is run (e.g., `2025-01-01` to `2025-01-07`).      |
| **Gross Pay**  | The total pay before any deductions (e.g., taxes, benefits).                                           |
| **Pay Code**   | A category for classifying different types of paid time (e.g., `base_pay`, `overtime`, `holiday`).       |
| **Work Week**  | For overtime calculation, a fixed seven-day period (e.g., Monday to Sunday).                           |

## Input Data

### 1. Pay Period

The primary input that defines the scope of the calculation.

| start_date | end_date   |
| ---------- | ---------- |
| 2025-01-01 | 2025-01-07 |

### 2. Shifts

Raw time tracking data for employees.

| entity_id | entity_type   | start_datetime      | finish_datetime     | break_minutes | pay_code | employee_number |
| --------- | ------------- | ------------------- | ------------------- | ------------- | -------- | --------------- |
| 1         | shift         | 2025-01-01 08:00:00 | 2025-01-01 18:00:00 | 60            | base_pay | EMP001          |
| 2         | shift         | 2025-01-02 08:00:00 | 2025-01-02 18:00:00 | 60            | base_pay | EMP001          |
| 3         | shift         | 2025-01-03 08:00:00 | 2025-01-03 18:00:00 | 60            | base_pay | EMP001          |
| 4         | shift         | 2025-01-04 08:00:00 | 2025-01-04 18:00:00 | 60            | base_pay | EMP001          |
| 5         | shift         | 2025-01-05 08:00:00 | 2025-01-05 18:00:00 | 60            | holiday  | EMP001          |

### 3. Employee Config

Configuration data for each employee.

| employee_number | weekly_overtime_threshold | base_hourly_rate | holiday_rate_multiplier |
| --------------- | ------------------------- | ---------------- | ----------------------- |
| EMP001          | 40                        | 20.00            | 1.5                     |

## Rule Definitions

The payroll calculation is executed through a series of rules with different priorities. This ensures that initial calculations (like determining the hours for each shift) are completed before subsequent rules perform aggregations.

### Rule 1: Calculate Shift Hours
```rust
Rule {
    id: "calculate_shift_hours".to_string(),
    name: "Calculate Payable Hours for Each Shift".to_string(),
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
    tags: vec!["payroll".to_string(), "time_calculation".to_string()],
    created_at: chrono::Utc::now().timestamp(),
    updated_at: chrono::Utc::now().timestamp(),
}
```

### Rule 2: Calculate Weekly Overtime
```rust
Rule {
    id: "calculate_overtime".to_string(),
    name: "Calculate Weekly Overtime".to_string(),
    description: "Aggregates base pay shift hours and creates an overtime fact if the weekly threshold is exceeded.".to_string(),
    conditions: vec![Condition {
        condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
            field: "entity_type".to_string(),
            operator: SimpleOperator::Equal as i32,
            value: Some(Value {
                value: Some(value::Value::StringValue("employee_config".to_string())),
            }),
        })),
    }],
    actions: vec![
        Action {
            action_type: Some(action::ActionType::CallCalculator(CallCalculatorAction {
                calculator_name: "threshold_check".to_string(),
                input_mapping: HashMap::from([
                    ("aggregate_field".to_string(), "calculated_hours".to_string()),
                    ("filter_condition".to_string(), "entity_type == 'shift' && employee_number == current_fact.employee_number && pay_code == 'base_pay'".to_string()),
                    ("threshold_field".to_string(), "weekly_overtime_threshold".to_string()),
                    ("operator".to_string(), "GreaterThan".to_string()),
                ]),
                output_field: "overtime_calculation".to_string(),
            })),
        },
        Action {
            action_type: Some(action::ActionType::CreateFact(CreateFactAction {
                fields: HashMap::from([
                    ("entity_type".to_string(), Value { value: Some(value::Value::StringValue("overtime".to_string())) }),
                    ("employee_number".to_string(), Value { value: Some(value::Value::StringValue("{{current_fact.employee_number}}".to_string())) }),
                    ("pay_code".to_string(), Value { value: Some(value::Value::StringValue("overtime".to_string())) }),
                    ("hours".to_string(), Value { value: Some(value::Value::StringValue("{{overtime_calculation.excess_amount}}".to_string())) }),
                    ("pay_rate".to_string(), Value { value: Some(value::Value::StringValue("{{current_fact.base_hourly_rate}}".to_string())) }),
                ]),
            })),
        },
    ],
    priority: 100,
    enabled: true,
    tags: vec!["payroll".to_string(), "overtime".to_string()],
    created_at: chrono::Utc::now().timestamp(),
    updated_at: chrono::Utc::now().timestamp(),
}
```

### Rule 3: Calculate Gross Pay
```rust
Rule {
    id: "calculate_gross_pay".to_string(),
    name: "Calculate Gross Pay".to_string(),
    description: "Calculates the gross pay for any fact with hours and a pay rate.".to_string(),
    conditions: vec![
        Condition {
            condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                field: "hours".to_string(),
                operator: SimpleOperator::GreaterThan as i32,
                value: Some(Value {
                    value: Some(value::Value::NumberValue(0.0)),
                }),
            })),
        },
        Condition {
            condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                field: "pay_rate".to_string(),
                operator: SimpleOperator::GreaterThan as i32,
                value: Some(Value {
                    value: Some(value::Value::NumberValue(0.0)),
                }),
            })),
        },
    ],
    actions: vec![Action {
        action_type: Some(action::ActionType::CallCalculator(CallCalculatorAction {
            calculator_name: "multiply".to_string(),
            input_mapping: HashMap::from([
                ("multiplicand".to_string(), "hours".to_string()),
                ("multiplier".to_string(), "pay_rate".to_string()),
            ]),
            output_field: "gross_pay".to_string(),
        })),
    }],
    priority: 50,
    enabled: true,
    tags: vec!["payroll".to_string(), "gross_pay".to_string()],
    created_at: chrono::Utc::now().timestamp(),
    updated_at: chrono::Utc::now().timestamp(),
}
```

*Note: The calculator actions use input mappings to handle aggregations, fact lookups, and templating for dynamic fact creation.*

## Final Output

The final output is a set of facts, where each fact has a calculated `gross_pay` component.

| entity_id | entity_type | employee_number | pay_code | hours | pay_rate | gross_pay |
| --------- | ----------- | --------------- | -------- | ----- | -------- | --------- |
| 1         | shift       | EMP001          | base_pay | 9     | 20.00    | 180.00    |
| 2         | shift       | EMP001          | base_pay | 9     | 20.00    | 180.00    |
| 3         | shift       | EMP001          | base_pay | 9     | 20.00    | 180.00    |
| 4         | shift       | EMP001          | base_pay | 9     | 20.00    | 180.00    |
| 5         | shift       | EMP001          | holiday  | 9     | 30.00    | 270.00    |
| auto-gen  | overtime    | EMP001          | overtime | 4     | 20.00    | 80.00     |

*Note: In this example, the first four 9-hour shifts total 36 hours. The fifth shift is a holiday and does not count towards the 40-hour overtime threshold. If there were another 9-hour `base_pay` shift, 5 hours would be paid as `base_pay` (reaching the 40-hour threshold) and 4 hours would be converted to `overtime`.*

## API Request Example

Here is a complete example of what would be sent to the `/evaluate` endpoint.

### Input

```json
{
  "rules": [
    {
      "id": "calculate_shift_hours",
      "priority": 200,
      "conditions": [ { "field": "entity_type", "operator": "equal", "value": "shift" } ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "time_between_datetime",
          "input_mapping": {
            "start_field": "start_datetime", "end_field": "finish_datetime", "unit": "hours"
          },
          "output_field": "hours"
        }
      ]
    },
    {
      "id": "calculate_overtime",
      "priority": 100,
      "conditions": [ { "field": "entity_type", "operator": "equal", "value": "employee_config" } ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "threshold_check",
          "input_mapping": {
            "value": { "source_type": "aggregate", "source_field": "hours", "filter": "entity_type == 'shift' && employee_number == current_fact.employee_number && pay_code == 'base_pay'" },
            "threshold": "weekly_overtime_threshold",
            "operator": { "value": "GreaterThan" }
          },
          "output_field": "overtime_calculation"
        },
        {
            "type": "create_fact",
            "fact_id": "overtime_{{current_fact.employee_number}}",
            "fact_data": {
                "entity_type": "overtime", "employee_number": "{{current_fact.employee_number}}", "pay_code": "overtime",
                "hours": "{{overtime_calculation.violation_amount}}", "pay_rate": "{{current_fact.base_hourly_rate}}"
            },
            "condition": "{{overtime_calculation.passes}}"
        }
      ]
    }
  ],
  "facts": [
    {
      "id": "emp_config_001",
      "data": { "entity_type": "employee_config", "employee_number": "EMP001", "weekly_overtime_threshold": 40, "base_hourly_rate": 20.00, "holiday_rate_multiplier": 1.5 }
    },
    { "id": "1", "data": { "entity_type": "shift", "start_datetime": "2025-01-01 08:00:00", "finish_datetime": "2025-01-01 18:00:00", "break_minutes": 60, "pay_code": "base_pay", "employee_number": "EMP001" } },
    { "id": "2", "data": { "entity_type": "shift", "start_datetime": "2025-01-02 08:00:00", "finish_datetime": "2025-01-02 18:00:00", "break_minutes": 60, "pay_code": "base_pay", "employee_number": "EMP001" } },
    { "id": "3", "data": { "entity_type": "shift", "start_datetime": "2025-01-03 08:00:00", "finish_datetime": "2025-01-03 18:00:00", "break_minutes": 60, "pay_code": "base_pay", "employee_number": "EMP001" } },
    { "id": "4", "data": { "entity_type": "shift", "start_datetime": "2025-01-04 08:00:00", "finish_datetime": "2025-01-04 18:00:00", "break_minutes": 60, "pay_code": "base_pay", "employee_number": "EMP001" } },
    { "id": "5", "data": { "entity_type": "shift", "start_datetime": "2025-01-05 08:00:00", "finish_datetime": "2025-01-05 18:00:00", "break_minutes": 60, "pay_code": "base_pay", "employee_number": "EMP001" } }
  ]
}
```

### Expected Output

```json
{
  "request_id": "req_f4a9b1c2-a1b2-4c3d-8e4f-5a6b7c8d9e0f",
  "results": [
    {
      "id": "emp_config_001",
      "data": {
        "entity_type": "employee_config",
        "employee_number": "EMP001",
        "weekly_overtime_threshold": 40,
        "base_hourly_rate": 20.00,
        "holiday_rate_multiplier": 1.5,
        "overtime_calculation": {
          "passes": true,
          "value": 45.0,
          "threshold": 40.0,
          "operator": "GreaterThan",
          "violation_amount": 5.0,
          "status": "compliant"
        }
      }
    },
    { "id": "1", "data": { "hours": 9.0, ... } },
    { "id": "2", "data": { "hours": 9.0, ... } },
    { "id": "3", "data": { "hours": 9.0, ... } },
    { "id": "4", "data": { "hours": 9.0, ... } },
    { "id": "5", "data": { "hours": 9.0, ... } },
    {
      "id": "overtime_EMP001",
      "data": {
        "entity_type": "overtime",
        "employee_number": "EMP001",
        "pay_code": "overtime",
        "hours": 5.0,
        "pay_rate": 20.00
      }
    }
  ],
  "facts_processed": 7,
  "rules_evaluated": 2,
  "rules_fired": 7
}
```

## Understanding the Calculator Result Object

You've asked a great question: why does the `threshold_check` calculator return a detailed object instead of a simple `true` or `false`? This is a deliberate and crucial design choice for making the rules engine powerful and transparent.

Here’s the breakdown:

1.  **Context is King**: A simple boolean (`true`/`false`) tells you *if* a condition was met, but it doesn't tell you *why* or *by how much*. The result object provides rich context that is essential for complex logic.

2.  **Enabling Chained Rules**: The object's properties can be used as inputs for subsequent rules. In the payroll example, the `calculate_overtime` rule runs first. Its result object looks like this:

    ```json
    {
      "passes": true,            // The condition (hours > 40) was met.
      "value": 45.0,             // The actual value that was checked (total hours).
      "threshold": 40.0,           // The threshold it was checked against.
      "violation_amount": 5.0    // The amount by which the threshold was exceeded.
    }
    ```

    A second action in the same rule, `create_fact`, uses this result. It checks if `passes` is true, and if so, it uses the `violation_amount` (5.0) to set the `hours` for the new overtime fact. This would be impossible with a simple boolean result.

3.  **Improved Auditability and Debugging**: When you are examining the engine's output, the result object gives you a complete picture of the calculation. You can see the exact inputs (`value`, `threshold`) and the precise outcome (`violation_amount`). This makes it much easier to audit the results for correctness and to debug rules that aren't behaving as expected.

4.  **Flexibility**: This approach is highly flexible. The same `threshold_check` can be used for many different scenarios (compliance, payroll, budget alerts) precisely because its output is so descriptive. The rules that consume its output have all the information they need to make nuanced decisions.

In short, returning an object from the calculator transforms it from a simple verifier into a powerful analytical tool that enriches the data and enables more sophisticated, multi-stage workflows. It's a core principle of the engine's design to provide rich, contextual information rather than simple binary answers.

## Performance and Scalability

The payroll engine must be designed to handle large-scale enterprise scenarios. The system should be capable of efficiently processing a payroll run for **10,000 employees** over a **3-month period**, with each employee averaging **10 shifts per week**.

This translates to approximately **1.3 million shift facts** (10,000 employees * 10 shifts/week * 13 weeks) plus employee configuration facts. The entire payroll calculation for this volume should be completed in a timely manner, ideally within a few minutes.
