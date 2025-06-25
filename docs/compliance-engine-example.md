# Compliance Engine Example

The compliance engine will ingest worked and planned shift data and will determine if an employee is compliant.

## Student Visa Rule

When an employee is a student, they are not allowed to work more than a certain amount of time per week spanning Mon-Sun.
The threshold is based on their degree and will be supplied as a config parameter.

## Data

The facts will be shift data in the form of

- entity_id: shift_id
- entity_type: worked_shift/planned_shift
- start_datetime: start date time of the shift in UTC
- finish_datetime: finish date time of the shift in UTC

### Example Input JSON

```json
{
  "facts": [
    {
      "entity_id": "shift_001",
      "entity_type": "worked_shift",
      "employee_id": "emp_123",
      "start_datetime": "2024-06-17T09:00:00Z",
      "finish_datetime": "2024-06-17T17:00:00Z"
    },
    {
      "entity_id": "shift_002", 
      "entity_type": "worked_shift",
      "employee_id": "emp_123",
      "start_datetime": "2024-06-18T10:00:00Z",
      "finish_datetime": "2024-06-18T18:00:00Z"
    },
    {
      "entity_id": "shift_003",
      "entity_type": "planned_shift", 
      "employee_id": "emp_123",
      "start_datetime": "2024-06-19T09:00:00Z",
      "finish_datetime": "2024-06-19T19:00:00Z"
    }
  ],
  "configuration": {
    "employee_id": "emp_123",
    "is_student_visa": true,
    "weekly_hours_threshold": 20
  }
}

## Rules engine

The rules engine will be provided with 2 types of facts:

1. the shift data
2. the configuration
  2.1. if the employee is under a student visa (boolean)
  2.2. what the threshold is

The rules engine has the following reposibilities:

- calculate the amount of `minutes` between start and finish and update the fact with the value
- calculate the amount of `units`, e.g. `minute` / 60 and update the fact with the value
- check per week spanning Mon-Sun how much an employee has worked
- check if the amount of `units` per week reaches the threshold
  - if it goes over the threshold, the rules engine will return a `non_compliance` for that employee
  - if it stays under or equal to the threshold, the rules engine will return a `compliant` for that employee

## Output

The expected output is a list of employees with either a `compliant` or `non_compliant` status, the offending weeks as dates (Mon-Sun) and a breakdown of the exceeded hours and threshold.

### Example Output JSON

```json
{
  "compliance_results": [
    {
      "employee_id": "emp_123",
      "status": "non_compliant",
      "violations": [
        {
          "week_start": "2024-06-17",
          "week_end": "2024-06-23", 
          "total_hours": 26,
          "threshold": 20,
          "excess_hours": 6,
          "shifts": [
            {
              "entity_id": "shift_001",
              "date": "2024-06-17",
              "hours": 8,
              "type": "worked_shift"
            },
            {
              "entity_id": "shift_002", 
              "date": "2024-06-18",
              "hours": 8,
              "type": "worked_shift"
            },
            {
              "entity_id": "shift_003",
              "date": "2024-06-19", 
              "hours": 10,
              "type": "planned_shift"
            }
          ]
        }
      ]
    }
  ]
}
```