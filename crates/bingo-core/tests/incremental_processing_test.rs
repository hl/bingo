//! Incremental Processing Test
//!
//! This test validates the RETE algorithm's O(Δfacts) incremental processing
//! capability where only new/changed facts trigger rule evaluation.

use bingo_core::BingoEngine;
use bingo_core::types::*;
use std::collections::HashMap;

#[test]
fn test_incremental_fact_addition() {
    let engine = BingoEngine::new().expect("Engine creation failed");

    // Create a rule that requires high score and active status
    let rule = Rule {
        id: 1,
        name: "High Performer".to_string(),
        conditions: vec![
            Condition::Simple {
                field: "score".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Integer(90),
            },
            Condition::Simple {
                field: "status".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("active".to_string()),
            },
        ],
        actions: vec![Action {
            action_type: ActionType::Log { message: "High performer detected".to_string() },
        }],
    };

    engine.add_rule(rule).expect("Rule addition failed");

    // Add first fact (partial match)
    let mut fields1 = HashMap::new();
    fields1.insert("score".to_string(), FactValue::Integer(95));
    fields1.insert(
        "status".to_string(),
        FactValue::String("pending".to_string()),
    );
    fields1.insert("user_id".to_string(), FactValue::String("U001".to_string()));

    let fact1 = Fact::new(1, FactData { fields: fields1 });

    // Using the working memory API for incremental processing
    let results1 = engine
        .add_fact_to_working_memory(fact1)
        .expect("Incremental fact addition failed");

    // Should not fire because status is not "active"
    assert_eq!(results1.len(), 0, "Rule should not fire for partial match");

    // Add second fact that completes the pattern
    let mut fields2 = HashMap::new();
    fields2.insert("score".to_string(), FactValue::Integer(85)); // Lower score
    fields2.insert(
        "status".to_string(),
        FactValue::String("active".to_string()),
    );
    fields2.insert("user_id".to_string(), FactValue::String("U002".to_string()));

    let fact2 = Fact::new(2, FactData { fields: fields2 });
    let results2 = engine
        .add_fact_to_working_memory(fact2)
        .expect("Incremental fact addition failed");

    // Should not fire because score is not > 90
    assert_eq!(
        results2.len(),
        0,
        "Rule should not fire for insufficient score"
    );

    // Add third fact that matches all conditions
    let mut fields3 = HashMap::new();
    fields3.insert("score".to_string(), FactValue::Integer(95));
    fields3.insert(
        "status".to_string(),
        FactValue::String("active".to_string()),
    );
    fields3.insert("user_id".to_string(), FactValue::String("U003".to_string()));

    let fact3 = Fact::new(3, FactData { fields: fields3 });
    let results3 = engine
        .add_fact_to_working_memory(fact3)
        .expect("Incremental fact addition failed");

    // Should fire because all conditions are met
    assert_eq!(results3.len(), 1, "Rule should fire for complete match");
    assert_eq!(results3[0].rule_id, 1, "Correct rule should fire");

    println!("✅ Incremental fact addition test passed");
    let result_len = results3.len();
    println!("   Total rule activations: {result_len}");
}

#[test]
fn test_fact_retraction() {
    let engine = BingoEngine::new().expect("Engine creation failed");

    // Create a simple rule
    let rule = Rule {
        id: 2,
        name: "Active User".to_string(),
        conditions: vec![Condition::Simple {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("active".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Active user found".to_string() },
        }],
    };

    engine.add_rule(rule).expect("Rule addition failed");

    // Add a fact
    let mut fields = HashMap::new();
    fields.insert(
        "status".to_string(),
        FactValue::String("active".to_string()),
    );
    fields.insert("user_id".to_string(), FactValue::String("U001".to_string()));

    let fact = Fact::new(1, FactData { fields });

    let add_results = engine.add_fact_to_working_memory(fact).expect("Fact addition failed");

    assert_eq!(add_results.len(), 1, "Rule should fire when fact is added");

    // Verify fact is in working memory
    let (fact_count, _) = engine.get_working_memory_stats();
    assert_eq!(fact_count, 1, "Working memory should contain one fact");

    // Retract the fact
    let affected_rules = engine.remove_fact_from_working_memory(1).expect("Fact retraction failed");

    assert_eq!(
        affected_rules.len(),
        1,
        "One rule should be affected by retraction"
    );
    assert_eq!(
        affected_rules[0].rule_id, 2,
        "Correct rule should be affected"
    );

    // Verify fact is removed from working memory
    let (fact_count_after, _) = engine.get_working_memory_stats();
    assert_eq!(
        fact_count_after, 0,
        "Working memory should be empty after retraction"
    );

    println!("✅ Fact retraction test passed");
    println!("   Affected rules: {affected_rules:?}");
}

#[test]
fn test_incremental_vs_batch_processing() {
    let incremental_engine = BingoEngine::new().expect("Engine creation failed");
    let batch_engine = BingoEngine::new().expect("Engine creation failed");

    // Create identical rules in both engines
    let rule = Rule {
        id: 3,
        name: "Performance Comparison".to_string(),
        conditions: vec![
            Condition::Simple {
                field: "category".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("premium".to_string()),
            },
            Condition::Simple {
                field: "amount".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Float(1000.0),
            },
        ],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Premium transaction".to_string() },
        }],
    };

    incremental_engine.add_rule(rule.clone()).expect("Rule addition failed");
    batch_engine.add_rule(rule).expect("Rule addition failed");

    // Create test facts
    let mut facts = Vec::new();
    for i in 1..=5 {
        let mut fields = HashMap::new();
        fields.insert(
            "category".to_string(),
            FactValue::String("premium".to_string()),
        );
        fields.insert("amount".to_string(), FactValue::Float(1500.0));
        fields.insert(
            "transaction_id".to_string(),
            FactValue::String(format!("T{i:03}")),
        );

        facts.push(Fact::new(i, FactData { fields }));
    }

    // Test incremental processing
    let mut incremental_total_activations = 0;
    for fact in &facts {
        let results = incremental_engine
            .add_fact_to_working_memory(fact.clone())
            .expect("Incremental processing failed");
        incremental_total_activations += results.len();
    }

    // Test batch processing
    let batch_results = batch_engine.process_facts(facts.clone()).expect("Batch processing failed");

    // Both should produce the same number of rule activations
    assert_eq!(
        incremental_total_activations,
        batch_results.len(),
        "Incremental and batch processing should produce same results"
    );

    // Verify working memory state
    let (incremental_facts, _) = incremental_engine.get_working_memory_stats();
    assert_eq!(
        incremental_facts,
        facts.len(),
        "All facts should be in working memory"
    );

    println!("✅ Incremental vs batch processing test passed");
    println!("   Incremental activations: {incremental_total_activations}");
    let batch_len = batch_results.len();
    println!("   Batch activations: {batch_len}");
    println!("   Facts in working memory: {incremental_facts}");
}

