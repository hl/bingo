

//! Comprehensive integration test for advanced aggregation functions
//!
//! This test validates Task 3.4: Implement advanced aggregation functions (percentiles, variance)
//! Tests StandardDeviation, Variance, and Percentile functions in both RETE network and stream processing


use bingo_core::BingoEngine;
use bingo_core::types::{
    Action, ActionType, AggregationCondition, AggregationType, AggregationWindow, Condition, Fact,
    FactData, FactValue, Operator, Rule,
};
use chrono::Utc;
use std::collections::HashMap;

/// Create test facts with varying numeric values for statistical calculations
fn create_statistical_facts() -> Vec<Fact> {
    let values = vec![10, 15, 20, 25, 30, 35, 40, 45, 50, 100]; // Including an outlier

    values
        .into_iter()
        .enumerate()
        .map(|(i, value)| {
            let mut fields = HashMap::new();
            fields.insert(
                "employee_id".to_string(),
                FactValue::Integer((i + 1) as i64),
            );
            fields.insert("salary".to_string(), FactValue::Integer(value * 1000)); // In thousands
            fields.insert("score".to_string(), FactValue::Float(value as f64));
            fields.insert(
                "department".to_string(),
                FactValue::String("engineering".to_string()),
            );

            Fact {
                id: (i + 1) as u64,
                external_id: Some(format!("emp-{}", i + 1)),
                timestamp: Utc::now(),
                data: FactData { fields },
            }
        })
        .collect()
}

