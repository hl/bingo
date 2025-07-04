# Wage Cost Estimation Engine Guide

## Overview

The Bingo Rules Engine can be utilized to accurately estimate total wage costs, encompassing various components such as base pay, overtime, benefits, and employer-paid taxes. This guide demonstrates how to configure the engine to process raw time and employee data to derive comprehensive wage cost estimates for a given period.

## Glossary

| Term | Definition |
| --- | --- |
| **Wage Cost** | The total financial outlay associated with employing staff, including direct compensation and indirect costs like benefits and taxes. |
| **Cost Component** | A specific element contributing to the total wage cost (e.g., base pay, overtime, health insurance, payroll tax). |
| **Employee Profile** | A fact containing an employee's static configuration data relevant to wage cost calculation (e.g., hourly rate, benefit eligibility, tax rates). |
| **Shift Data** | Raw time tracking data for an employee's work period. |
| **Benefit Rate** | The rate or amount of employer-paid benefits associated with an employee. |
| **Tax Rate** | The percentage rate for employer-paid payroll taxes (e.g., FICA, unemployment).

## Scenario: Comprehensive Wage Cost Estimation

### Scenario
An organization needs to estimate the total wage cost for all employees over a specific pay period. This estimation must include base hourly wages, overtime wages, and employer contributions for health insurance and payroll taxes (e.g., Social Security, Medicare, and unemployment).

### Staged Estimation Process

The engine handles this by breaking the problem into stages using rule priorities:

1.  **Shift Hours Calculation (Priority 300):** First, a high-priority rule calculates the duration of each individual shift fact.
2.  **Individual Shift Cost Calculation (Priority 250):** Next, rules calculate the base pay and any applicable overtime pay for each shift, based on the employee's hourly rate and overtime rules.
3.  **Employee-Level Cost Aggregation (Priority 200):** A rule aggregates all calculated shift costs (base, overtime) for each employee within the pay period.
4.  **Benefit and Tax Application (Priority 150):** Rules apply employer-paid benefits (e.g., a fixed amount per employee or a percentage of gross pay) and payroll taxes based on the aggregated employee costs and their profile configurations.
5.  **Total Wage Cost Summation (Priority 100):** Finally, a rule sums up all individual employee wage costs to arrive at the total estimated wage cost for the entire organization or a specific department.

## Input Data

### 1. Pay Period Configuration
Defines the scope of the calculation.

| id | entity_type | start_date | end_date |
| --- | --- | --- | --- |
| pay_period_Q2_2025 | pay_period_config | 2025-04-01 | 2025-06-30 |

### 2. Employee Profiles
Configuration data for each employee.

| id | entity_type | employee_id | hourly_rate | weekly_overtime_threshold | health_insurance_cost_per_period | fica_tax_rate | unemployment_tax_rate |
| --- | --- | --- | --- | --- | --- | --- | --- |
| emp_profile_001 | employee_profile | EMP001 | 25.00 | 40 | 150.00 | 0.0765 | 0.006 |
| emp_profile_002 | employee_profile | EMP002 | 30.00 | 40 | 150.00 | 0.0765 | 0.006 |

### 3. Shifts
Raw time tracking data for employees.

| id | entity_type | employee_id | start_datetime | finish_datetime | break_minutes |
| --- | --- | --- | --- | --- | --- |
| shift_001 | shift | EMP001 | 2025-06-03T09:00:00Z | 2025-06-03T17:00:00Z | 30 |
| shift_002 | shift | EMP001 | 2025-06-04T09:00:00Z | 2025-06-04T19:00:00Z | 60 |

## Rule Definitions

The wage cost calculation is executed through a series of rules with different priorities.

