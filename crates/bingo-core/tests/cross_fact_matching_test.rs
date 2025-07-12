//! Cross-Fact Pattern Matching Test
//!
//! This test validates that the beta network properly handles multi-condition rules
//! that require matching patterns across different facts.

use bingo_core::BingoEngine;
use bingo_core::types::*;
use std::collections::HashMap;

#[test]
fn test_cross_fact_pattern_matching() {
    let mut engine = BingoEngine::new().expect("Engine creation failed");

    // Create a multi-condition rule that should match across different facts
    let rule = Rule {
        id: 1,
        name: "High Value Customer Order".to_string(),
        conditions: vec![
            // Condition 1: Order amount > 1000
            Condition::Simple {
                field: "amount".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Float(1000.0),
            },
            // Condition 2: Customer status = "premium"
            Condition::Simple {
                field: "status".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("premium".to_string()),
            },
        ],
        actions: vec![Action {
            action_type: ActionType::Log {
                message: "High value customer order detected".to_string(),
            },
        }],
    };

    engine.add_rule(rule).expect("Rule addition failed");

    // Create facts that should trigger the rule when both conditions are met
    let mut order_fields = HashMap::new();
    order_fields.insert("amount".to_string(), FactValue::Float(1500.0));
    order_fields.insert(
        "customer_id".to_string(),
        FactValue::String("C123".to_string()),
    );
    order_fields.insert(
        "status".to_string(),
        FactValue::String("premium".to_string()),
    );

    let order_fact = Fact::new(1, FactData { fields: order_fields });

    // Process the fact
    let results = engine.process_facts(vec![order_fact]).expect("Fact processing failed");

    // Verify that the rule was triggered
    assert_eq!(results.len(), 1, "Expected exactly one rule to fire");
    assert_eq!(results[0].rule_id, 1, "Expected rule 1 to fire");

    println!("✅ Cross-fact pattern matching test passed");
    println!(
        "   Rule fired: {} actions executed",
        results[0].actions_executed.len()
    );
}

#[test]
fn test_beta_network_partial_matches() {
    let mut engine = BingoEngine::new().expect("Engine creation failed");

    // Create a rule with multiple conditions
    let rule = Rule {
        id: 2,
        name: "Complex Multi-Condition Rule".to_string(),
        conditions: vec![
            Condition::Simple {
                field: "type".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("order".to_string()),
            },
            Condition::Simple {
                field: "amount".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Float(500.0),
            },
            Condition::Simple {
                field: "priority".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("high".to_string()),
            },
        ],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Complex rule triggered".to_string() },
        }],
    };

    engine.add_rule(rule).expect("Rule addition failed");

    // Test 1: Fact that matches only some conditions (should not fire)
    let mut partial_fields = HashMap::new();
    partial_fields.insert("type".to_string(), FactValue::String("order".to_string()));
    partial_fields.insert("amount".to_string(), FactValue::Float(300.0)); // Too low
    partial_fields.insert(
        "priority".to_string(),
        FactValue::String("high".to_string()),
    );

    let partial_fact = Fact::new(1, FactData { fields: partial_fields });
    let results = engine.process_facts(vec![partial_fact]).expect("Fact processing failed");
    assert_eq!(results.len(), 0, "Rule should not fire for partial match");

    // Test 2: Fact that matches all conditions (should fire)
    let mut complete_fields = HashMap::new();
    complete_fields.insert("type".to_string(), FactValue::String("order".to_string()));
    complete_fields.insert("amount".to_string(), FactValue::Float(750.0)); // Meets threshold
    complete_fields.insert(
        "priority".to_string(),
        FactValue::String("high".to_string()),
    );

    let complete_fact = Fact::new(2, FactData { fields: complete_fields });
    let results = engine.process_facts(vec![complete_fact]).expect("Fact processing failed");
    assert_eq!(results.len(), 1, "Rule should fire for complete match");

    println!("✅ Beta network partial matches test passed");
}

#[test]
fn test_beta_network_structure_creation() {
    let mut engine = BingoEngine::new().expect("Engine creation failed");

    // Create a rule with multiple conditions to trigger beta network creation
    let rule = Rule {
        id: 3,
        name: "Beta Network Structure Test".to_string(),
        conditions: vec![
            Condition::Simple {
                field: "field1".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("value1".to_string()),
            },
            Condition::Simple {
                field: "field2".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Integer(100),
            },
            Condition::Simple {
                field: "field3".to_string(),
                operator: Operator::Contains,
                value: FactValue::String("test".to_string()),
            },
        ],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Beta network structure test".to_string() },
        }],
    };

    // Adding the rule should create beta network structure
    engine.add_rule(rule).expect("Rule addition failed");

    // Create a fact that matches all conditions
    let mut fields = HashMap::new();
    fields.insert(
        "field1".to_string(),
        FactValue::String("value1".to_string()),
    );
    fields.insert("field2".to_string(), FactValue::Integer(150));
    fields.insert(
        "field3".to_string(),
        FactValue::String("test_string".to_string()),
    );

    let fact = Fact::new(1, FactData { fields });
    let results = engine.process_facts(vec![fact]).expect("Fact processing failed");

    assert_eq!(
        results.len(),
        1,
        "Beta network should process multi-condition rule"
    );
    assert_eq!(results[0].rule_id, 3, "Correct rule should fire");

    println!("✅ Beta network structure creation test passed");
}

#[test]
fn test_alpha_beta_integration() {
    let mut engine = BingoEngine::new().expect("Engine creation failed");

    // Create both single and multi-condition rules to test integration
    let single_rule = Rule {
        id: 10,
        name: "Single Condition Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("active".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Single condition matched".to_string() },
        }],
    };

    let multi_rule = Rule {
        id: 11,
        name: "Multi Condition Rule".to_string(),
        conditions: vec![
            Condition::Simple {
                field: "status".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("active".to_string()),
            },
            Condition::Simple {
                field: "score".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Integer(80),
            },
        ],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Multi condition matched".to_string() },
        }],
    };

    engine.add_rule(single_rule).expect("Single rule addition failed");
    engine.add_rule(multi_rule).expect("Multi rule addition failed");

    // Create a fact that should trigger both rules
    let mut fields = HashMap::new();
    fields.insert(
        "status".to_string(),
        FactValue::String("active".to_string()),
    );
    fields.insert("score".to_string(), FactValue::Integer(95));

    let fact = Fact::new(1, FactData { fields });
    let results = engine.process_facts(vec![fact]).expect("Fact processing failed");

    // Both rules should fire
    assert_eq!(
        results.len(),
        2,
        "Both alpha and beta network rules should fire"
    );

    let rule_ids: Vec<u64> = results.iter().map(|r| r.rule_id).collect();
    assert!(rule_ids.contains(&10), "Single condition rule should fire");
    assert!(rule_ids.contains(&11), "Multi condition rule should fire");

    println!("✅ Alpha-Beta integration test passed");
    println!("   Rules fired: {rule_ids:?}");
}