#[test]
fn test_working_memory_lifecycle() {
    let engine = BingoEngine::new().expect("Engine creation failed");

    // Create a rule
    let rule = Rule {
        id: 4,
        name: "Lifecycle Test".to_string(),
        conditions: vec![Condition::Simple {
            field: "type".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("order".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Order processed".to_string() },
        }],
    };

    engine.add_rule(rule).expect("Rule addition failed");

    // Initial state
    let (initial_count, _) = engine.get_working_memory_stats();
    assert_eq!(initial_count, 0, "Working memory should start empty");

    // Add facts incrementally
    for i in 1..=3 {
        let mut fields = HashMap::new();
        fields.insert("type".to_string(), FactValue::String("order".to_string()));
        fields.insert("order_id".to_string(), FactValue::String(format!("ORD{i}")));

        let fact = Fact::new(i, FactData { fields });
        let results = engine.add_fact_to_working_memory(fact).expect("Fact addition failed");

        assert_eq!(results.len(), 1, "Each fact should trigger the rule");

        let (current_count, _) = engine.get_working_memory_stats();
        assert_eq!(current_count, i as usize, "Working memory should grow");
    }

    // Remove facts
    for i in 1..=3 {
        let affected_rules =
            engine.remove_fact_from_working_memory(i).expect("Fact removal failed");
        assert_eq!(
            affected_rules.len(),
            1,
            "Rule should be affected by retraction"
        );

        let (current_count, _) = engine.get_working_memory_stats();
        assert_eq!(
            current_count,
            3 - i as usize,
            "Working memory should shrink"
        );
    }

    // Final state
    let (final_count, _) = engine.get_working_memory_stats();
    assert_eq!(final_count, 0, "Working memory should be empty");

    println!("✅ Working memory lifecycle test passed");
}

#[test]
fn test_alpha_memory_integration() {
    let engine = BingoEngine::new().expect("Engine creation failed");

    // Create a rule to track alpha memory integration
    let rule = Rule {
        id: 5,
        name: "Alpha Memory Test".to_string(),
        conditions: vec![Condition::Simple {
            field: "priority".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("high".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::Log { message: "High priority item".to_string() },
        }],
    };

    engine.add_rule(rule).expect("Rule addition failed");

    // Check initial alpha memory state
    let (initial_memories, _initial_patterns, initial_processed) = engine.get_alpha_memory_info();
    assert!(
        initial_memories > 0,
        "Alpha memories should be created for rule"
    );

    // Add facts and track alpha memory statistics
    for i in 1..=5 {
        let mut fields = HashMap::new();
        fields.insert(
            "priority".to_string(),
            FactValue::String("high".to_string()),
        );
        fields.insert("task_id".to_string(), FactValue::String(format!("T{i}")));

        let fact = Fact::new(i, FactData { fields });
        let results = engine.add_fact_to_working_memory(fact).expect("Fact addition failed");

        assert_eq!(
            results.len(),
            1,
            "Each high priority fact should trigger rule"
        );
    }

    // Check final alpha memory state
    let (final_memories, _final_patterns, final_processed) = engine.get_alpha_memory_info();
    assert_eq!(
        final_memories, initial_memories,
        "Number of alpha memories should be stable"
    );
    assert!(
        final_processed > initial_processed,
        "More facts should have been processed"
    );

    // Get detailed alpha memory statistics
    let alpha_stats = engine.get_alpha_memory_stats();
    assert!(
        alpha_stats.total_facts_processed >= 5,
        "At least 5 facts should be processed"
    );
    assert!(
        alpha_stats.total_matches_found >= 5,
        "At least 5 matches should be found"
    );

    println!("✅ Alpha memory integration test passed");
    println!("   Alpha memories: {final_memories}");
    println!("   Facts processed: {final_processed}");
    println!(
        "   Match rate: {:.1}%",
        (alpha_stats.total_matches_found as f64 / alpha_stats.total_facts_processed as f64) * 100.0
    );
}