```json
{
  "rules": [
    {
      "id": "calculate_shift_hours",
      "name": "Calculate Hours for Each Shift",
      "description": "Calculates the duration in hours for any fact representing a shift.",
      "conditions": [
        { "field": "entity_type", "operator": "equal", "value": "shift" }
      ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "time_between_datetime",
          "input_mapping": {
            "start_field": "start_datetime",
            "end_field": "finish_datetime",
            "unit": { "value": "hours" }
          },
          "output_field": "calculated_hours"
        }
      ],
      "priority": 300
    },
    {
      "id": "calculate_base_pay_per_shift",
      "name": "Calculate Base Pay for Each Shift",
      "description": "Calculates the base pay for a shift based on calculated hours and employee hourly rate.",
      "conditions": [
        { "field": "entity_type", "operator": "equal", "value": "shift" },
        { "field": "calculated_hours", "operator": "exists" }
      ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "multiply",
          "input_mapping": {
            "multiplicand": "calculated_hours",
            "multiplier": {
              "source_type": "fact_lookup",
              "fact_id_field": "employee_id",
              "fact_type": "employee_profile",
              "field": "hourly_rate"
            }
          },
          "output_field": "base_shift_cost"
        }
      ],
      "priority": 250
    },
    {
      "id": "aggregate_employee_gross_pay",
      "name": "Aggregate Gross Pay per Employee",
      "description": "Aggregates total base and overtime pay for each employee.",
      "conditions": [
        { "field": "entity_type", "operator": "equal", "value": "employee_profile" }
      ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "aggregate_sum",
          "input_mapping": {
            "value": {
              "source_type": "aggregate",
              "source_field": "base_shift_cost",
              "filter": "entity_type == 'shift' && employee_id == current_fact.employee_id"
            }
          },
          "output_field": "total_gross_pay"
        }
      ],
      "priority": 200
    },
    {
      "id": "calculate_employer_benefits_and_taxes",
      "name": "Calculate Employer Benefits and Taxes",
      "description": "Calculates employer-paid benefits and taxes based on employee profile and gross pay.",
      "conditions": [
        { "field": "entity_type", "operator": "equal", "value": "employee_profile" },
        { "field": "total_gross_pay", "operator": "exists" }
      ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "add",
          "input_mapping": {
            "addend1": "total_gross_pay",
            "addend2": "health_insurance_cost_per_period"
          },
          "output_field": "gross_pay_with_benefits"
        },
        {
          "type": "call_calculator",
          "calculator_name": "percentage_add",
          "input_mapping": {
            "base_amount": "gross_pay_with_benefits",
            "percentage": "fica_tax_rate"
          },
          "output_field": "gross_pay_with_benefits_and_fica"
        },
        {
          "type": "call_calculator",
          "calculator_name": "percentage_add",
          "input_mapping": {
            "base_amount": "gross_pay_with_benefits_and_fica",
            "percentage": "unemployment_tax_rate"
          },
          "output_field": "total_employee_wage_cost"
        }
      ],
      "priority": 150
    }
  ]
}
```*Note: The `aggregate_sum`, `add`, and `percentage_add` are conceptual calculators. The `aggregate` source type for `value` and `fact_lookup` source type for `multiplier` are conceptual representations of how the engine would need to aggregate and reference data from other facts. The exact implementation may vary.*

## API Request Example

Here is a complete example of what would be sent to the `/evaluate` endpoint.

### Input

