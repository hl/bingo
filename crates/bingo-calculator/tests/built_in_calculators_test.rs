use std::collections::HashMap;

use bingo_calculator::FactValue;
use bingo_calculator::built_in::add::AddCalculator;
use bingo_calculator::built_in::multiply::MultiplyCalculator;
use bingo_calculator::built_in::percentage_add::PercentageAddCalculator;
use bingo_calculator::built_in::percentage_deduct::PercentageDeductCalculator;
use bingo_calculator::built_in::proportional_allocator::ProportionalAllocatorCalculator;
use bingo_calculator::built_in::time_between_datetime::TimeBetweenDatetimeCalculator;
use bingo_calculator::plugin::CalculatorPlugin;

fn calculate_with<C: CalculatorPlugin>(calculator: C, inputs: &[(&str, FactValue)]) -> String {
    let var_refs: HashMap<String, &FactValue> =
        inputs.iter().map(|(k, v)| (k.to_string(), v)).collect();
    calculator.calculate(&var_refs).unwrap().as_string()
}

#[test]
fn multiply_calculator_works() {
    let result = calculate_with(
        MultiplyCalculator,
        &[("a", FactValue::Float(2.0)), ("b", FactValue::Float(3.5))],
    );
    assert_eq!(result, (2.0 * 3.5).to_string());
}

#[test]
fn add_calculator_works() {
    let result = calculate_with(
        AddCalculator,
        &[("a", FactValue::Float(10.0)), ("b", FactValue::Float(15.5))],
    );
    assert_eq!(result, 25.5.to_string());
}

#[test]
fn percentage_add_calculator_works() {
    let result = calculate_with(
        PercentageAddCalculator,
        &[("amount", FactValue::Float(100.0)), ("percentage", FactValue::Float(0.1))],
    );
    assert_eq!(result, 110.0.to_string());
}

#[test]
fn percentage_deduct_calculator_works() {
    let result = calculate_with(
        PercentageDeductCalculator,
        &[("amount", FactValue::Float(200.0)), ("percentage", FactValue::Float(0.25))],
    );
    assert_eq!(result, 150.0.to_string());
}

#[test]
fn proportional_allocator_works() {
    let result = calculate_with(
        ProportionalAllocatorCalculator,
        &[
            ("total_amount", FactValue::Float(1000.0)),
            ("individual_value", FactValue::Float(10.0)),
            ("total_value", FactValue::Float(100.0)),
        ],
    );
    assert_eq!(result, 100.0.to_string());
}

#[test]
fn time_between_datetime_works() {
    let result = calculate_with(
        TimeBetweenDatetimeCalculator,
        &[
            (
                "start_datetime",
                FactValue::String("2024-01-01T00:00:00Z".to_string()),
            ),
            (
                "finish_datetime",
                FactValue::String("2024-01-02T00:00:00Z".to_string()),
            ),
            ("units", FactValue::String("hours".to_string())),
        ],
    );
    assert_eq!(result, 24.0.to_string());
}

