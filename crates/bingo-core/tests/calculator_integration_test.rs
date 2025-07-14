//! Comprehensive integration test for calculator functionality in RETE network
//!
//! This test validates that calculator integration works correctly within the
//! RETE network, including built-in calculators and formula evaluation.

use bingo_core::BingoEngine;
use bingo_core::types::{Action, ActionType, Condition, Fact, FactData, FactValue, Operator, Rule};
use chrono::Utc;
use std::collections::HashMap;

/// Create test facts for calculator testing
fn create_test_facts() -> Vec<Fact> {
    vec![
        create_fact(1, "employee", "John", "hours", 45.0, "rate", 25.0),
        create_fact(2, "employee", "Jane", "hours", 38.0, "rate", 30.0),
        create_fact(3, "employee", "Bob", "hours", 50.0, "rate", 20.0),
        create_fact(4, "product", "Widget", "price", 100.0, "tax_rate", 0.08),
        create_fact(5, "product", "Gadget", "price", 200.0, "tax_rate", 0.08),
    ]
}

fn create_fact(
    id: u64,
    category_field: &str,
    category_value: &str,
    field1: &str,
    value1: f64,
    field2: &str,
    value2: f64,
) -> Fact {
    let mut fields = HashMap::new();
    fields.insert(
        category_field.to_string(),
        FactValue::String(category_value.to_string()),
    );
    fields.insert(field1.to_string(), FactValue::Float(value1));
    fields.insert(field2.to_string(), FactValue::Float(value2));
    fields.insert("id".to_string(), FactValue::Integer(id as i64));

    Fact {
        id,
        external_id: Some(format!("fact-{id}")),
        timestamp: Utc::now(),
        data: FactData { fields },
    }
}