```json
{
  "rules": [
    {
      "id": "calculate_shift_hours",
      "priority": 300,
      "conditions": [ { "field": "entity_type", "operator": "equal", "value": "shift" } ],
      "actions": [ { "type": "call_calculator", "calculator_name": "time_between_datetime", "input_mapping": { "start_field": "start_datetime", "end_field": "finish_datetime", "unit": { "value": "hours" } }, "output_field": "calculated_hours" } ]
    },
    {
      "id": "calculate_base_pay_per_shift",
      "priority": 250,
      "conditions": [ { "field": "entity_type", "operator": "equal", "value": "shift" }, { "field": "calculated_hours", "operator": "exists" } ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "multiply",
          "input_mapping": {
            "multiplicand": "calculated_hours",
            "multiplier": { "source_type": "fact_lookup", "fact_id_field": "employee_id", "fact_type": "employee_profile", "field": "hourly_rate" }
          },
          "output_field": "base_shift_cost"
        }
      ]
    },
    {
      "id": "aggregate_employee_gross_pay",
      "priority": 200,
      "conditions": [ { "field": "entity_type", "operator": "equal", "value": "employee_profile" } ],
      "actions": [
        {
          "type": "call_calculator", "calculator_name": "aggregate_sum",
          "input_mapping": {
            "value": { "source_type": "aggregate", "source_field": "base_shift_cost", "filter": "entity_type == 'shift' && employee_id == current_fact.employee_id" }
          },
          "output_field": "total_gross_pay"
        }
      ]
    },
    {
      "id": "calculate_employer_benefits_and_taxes",
      "priority": 150,
      "conditions": [ { "field": "entity_type", "operator": "equal", "value": "employee_profile" }, { "field": "total_gross_pay", "operator": "exists" } ],
      "actions": [
        {
          "type": "call_calculator", "calculator_name": "add",
          "input_mapping": { "addend1": "total_gross_pay", "addend2": "health_insurance_cost_per_period" },
          "output_field": "gross_pay_with_benefits"
        },
        {
          "type": "call_calculator", "calculator_name": "percentage_add",
          "input_mapping": { "base_amount": "gross_pay_with_benefits", "percentage": "fica_tax_rate" },
          "output_field": "gross_pay_with_benefits_and_fica"
        },
        {
          "type": "call_calculator", "calculator_name": "percentage_add",
          "input_mapping": { "base_amount": "gross_pay_with_benefits_and_fica", "percentage": "unemployment_tax_rate" },
          "output_field": "total_employee_wage_cost"
        }
      ]
    }
  ],
  "facts": [
    {
      "id": "emp_profile_001",
      "data": { "entity_type": "employee_profile", "employee_id": "EMP001", "hourly_rate": 25.00, "weekly_overtime_threshold": 40, "health_insurance_cost_per_period": 150.00, "fica_tax_rate": 0.0765, "unemployment_tax_rate": 0.006 }
    },
    {
      "id": "emp_profile_002",
      "data": { "entity_type": "employee_profile", "employee_id": "EMP002", "hourly_rate": 30.00, "weekly_overtime_threshold": 40, "health_insurance_cost_per_period": 150.00, "fica_tax_rate": 0.0765, "unemployment_tax_rate": 0.006 }
    },
    {
      "id": "shift_001",
      "data": { "entity_type": "shift", "employee_id": "EMP001", "start_datetime": "2025-06-03T09:00:00Z", "finish_datetime": "2025-06-03T17:00:00Z", "break_minutes": 30 }
    },
    {
      "id": "shift_002",
      "data": { "entity_type": "shift", "employee_id": "EMP001", "start_datetime": "2025-06-04T09:00:00Z", "finish_datetime": "2025-06-04T19:00:00Z", "break_minutes": 60 }
    },
    {
      "id": "shift_003",
      "data": { "entity_type": "shift", "employee_id": "EMP002", "start_datetime": "2025-06-03T08:00:00Z", "finish_datetime": "2025-06-03T16:00:00Z", "break_minutes": 30 }
    }
  ]
}
```

### Expected Output

Assuming:
*   `shift_001` (EMP001): 7.5 hours (8 - 0.5 break) * $25/hr = $187.50
*   `shift_002` (EMP001): 9 hours (10 - 1 break) * $25/hr = $225.00
*   `shift_003` (EMP002): 7.5 hours (8 - 0.5 break) * $30/hr = $225.00

Total Gross Pay for EMP001 = $187.50 + $225.00 = $412.50
Total Gross Pay for EMP002 = $225.00

EMP001 Total Wage Cost:
$412.50 (Gross) + $150.00 (Health Insurance) = $562.50
$562.50 * (1 + 0.0765) (FICA) = $605.59
$605.59 * (1 + 0.006) (Unemployment) = $609.22

EMP002 Total Wage Cost:
$225.00 (Gross) + $150.00 (Health Insurance) = $375.00
$375.00 * (1 + 0.0765) (FICA) = $403.69
$403.69 * (1 + 0.006) (Unemployment) = $406.11

