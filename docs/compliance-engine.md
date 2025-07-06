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

### API Request Format

This example shows how to structure the rules and facts to execute this staged process in a single call.

**gRPC Method:** `EngineService.Evaluate`

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
            "unit": "hours"
          },
          "output_field": "calculated_hours"
        }
      ],
      "priority": 200
    },
    {
      "id": "student_visa_compliance_check",
      "name": "Student Visa Weekly Hours Compliance",
      "description": "Aggregates shift hours and checks them against the employee's weekly limit.",
      "conditions": [
        { "field": "entity_type", "operator": "equal", "value": "employee" },
        { "field": "is_student_visa", "operator": "equal", "value": true }
      ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "limit_validator",
          "input_mapping": {
            "value": {
              "source_type": "aggregate",
              "source_field": "calculated_hours",
              "filter": "entity_type == 'shift' && employee_id == current_fact.employee_id"
            },
            "warning_threshold": "weekly_hours_warning",
            "critical_threshold": null,
            "max_threshold": "weekly_hours_limit"
          },
          "output_field": "compliance_result"
        }
      ],
      "priority": 100
    }
  ],
  "facts": [
    {
      "id": "emp_123",
      "data": {
        "entity_type": "employee",
        "employee_id": "emp_123",
        "name": "Alice Johnson",
        "is_student_visa": true,
        "weekly_hours_limit": 20.0,
        "weekly_hours_warning": 18.0
      }
    },
    {
      "id": "shift_001",
      "data": {
        "entity_type": "shift",
        "employee_id": "emp_123",
        "start_datetime": "2024-06-17T09:00:00Z",
        "finish_datetime": "2024-06-17T17:00:00Z"
      }
    },
    {
      "id": "shift_002",
      "data": {
        "entity_type": "shift",
        "employee_id": "emp_123",
        "start_datetime": "2024-06-18T10:00:00Z",
        "finish_datetime": "2024-06-18T18:00:00Z"
      }
    },
    {
      "id": "shift_003",
      "data": {
        "entity_type": "shift",
        "employee_id": "emp_123",
        "start_datetime": "2024-06-19T09:00:00Z",
        "finish_datetime": "2024-06-19T17:30:00Z"
      }
    }
  ]
}
```
*Note: The `aggregate` source type in the `student_visa_compliance_check` rule is a conceptual representation of how the engine would need to aggregate data from other facts. The exact implementation may vary.*

### Expected Response

The response shows the result of the compliance check for the employee fact, which is the one that aggregates the data.

```json
{
  "request_id": "req_12345",
  "results": [
    {
      "rule_id": "student_visa_compliance_check",
      "fact_id": "emp_123",
      "actions_executed": [
        {
          "type": "calculator_result",
          "calculator": "limit_validator",
          "result": {
            "severity": "breach",
            "status": "Value 24.5 has breached maximum threshold 20",
            "value": 24.5,
            "utilization_percent": 122.5
          }
        }
      ]
    }
  ],
  "rules_processed": 2,
  "facts_processed": 4,
  "rules_fired": 4
}
```

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
