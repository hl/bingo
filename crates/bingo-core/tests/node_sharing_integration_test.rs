//! Integration tests for RETE network node sharing optimization
//!
//! This test validates that identical nodes are shared across rules to reduce
//! memory usage and improve scalability while maintaining correct behavior.

use bingo_core::*;
use std::collections::HashMap;

#[test]
fn test_alpha_node_sharing_across_rules() {
    let mut engine = ReteNetwork::new().unwrap();

    // Create multiple rules with identical conditions to test alpha node sharing
    let shared_condition = Condition::Simple {
        field: "status".to_string(),
        operator: Operator::Equal,
        value: FactValue::String("active".to_string()),
    };

    let rules = vec![
        Rule {
            id: 1,
            name: "rule_1".to_string(),
            conditions: vec![shared_condition.clone()],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "processed_by_rule_1".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        },
        Rule {
            id: 2,
            name: "rule_2".to_string(),
            conditions: vec![shared_condition.clone()],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "processed_by_rule_2".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        },
        Rule {
            id: 3,
            name: "rule_3".to_string(),
            conditions: vec![shared_condition.clone()],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "processed_by_rule_3".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        },
    ];

    // Add all rules
    for rule in &rules {
        engine.add_rule(rule.clone()).unwrap();
    }

    // Check node sharing statistics
    let sharing_stats = engine.get_node_sharing_stats();
    println!("Alpha Node Sharing Test:");
    println!(
        "  Total alpha nodes created: {}",
        sharing_stats.alpha_nodes_total
    );
    println!("  Alpha nodes shared: {}", sharing_stats.alpha_shares_found);
    println!(
        "  Alpha sharing rate: {:.1}%",
        sharing_stats.alpha_sharing_rate
    );
    println!("  Alpha nodes active: {}", sharing_stats.alpha_nodes_active);

    // Should have created only 1 alpha node but found 2 shares (rules 2 and 3 shared with rule 1)
    assert_eq!(
        sharing_stats.alpha_nodes_total, 1,
        "Only 1 alpha node was actually created"
    );
    assert_eq!(
        sharing_stats.alpha_shares_found, 2,
        "Rules 2 and 3 should share with rule 1"
    );
    assert_eq!(
        sharing_stats.alpha_nodes_active, 1,
        "Only 1 unique alpha node should exist"
    );
    assert!(
        sharing_stats.alpha_sharing_rate > 100.0,
        "Sharing rate should be high (2 shares / 1 created)"
    );

    // Test memory savings
    let memory_savings = engine.get_memory_savings();
    println!("  Alpha nodes saved: {}", memory_savings.alpha_nodes_saved);
    println!("  Memory saved: {}", memory_savings.to_human_readable());

    assert_eq!(memory_savings.alpha_nodes_saved, 2);
    assert!(memory_savings.total_memory_saved_bytes > 0);

    // Test that facts are processed correctly by all rules
    let mut fields = HashMap::new();
    fields.insert(
        "status".to_string(),
        FactValue::String("active".to_string()),
    );
    fields.insert("id".to_string(), FactValue::Integer(123));

    let fact = Fact { id: 1, data: FactData { fields } };

    let results = engine.process_facts(vec![fact]).unwrap();
    println!("  Results generated: {}", results.len());

    // Should generate 3 results (one from each rule)
    assert_eq!(
        results.len(),
        3,
        "All 3 rules should fire and produce results"
    );

    // Verify each rule produced the expected result
    let rule_1_results: Vec<_> = results
        .iter()
        .filter(|r| r.data.fields.get("processed_by_rule_1") == Some(&FactValue::Boolean(true)))
        .collect();
    let rule_2_results: Vec<_> = results
        .iter()
        .filter(|r| r.data.fields.get("processed_by_rule_2") == Some(&FactValue::Boolean(true)))
        .collect();
    let rule_3_results: Vec<_> = results
        .iter()
        .filter(|r| r.data.fields.get("processed_by_rule_3") == Some(&FactValue::Boolean(true)))
        .collect();

    assert_eq!(rule_1_results.len(), 1, "Rule 1 should fire once");
    assert_eq!(rule_2_results.len(), 1, "Rule 2 should fire once");
    assert_eq!(rule_3_results.len(), 1, "Rule 3 should fire once");

    println!("  ✓ All rules fired correctly despite shared alpha node");
}

