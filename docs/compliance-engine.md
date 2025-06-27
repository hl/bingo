# Compliance Engine Guide

## Overview

The Bingo Rules Engine provides comprehensive compliance checking capabilities through its simplified `/evaluate` API endpoint. This guide demonstrates how to implement student visa compliance checking and other compliance scenarios using the engine's predefined calculators.

## Student Visa Compliance Example

### Scenario
When an employee is a student visa holder, they are not allowed to work more than 20 hours per week (Monday-Sunday). The system needs to:
- Calculate hours worked from shift start/end times
- Aggregate weekly totals per employee
- Flag compliance violations with detailed reporting

### API Request Format

**Endpoint:** `POST /evaluate`

```json
{
  "rules": [
    {
      "id": "student_visa_compliance",
      "name": "Student Visa Weekly Hours Compliance",
      "description": "Ensure student visa holders don't exceed 20 hours per week",
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
            "value": "weekly_hours",
            "threshold": "weekly_limit",
            "operator": "LessThanOrEqual"
          },
          "output_field": "compliance_status"
        }
      ],
      "enabled": true,
      "tags": ["compliance", "student_visa", "legal"],
      "priority": 100,
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  ],
  "facts": [
    {
      "id": "emp_123_week_2024_25",
      "data": {
        "employee_id": "emp_123",
        "name": "Alice Johnson",
        "is_student_visa": true,
        "weekly_hours": 24.5,
        "weekly_limit": 20.0,
        "week_start": "2024-06-17",
        "week_end": "2024-06-23",
        "shifts": [
          {
            "shift_id": "shift_001",
            "start_datetime": "2024-06-17T09:00:00Z",
            "finish_datetime": "2024-06-17T17:00:00Z",
            "hours": 8.0,
            "type": "worked_shift"
          },
          {
            "shift_id": "shift_002",
            "start_datetime": "2024-06-18T10:00:00Z",
            "finish_datetime": "2024-06-18T18:00:00Z",
            "hours": 8.0,
            "type": "worked_shift"
          },
          {
            "shift_id": "shift_003",
            "start_datetime": "2024-06-19T09:00:00Z",
            "finish_datetime": "2024-06-19T17:30:00Z",
            "hours": 8.5,
            "type": "planned_shift"
          }
        ]
      },
      "created_at": "2024-06-19T00:00:00Z"
    }
  ]
}
```

### Expected Response

```json
{
  "request_id": "req_12345",
  "results": [
    {
      "rule_id": "student_visa_compliance",
      "fact_id": "emp_123_week_2024_25",
      "actions_executed": [
        {
          "type": "calculator_result",
          "calculator": "threshold_checker",
          "result": "Object({\"passes\": Boolean(false), \"value\": Float(24.5), \"threshold\": Float(20.0), \"operator\": String(\"LessThanOrEqual\"), \"violation_amount\": Float(4.5), \"status\": String(\"non_compliant\")})"
        }
      ]
    }
  ],
  "rules_processed": 1,
  "facts_processed": 1,
  "rules_fired": 1,
  "processing_time_ms": 2,
  "stats": {
    "total_facts": 1,
    "total_rules": 1,
    "network_nodes": 2,
    "memory_usage_bytes": 1024
  }
}
```

## Multi-Employee Batch Processing

You can process multiple employees in a single request:

```json
{
  "rules": [
    {
      "id": "student_visa_compliance",
      "name": "Student Visa Weekly Hours Compliance",
      "description": "Batch compliance check for multiple employees",
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
          "calculator_name": "limit_validator",
          "input_mapping": {
            "value": "weekly_hours",
            "warning_threshold": "warning_limit",
            "critical_threshold": "critical_limit",
            "max_threshold": "legal_limit"
          },
          "output_field": "compliance_analysis"
        }
      ],
      "enabled": true,
      "tags": ["compliance", "batch"],
      "priority": 100,
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  ],
  "facts": [
    {
      "id": "emp_001_week_25",
      "data": {
        "employee_id": "emp_001",
        "name": "Alice Johnson",
        "is_student_visa": true,
        "weekly_hours": 18.0,
        "warning_limit": 16.0,
        "critical_limit": 18.0,
        "legal_limit": 20.0
      },
      "created_at": "2024-06-19T00:00:00Z"
    },
    {
      "id": "emp_002_week_25",
      "data": {
        "employee_id": "emp_002",
        "name": "Bob Smith",
        "is_student_visa": true,
        "weekly_hours": 22.5,
        "warning_limit": 16.0,
        "critical_limit": 18.0,
        "legal_limit": 20.0
      },
      "created_at": "2024-06-19T00:00:00Z"
    }
  ]
}
```

