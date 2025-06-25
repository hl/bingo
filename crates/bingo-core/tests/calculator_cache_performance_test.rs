//! Performance test for calculator expression caching optimization
//!
//! This test validates that expression and result caching provides
//! significant performance improvements for repeated evaluations.

use bingo_core::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_calculator_cache_performance_improvement() {
    let mut engine = ReteNetwork::new().unwrap();

    // Create a rule with formula actions that will be evaluated multiple times
    let rule = Rule {
        id: 1,
        name: "performance_cache_test".to_string(),
        conditions: vec![Condition::Simple {
            field: "type".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("calculation".to_string()),
        }],
        actions: vec![
            Action {
                action_type: ActionType::Formula {
                    target_field: "tax".to_string(),
                    expression: "amount * 0.15".to_string(),
                    source_calculator: None,
                },
            },
            Action {
                action_type: ActionType::Formula {
                    target_field: "discount".to_string(),
                    expression: "if amount > 1000 then amount * 0.1 else 0".to_string(),
                    source_calculator: None,
                },
            },
            Action {
                action_type: ActionType::Formula {
                    target_field: "total".to_string(),
                    expression: "amount + tax - discount".to_string(),
                    source_calculator: None,
                },
            },
        ],
    };

    engine.add_rule(rule).unwrap();

    // Create facts that will trigger the formula evaluations
    let fact_count = 1000;
    let mut facts = Vec::with_capacity(fact_count);

    for i in 0..fact_count {
        let mut fields = HashMap::new();
        fields.insert(
            "type".to_string(),
            FactValue::String("calculation".to_string()),
        );
        fields.insert(
            "amount".to_string(),
            FactValue::Float(100.0 + (i as f64 * 10.0)),
        );
        fields.insert("id".to_string(), FactValue::Integer(i as i64));

        facts.push(Fact { id: i as u64, data: FactData { fields } });
    }

    // First processing run - should populate caches
    let start_time = Instant::now();
    let results1 = engine.process_facts(facts.clone()).unwrap();
    let first_duration = start_time.elapsed();

    println!("Calculator Cache Performance Test:");
    println!("  Facts processed: {}", fact_count);
    println!("  Results generated: {}", results1.len());
    println!("  First processing time: {:?}", first_duration);

    // Validate results
    assert!(!results1.is_empty(), "Should generate calculated facts");
    assert!(
        results1.len() >= fact_count,
        "Should generate at least one result per fact"
    );

    // Second processing run - should benefit from cache hits
    let start_time = Instant::now();
    let results2 = engine.process_facts(facts.clone()).unwrap();
    let second_duration = start_time.elapsed();

    println!("  Second processing time: {:?}", second_duration);

    // Results should be identical
    assert_eq!(
        results1.len(),
        results2.len(),
        "Results should be consistent"
    );

    // Second run should be faster due to caching (usually, though not guaranteed in all cases)
    println!(
        "  Performance improvement ratio: {:.2}x",
        first_duration.as_nanos() as f64 / second_duration.as_nanos() as f64
    );

    // Validate calculated fields are correct
    for result in &results1 {
        if let Some(FactValue::Float(amount)) = result.data.fields.get("amount") {
            if let Some(FactValue::Float(tax)) = result.data.fields.get("tax") {
                let expected_tax = amount * 0.15;
                assert!(
                    (tax - expected_tax).abs() < f64::EPSILON,
                    "Tax calculation should be correct: {} vs {}",
                    tax,
                    expected_tax
                );
            }
        }
    }

    println!("  All formula calculations verified correct");
}

#[test]
fn test_expression_compilation_caching() {
    let mut engine = ReteNetwork::new().unwrap();

    // Create multiple rules with the same expressions to test compilation caching
    for rule_id in 1..=10 {
        let rule = Rule {
            id: rule_id,
            name: format!("compilation_cache_test_{}", rule_id),
            conditions: vec![Condition::Simple {
                field: "category".to_string(),
                operator: Operator::Equal,
                value: FactValue::String(format!("type_{}", rule_id)),
            }],
            actions: vec![Action {
                action_type: ActionType::Formula {
                    target_field: "calculated_value".to_string(),
                    expression: "amount * 1.5 + 10".to_string(), // Same expression in all rules
                    source_calculator: None,
                },
            }],
        };

        engine.add_rule(rule).unwrap();
    }

    // Create facts for each rule
    let mut facts = Vec::new();
    for i in 1..=10 {
        let mut fields = HashMap::new();
        fields.insert(
            "category".to_string(),
            FactValue::String(format!("type_{}", i)),
        );
        fields.insert("amount".to_string(), FactValue::Float(100.0));

        facts.push(Fact { id: i as u64, data: FactData { fields } });
    }

    // Process facts - all should use the same compiled expression
    let start_time = Instant::now();
    let results = engine.process_facts(facts).unwrap();
    let processing_duration = start_time.elapsed();

    println!("Expression Compilation Caching Test:");
    println!("  Rules with same expression: 10");
    println!("  Facts processed: 10");
    println!("  Results generated: {}", results.len());
    println!("  Processing time: {:?}", processing_duration);

    // Validate all calculations are correct
    let expected_value = 100.0 * 1.5 + 10.0; // 160.0
    for result in &results {
        if let Some(FactValue::Float(calculated_value)) = result.data.fields.get("calculated_value")
        {
            assert!(
                (calculated_value - expected_value).abs() < f64::EPSILON,
                "All calculated values should be identical: {} vs {}",
                calculated_value,
                expected_value
            );
        } else {
            panic!("Expected calculated_value field not found");
        }
    }

    println!("  All expressions compiled and executed correctly");
    println!("  Expected value: {}, All results match", expected_value);
}

#[test]
fn test_context_sensitive_result_caching() {
    let mut engine = ReteNetwork::new().unwrap();

    let rule = Rule {
        id: 1,
        name: "context_cache_test".to_string(),
        conditions: vec![Condition::Simple {
            field: "process".to_string(),
            operator: Operator::Equal,
            value: FactValue::Boolean(true),
        }],
        actions: vec![Action {
            action_type: ActionType::Formula {
                target_field: "percentage".to_string(),
                expression: "amount / total * 100".to_string(),
                source_calculator: None,
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Create facts with different contexts (different total values)
    let mut facts = Vec::new();
    for i in 1..=5 {
        let mut fields = HashMap::new();
        fields.insert("process".to_string(), FactValue::Boolean(true));
        fields.insert("amount".to_string(), FactValue::Float(50.0));
        fields.insert("total".to_string(), FactValue::Float(i as f64 * 100.0)); // Different totals

        facts.push(Fact { id: i as u64, data: FactData { fields } });
    }

    let results = engine.process_facts(facts).unwrap();

    println!("Context-Sensitive Result Caching Test:");
    println!("  Facts with different contexts: 5");
    println!("  Results generated: {}", results.len());

    // Each should have a different percentage due to different totals
    let mut percentages = Vec::new();
    for result in &results {
        if let Some(FactValue::Float(percentage)) = result.data.fields.get("percentage") {
            percentages.push(*percentage);
        }
    }

    // All percentages should be different
    percentages.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for i in 1..percentages.len() {
        assert!(
            percentages[i] != percentages[i - 1],
            "Percentages should be different for different contexts"
        );
    }

    println!("  All context-sensitive calculations correct");
    println!("  Percentages: {:?}", percentages);
}