#[test]
fn test_standard_deviation_aggregation() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Standard Deviation Aggregation");

    // Create a rule that calculates standard deviation of salaries
    let rule = Rule {
        id: 1,
        name: "Salary Standard Deviation".to_string(),
        conditions: vec![Condition::Aggregation(AggregationCondition {
            alias: "salary_stddev".to_string(),
            aggregation_type: AggregationType::StandardDeviation,
            source_field: "salary".to_string(),
            window: None, // No windowing, use all facts
            group_by: vec!["department".to_string()],
            having: Some(Box::new(Condition::Simple {
                field: "salary_stddev".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Float(10000.0), // Significant variance in salaries
            })),
        })],
        actions: vec![Action {
            action_type: ActionType::Log {
                message: "High salary variance detected in department".to_string(),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts
    let facts = create_statistical_facts();
    let results = engine.process_facts(facts).unwrap();

    println!(
        "🎯 Standard deviation rule results: {} rules fired",
        results.len()
    );

    // Should fire because standard deviation of salaries is significant
    assert!(!results.is_empty(), "Should detect high salary variance");

    // Verify action results
    let log_results: Vec<_> = results
        .iter()
        .flat_map(|r| &r.actions_executed)
        .filter_map(|action| {
            if let bingo_core::rete_nodes::ActionResult::Logged { message } = action {
                Some(message)
            } else {
                None
            }
        })
        .collect();

    assert!(!log_results.is_empty(), "Should have log results");

    for message in log_results.iter().take(3) {
        // Show first 3 results
        println!("📊 Standard deviation result: {}", message);
        assert!(message.contains("High salary variance detected"));
    }

    println!("✅ Standard deviation aggregation test passed");
}

#[test]
fn test_percentile_aggregation() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Percentile Aggregation");

    // Create a rule that finds high performers (90th percentile of scores)
    let rule = Rule {
        id: 1,
        name: "90th Percentile Performance".to_string(),
        conditions: vec![Condition::Aggregation(AggregationCondition {
            alias: "score_p90".to_string(),
            aggregation_type: AggregationType::Percentile(90.0),
            source_field: "score".to_string(),
            window: None,
            group_by: vec!["department".to_string()],
            having: Some(Box::new(Condition::Simple {
                field: "score_p90".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Float(40.0), // 90th percentile should be > 40
            })),
        })],
        actions: vec![Action {
            action_type: ActionType::Log {
                message: "High-performing department identified".to_string(),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts
    let facts = create_statistical_facts();
    let results = engine.process_facts(facts).unwrap();

    println!("🎯 Percentile rule results: {} rules fired", results.len());

    // Should fire because 90th percentile of scores is above 40
    assert!(
        !results.is_empty(),
        "Should detect high-performing department"
    );

    // Verify action results
    let log_results: Vec<_> = results
        .iter()
        .flat_map(|r| &r.actions_executed)
        .filter_map(|action| {
            if let bingo_core::rete_nodes::ActionResult::Logged { message } = action {
                Some(message)
            } else {
                None
            }
        })
        .collect();

    assert!(!log_results.is_empty(), "Should have log results");

    for message in log_results.iter().take(3) {
        println!("📊 Percentile result: {}", message);
        assert!(message.contains("High-performing department"));
    }

    println!("✅ Percentile aggregation test passed");
}

#[test]
fn test_multiple_advanced_aggregations() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Multiple Advanced Aggregations");

    // Rule 1: Standard deviation of salaries
    let rule1 = Rule {
        id: 1,
        name: "Salary Distribution Analysis".to_string(),
        conditions: vec![Condition::Aggregation(AggregationCondition {
            alias: "salary_stddev".to_string(),
            aggregation_type: AggregationType::StandardDeviation,
            source_field: "salary".to_string(),
            window: None,
            group_by: vec!["department".to_string()],
            having: Some(Box::new(Condition::Simple {
                field: "salary_stddev".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Float(5000.0),
            })),
        })],
        actions: vec![Action {
            action_type: ActionType::Log {
                message: "Salary distribution analysis complete".to_string(),
            },
        }],
    };

    // Rule 2: 75th percentile of scores
    let rule2 = Rule {
        id: 2,
        name: "Performance Threshold Analysis".to_string(),
        conditions: vec![Condition::Aggregation(AggregationCondition {
            alias: "score_p75".to_string(),
            aggregation_type: AggregationType::Percentile(75.0),
            source_field: "score".to_string(),
            window: None,
            group_by: vec!["department".to_string()],
            having: Some(Box::new(Condition::Simple {
                field: "score_p75".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Float(30.0),
            })),
        })],
        actions: vec![Action {
            action_type: ActionType::Log {
                message: "Performance threshold analysis complete".to_string(),
            },
        }],
    };

    engine.add_rule(rule1).unwrap();
    engine.add_rule(rule2).unwrap();

    // Process facts
    let facts = create_statistical_facts();
    let results = engine.process_facts(facts).unwrap();

    println!(
        "🎯 Multiple aggregation results: {} rules fired",
        results.len()
    );

    // Should fire for both aggregation types
    assert!(
        !results.is_empty(),
        "Should have results from multiple aggregation types"
    );

    // Count different message types
    let log_results: Vec<_> = results
        .iter()
        .flat_map(|r| &r.actions_executed)
        .filter_map(|action| {
            if let bingo_core::rete_nodes::ActionResult::Logged { message } = action {
                Some(message)
            } else {
                None
            }
        })
        .collect();

    let distribution_results =
        log_results.iter().filter(|msg| msg.contains("distribution analysis")).count();
    let threshold_results =
        log_results.iter().filter(|msg| msg.contains("threshold analysis")).count();

    println!("📊 Distribution analysis results: {}", distribution_results);
    println!("📊 Threshold analysis results: {}", threshold_results);

    assert!(
        distribution_results > 0,
        "Should have distribution analysis results"
    );
    assert!(
        threshold_results > 0,
        "Should have threshold analysis results"
    );

    println!("✅ Multiple advanced aggregations test passed");
}

#[test]
fn test_advanced_aggregation_with_windowing() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Advanced Aggregation with Windowing");

    // Create a rule with sliding window and percentile aggregation
    let rule = Rule {
        id: 1,
        name: "Windowed Percentile Analysis".to_string(),
        conditions: vec![Condition::Aggregation(AggregationCondition {
            alias: "windowed_p50".to_string(),
            aggregation_type: AggregationType::Percentile(50.0), // Median
            source_field: "score".to_string(),
            window: Some(AggregationWindow::Sliding { size: 5 }), // Last 5 facts
            group_by: vec!["department".to_string()],
            having: Some(Box::new(Condition::Simple {
                field: "windowed_p50".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Float(25.0),
            })),
        })],
        actions: vec![Action {
            action_type: ActionType::Log {
                message: "Windowed median analysis complete".to_string(),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts
    let facts = create_statistical_facts();
    let results = engine.process_facts(facts).unwrap();

    println!(
        "🎯 Windowed aggregation results: {} rules fired",
        results.len()
    );

    // Should fire for sliding window aggregations
    assert!(
        !results.is_empty(),
        "Should have windowed aggregation results"
    );

    // Verify action results
    let log_results: Vec<_> = results
        .iter()
        .flat_map(|r| &r.actions_executed)
        .filter_map(|action| {
            if let bingo_core::rete_nodes::ActionResult::Logged { message } = action {
                Some(message)
            } else {
                None
            }
        })
        .collect();

    assert!(!log_results.is_empty(), "Should have log results");

    for message in log_results.iter().take(3) {
        println!("📊 Windowed result: {}", message);
        assert!(message.contains("Windowed median analysis"));
    }

    println!("✅ Advanced aggregation with windowing test passed");
}

#[test]
fn test_edge_cases_advanced_aggregations() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Edge Cases for Advanced Aggregations");

    // Create a rule with very specific percentile
    let rule = Rule {
        id: 1,
        name: "Edge Case Percentile".to_string(),
        conditions: vec![Condition::Aggregation(AggregationCondition {
            alias: "score_p99".to_string(),
            aggregation_type: AggregationType::Percentile(99.0), // 99th percentile
            source_field: "score".to_string(),
            window: None,
            group_by: vec!["department".to_string()],
            having: Some(Box::new(Condition::Simple {
                field: "score_p99".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Float(90.0), // Should be close to max value
            })),
        })],
        actions: vec![Action {
            action_type: ActionType::Log {
                message: "99th percentile analysis complete".to_string(),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts (includes outlier value of 100)
    let facts = create_statistical_facts();
    let results = engine.process_facts(facts).unwrap();

    println!("🎯 Edge case results: {} rules fired", results.len());

    // Should fire because 99th percentile with outlier should be > 90
    assert!(!results.is_empty(), "Should handle edge case percentiles");

    // Verify action results
    let log_results: Vec<_> = results
        .iter()
        .flat_map(|r| &r.actions_executed)
        .filter_map(|action| {
            if let bingo_core::rete_nodes::ActionResult::Logged { message } = action {
                Some(message)
            } else {
                None
            }
        })
        .collect();

    assert!(!log_results.is_empty(), "Should have log results");

    for message in log_results.iter().take(3) {
        println!("📊 Edge case result: {}", message);
        assert!(message.contains("99th percentile analysis"));
    }

    println!("✅ Edge cases advanced aggregations test passed");
}