#[test]
fn time_between_datetime_workday_midnight_time_before() {
    // Example: workday at 00:00, start at 18:00, finish at 02:00 next day
    // time_before should return 6 hours (18:00 to 00:00)
    let result = calculate_with(
        TimeBetweenDatetimeCalculator,
        &[
            (
                "start_datetime",
                FactValue::String("2025-01-01T18:00:00Z".to_string()),
            ),
            (
                "finish_datetime",
                FactValue::String("2025-01-02T02:00:00Z".to_string()),
            ),
            (
                "workday",
                FactValue::String(r#"{"hours": 0, "minutes": 0}"#.to_string()),
            ),
            ("part", FactValue::String("time_before".to_string())),
            ("units", FactValue::String("hours".to_string())),
        ],
    );
    assert_eq!(result, 6.0.to_string());
}

#[test]
fn time_between_datetime_workday_midnight_time_after() {
    // Example: workday at 00:00, start at 18:00, finish at 02:00 next day
    // time_after should return 2 hours (00:00 to 02:00)
    let result = calculate_with(
        TimeBetweenDatetimeCalculator,
        &[
            (
                "start_datetime",
                FactValue::String("2025-01-01T18:00:00Z".to_string()),
            ),
            (
                "finish_datetime",
                FactValue::String("2025-01-02T02:00:00Z".to_string()),
            ),
            (
                "workday",
                FactValue::String(r#"{"hours": 0, "minutes": 0}"#.to_string()),
            ),
            ("part", FactValue::String("time_after".to_string())),
            ("units", FactValue::String("hours".to_string())),
        ],
    );
    assert_eq!(result, 2.0.to_string());
}

#[test]
fn time_between_datetime_workday_2230_time_before() {
    // Example: workday at 22:30, start at 18:00, finish at 02:00 next day
    // time_before should return 4.5 hours (18:00 to 22:30)
    let result = calculate_with(
        TimeBetweenDatetimeCalculator,
        &[
            (
                "start_datetime",
                FactValue::String("2025-01-01T18:00:00Z".to_string()),
            ),
            (
                "finish_datetime",
                FactValue::String("2025-01-02T02:00:00Z".to_string()),
            ),
            (
                "workday",
                FactValue::String(r#"{"hours": 22, "minutes": 30}"#.to_string()),
            ),
            ("part", FactValue::String("time_before".to_string())),
            ("units", FactValue::String("hours".to_string())),
        ],
    );
    assert_eq!(result, 4.5.to_string());
}

#[test]
fn time_between_datetime_workday_2230_time_after() {
    // Example: workday at 22:30, start at 18:00, finish at 02:00 next day
    // time_after should return 3.5 hours (22:30 to 02:00)
    let result = calculate_with(
        TimeBetweenDatetimeCalculator,
        &[
            (
                "start_datetime",
                FactValue::String("2025-01-01T18:00:00Z".to_string()),
            ),
            (
                "finish_datetime",
                FactValue::String("2025-01-02T02:00:00Z".to_string()),
            ),
            (
                "workday",
                FactValue::String(r#"{"hours": 22, "minutes": 30}"#.to_string()),
            ),
            ("part", FactValue::String("time_after".to_string())),
            ("units", FactValue::String("hours".to_string())),
        ],
    );
    assert_eq!(result, 3.5.to_string());
}

#[test]
fn time_between_datetime_minutes_output() {
    // Test minutes output unit
    let result = calculate_with(
        TimeBetweenDatetimeCalculator,
        &[
            (
                "start_datetime",
                FactValue::String("2025-01-01T18:00:00Z".to_string()),
            ),
            (
                "finish_datetime",
                FactValue::String("2025-01-01T20:30:00Z".to_string()),
            ),
            ("units", FactValue::String("minutes".to_string())),
        ],
    );
    assert_eq!(result, 150.0.to_string()); // 2.5 hours = 150 minutes
}

#[test]
fn time_between_datetime_workday_same_day() {
    // Test workday calculation within the same day
    let result = calculate_with(
        TimeBetweenDatetimeCalculator,
        &[
            (
                "start_datetime",
                FactValue::String("2025-01-01T08:00:00Z".to_string()),
            ),
            (
                "finish_datetime",
                FactValue::String("2025-01-01T17:00:00Z".to_string()),
            ),
            (
                "workday",
                FactValue::String(r#"{"hours": 12, "minutes": 0}"#.to_string()),
            ),
            ("part", FactValue::String("time_before".to_string())),
            ("units", FactValue::String("hours".to_string())),
        ],
    );
    assert_eq!(result, 4.0.to_string()); // 08:00 to 12:00 = 4 hours
}

#[test]
fn weighted_average_calculator_works() {
    use bingo_calculator::built_in::weighted_average::WeightedAverageCalculator;
    let items = vec![
        FactValue::Object(
            [
                ("value".to_string(), FactValue::Float(5.0)),
                ("weight".to_string(), FactValue::Float(1.0)),
            ]
            .iter()
            .cloned()
            .collect(),
        ),
        FactValue::Object(
            [
                ("value".to_string(), FactValue::Float(15.0)),
                ("weight".to_string(), FactValue::Float(3.0)),
            ]
            .iter()
            .cloned()
            .collect(),
        ),
    ];
    let result = calculate_with(
        WeightedAverageCalculator::new(),
        &[("items", FactValue::Array(items))],
    );
    // (5*1 + 15*3) / 4 = 12.5
    assert_eq!(result, 12.5.to_string());
}

#[test]
fn threshold_check_calculator_works() {
    use bingo_calculator::ThresholdCheckCalculator;
    let result_true = calculate_with(
        ThresholdCheckCalculator,
        &[("value", FactValue::Float(10.0)), ("threshold", FactValue::Float(5.0))],
    );
    assert_eq!(result_true, "true");

    let result_false = calculate_with(
        ThresholdCheckCalculator,
        &[("value", FactValue::Float(3.0)), ("threshold", FactValue::Float(5.0))],
    );
    assert_eq!(result_false, "false");
}

#[test]
fn limit_validate_calculator_works() {
    use bingo_calculator::LimitValidateCalculator;
    let result_ok = calculate_with(
        LimitValidateCalculator,
        &[
            ("value", FactValue::Float(50.0)),
            ("min", FactValue::Float(0.0)),
            ("max", FactValue::Float(100.0)),
        ],
    );
    assert_eq!(result_ok, "true");

    let result_fail = calculate_with(
        LimitValidateCalculator,
        &[
            ("value", FactValue::Float(150.0)),
            ("min", FactValue::Float(0.0)),
            ("max", FactValue::Float(100.0)),
        ],
    );
    assert_eq!(result_fail, "false");
}
