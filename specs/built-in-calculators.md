# Built-in Calculators Specification

## Overview
The Bingo Rules Engine utilizes a system of **predefined, compiled calculators** to execute all calculations. This architectural choice prioritizes maximum performance, type safety, and auditability. Unlike systems that interpret user-defined formulas at runtime, Bingo invokes these highly optimized, built-in calculators directly.

This specification details the available built-in calculators and their integration within the rule engine.

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
-   **Purpose**: Calculates the duration in hours between two specified datetime fields.
-   **Use Cases**: Determining shift durations, tracking time spent on tasks, calculating elapsed time for processes.
-   **Inputs**:
    -   `start_datetime` (string): The beginning datetime, expected in ISO 8601 format.
    -   `end_datetime` (string): The ending datetime, expected in ISO 8601 format.
    -   `unit` (string, optional): The unit for the duration. Supported values: `hours`, `minutes`, `seconds`. Defaults to `hours`.
-   **Output**: A floating-point number representing the total hours between the `start_datetime` and `end_datetime`.

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

## Benefits of Built-in Calculators
The adoption of built-in calculators offers several significant advantages:

1.  **High Performance**: Implemented as compiled Rust code, these calculators deliver substantially faster execution (typically 10-100x) compared to interpreted formula engines.
2.  **Type Safety**: Input validation occurs at compile time, effectively preventing a broad spectrum of runtime errors and ensuring data integrity.
3.  **Reusability**: Generic calculator implementations can be seamlessly applied across diverse business domains, including compliance, payroll, and cost management, promoting code reuse and consistency.
4.  **Auditability**: As part of the version-controlled codebase, all changes to calculator logic are fully auditable, providing a clear historical record of business rule evolution.
5.  **Operational Simplicity**: The calculator logic is embedded within the standard application deployment, eliminating the need for separate rule distribution or complex management systems.

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