#[test]
fn test_beta_node_sharing_across_rules() {
    let mut engine = ReteNetwork::new().unwrap();

    // Create rules with multiple conditions that should result in shared beta nodes
    // Both conditions reference user_id so they can be joined
    let condition1 = Condition::Simple {
        field: "user_id".to_string(),
        operator: Operator::Equal,
        value: FactValue::Integer(42),
    };

    let condition2 = Condition::Simple {
        field: "user_id".to_string(),
        operator: Operator::GreaterThan,
        value: FactValue::Integer(0),
    };

    let rules = vec![
        Rule {
            id: 1,
            name: "engineering_user_rule_1".to_string(),
            conditions: vec![condition1.clone(), condition2.clone()],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "engineering_user_action_1".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        },
        Rule {
            id: 2,
            name: "engineering_user_rule_2".to_string(),
            conditions: vec![condition1.clone(), condition2.clone()],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "engineering_user_action_2".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        },
    ];

    // Add rules
    for rule in &rules {
        engine.add_rule(rule.clone()).unwrap();
    }

    // Check node sharing statistics
    let sharing_stats = engine.get_node_sharing_stats();
    println!("Beta Node Sharing Test:");
    println!(
        "  Total alpha nodes created: {}",
        sharing_stats.alpha_nodes_total
    );
    println!("  Alpha nodes shared: {}", sharing_stats.alpha_shares_found);
    println!(
        "  Total beta nodes created: {}",
        sharing_stats.beta_nodes_total
    );
    println!("  Beta nodes shared: {}", sharing_stats.beta_shares_found);
    println!(
        "  Beta sharing rate: {:.1}%",
        sharing_stats.beta_sharing_rate
    );
    println!(
        "  Overall sharing rate: {:.1}%",
        sharing_stats.overall_sharing_rate()
    );

    // Should have shared both alpha nodes (conditions) and beta nodes (joins)
    assert!(
        sharing_stats.alpha_shares_found > 0,
        "Should share alpha nodes"
    );
    assert!(
        sharing_stats.beta_shares_found > 0,
        "Should share beta nodes"
    );
    assert!(
        sharing_stats.overall_sharing_rate() > 0.0,
        "Should have overall sharing"
    );

    // Test memory savings
    let memory_savings = engine.get_memory_savings();
    println!(
        "  Total nodes saved: {}",
        memory_savings.total_nodes_saved()
    );
    println!("  Memory saved: {}", memory_savings.to_human_readable());

    assert!(memory_savings.total_nodes_saved() > 0);
    assert!(memory_savings.total_memory_saved_bytes > 0);
}

