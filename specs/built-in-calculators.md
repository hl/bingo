# Built-in Calculators Specification

## Overview
The Bingo RETE Rules Engine features a **high-performance plugin-based calculator system** with **predefined, compiled calculators** optimized for enterprise workloads. This architecture delivers exceptional performance (10-100x faster than interpreted formulas), type safety, and full auditability through version-controlled business logic.

The calculator system integrates seamlessly with the RETE algorithm, supporting complex business domains including compliance monitoring, payroll processing, and TRONC (tip distribution) calculations.

## 🚀 Calculator System Architecture

### Plugin Interface Design
- **Unified Interface**: All calculators implement the same `Calculator` trait for consistent behavior
- **Type Safety**: Compile-time input validation prevents runtime errors
- **Performance**: Compiled Rust code with optimized algorithms
- **Error Handling**: Structured error facts for graceful failure handling
- **Extensibility**: Plugin architecture allows custom calculator development

### Integration with RETE Engine
- **Lazy Evaluation**: Calculators are invoked only when rule conditions are met
- **Result Caching**: Calculator outputs are cached within session scope
- **Parallel Execution**: Independent calculations can run concurrently
- **Memory Efficiency**: Optimized memory usage with object pooling

## The `call_calculator` Action
To integrate a calculator into a rule, the `call_calculator` action type is used. This action facilitates the execution of a specified built-in calculator and requires the following properties:

-   `calculator_name`: The unique identifier for the built-in calculator to be executed.
-   `input_mapping`: A dictionary that defines the mapping between the calculator's required input parameters and fields from the current fact being processed.
-   `output_field`: The name of the field within the fact where the calculator's computed result will be stored.

### Example Usage
The following JSON snippet demonstrates how to invoke the `time_between_datetime` calculator. It maps `start_datetime` and `finish_datetime` from the incoming fact to the calculator's `start_field` and `end_field` inputs, respectively. The calculated duration is then stored in a new fact field named `hours_worked`.

```json
{
  "type": "call_calculator",
  "calculator_name": "time_between_datetime",
  "input_mapping": {
    "start_field": "start_datetime",
    "end_field": "finish_datetime"
  },
  "output_field": "hours_worked"
}
```

## Available Calculators
This section enumerates the built-in calculators provided by the engine, detailing their purpose, use cases, input parameters, and expected output.

### Mathematical Calculators

#### `proportional_allocator`
-   **Purpose**: Distributes a total amount proportionally based on an individual's value relative to a total value.
-   **Use Cases**: Allocating shared costs, distributing bonuses based on performance metrics.
-   **Inputs**:
    -   `total_amount` (number): The total amount to be allocated.
    -   `individual_value` (number): The value associated with the individual entity.
    -   `total_value` (number): The sum of all individual values.
-   **Output**: The proportionally allocated amount for the individual.

#### `weighted_average`
-   **Purpose**: Computes the weighted average of a set of items.
-   **Use Cases**: Calculating average costs with varying quantities, determining average scores with different weights.
-   **Inputs**:
    -   `items` (array of objects): Each object must contain:
        -   `value` (number): The value of the item.
        -   `weight` (number): The weight of the item.
-   **Output**: The weighted average of the items.

#### `multiply`
-   **Purpose**: Computes the product of two numerical inputs.
-   **Use Cases**: Calculation of gross pay, application of multipliers (e.g., tax rates), determination of total costs.
-   **Inputs**:
    -   `multiplicand` (number): The first number in the multiplication operation.
    -   `multiplier` (number): The second number to multiply by.
-   **Output**: The numerical product of `multiplicand` and `multiplier`.

#### `add`
-   **Purpose**: Computes the sum of two numerical inputs.
-   **Use Cases**: Summing up cost components, accumulating values.
-   **Inputs**:
    -   `addend1` (number): The first number in the addition.
    -   `addend2` (number): The second number in the addition.
-   **Output**: The numerical sum of `addend1` and `addend2`.

#### `percentage_deduct`
-   **Purpose**: Deducts a percentage from a total amount.
-   **Use Cases**: Applying administration fees, calculating discounts.
-   **Inputs**:
    -   `total_amount` (number): The base amount from which to deduct.
    -   `percentage` (number): The percentage to deduct, expressed as a decimal (e.g., 0.05 for 5%).
-   **Output**: The `total_amount` after the percentage deduction.

#### `percentage_add`
-   **Purpose**: Adds a percentage of a base amount to the base amount.
-   **Use Cases**: Calculating taxes, adding service charges.
-   **Inputs**:
    -   `base_amount` (number): The base amount.
    -   `percentage` (number): The percentage to add, expressed as a decimal (e.g., 0.075 for 7.5%).
-   **Output**: The `base_amount` after the percentage has been added.

### Time & Duration Calculators

