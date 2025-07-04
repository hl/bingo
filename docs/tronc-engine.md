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

```json
{
  "rules": [
    {
      "id": "deduct_admin_fee",
      "name": "Deduct Administration Fee from TRONC Pool",
      "description": "Deducts a specified percentage as an administration fee from the total TRONC amount.",
      "conditions": [
        { "field": "entity_type", "operator": "equal", "value": "tronc_distribution_config" }
      ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "percentage_deduct",
          "input_mapping": {
            "total_amount": "total_tronc_amount",
            "percentage": "administration_fee_percentage"
          },
          "output_field": "adjusted_tronc_amount"
        }
      ],
      "priority": 300
    },
    {
      "id": "calculate_weighted_eligible_hours",
      "name": "Calculate Total Weighted Eligible Hours for TRONC Distribution",
      "description": "Aggregates total weighted hours worked by eligible employees for a given distribution period, considering role-based weights.",
      "conditions": [
        { "field": "entity_type", "operator": "equal", "value": "tronc_distribution_config" }
      ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "aggregate_weighted_sum",
          "input_mapping": {
            "value": {
              "source_type": "aggregate",
              "source_field": "hours_worked",
              "filter": "entity_type == 'employee_shift' && shift_date == current_fact.distribution_date",
              "weight_field": "role",
              "weight_lookup_fact_type": "role_weight_config",
              "weight_lookup_field": "weight"
            }
          },
          "output_field": "total_weighted_eligible_hours"
        }
      ],
      "priority": 200
    },
    {
      "id": "allocate_tronc_to_shift",
      "name": "Allocate TRONC to Employee Shifts",
      "description": "Calculates each shift's proportional share of the adjusted TRONC pool based on weighted hours.",
      "conditions": [
        { "field": "entity_type", "operator": "equal", "value": "employee_shift" }
      ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "allocate_proportional",
          "input_mapping": {
            "total_amount": {
              "source_type": "fact_lookup",
              "fact_id": "tronc_config_2025-06-28",
              "field": "adjusted_tronc_amount"
            },
            "individual_value": {
              "source_type": "current_fact",
              "source_field": "hours_worked",
              "weight_field": "role",
              "weight_lookup_fact_type": "role_weight_config",
              "weight_lookup_field": "weight"
            },
            "total_value": {
              "source_type": "fact_lookup",
              "fact_id": "tronc_config_2025-06-28",
              "field": "total_weighted_eligible_hours"
            }
          },
          "output_field": "tronc_allocated_amount"
        }
      ],
      "priority": 100
    }
  ]
}
```
*Note: The `aggregate` and `fact_lookup` source types are part of the engine's powerful data access DSL. They allow rules to dynamically aggregate data from across the fact network and look up specific values from other facts by their ID, enabling complex, multi-stage calculations.*

## API Request Example

Here is a complete example of what would be sent to the `/evaluate` endpoint.

### Input

