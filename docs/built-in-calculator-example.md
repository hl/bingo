# Built-in Calculator Engine Example

The compliance engine uses built-in, compiled calculators for maximum performance and auditability. This approach provides reusable calculators that can be applied across compliance, payroll, cost control, and other business domains.

## Student Visa Compliance Example

This example shows how to use the generic built-in calculators to implement student visa hour limit compliance checking.

### API Usage

**Endpoint:** `POST /facts/process`

#### Input (Updated for Built-in Calculators)

```json
{
  "facts": [
    {
      "id": "shift_001",
      "data": {
        "entity_id": "shift_001",
        "entity_type": "worked_shift",
        "employee_id": "emp_123",
        "start_datetime": "2024-06-17T09:00:00Z",
        "finish_datetime": "2024-06-17T17:00:00Z"
      }
    },
    {
      "id": "shift_002", 
      "data": {
        "entity_id": "shift_002",
        "entity_type": "worked_shift",
        "employee_id": "emp_123",
        "start_datetime": "2024-06-18T10:00:00Z",
        "finish_datetime": "2024-06-18T18:00:00Z"
      }
    },
    {
      "id": "shift_003",
      "data": {
        "entity_id": "shift_003",
        "entity_type": "planned_shift", 
        "employee_id": "emp_123",
        "start_datetime": "2024-06-19T09:00:00Z",
        "finish_datetime": "2024-06-19T19:00:00Z"
      }
    },
    {
      "id": "config_emp_123",
      "data": {
        "entity_type": "employee_config",
        "employee_id": "emp_123",
        "is_student_visa": true,
        "weekly_hours_threshold": 20
      }
    }
  ],
  "rules": [
    {
      "id": "calculate_shift_hours",
      "name": "Calculate Hours for Each Shift",
      "conditions": [
        {
          "type": "simple",
          "field": "entity_type",
          "operator": "contains",
          "value": "shift"
        }
      ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "hours_between_datetime",
          "input_mapping": {
            "start_field": "start_datetime",
            "end_field": "finish_datetime"
          },
          "output_field": "hours_worked"
        }
      ]
    },
    {
      "id": "student_visa_compliance_check",
      "name": "Student Visa Hours Compliance",
      "conditions": [
        {
          "type": "simple",
          "field": "is_student_visa",
          "operator": "equal",
          "value": true
        }
      ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "threshold_checker",
          "input_mapping": {
            "value": "weekly_total_hours",
            "threshold": "weekly_hours_threshold",
            "operator": "LessThanOrEqual"
          },
          "output_field": "compliance_result"
        }
      ]
    }
  ]
}
```

#### Expected Response

```json
{
  "request_id": "req_123e4567-e89b-12d3-a456-426614174000",
  "results": [
    {
      "id": "shift_001",
      "data": {
        "entity_id": "shift_001",
        "entity_type": "worked_shift",
        "employee_id": "emp_123",
        "start_datetime": "2024-06-17T09:00:00Z",
        "finish_datetime": "2024-06-17T17:00:00Z",
        "hours_worked": 8.0
      },
      "created_at": "2024-06-24T14:30:00Z"
    },
    {
      "id": "shift_002",
      "data": {
        "entity_id": "shift_002",
        "entity_type": "worked_shift",
        "employee_id": "emp_123", 
        "start_datetime": "2024-06-18T10:00:00Z",
        "finish_datetime": "2024-06-18T18:00:00Z",
        "hours_worked": 8.0
      },
      "created_at": "2024-06-24T14:30:00Z"
    },
    {
      "id": "shift_003",
      "data": {
        "entity_id": "shift_003",
        "entity_type": "planned_shift",
        "employee_id": "emp_123",
        "start_datetime": "2024-06-19T09:00:00Z", 
        "finish_datetime": "2024-06-19T19:00:00Z",
        "hours_worked": 10.0
      },
      "created_at": "2024-06-24T14:30:00Z"
    },
    {
      "id": "config_emp_123",
      "data": {
        "entity_type": "employee_config",
        "employee_id": "emp_123",
        "is_student_visa": true,
        "weekly_hours_threshold": 20,
        "compliance_result": {
          "passes": false,
          "value": 26.0,
          "threshold": 20.0,
          "operator": "LessThanOrEqual",
          "violation_amount": 6.0,
          "status": "non_compliant"
        }
      },
      "created_at": "2024-06-24T14:30:00Z"
    }
  ],
  "facts_processed": 4,
  "rules_evaluated": 2,
  "rules_fired": 4,
  "processing_time_ms": 12,
  "stats": {
    "total_facts": 4,
    "total_rules": 2,
    "network_nodes": 8,
    "memory_usage_bytes": 2048576
  }
}
```