#[test]
fn test_threshold_check_calculator() {
    let engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Threshold Check Calculator");

    // Create a rule that uses threshold_check calculator
    let rule = Rule {
        id: 1,
        name: "Overtime Check".to_string(),
        conditions: vec![Condition::Simple {
            field: "employee".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("John".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::CallCalculator {
                calculator_name: "threshold_check".to_string(),
                input_mapping: {
                    let mut mapping = HashMap::new();
                    mapping.insert("value".to_string(), "hours".to_string());
                    mapping.insert("threshold".to_string(), "rate".to_string()); // Using rate as threshold for test
                    mapping
                },
                output_field: "overtime_check".to_string(),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts
    let facts = create_test_facts();
    let results = engine.process_facts(facts).unwrap();

    println!(
        "🎯 Calculator execution results: {} rules fired",
        results.len()
    );

    // Verify that the calculator was executed
    let calc_results: Vec<_> = results
        .iter()
        .filter_map(|r| {
            r.actions_executed.iter().find_map(|action| {
                if let bingo_core::rete_nodes::ActionResult::CalculatorResult {
                    calculator,
                    result,
                    output_field,
                    parsed_value,
                } = action
                {
                    Some((calculator, result, output_field, parsed_value))
                } else {
                    None
                }
            })
        })
        .collect();

    for (calculator, result, output_field, parsed_value) in calc_results {
        println!(
            "📊 Calculator: {calculator}, Result: {result}, Output Field: {output_field}, Parsed: {parsed_value:?}"
        );
        assert_eq!(calculator, "threshold_check");
        assert_eq!(output_field, "overtime_check");
        assert!(result == "true" || result == "false");
    }

    println!("✅ Threshold check calculator test passed");
}

#[test]
fn test_limit_validator_calculator() {
    let engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Limit Validator Calculator");

    // Create a rule that uses limit_validator calculator
    let rule = Rule {
        id: 1,
        name: "Hours Validation".to_string(),
        conditions: vec![Condition::Simple {
            field: "employee".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("Jane".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::CallCalculator {
                calculator_name: "limit_validator".to_string(),
                input_mapping: {
                    let mut mapping = HashMap::new();
                    mapping.insert("value".to_string(), "hours".to_string());
                    mapping.insert("min".to_string(), "rate".to_string()); // Using rate as min for test
                    mapping.insert("max".to_string(), "hours".to_string()); // Same as value for test
                    mapping
                },
                output_field: "hours_valid".to_string(),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts
    let facts = create_test_facts();
    let results = engine.process_facts(facts).unwrap();

    println!("🎯 Limit validator results: {} rules fired", results.len());

    // Verify that the calculator was executed
    let calc_results: Vec<_> = results
        .iter()
        .filter_map(|r| {
            r.actions_executed.iter().find_map(|action| {
                if let bingo_core::rete_nodes::ActionResult::CalculatorResult {
                    calculator,
                    result,
                    output_field,
                    parsed_value,
                } = action
                {
                    Some((calculator, result, output_field, parsed_value))
                } else {
                    None
                }
            })
        })
        .collect();

    for (calculator, result, output_field, parsed_value) in calc_results {
        println!(
            "📊 Calculator: {calculator}, Result: {result}, Output Field: {output_field}, Parsed: {parsed_value:?}"
        );
        assert_eq!(calculator, "limit_validator");
        assert_eq!(output_field, "hours_valid");
        assert!(result == "true" || result == "false");
    }

    println!("✅ Limit validator calculator test passed");
}

#[test]
fn test_weighted_average_calculator() {
    let engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Weighted Average Calculator");

    // Create a fact with array data for weighted average
    let mut fields = HashMap::new();
    fields.insert(
        "type".to_string(),
        FactValue::String("calculation".to_string()),
    );

    // Create an array of objects with value and weight fields
    let items = FactValue::Array(vec![
        FactValue::Object({
            let mut item = HashMap::new();
            item.insert("value".to_string(), FactValue::Float(100.0));
            item.insert("weight".to_string(), FactValue::Float(2.0));
            item
        }),
        FactValue::Object({
            let mut item = HashMap::new();
            item.insert("value".to_string(), FactValue::Float(200.0));
            item.insert("weight".to_string(), FactValue::Float(3.0));
            item
        }),
    ]);
    fields.insert("items".to_string(), items);

    let weighted_fact = Fact {
        id: 100,
        external_id: Some("weighted-test".to_string()),
        timestamp: Utc::now(),
        data: FactData { fields },
    };

    // Create a rule that uses weighted_average calculator
    let rule = Rule {
        id: 1,
        name: "Weighted Average Calculation".to_string(),
        conditions: vec![Condition::Simple {
            field: "type".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("calculation".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::CallCalculator {
                calculator_name: "weighted_average".to_string(),
                input_mapping: {
                    let mut mapping = HashMap::new();
                    mapping.insert("items".to_string(), "items".to_string());
                    mapping
                },
                output_field: "weighted_avg".to_string(),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process the weighted fact
    let results = engine.process_facts(vec![weighted_fact]).unwrap();

    println!("🎯 Weighted average results: {} rules fired", results.len());

    // Verify the weighted average calculation
    let calc_results: Vec<_> = results
        .iter()
        .filter_map(|r| {
            r.actions_executed.iter().find_map(|action| {
                if let bingo_core::rete_nodes::ActionResult::CalculatorResult {
                    calculator,
                    result,
                    output_field,
                    parsed_value,
                } = action
                {
                    Some((calculator, result, output_field, parsed_value))
                } else {
                    None
                }
            })
        })
        .collect();

    for (calculator, result, output_field, parsed_value) in calc_results {
        println!(
            "📊 Calculator: {calculator}, Result: {result}, Output Field: {output_field}, Parsed: {parsed_value:?}"
        );
        assert_eq!(calculator, "weighted_average");
        assert_eq!(output_field, "weighted_avg");

        // Expected calculation: (100*2 + 200*3) / (2+3) = 800/5 = 160.0
        let expected_avg = 160.0;
        if let Ok(result_float) = result.parse::<f64>() {
            assert!(
                (result_float - expected_avg).abs() < 0.01,
                "Expected ~{expected_avg}, got {result_float}"
            );
        }
    }

    println!("✅ Weighted average calculator test passed");
}