```json
{
  "rules": [
    {
      "id": "deduct_admin_fee",
      "priority": 300,
      "conditions": [ { "field": "entity_type", "operator": "equal", "value": "tronc_distribution_config" } ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "percentage_deduct",
          "input_mapping": {
            "total_amount": "total_tronc_amount",
            "percentage": "administration_fee_percentage"
          },
          "output_field": "adjusted_tronc_amount"
        }
      ]
    },
    {
      "id": "calculate_weighted_eligible_hours",
      "priority": 200,
      "conditions": [ { "field": "entity_type", "operator": "equal", "value": "tronc_distribution_config" } ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "aggregate_weighted_sum",
          "input_mapping": {
            "value": {
              "source_type": "aggregate",
              "source_field": "hours_worked",
              "filter": "entity_type == 'employee_shift' && shift_date == current_fact.distribution_date",
              "weight_field": "role",
              "weight_lookup_fact_type": "role_weight_config",
              "weight_lookup_field": "weight"
            }
          },
          "output_field": "total_weighted_eligible_hours"
        }
      ]
    },
    {
      "id": "allocate_tronc_to_shift",
      "priority": 100,
      "conditions": [ { "field": "entity_type", "operator": "equal", "value": "employee_shift" } ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "allocate_proportional",
          "input_mapping": {
            "total_amount": {
              "source_type": "fact_lookup",
              "fact_id": "tronc_config_2025-06-28",
              "field": "adjusted_tronc_amount"
            },
            "individual_value": {
              "source_type": "current_fact",
              "source_field": "hours_worked",
              "weight_field": "role",
              "weight_lookup_fact_type": "role_weight_config",
              "weight_lookup_field": "weight"
            },
            "total_value": {
              "source_type": "fact_lookup",
              "fact_id": "tronc_config_2025-06-28",
              "field": "total_weighted_eligible_hours"
            }
          },
          "output_field": "tronc_allocated_amount"
        }
      ]
    }
  ],
  "facts": [
    {
      "id": "tronc_config_2025-06-28",
      "data": { "entity_type": "tronc_distribution_config", "total_tronc_amount": 500.00, "administration_fee_percentage": 0.05, "distribution_date": "2025-06-28" }
    },
    { "id": "role_weight_waiter", "data": { "entity_type": "role_weight_config", "role": "waiter", "weight": 1.0 } },
    { "id": "role_weight_bartender", "data": { "entity_type": "role_weight_config", "role": "bartender", "weight": 1.2 } },
    { "id": "shift_001", "data": { "entity_type": "employee_shift", "employee_id": "emp_A", "hours_worked": 8.0, "shift_date": "2025-06-28", "role": "waiter" } },
    { "id": "shift_002", "data": { "entity_type": "employee_shift", "employee_id": "emp_B", "hours_worked": 6.0, "shift_date": "2025-06-28", "role": "waiter" } },
    { "id": "shift_003", "data": { "entity_type": "employee_shift", "employee_id": "emp_C", "hours_worked": 10.0, "shift_date": "2025-06-28", "role": "bartender" } }
  ]
}
```

### Expected Output

Assuming `adjusted_tronc_amount` is calculated as 475.00 (500 - 500 * 0.05) and `total_weighted_eligible_hours` is 26.0 (8*1.0 + 6*1.0 + 10*1.2):

*   `shift_001`: (8.0 / 26.0) * 475.00 = 146.15
*   `shift_002`: (6.0 / 26.0) * 475.00 = 109.62
*   `shift_003`: (12.0 / 26.0) * 475.00 = 219.23

```json
{
  "request_id": "req_example_tronc_distribution",
  "results": [
    {
      "id": "tronc_config_2025-06-28",
      "data": {
        "entity_type": "tronc_distribution_config",
        "total_tronc_amount": 500.00,
        "administration_fee_percentage": 0.05,
        "distribution_date": "2025-06-28",
        "adjusted_tronc_amount": 475.00,
        "total_weighted_eligible_hours": 26.0
      }
    },
    {
      "id": "role_weight_waiter",
      "data": {
        "entity_type": "role_weight_config",
        "role": "waiter",
        "weight": 1.0
      }
    },
    {
      "id": "role_weight_bartender",
      "data": {
        "entity_type": "role_weight_config",
        "role": "bartender",
        "weight": 1.2
      }
    },
    {
      "id": "shift_001",
      "data": {
        "entity_type": "employee_shift",
        "employee_id": "emp_A",
        "hours_worked": 8.0,
        "shift_date": "2025-06-28",
        "role": "waiter",
        "tronc_allocated_amount": 146.15
      }
    },
    {
      "id": "shift_002",
      "data": {
        "entity_type": "employee_shift",
        "employee_id": "emp_B",
        "hours_worked": 6.0,
        "shift_date": "2025-06-28",
        "role": "waiter",
        "tronc_allocated_amount": 109.62
      }
    },
    {
      "id": "shift_003",
      "data": {
        "entity_type": "employee_shift",
        "employee_id": "emp_C",
        "hours_worked": 10.0,
        "shift_date": "2025-06-28",
        "role": "bartender",
        "tronc_allocated_amount": 219.23
      }
    }
  ],
  "facts_processed": 6,
  "rules_evaluated": 3,
  "rules_fired": 6
}
```

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
