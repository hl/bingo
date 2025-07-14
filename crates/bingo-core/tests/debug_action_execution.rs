//! Debug test to understand why actions are not being executed

use bingo_core::BingoEngine;
use bingo_core::types::{Action, ActionType, Condition, Fact, FactData, FactValue, Operator, Rule};
use chrono::Utc;
use std::collections::HashMap;

#[test]
fn debug_simple_action_execution() {
    let engine = BingoEngine::new().unwrap();

    println!("🔍 Debug: Creating simple test rule");

    // Create the simplest possible rule with a Log action
    let rule = Rule {
        id: 1,
        name: "Simple Log Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "test_field".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("test_value".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::Log { message: "This is a test log message".to_string() },
        }],
    };

    println!("🔍 Debug: Adding rule to engine");
    engine.add_rule(rule).unwrap();

    // Create a fact that matches the condition
    let mut fields = HashMap::new();
    fields.insert(
        "test_field".to_string(),
        FactValue::String("test_value".to_string()),
    );
    let fact = Fact {
        id: 1,
        external_id: Some("test-fact-1".to_string()),
        timestamp: Utc::now(),
        data: FactData { fields },
    };

    println!("🔍 Debug: Processing fact");
    let results = engine.process_facts(vec![fact]).unwrap();

    println!("🔍 Debug: Results count: {}", results.len());
    for (i, result) in results.iter().enumerate() {
        println!(
            "🔍 Debug: Result {}: rule_id={}, fact_id={}, actions_count={}",
            i,
            result.rule_id,
            result.fact_id,
            result.actions_executed.len()
        );

        for (j, action) in result.actions_executed.iter().enumerate() {
            println!("🔍 Debug:   Action {j}: {action:?}");
        }
    }

    // Check if we have any results and actions
    assert!(!results.is_empty(), "Should have at least one result");
    assert!(
        !results[0].actions_executed.is_empty(),
        "Should have at least one action executed"
    );
}

#[test]
fn debug_increment_field_execution() {
    let engine = BingoEngine::new().unwrap();

    println!("🔍 Debug: Creating IncrementField test rule");

    let rule = Rule {
        id: 1,
        name: "Increment Field Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "name".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("Alice".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::IncrementField {
                field: "score".to_string(),
                increment: FactValue::Integer(10),
            },
        }],
    };

    println!("🔍 Debug: Adding IncrementField rule to engine");
    engine.add_rule(rule).unwrap();

    // Create a fact that matches the condition
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), FactValue::String("Alice".to_string()));
    fields.insert("score".to_string(), FactValue::Integer(50));
    let fact = Fact {
        id: 1,
        external_id: Some("alice-1".to_string()),
        timestamp: Utc::now(),
        data: FactData { fields },
    };

    println!("🔍 Debug: Processing fact with IncrementField action");
    let results = engine.process_facts(vec![fact]).unwrap();

    println!("🔍 Debug: IncrementField Results count: {}", results.len());
    for (i, result) in results.iter().enumerate() {
        println!(
            "🔍 Debug: Result {}: rule_id={}, fact_id={}, actions_count={}",
            i,
            result.rule_id,
            result.fact_id,
            result.actions_executed.len()
        );

        for (j, action) in result.actions_executed.iter().enumerate() {
            println!("🔍 Debug:   Action {j}: {action:?}");
        }
    }

    // Check if we have any results and actions
    assert!(!results.is_empty(), "Should have at least one result");
    assert!(
        !results[0].actions_executed.is_empty(),
        "Should have at least one action executed"
    );
}