#[test]
fn test_node_sharing_with_rule_removal() {
    let mut engine = ReteNetwork::new().unwrap();

    // Create shared condition
    let shared_condition = Condition::Simple {
        field: "category".to_string(),
        operator: Operator::Equal,
        value: FactValue::String("premium".to_string()),
    };

    // Add first rule
    let rule1 = Rule {
        id: 1,
        name: "premium_rule_1".to_string(),
        conditions: vec![shared_condition.clone()],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "premium_processing".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };
    engine.add_rule(rule1).unwrap();

    // Add second rule with same condition
    let rule2 = Rule {
        id: 2,
        name: "premium_rule_2".to_string(),
        conditions: vec![shared_condition.clone()],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "premium_priority".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };
    engine.add_rule(rule2).unwrap();

    let stats_after_adding = engine.get_node_sharing_stats();
    println!("Node Sharing with Rule Removal Test:");
    println!("  After adding 2 rules:");
    println!(
        "    Alpha shares found: {}",
        stats_after_adding.alpha_shares_found
    );
    println!(
        "    Alpha nodes active: {}",
        stats_after_adding.alpha_nodes_active
    );

    // Should have shared the alpha node
    assert_eq!(
        stats_after_adding.alpha_shares_found, 1,
        "Should have 1 alpha node share"
    );
    assert_eq!(
        stats_after_adding.alpha_nodes_active, 1,
        "Should have 1 active alpha node"
    );

    // Remove first rule
    engine.remove_rule(1).unwrap();

    let stats_after_removal = engine.get_node_sharing_stats();
    println!("  After removing rule 1:");
    println!(
        "    Alpha shares found: {}",
        stats_after_removal.alpha_shares_found
    );
    println!(
        "    Alpha nodes active: {}",
        stats_after_removal.alpha_nodes_active
    );

    // Alpha node should still exist (used by rule 2)
    assert_eq!(
        stats_after_removal.alpha_nodes_active, 1,
        "Alpha node should still exist for rule 2"
    );

    // Test that rule 2 still works
    let mut fields = HashMap::new();
    fields.insert(
        "category".to_string(),
        FactValue::String("premium".to_string()),
    );

    let fact = Fact { id: 1, data: FactData { fields } };

    let results = engine.process_facts(vec![fact]).unwrap();
    println!("    Results after rule removal: {}", results.len());

    assert_eq!(results.len(), 1, "Rule 2 should still fire");
    assert!(results[0].data.fields.get("premium_priority") == Some(&FactValue::Boolean(true)));

    // Remove second rule
    engine.remove_rule(2).unwrap();

    let stats_after_all_removal = engine.get_node_sharing_stats();
    println!("  After removing rule 2:");
    println!(
        "    Alpha nodes active: {}",
        stats_after_all_removal.alpha_nodes_active
    );

    // Now the alpha node should be removed
    assert_eq!(
        stats_after_all_removal.alpha_nodes_active, 0,
        "All shared nodes should be cleaned up"
    );
}

#[test]
fn test_mixed_shared_and_unique_nodes() {
    let mut engine = ReteNetwork::new().unwrap();

    // Create some shared and some unique conditions
    let shared_condition = Condition::Simple {
        field: "status".to_string(),
        operator: Operator::Equal,
        value: FactValue::String("active".to_string()),
    };

    let unique_condition1 = Condition::Simple {
        field: "priority".to_string(),
        operator: Operator::GreaterThan,
        value: FactValue::Integer(5),
    };

    let unique_condition2 = Condition::Simple {
        field: "region".to_string(),
        operator: Operator::Equal,
        value: FactValue::String("us-west".to_string()),
    };

    let rules = vec![
        Rule {
            id: 1,
            name: "shared_rule_1".to_string(),
            conditions: vec![shared_condition.clone()],
            actions: vec![Action {
                action_type: ActionType::Log { message: "Rule 1".to_string() },
            }],
        },
        Rule {
            id: 2,
            name: "shared_rule_2".to_string(),
            conditions: vec![shared_condition.clone()], // Same as rule 1
            actions: vec![Action {
                action_type: ActionType::Log { message: "Rule 2".to_string() },
            }],
        },
        Rule {
            id: 3,
            name: "unique_rule_1".to_string(),
            conditions: vec![unique_condition1],
            actions: vec![Action {
                action_type: ActionType::Log { message: "Rule 3".to_string() },
            }],
        },
        Rule {
            id: 4,
            name: "unique_rule_2".to_string(),
            conditions: vec![unique_condition2],
            actions: vec![Action {
                action_type: ActionType::Log { message: "Rule 4".to_string() },
            }],
        },
    ];

    for rule in &rules {
        engine.add_rule(rule.clone()).unwrap();
    }

    let stats = engine.get_node_sharing_stats();
    println!("Mixed Shared and Unique Nodes Test:");
    println!("  Total alpha nodes created: {}", stats.alpha_nodes_total);
    println!("  Alpha shares found: {}", stats.alpha_shares_found);
    println!("  Alpha nodes active: {}", stats.alpha_nodes_active);
    println!("  Alpha sharing rate: {:.1}%", stats.alpha_sharing_rate);

    // Should have created 3 unique nodes, with 1 share (rule 2 shares with rule 1), 3 active nodes
    assert_eq!(stats.alpha_nodes_total, 3); // 3 unique nodes created
    assert_eq!(stats.alpha_shares_found, 1); // 1 sharing event
    assert_eq!(stats.alpha_nodes_active, 3); // shared + unique1 + unique2
    assert!((stats.alpha_sharing_rate - 33.3).abs() < 0.1); // 1/3 = 33.3%

    let memory_savings = engine.get_memory_savings();
    println!("  Memory saved: {}", memory_savings.to_human_readable());
    println!(
        "  Nodes that would exist without sharing: {}",
        stats.nodes_without_sharing()
    );

    assert_eq!(memory_savings.alpha_nodes_saved, 1);
    assert_eq!(stats.nodes_without_sharing(), 4); // 3 created + 1 shared
}

