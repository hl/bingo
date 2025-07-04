# JSON-Based Integration Test Framework

This directory contains a comprehensive JSON-based testing framework for validating the Bingo RETE Rules Engine API. Test cases are defined in JSON files with inputs and expected outputs.

## Directory Structure

```
tests/
├── json-tests/
│   ├── basic/           # Basic functionality tests
│   ├── streaming/       # Streaming API tests  
│   ├── caching/         # Engine cache tests
│   ├── calculator/      # Calculator DSL tests
│   ├── multi-rule/      # Multi-rule processing tests
│   ├── performance/     # Performance validation tests
│   └── edge-cases/      # Edge case and error handling tests
├── json_test_runner.rs  # Test framework implementation
└── README.md           # This file
```

## JSON Test Format

Each test case consists of three files:
- `{test_name}_input.json` - Input data (facts, rules, configuration)
- `{test_name}_expected.json` - Expected output structure
- `{test_name}_metadata.json` - Test metadata and validation rules

### Input File Format (`*_input.json`)

```json
{
  "facts": [
    {
      "id": "fact-1",
      "data": {
        "employee_id": 12345,
        "hours_worked": 42.5,
        "status": "active"
      },
      "created_at": "2024-01-01T00:00:00Z"
    }
  ],
  "rules": [
    {
      "id": "rule-1",
      "name": "Overtime Detection",
      "description": "Detect overtime hours",
      "conditions": [
        {
          "field": "hours_worked",
          "operator": "greater_than",
          "value": 40.0
        }
      ],
      "actions": [
        {
          "type": "set_field",
          "field": "overtime",
          "value": true
        }
      ],
      "priority": 100,
      "enabled": true,
      "tags": ["payroll", "overtime"],
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  ],
  "response_format": "json",
  "streaming_config": {
    "result_threshold": 100,
    "chunk_size": 50,
    "include_progress": true
  }
}
```

### Expected Output Format (`*_expected.json`)

```json
{
  "validation_rules": {
    "status_code": 200,
    "content_type": "application/json",
    "headers": {
      "x-request-id": "exists",
      "x-processing-time": "numeric"
    },
    "response_structure": {
      "request_id": "string",
      "results": "array",
      "rules_processed": "number",
      "facts_processed": "number", 
      "rules_fired": "number",
      "processing_time_ms": "number",
      "stats": "object"
    }
  },
  "exact_matches": {
    "facts_processed": 1,
    "rules_processed": 1,
    "rules_fired": 1
  },
  "range_validations": {
    "processing_time_ms": {"min": 0, "max": 1000}
  },
  "array_validations": {
    "results": {
      "min_length": 1,
      "max_length": 1,
      "item_structure": {
        "id": "string",
        "data": "object"
      }
    }
  }
}
```

### Metadata Format (`*_metadata.json`)

```json
{
  "test_name": "Basic Overtime Detection",
  "description": "Tests basic rule firing for overtime detection",
  "category": "basic",
  "tags": ["payroll", "rules", "basic"],
  "timeout_ms": 5000,
  "expected_duration_ms": 100,
  "skip_reason": null,
  "prerequisites": [],
  "author": "test-framework",
  "created_at": "2024-01-01T00:00:00Z"
}
```

## Running Tests

### Run All JSON Tests
```bash
cargo test json_test_framework
```

### Run Specific Category
```bash
cargo test json_test_framework -- --test-category=basic
cargo test json_test_framework -- --test-category=streaming
```

### Run Specific Test
```bash
cargo test json_test_framework -- --test-name=overtime_detection
```

### Validation Options
```bash
# Skip performance validations for faster testing
cargo test json_test_framework -- --skip-performance

# Verbose output with detailed comparisons
cargo test json_test_framework -- --verbose

# Generate coverage report
cargo test json_test_framework -- --coverage
```

## Test Features

### Supported Validation Types

1. **Exact Value Matching**: Validates specific field values
2. **Type Validation**: Ensures fields have correct types
3. **Range Validation**: Validates numeric ranges (time, counts, etc.)
4. **Array Validation**: Validates array structure and contents
5. **Header Validation**: Validates HTTP headers and metadata
6. **Performance Validation**: Validates response times and throughput

### Supported API Features

- ✅ Standard JSON responses
- ✅ NDJSON streaming responses  
- ✅ Engine caching with ETag headers
- ✅ Calculator DSL expressions
- ✅ Multi-rule processing
- ✅ Error handling and validation
- ✅ Performance metrics
- ✅ Operational hardening (rate limiting, etc.)

### Error Handling Tests

The framework includes comprehensive error handling validation:
- Invalid input formats
- Malformed JSON
- Security violations
- Resource limits exceeded
- Network failures
- Cache misses

## Writing New Tests

### 1. Create Test Files

Create three files in the appropriate category directory:
```bash
tests/json-tests/basic/my_test_input.json
tests/json-tests/basic/my_test_expected.json  
tests/json-tests/basic/my_test_metadata.json
```

### 2. Define Input Data

Focus on the specific functionality you want to test. Include realistic facts and rules that exercise the feature.

### 3. Define Expected Output

Use validation rules rather than hardcoded values where possible. This makes tests more resilient to minor implementation changes.

### 4. Add Metadata

Include clear descriptions and appropriate categorization.

### 5. Run and Validate

```bash
cargo test json_test_framework -- --test-name=my_test
```

## Best Practices

1. **Use Realistic Data**: Test with data that represents actual use cases
2. **Test Edge Cases**: Include boundary conditions and error scenarios
3. **Performance Aware**: Include timing validations for critical paths
4. **Maintainable**: Use validation rules rather than exact matches where possible
5. **Well Documented**: Include clear descriptions and test purposes
6. **Isolated**: Each test should be independent and repeatable

## Integration with CI/CD

The JSON test framework is designed for CI/CD integration:
- Fast execution (<30s for all tests)
- Clear pass/fail reporting
- Detailed failure diagnostics
- Coverage reporting
- Performance regression detection