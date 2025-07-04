# Bingo RETE Rules Engine - Comprehensive API Documentation

## Overview

The Bingo RETE Rules Engine provides a high-performance rules processing system built on the RETE algorithm. This documentation complements the OpenAPI specifications by providing practical usage guidance, architectural context, and real-world examples.

## Architecture Summary

The Bingo engine follows a modular architecture:

- **bingo-core**: Core RETE algorithm implementation, fact storage, and rule processing
- **bingo-api**: HTTP/JSON API layer with caching and operational features
- **bingo-calculator**: Business logic calculators for payroll, tax, and compliance
- **bingo-prelude**: Common types and utilities shared across crates

## Core API Operations

### 1. Fact Management

#### Adding Facts

Facts represent data points that the rules engine processes. Each fact contains:
- `id`: Unique identifier (auto-generated if not provided)
- `external_id`: Optional external system reference
- `timestamp`: Event timestamp (defaults to current time)
- `data.fields`: Key-value pairs containing the fact data

```json
POST /api/facts
{
  "external_id": "employee_001",
  "data": {
    "fields": {
      "employee_id": "EMP001",
      "salary": 75000,
      "department": "Engineering",
      "hire_date": "2023-01-15T00:00:00Z",
      "status": "active"
    }
  }
}
```

**Response:**
```json
{
  "id": 12345,
  "external_id": "employee_001",
  "timestamp": "2024-01-15T10:30:00Z",
  "data": {
    "fields": {
      "employee_id": "EMP001",
      "salary": 75000,
      "department": "Engineering",
      "hire_date": "2023-01-15T00:00:00Z",
      "status": "active"
    }
  }
}
```

#### Querying Facts

Retrieve facts by ID or search criteria:

```bash
# Get specific fact
GET /api/facts/12345

# Search by field values
GET /api/facts?field=department&value=Engineering
GET /api/facts?field=salary&operator=GreaterThan&value=70000
```

### 2. Rule Management

#### Rule Structure

Rules consist of conditions (when to fire) and actions (what to do):

```json
POST /api/rules
{
  "name": "High Salary Alert",
  "conditions": [
    {
      "field": "salary",
      "operator": "GreaterThan",
      "value": 100000
    },
    {
      "field": "department",
      "operator": "Equal",
      "value": "Engineering"
    }
  ],
  "actions": [
    {
      "action_type": {
        "Log": {
          "message": "High salary employee in Engineering: {employee_id}"
        }
      }
    },
    {
      "action_type": {
        "TriggerAlert": {
          "alert_type": "salary_review",
          "message": "Employee {employee_id} has high salary",
          "severity": "Medium",
          "metadata": {
            "employee_id": "{employee_id}",
            "salary": "{salary}"
          }
        }
      }
    }
  ]
}
```

#### Complex Conditions

Rules support complex logical operations:

```json
{
  "conditions": [
    {
      "Complex": {
        "operator": "Or",
        "conditions": [
          {
            "field": "department",
            "operator": "Equal",
            "value": "Sales"
          },
          {
            "Complex": {
              "operator": "And",
              "conditions": [
                {
                  "field": "department",
                  "operator": "Equal",
                  "value": "Engineering"
                },
                {
                  "field": "experience_years",
                  "operator": "GreaterThan",
                  "value": 5
                }
              ]
            }
          }
        ]
      }
    }
  ]
}
```

### 3. Rule Processing

#### Single Fact Processing

Process a single fact against all rules:

```json
POST /api/process
{
  "fact": {
    "data": {
      "fields": {
        "employee_id": "EMP002",
        "salary": 120000,
        "department": "Engineering"
      }
    }
  }
}
```

**Response:**
```json
{
  "results": [
    {
      "rule_id": 1,
      "rule_name": "High Salary Alert",
      "actions_executed": [
        {
          "action_type": "Log",
          "result": "success",
          "message": "High salary employee in Engineering: EMP002"
        },
        {
          "action_type": "TriggerAlert",
          "result": "success",
          "alert_id": "alert_12345"
        }
      ]
    }
  ],
  "processing_time_ms": 15,
  "facts_created": []
}
```

#### Batch Processing

Process multiple facts efficiently:

```json
POST /api/process/batch
{
  "facts": [
    {
      "data": {
        "fields": {
          "employee_id": "EMP003",
          "salary": 95000,
          "department": "Marketing"
        }
      }
    },
    {
      "data": {
        "fields": {
          "employee_id": "EMP004",
          "salary": 110000,
          "department": "Engineering"
        }
      }
    }
  ]
}
```

### 4. Calculator Integration

#### Built-in Calculators

The engine includes pre-built calculators for common business logic:

```json
POST /api/rules
{
  "name": "Tax Calculation",
  "conditions": [
    {
      "field": "salary",
      "operator": "GreaterThan",
      "value": 0
    }
  ],
  "actions": [
    {
      "action_type": {
        "CallCalculator": {
          "calculator_name": "tax_calculator",
          "input_mapping": {
            "gross_salary": "salary",
            "tax_year": "2024"
          },
          "output_field": "tax_amount"
        }
      }
    }
  ]
}
```

#### Available Calculators

- `threshold_check`: Validates values against thresholds
- `limit_validate`: Ensures values don't exceed limits
- `percentage_calculator`: Applies percentage calculations
- `tiered_rate_calculator`: Applies progressive rates

### 5. Stream Processing

#### Window-based Processing

For time-series data processing:

```json
POST /api/rules
{
  "name": "Hourly Transaction Volume",
  "conditions": [
    {
      "Stream": {
        "window_spec": {
          "Tumbling": {
            "duration_ms": 3600000
          }
        },
        "aggregation": {
          "Count": {}
        },
        "filter": {
          "field": "transaction_type",
          "operator": "Equal",
          "value": "purchase"
        },
        "having": {
          "field": "count",
          "operator": "GreaterThan",
          "value": 1000
        },
        "alias": "hourly_count"
      }
    }
  ],
  "actions": [
    {
      "action_type": {
        "TriggerAlert": {
          "alert_type": "high_volume",
          "message": "High transaction volume: {hourly_count}",
          "severity": "High",
          "metadata": {}
        }
      }
    }
  ]
}
```

## Data Types and Validation

### FactValue Types

Facts support multiple data types:

```json
{
  "string_field": "text value",
  "integer_field": 42,
  "float_field": 3.14159,
  "boolean_field": true,
  "date_field": "2024-01-15T10:30:00Z",
  "array_field": [1, 2, 3],
  "object_field": {
    "nested_key": "nested_value"
  },
  "null_field": null
}
```

### Operators

Available comparison operators:
- `Equal` / `NotEqual`
- `GreaterThan` / `LessThan`
- `GreaterThanOrEqual` / `LessThanOrEqual`
- `Contains` (for strings and arrays)

### Logical Operators

For complex conditions:
- `And`: All sub-conditions must be true
- `Or`: At least one sub-condition must be true
- `Not`: Inverts the result of sub-conditions

## Performance Considerations

### Caching

The API layer includes intelligent caching:
- **Ruleset Cache**: Compiled rules cached with TTL
- **Fact Lookup Cache**: Frequently accessed facts cached in memory
- **Calculator Result Cache**: Expensive calculations cached by input hash

### Indexing

Automatic indexing on commonly queried fields:
- `entity_id`, `id`, `user_id`, `customer_id`
- `status`, `category`
- Custom fields configurable via API

### Batch Operations

For high-throughput scenarios:
- Use batch processing endpoints
- Pre-allocate fact IDs when possible
- Consider streaming APIs for continuous data

## Error Handling

### Standard Error Response

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid field type",
    "details": {
      "field": "salary",
      "expected_type": "number",
      "received_type": "string"
    },
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

### Common Error Codes

- `VALIDATION_ERROR`: Invalid input data
- `RULE_COMPILATION_ERROR`: Rule syntax errors
- `CALCULATOR_ERROR`: Calculator execution failures
- `TIMEOUT_ERROR`: Processing timeout exceeded
- `RESOURCE_LIMIT_ERROR`: Memory or fact limits exceeded

## Monitoring and Observability

### Health Checks

```bash
GET /health
```

Response:
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 86400,
  "memory_usage_mb": 256,
  "active_rules": 42,
  "total_facts": 10000,
  "cache_hit_rate": 85.5
}
```

### Metrics Endpoint

```bash
GET /metrics
```

Provides Prometheus-compatible metrics for:
- Request rates and latencies
- Rule execution statistics
- Cache performance
- Memory usage patterns

### Debugging

Enable detailed logging:
```bash
GET /api/debug/engine-state
GET /api/debug/rule/{rule_id}/trace
GET /api/debug/fact/{fact_id}/matches
```

## Security Considerations

### Authentication

The API supports multiple authentication methods:
- Bearer tokens
- API keys
- mTLS for service-to-service

### Input Validation

All inputs are validated:
- Field type checking
- Value range validation
- Rule complexity limits
- Fact size limits

### Rate Limiting

Configurable rate limits:
- Per-client request limits
- Processing complexity limits
- Memory usage quotas

## Configuration

### Environment Variables

Key configuration options:
```bash
BINGO_MAX_FACTS=1000000          # Maximum facts in memory
BINGO_RULE_CACHE_TTL=3600        # Rule cache TTL in seconds
BINGO_LOG_LEVEL=info             # Logging level
BINGO_WORKERS=4                  # Processing thread count
BINGO_CALCULATOR_TIMEOUT=30      # Calculator timeout in seconds
```

### Runtime Configuration

Update configuration via API:
```json
POST /api/config
{
  "cache_settings": {
    "ruleset_ttl_seconds": 7200,
    "fact_cache_size": 10000
  },
  "processing_limits": {
    "max_rules_per_fact": 1000,
    "max_actions_per_rule": 50
  }
}
```

## Best Practices

### Rule Design
1. Keep conditions simple and specific
2. Use indexed fields in conditions when possible
3. Avoid complex nested conditions unless necessary
4. Test rules with representative data

### Fact Management
1. Use meaningful external IDs for tracking
2. Include timestamps for time-based processing
3. Normalize field names across fact types
4. Batch fact submissions when possible

### Performance Optimization
1. Monitor cache hit rates
2. Use batch processing for high volumes
3. Consider fact retention policies
4. Profile rule execution times

### Deployment
1. Use health checks for load balancer configuration
2. Monitor memory usage and garbage collection
3. Configure appropriate timeouts
4. Use circuit breakers for external dependencies

## Migration and Compatibility

### API Versioning

The API uses semantic versioning:
- Breaking changes increment major version
- New features increment minor version
- Bug fixes increment patch version

### Backward Compatibility

Version compatibility guarantees:
- v1.x APIs remain stable within major version
- Deprecation notices provided 6 months before removal
- Migration tools provided for major version upgrades

### Data Migration

For engine upgrades:
1. Export existing facts and rules
2. Test with new engine version
3. Perform rolling deployment
4. Validate rule execution results

This documentation provides the foundation for effectively using the Bingo RETE Rules Engine API. For specific implementation details, refer to the OpenAPI specifications and example repositories.