## Available Built-in Calculators

### Time & Duration Calculators

#### `hours_between_datetime`
- **Purpose**: Calculate hours between two datetime fields
- **Use Cases**: Shift duration, overtime calculation, billing hours, project time tracking
- **Required Inputs**: 
  - `start_field` (string): Name of the field containing start datetime
  - `end_field` (string): Name of the field containing end datetime
- **Output**: Float representing hours between the datetimes

#### `time_difference` 
- **Purpose**: Calculate time difference with configurable units
- **Use Cases**: Break duration, meeting length, project phases, billing periods
- **Required Inputs**:
  - `start_field` (string): Name of the field containing start datetime
  - `end_field` (string): Name of the field containing end datetime
  - `unit` (string, optional): "seconds", "minutes", "hours", "days" (default: "hours")
- **Output**: Object with time difference in multiple formats

### Threshold & Validation Calculators

#### `threshold_checker`
- **Purpose**: Generic threshold validation for compliance, limits, and validations
- **Use Cases**: Student visa hours, overtime thresholds, budget limits, performance targets, cost control
- **Required Inputs**:
  - `value` (float): Value to check
  - `threshold` (float): Threshold to compare against
  - `operator` (string, optional): "LessThan", "LessThanOrEqual", "GreaterThan", "GreaterThanOrEqual", "Equal", "NotEqual" (default: "LessThanOrEqual")
- **Output**: Object with compliance status and violation details

#### `limit_validator`
- **Purpose**: Multi-tier limit validation with severity levels
- **Use Cases**: Multi-tier overtime rates, budget alerts, performance ratings, cost tiers
- **Required Inputs**:
  - `value` (float): Value to validate
  - `warning_threshold` (float, optional): Warning level threshold
  - `critical_threshold` (float, optional): Critical level threshold  
  - `max_threshold` (float, optional): Maximum allowed threshold
- **Output**: Object with severity level, status, and utilization details

## Alternative Use Cases

### Overtime Calculation
```json
{
  "type": "call_calculator",
  "calculator_name": "limit_validator", 
  "input_mapping": {
    "value": "weekly_hours",
    "warning_threshold": "40",
    "critical_threshold": "48", 
    "max_threshold": "60"
  },
  "output_field": "overtime_status"
}
```

### Budget Monitoring
```json
{
  "type": "call_calculator",
  "calculator_name": "threshold_checker",
  "input_mapping": {
    "value": "current_spend",
    "threshold": "budget_limit",
    "operator": "LessThanOrEqual"
  },
  "output_field": "budget_compliance"
}
```

### Meeting Duration Tracking
```json
{
  "type": "call_calculator",
  "calculator_name": "time_difference",
  "input_mapping": {
    "start_field": "meeting_start",
    "end_field": "meeting_end", 
    "unit": "minutes"
  },
  "output_field": "meeting_duration"
}
```

## Benefits of Built-in Calculators

1. **High Performance**: Compiled Rust execution vs interpreted rules (10-100x faster)
2. **Type Safety**: Compile-time validation prevents runtime errors
3. **Reusability**: Generic calculators work across compliance, payroll, cost control
4. **Auditability**: Version-controlled business logic with clear change history
5. **Operational Simplicity**: Standard application deployments, no rule distribution
6. **Unlimited Concurrency**: No shared state allows horizontal scaling

## Calculator Error Handling

When calculators encounter errors, they create structured error facts:

```json
{
  "entity_type": "calculator_error",
  "calculator_name": "threshold_checker",
  "error_code": "MISSING_REQUIRED_FIELD",
  "error_message": "Required field 'value' is missing",
  "triggering_fact_id": "shift_001",
  "details": {
    "field": "value"
  }
}
```

These error facts can be processed by additional rules for error handling, notifications, or fallback logic.