#### `time_between_datetime`
-   **Purpose**: Calculates time differences between datetime values with optional workday-aware splitting capabilities.
-   **Use Cases**: 
    -   Simple time calculations: shift durations, elapsed time tracking
    -   Workday calculations: splitting time periods around workday boundaries, payroll calculations with work/non-work time separation
-   **Inputs**:
    -   `start_datetime` (string): The beginning datetime in RFC3339/ISO 8601 format
    -   `finish_datetime` (string): The ending datetime in RFC3339/ISO 8601 format
    -   `units` (string, optional): Output unit - `hours` (default), `minutes`, or `seconds`
    -   `workday` (object, optional): Workday boundary time as JSON object with `hours` (0-23) and `minutes` (0-59) fields
    -   `part` (string, optional): When using workday mode - `time_before` (start to workday boundary) or `time_after` (workday boundary to finish)
-   **Output**: A floating-point number representing the calculated time duration in the specified units
-   **Examples**:
    -   Simple: `{"start_datetime": "2025-01-01T08:00:00Z", "finish_datetime": "2025-01-01T17:00:00Z"}` → 9.0 hours
    -   Workday: `{"start_datetime": "2025-01-01T18:00:00Z", "finish_datetime": "2025-01-02T02:00:00Z", "workday": {"hours": 0, "minutes": 0}, "part": "time_before"}` → 6.0 hours

### Threshold & Validation Calculators

#### `threshold_check`
-   **Purpose**: A versatile calculator designed for validating a given value against a defined threshold using various comparison operators.
-   **Use Cases**: Implementing compliance checks (e.g., regulatory limits), validating budget adherence, assessing performance against targets.
-   **Inputs**:
    -   `value` (number): The numerical value to be evaluated.
    -   `threshold` (number): The numerical threshold for comparison.
    -   `operator` (string, optional): The comparison operator to apply. Supported values include `LessThan`, `LessThanOrEqual`, `GreaterThan`, `GreaterThanOrEqual`, `Equal`, or `NotEqual`. Defaults to `LessThanOrEqual` if not specified.
-   **Output**: A rich JSON object containing the compliance status, the magnitude of any violation, and the original input parameters. For a detailed example of the output structure, refer to the [Payroll Engine Guide](../docs/payroll-engine.md).

#### `limit_validate`
-   **Purpose**: A multi-tier validation calculator for checking a value against a series of escalating thresholds.
-   **Use Cases**: Monitoring resource utilization (e.g., data usage), tiered compliance checks, service level agreement (SLA) monitoring.
-   **Inputs**:
    -   `value` (number): The numerical value to be evaluated.
    -   `max_threshold` (number): The absolute maximum threshold. A value greater than or equal to this results in a `Breach`.
    -   `critical_threshold` (number, optional): The threshold for a `Critical` severity level.
    -   `warning_threshold` (number, optional): The threshold for a `Warning` severity level.
-   **Output**: A JSON object containing the `severity` (`Ok`, `Warning`, `Critical`, or `Breach`), a descriptive `status`, the original `value`, and the `utilization_percent` relative to the `max_threshold`.

## 🏆 Benefits of Plugin-based Calculator System

### Performance Benefits
1.  **Exceptional Speed**: Compiled Rust implementations deliver 10-100x faster execution than interpreted formula engines
2.  **Memory Efficiency**: Optimized memory usage with object pooling and arena allocation
3.  **Parallel Processing**: Calculators can execute concurrently for independent calculations
4.  **RETE Integration**: Seamless integration with O(Δfacts) complexity optimization

### Enterprise Features
5.  **Type Safety**: Compile-time input validation prevents runtime errors and ensures data integrity
6.  **Full Auditability**: Version-controlled business logic with complete change history
7.  **Production Hardening**: Zero-warning policy, comprehensive testing, and enterprise-grade error handling
8.  **Thread Safety**: Full Send + Sync implementation for concurrent processing

### Operational Advantages
9.  **Deployment Simplicity**: Embedded calculator logic eliminates separate rule distribution systems
10. **Extensibility**: Plugin architecture enables custom calculator development without core engine changes
11. **Consistency**: Uniform interface across all calculators ensures predictable behavior
12. **Business Domain Support**: Pre-built calculators for compliance, payroll, and TRONC workflows

## Error Handling
In the event of an error during calculator execution (e.g., a missing required input field), the engine generates a structured error fact. This mechanism enables the creation of rules specifically designed to gracefully handle such errors, for instance, by triggering notifications or applying alternative fallback logic.

### Example Error Fact
```json
{
  "entity_type": "calculator_error",
  "calculator_name": "threshold_check",
  "error_code": "MISSING_REQUIRED_FIELD",
  "error_message": "Required field 'value' is missing",
  "triggering_fact_id": "shift_001",
  "details": {
    "field": "value"
  }
}
```
