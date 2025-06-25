# Compliance Engine API Workflow

## Overview

The Bingo engine uses a **two-step workflow** for compliance processing:

1. **Define business logic** by creating calculators via `POST /calculators`
2. **Process data** by submitting facts via `POST /evaluate`

This separation allows you to define the compliance rules once, then process multiple batches of shift data against those rules.

## Student Visa Example Workflow

### Step 1: Create Compliance Calculator

**Endpoint:** `POST /calculators`

```json
{
  "calculator": {
    "id": "student_visa_compliance",
    "description": "Student visa weekly hours compliance check",
    "calculator_type": "ComplianceCalculator",
    "compliance_type": "weekly_hours_limit",
    "target_field": "compliance_status",
    "validation_rules": [
      {
        "rule_type": "weekly_hours_limit",
        "time_calculation": "(finish_datetime - start_datetime) / 3600",
        "aggregation": {
          "field": "hours_worked",
          "method": "sum",
          "group_by": ["employee_id"],
          "time_window": {
            "type": "week",
            "start_day": "monday"
          }
        },
        "threshold_check": {
          "field": "weekly_total_hours",
          "operator": "LessThanOrEqual",
          "threshold_field": "weekly_hours_threshold"
        }
      }
    ],
    "conditions": [
      {
        "field": "is_student_visa",
        "operator": "Equal",
        "value": true
      }
    ],
    "enforcement_actions": [
      {
        "condition": "exceeds_threshold",
        "action": {
          "type": "set_compliance_status",
          "value": "non_compliant"
        }
      },
      {
        "condition": "within_threshold",
        "action": {
          "type": "set_compliance_status", 
          "value": "compliant"
        }
      }
    ]
  }
}
```

### Step 2: Process Shift Data

**Endpoint:** `POST /evaluate`

```json
{
  "facts": [
    {
      "id": 1,
      "data": {
        "fields": {
          "entity_id": "shift_001",
          "entity_type": "worked_shift",
          "employee_id": "emp_123",
          "start_datetime": "2024-06-17T09:00:00Z",
          "finish_datetime": "2024-06-17T17:00:00Z",
          "is_student_visa": true,
          "weekly_hours_threshold": 20
        }
      }
    },
    {
      "id": 2,
      "data": {
        "fields": {
          "entity_id": "shift_002", 
          "entity_type": "worked_shift",
          "employee_id": "emp_123",
          "start_datetime": "2024-06-18T10:00:00Z",
          "finish_datetime": "2024-06-18T18:00:00Z",
          "is_student_visa": true,
          "weekly_hours_threshold": 20
        }
      }
    },
    {
      "id": 3,
      "data": {
        "fields": {
          "entity_id": "shift_003",
          "entity_type": "planned_shift", 
          "employee_id": "emp_123",
          "start_datetime": "2024-06-19T09:00:00Z",
          "finish_datetime": "2024-06-19T19:00:00Z",
          "is_student_visa": true,
          "weekly_hours_threshold": 20
        }
      }
    }
  ]
}
```

### Expected Response

```json
{
  "results": [
    {
      "id": 1,
      "data": {
        "fields": {
          "entity_id": "shift_001",
          "employee_id": "emp_123",
          "hours_worked": 8,
          "compliance_status": "non_compliant",
          "weekly_total_hours": 26,
          "weekly_hours_threshold": 20,
          "excess_hours": 6
        }
      }
    }
  ],
  "stats": {
    "facts_processed": 3,
    "calculators_executed": 1,
    "compliance_violations": 1
  },
  "validation_report": {
    "violations": [
      {
        "employee_id": "emp_123",
        "violation_type": "weekly_hours_exceeded",
        "week_start": "2024-06-17",
        "total_hours": 26,
        "threshold": 20,
        "excess_hours": 6
      }
    ]
  }
}
```

## Key Points

1. **Calculator Definition is Persistent**: Once created via `POST /calculators`, the business logic is stored and applied to all future fact submissions.

2. **Facts Include Configuration**: Each fact must include the relevant configuration data (`is_student_visa`, `weekly_hours_threshold`) so the calculator can apply the rules.

3. **Automatic Processing**: The engine automatically matches facts to calculators based on the conditions and executes the compliance logic.

4. **Structured Output**: Results include both the processed facts and detailed compliance reports with violation information.

## Alternative: Combined Approach

You could also embed the rules directly in the `/evaluate` request, but the calculator approach is recommended for reusable compliance logic.