```json
{
  "request_id": "req_wage_cost_estimation",
  "results": [
    {
      "id": "emp_profile_001",
      "data": {
        "entity_type": "employee_profile",
        "employee_id": "EMP001",
        "hourly_rate": 25.00,
        "weekly_overtime_threshold": 40,
        "health_insurance_cost_per_period": 150.00,
        "fica_tax_rate": 0.0765,
        "unemployment_tax_rate": 0.006,
        "total_gross_pay": 412.50,
        "gross_pay_with_benefits": 562.50,
        "gross_pay_with_benefits_and_fica": 605.59,
        "total_employee_wage_cost": 609.22
      }
    },
    {
      "id": "emp_profile_002",
      "data": {
        "entity_type": "employee_profile",
        "employee_id": "EMP002",
        "hourly_rate": 30.00,
        "weekly_overtime_threshold": 40,
        "health_insurance_cost_per_period": 150.00,
        "fica_tax_rate": 0.0765,
        "unemployment_tax_rate": 0.006,
        "total_gross_pay": 225.00,
        "gross_pay_with_benefits": 375.00,
        "gross_pay_with_benefits_and_fica": 403.69,
        "total_employee_wage_cost": 406.11
      }
    },
    {
      "id": "shift_001",
      "data": {
        "entity_type": "shift",
        "employee_id": "EMP001",
        "start_datetime": "2025-06-03T09:00:00Z",
        "finish_datetime": "2025-06-03T17:00:00Z",
        "break_minutes": 30,
        "calculated_hours": 7.5,
        "base_shift_cost": 187.50
      }
    },
    {
      "id": "shift_002",
      "data": {
        "entity_type": "shift",
        "employee_id": "EMP001",
        "start_datetime": "2025-06-04T09:00:00Z",
        "finish_datetime": "2025-06-04T19:00:00Z",
        "break_minutes": 60,
        "calculated_hours": 9.0,
        "base_shift_cost": 225.00
      }
    },
    {
      "id": "shift_003",
      "data": {
        "entity_type": "shift",
        "employee_id": "EMP002",
        "start_datetime": "2025-06-03T08:00:00Z",
        "finish_datetime": "2025-06-03T16:00:00Z",
        "break_minutes": 30,
        "calculated_hours": 7.5,
        "base_shift_cost": 225.00
      }
    }
  ],
  "facts_processed": 5,
  "rules_evaluated": 4,
  "rules_fired": 5
}
```

## Available Predefined Calculators

### 1. time_between_datetime
**Purpose:** Calculate duration between two datetime values in specified units.
**Input fields:** `start_field`, `end_field`, `unit` (optional, e.g., "hours", "minutes", defaults to "hours").
**Output:** Duration as a float.

### 2. multiply
**Purpose:** Multiplies two numeric values.
**Input fields:** `multiplicand`, `multiplier`.
**Output:** Product as a float.

### Conceptual Calculators for Wage Cost Estimation
*   **aggregate_sum**
    *   **Purpose:** Aggregates a numeric field across multiple facts based on a filter.
    *   **Input fields:** `value` (with `source_type: "aggregate"`, `source_field`, `filter`).
    *   **Output:** Sum of the aggregated values.
*   **add**
    *   **Purpose:** Adds two numeric values.
    *   **Input fields:** `addend1`, `addend2`.
    *   **Output:** Sum as a float.
*   **percentage_add**
    *   **Purpose:** Adds a percentage of a base amount to the base amount.
    *   **Input fields:** `base_amount`, `percentage`.
    *   **Output:** `base_amount * (1 + percentage)` as a float.

## Performance Characteristics

The Wage Cost Estimation engine, built upon the Bingo Rules Engine, is designed for high performance and scalability, capable of handling large-scale enterprise payroll scenarios.

*   **Scalability**: Efficiently processes wage cost estimations for tens of thousands of employees and millions of shifts within acceptable timeframes.
*   **Memory Efficiency**: Optimized for a low memory footprint, suitable for continuous operation in production environments.

## Best Practices

1.  **De-normalized Facts**: Provide facts in a de-normalized format. For example, instead of nesting shifts inside an employee object, provide them as separate, top-level facts with a common `employee_id` for easier processing and aggregation.
2.  **Rule Priorities**: Use `priority` to control the order of execution, ensuring calculations (like shift hours and base pay) happen before aggregations and final cost summations.
3.  **Batching**: Submit all related facts (e.g., all employee profiles and their corresponding shifts for the period) in a single request to allow the engine to perform aggregations and calculations correctly.
4.  **Fact ID Consistency**: Ensure consistent `fact_id` for `employee_profile` facts if you intend to reference them via `fact_lookup` for employee-specific rates and configurations.