#[test]
fn test_performance_impact_of_node_sharing() {
    let mut engine = ReteNetwork::new().unwrap();

    // Create many rules with shared conditions to test performance
    let shared_condition = Condition::Simple {
        field: "type".to_string(),
        operator: Operator::Equal,
        value: FactValue::String("order".to_string()),
    };

    let rule_count = 50;

    let start_time = std::time::Instant::now();

    // Add many rules with the same condition
    for i in 1..=rule_count {
        let rule = Rule {
            id: i,
            name: format!("order_rule_{}", i),
            conditions: vec![shared_condition.clone()],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: format!("processed_by_rule_{}", i),
                    value: FactValue::Boolean(true),
                },
            }],
        };
        engine.add_rule(rule).unwrap();
    }

    let compilation_time = start_time.elapsed();

    let stats = engine.get_node_sharing_stats();
    let memory_savings = engine.get_memory_savings();

    println!("Performance Impact Test:");
    println!("  Rules added: {}", rule_count);
    println!("  Compilation time: {:?}", compilation_time);
    println!("  Total alpha nodes created: {}", stats.alpha_nodes_total);
    println!("  Alpha shares found: {}", stats.alpha_shares_found);
    println!("  Alpha nodes active: {}", stats.alpha_nodes_active);
    println!("  Alpha sharing rate: {:.1}%", stats.alpha_sharing_rate);
    println!("  Memory saved: {}", memory_savings.to_human_readable());

    // With node sharing, should have only 1 active alpha node despite 50 rules
    assert_eq!(stats.alpha_nodes_total, 1); // Only 1 node actually created
    assert_eq!(stats.alpha_shares_found, (rule_count - 1) as usize); // First rule creates, rest share
    assert_eq!(stats.alpha_nodes_active, 1);
    assert!(stats.alpha_sharing_rate > 1000.0); // Should be very high sharing rate (49 shares / 1 created)

    // Test that facts are still processed correctly
    let mut fields = HashMap::new();
    fields.insert("type".to_string(), FactValue::String("order".to_string()));
    fields.insert("order_id".to_string(), FactValue::Integer(12345));

    let fact = Fact { id: 1, data: FactData { fields } };

    let processing_start = std::time::Instant::now();
    let results = engine.process_facts(vec![fact]).unwrap();
    let processing_time = processing_start.elapsed();

    println!("  Fact processing time: {:?}", processing_time);
    println!("  Results generated: {}", results.len());

    // Should generate one result per rule
    assert_eq!(results.len(), rule_count as usize);

    // Verify all rules fired
    for i in 1..=rule_count {
        let field_name = format!("processed_by_rule_{}", i);
        let matching_results: Vec<_> = results
            .iter()
            .filter(|r| r.data.fields.get(&field_name) == Some(&FactValue::Boolean(true)))
            .collect();
        assert_eq!(
            matching_results.len(),
            1,
            "Rule {} should fire exactly once",
            i
        );
    }

    println!(
        "  ✓ All {} rules fired correctly with shared nodes",
        rule_count
    );
}