## Available Predefined Calculators

The Bingo engine includes several built-in calculators for compliance scenarios:

### 1. threshold_checker
**Purpose:** Simple threshold compliance validation
**Use cases:** Hours limits, budget limits, quantity thresholds

**Input fields:**
- `value` (required): The value to check
- `threshold` (required): The limit to check against
- `operator` (optional): Comparison operator (default: "LessThanOrEqual")

**Output:** Object with `passes`, `status`, `violation_amount`, etc.

### 2. limit_validator
**Purpose:** Multi-tier validation with warning/critical/breach levels
**Use cases:** Progressive alerting, tiered compliance monitoring

**Input fields:**
- `value` (required): The value to validate
- `warning_threshold` (optional): Warning level
- `critical_threshold` (optional): Critical level
- `max_threshold` (optional): Maximum allowed value

**Output:** Object with `severity`, `status`, `utilization_percent`, etc.

### 3. hours_between_datetime
**Purpose:** Calculate hours between two datetime values
**Use cases:** Shift duration calculations, time-based compliance

**Input fields:**
- `start_datetime`: Start time (ISO 8601 format)
- `end_datetime`: End time (ISO 8601 format)

**Output:** Hours as floating point number

## Common Compliance Patterns

### 1. Weekly Hours Validation
```json
{
  "type": "call_calculator",
  "calculator_name": "threshold_checker",
  "input_mapping": {
    "value": "weekly_hours",
    "threshold": "max_weekly_hours",
    "operator": "LessThanOrEqual"
  },
  "output_field": "weekly_compliance"
}
```

### 2. Progressive Alert System
```json
{
  "type": "call_calculator",
  "calculator_name": "limit_validator",
  "input_mapping": {
    "value": "current_hours",
    "warning_threshold": "warning_hours",
    "critical_threshold": "critical_hours",
    "max_threshold": "legal_limit"
  },
  "output_field": "alert_status"
}
```

### 3. Time-based Calculations
```json
{
  "type": "call_calculator",
  "calculator_name": "hours_between_datetime",
  "input_mapping": {
    "start_datetime": "shift_start",
    "end_datetime": "shift_end"
  },
  "output_field": "shift_duration"
}
```

## API Testing

### Health Check
```bash
curl http://localhost:3000/health
```

### Compliance Evaluation
```bash
curl -X POST http://localhost:3000/evaluate \
  -H "Content-Type: application/json" \
  -d '{"rules": [...], "facts": [...]}'
```

### OpenAPI Documentation
- **Swagger UI:** http://localhost:3000/swagger-ui/
- **ReDoc:** http://localhost:3000/redoc/
- **OpenAPI JSON:** http://localhost:3000/api-docs/openapi.json

## Performance Characteristics

The Bingo compliance engine delivers exceptional performance:

- **100K facts**: ~635ms processing time
- **1M facts**: ~6.59s processing time (4.5x faster than 30s target)
- **Memory efficient**: <3GB for enterprise-scale workloads
- **Linear scaling**: Predictable performance growth

## Best Practices

1. **Batch Processing**: Submit multiple facts in a single request for efficiency
2. **Rule Reuse**: Design rules to be reusable across different compliance scenarios
3. **Field Naming**: Use consistent field names across facts for easier rule writing
4. **Error Handling**: Always check the response for calculation errors
5. **Performance**: For large datasets, consider breaking into smaller batches

## Error Handling

The API returns structured errors:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid threshold value",
    "details": {
      "field": "weekly_limit",
      "value": -5,
      "constraint": "must_be_positive"
    },
    "request_id": "req_12345"
  }
}
```

This consolidated compliance engine guide provides everything needed to implement robust compliance checking using the Bingo Rules Engine's simplified API.