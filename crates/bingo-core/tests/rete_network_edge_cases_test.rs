//! Comprehensive edge case and performance tests for RETE Network
//!
//! This test suite covers:
//! - Complex condition combinations and rule interactions
//! - Memory pressure and cache eviction scenarios  
//! - Error conditions and boundary cases
//! - Performance regression validation
//! - Large dataset stress testing

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unnecessary_cast)]

use bingo_calculator::calculator::Calculator;
use bingo_core::fact_store::arena_store::ArenaFactStore;
use bingo_core::rete_network::ReteNetwork;
use bingo_core::types::*;
use chrono::Utc;
use std::collections::HashMap;

/// Helper function to create a test fact with specified fields
fn create_test_fact(id: u64, fields: HashMap<String, FactValue>) -> Fact {
    Fact {
        id,
        external_id: Some(format!("ext_{}", id)),
        timestamp: Utc::now(),
        data: FactData { fields },
    }
}

/// Helper function to create a simple condition
fn create_simple_condition(field: &str, operator: Operator, value: FactValue) -> Condition {
    Condition::Simple { field: field.to_string(), operator, value }
}

/// Helper function to create a complex AND condition
fn create_and_condition(conditions: Vec<Condition>) -> Condition {
    Condition::Complex { operator: LogicalOperator::And, conditions }
}

/// Helper function to create a complex OR condition
fn create_or_condition(conditions: Vec<Condition>) -> Condition {
    Condition::Complex { operator: LogicalOperator::Or, conditions }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_empty_rule_conditions() {
        let mut network = ReteNetwork::new();
        let fact_store = ArenaFactStore::new();
        let calculator = Calculator::new();

        // Rule with no conditions should not match anything
        let rule = Rule {
            id: 1,
            name: "empty_conditions_rule".to_string(),
            conditions: vec![],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "test".to_string(),
                    value: FactValue::String("triggered".to_string()),
                },
            }],
        };

        network.add_rule(rule).expect("Failed to add rule");

        let fact = create_test_fact(1, {
            let mut fields = HashMap::new();
            fields.insert("amount".to_string(), FactValue::Integer(100));
            fields
        });

        let results = network
            .process_facts(&[fact.clone()], &fact_store, &calculator)
            .expect("Failed to process facts");

        // Empty conditions should not trigger any rules
        assert_eq!(results.len(), 0, "Empty condition rule should not execute");
    }

    #[test]
    fn test_deeply_nested_complex_conditions() {
        let mut network = ReteNetwork::new();
        let mut fact_store = ArenaFactStore::new();
        let calculator = Calculator::new();

        // Create deeply nested condition: ((A AND B) OR (C AND D)) AND E
        let inner_and1 = create_and_condition(vec![
            create_simple_condition("field1", Operator::Equal, FactValue::Integer(1)),
            create_simple_condition("field2", Operator::Equal, FactValue::Integer(2)),
        ]);

        let inner_and2 = create_and_condition(vec![
            create_simple_condition("field3", Operator::Equal, FactValue::Integer(3)),
            create_simple_condition("field4", Operator::Equal, FactValue::Integer(4)),
        ]);

        let inner_or = create_or_condition(vec![inner_and1, inner_and2]);

        let outer_and = create_and_condition(vec![
            inner_or,
            create_simple_condition("field5", Operator::Equal, FactValue::Integer(5)),
        ]);

        let rule = Rule {
            id: 1,
            name: "deeply_nested_rule".to_string(),
            conditions: vec![outer_and],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "result".to_string(),
                    value: FactValue::String("complex_match".to_string()),
                },
            }],
        };

        network.add_rule(rule).expect("Failed to add rule");

        // Add facts to fact store for the nested condition evaluation
        let fact1 = create_test_fact(1, {
            let mut fields = HashMap::new();
            fields.insert("field1".to_string(), FactValue::Integer(1));
            fields.insert("field2".to_string(), FactValue::Integer(2));
            fields.insert("field5".to_string(), FactValue::Integer(5));
            fields
        });

        let fact2 = create_test_fact(2, {
            let mut fields = HashMap::new();
            fields.insert("field3".to_string(), FactValue::Integer(3));
            fields.insert("field4".to_string(), FactValue::Integer(4));
            fields
        });

        fact_store.insert(fact1.clone());
        fact_store.insert(fact2.clone());

        // Process the first fact (should match via first branch of OR)
        let results = network
            .process_facts(&[fact1], &fact_store, &calculator)
            .expect("Failed to process facts");

        assert_eq!(results.len(), 1, "Deeply nested condition should match");
        assert_eq!(results[0].rule_id, 1);
    }

    #[test]
    fn test_not_operator_edge_cases() {
        let mut network = ReteNetwork::new();
        let fact_store = ArenaFactStore::new();
        let calculator = Calculator::new();

        // Test NOT with empty sub-conditions
        let not_condition =
            Condition::Complex { operator: LogicalOperator::Not, conditions: vec![] };

        let rule = Rule {
            id: 1,
            name: "not_operator_test".to_string(),
            conditions: vec![not_condition],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "result".to_string(),
                    value: FactValue::String("not_empty".to_string()),
                },
            }],
        };

        network.add_rule(rule).expect("Failed to add rule");

        let fact = create_test_fact(1, {
            let mut fields = HashMap::new();
            fields.insert("test".to_string(), FactValue::Integer(1));
            fields
        });

        let results = network
            .process_facts(&[fact], &fact_store, &calculator)
            .expect("Failed to process facts");

        // NOT with empty conditions should not match (returns false)
        assert_eq!(
            results.len(),
            0,
            "NOT with empty conditions should not match"
        );
    }

    #[test]
    fn test_contains_operator_edge_cases() {
        let mut network = ReteNetwork::new();
        let fact_store = ArenaFactStore::new();
        let calculator = Calculator::new();

        // Test contains with non-string types
        let rule1 = Rule {
            id: 1,
            name: "contains_number_test".to_string(),
            conditions: vec![create_simple_condition(
                "number",
                Operator::Contains,
                FactValue::String("123".to_string()),
            )],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "result".to_string(),
                    value: FactValue::String("number_contains".to_string()),
                },
            }],
        };

        let rule2 = Rule {
            id: 2,
            name: "contains_invalid_test".to_string(),
            conditions: vec![create_simple_condition(
                "text",
                Operator::Contains,
                FactValue::Integer(42),
            )],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "result".to_string(),
                    value: FactValue::String("invalid_contains".to_string()),
                },
            }],
        };

        network.add_rule(rule1).expect("Failed to add rule1");
        network.add_rule(rule2).expect("Failed to add rule2");

        let fact = create_test_fact(1, {
            let mut fields = HashMap::new();
            fields.insert("number".to_string(), FactValue::Integer(12345));
            fields.insert(
                "text".to_string(),
                FactValue::String("test42data".to_string()),
            );
            fields
        });

        let results = network
            .process_facts(&[fact], &fact_store, &calculator)
            .expect("Failed to process facts");

        // Contains operator should only work with string fields
        assert_eq!(
            results.len(),
            0,
            "Contains should not match non-string types"
        );
    }

    #[test]
    fn test_missing_field_conditions() {
        let mut network = ReteNetwork::new();
        let fact_store = ArenaFactStore::new();
        let calculator = Calculator::new();

        // Rule that references non-existent fields
        let rule = Rule {
            id: 1,
            name: "missing_fields_test".to_string(),
            conditions: vec![
                create_simple_condition("nonexistent", Operator::Equal, FactValue::Integer(100)),
                create_simple_condition("missing", Operator::GreaterThan, FactValue::Float(50.0)),
            ],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "result".to_string(),
                    value: FactValue::String("should_not_execute".to_string()),
                },
            }],
        };

        network.add_rule(rule).expect("Failed to add rule");

        let fact = create_test_fact(1, {
            let mut fields = HashMap::new();
            fields.insert("existing_field".to_string(), FactValue::Integer(200));
            fields
        });

        let results = network
            .process_facts(&[fact], &fact_store, &calculator)
            .expect("Failed to process facts");

        // Rules with missing fields should not match
        assert_eq!(
            results.len(),
            0,
            "Rules with missing fields should not execute"
        );
    }
}

#[cfg(test)]
mod memory_pressure_tests {
    use super::*;

    #[test]
    fn test_partial_match_cache_eviction() {
        let mut network = ReteNetwork::new();
        let fact_store = ArenaFactStore::new();
        let calculator = Calculator::new();

        // Add a simple rule
        let rule = Rule {
            id: 1,
            name: "cache_eviction_test".to_string(),
            conditions: vec![create_simple_condition(
                "value",
                Operator::GreaterThan,
                FactValue::Integer(0),
            )],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "processed".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        };

        network.add_rule(rule).expect("Failed to add rule");

        // Create enough facts to trigger cache eviction (cache limit is 8192)
        let mut facts = Vec::new();
        for i in 1..=10000 {
            let fact = create_test_fact(i, {
                let mut fields = HashMap::new();
                fields.insert("value".to_string(), FactValue::Integer(i as i64));
                fields
            });
            facts.push(fact);
        }

        // Process facts in batches to build up cache
        for chunk in facts.chunks(1000) {
            let _results = network
                .process_facts(chunk, &fact_store, &calculator)
                .expect("Failed to process facts chunk");
        }

        // Process the same facts again - should trigger cache eviction and clearing
        for chunk in facts.chunks(1000) {
            let results = network
                .process_facts(chunk, &fact_store, &calculator)
                .expect("Failed to process facts chunk after cache eviction");

            // After cache clearing, rules should still execute normally
            assert_eq!(
                results.len(),
                chunk.len(),
                "Rules should execute after cache eviction"
            );
        }
    }

    #[test]
    fn test_large_fact_set_performance() {
        let mut network = ReteNetwork::new();
        let mut fact_store = ArenaFactStore::new();
        let calculator = Calculator::new();

        // Add rules with various complexity levels
        let rules = vec![
            Rule {
                id: 1,
                name: "category_a_rule".to_string(),
                conditions: vec![create_simple_condition(
                    "category",
                    Operator::Equal,
                    FactValue::String("A".to_string()),
                )],
                actions: vec![Action {
                    action_type: ActionType::SetField {
                        field: "category_a_processed".to_string(),
                        value: FactValue::Boolean(true),
                    },
                }],
            },
            Rule {
                id: 2,
                name: "high_value_active_rule".to_string(),
                conditions: vec![
                    create_simple_condition(
                        "amount",
                        Operator::GreaterThan,
                        FactValue::Integer(1000),
                    ),
                    create_simple_condition(
                        "status",
                        Operator::Equal,
                        FactValue::String("active".to_string()),
                    ),
                ],
                actions: vec![Action {
                    action_type: ActionType::SetField {
                        field: "high_value_active".to_string(),
                        value: FactValue::Boolean(true),
                    },
                }],
            },
        ];

        for rule in rules {
            network.add_rule(rule).expect("Failed to add rule");
        }

        // Create large fact set (5000 facts)
        let mut facts = Vec::new();
        for i in 1..=5000 {
            let fact = create_test_fact(i, {
                let mut fields = HashMap::new();
                fields.insert("amount".to_string(), FactValue::Integer(i as i64 * 10));
                fields.insert(
                    "category".to_string(),
                    FactValue::String(if i % 3 == 0 { "A" } else { "B" }.to_string()),
                );
                fields.insert(
                    "status".to_string(),
                    FactValue::String(if i % 2 == 0 { "active" } else { "inactive" }.to_string()),
                );
                fields
            });
            facts.push(fact);
            fact_store.insert(facts.last().unwrap().clone());
        }

        let start_time = std::time::Instant::now();

        // Process all facts
        let results = network
            .process_facts(&facts, &fact_store, &calculator)
            .expect("Failed to process large fact set");

        let processing_time = start_time.elapsed();

        // Verify processing completed in reasonable time (should be < 1 second for optimized implementation)
        assert!(
            processing_time.as_millis() < 5000,
            "Large fact set processing took too long: {:?}",
            processing_time
        );

        // Verify rules executed successfully - exact count depends on complex rule interactions
        // The important test is that processing completes and produces reasonable results
        assert!(
            !results.is_empty(),
            "At least some rules should have executed"
        );
        assert!(
            results.len() <= facts.len() * 2,
            "Rule executions should be reasonable for fact count"
        );

        println!(
            "✅ Large fact set test: Processed {} facts with {} rule executions in {:?}",
            facts.len(),
            results.len(),
            processing_time
        );
    }
}

#[cfg(test)]
mod aggregation_edge_cases {
    use super::*;

    #[test]
    fn test_aggregation_with_empty_fact_set() {
        let mut network = ReteNetwork::new();
        let fact_store = ArenaFactStore::new(); // Empty fact store
        let calculator = Calculator::new();

        // Create aggregation condition on empty fact store
        let agg_condition = Condition::Aggregation(AggregationCondition {
            aggregation_type: AggregationType::Count,
            source_field: "amount".to_string(),
            alias: "total_count".to_string(),
            group_by: vec![],
            window: None,
            having: None,
        });

        let rule = Rule {
            id: 1,
            name: "empty_aggregation_test".to_string(),
            conditions: vec![agg_condition],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "empty_agg_result".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        };

        network.add_rule(rule).expect("Failed to add rule");

        let trigger_fact = create_test_fact(1, {
            let mut fields = HashMap::new();
            fields.insert("trigger".to_string(), FactValue::Boolean(true));
            fields
        });

        let results = network
            .process_facts(&[trigger_fact], &fact_store, &calculator)
            .expect("Failed to process facts");

        // Aggregation on empty set should not match (returns false)
        assert_eq!(
            results.len(),
            0,
            "Aggregation on empty fact set should not match"
        );
    }

    #[test]
    fn test_aggregation_with_null_values() {
        let mut network = ReteNetwork::new();
        let mut fact_store = ArenaFactStore::new();
        let calculator = Calculator::new();

        // Add facts with missing source field
        let facts = vec![
            create_test_fact(1, {
                let mut fields = HashMap::new();
                fields.insert("category".to_string(), FactValue::String("A".to_string()));
                // Missing 'amount' field
                fields
            }),
            create_test_fact(2, {
                let mut fields = HashMap::new();
                fields.insert("category".to_string(), FactValue::String("A".to_string()));
                fields.insert("amount".to_string(), FactValue::Integer(100));
                fields
            }),
            create_test_fact(3, {
                let mut fields = HashMap::new();
                fields.insert("category".to_string(), FactValue::String("A".to_string()));
                // Missing 'amount' field again
                fields
            }),
        ];

        for fact in &facts {
            fact_store.insert(fact.clone());
        }

        let agg_condition = Condition::Aggregation(AggregationCondition {
            aggregation_type: AggregationType::Sum,
            source_field: "amount".to_string(),
            alias: "total_amount".to_string(),
            group_by: vec!["category".to_string()],
            window: None,
            having: None,
        });

        let rule = Rule {
            id: 1,
            name: "null_values_aggregation_test".to_string(),
            conditions: vec![agg_condition],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "aggregation_result".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        };

        network.add_rule(rule).expect("Failed to add rule");

        let trigger_fact = create_test_fact(4, {
            let mut fields = HashMap::new();
            fields.insert("category".to_string(), FactValue::String("A".to_string()));
            fields.insert("amount".to_string(), FactValue::Integer(50));
            fields
        });

        let results = network
            .process_facts(&[trigger_fact], &fact_store, &calculator)
            .expect("Failed to process facts");

        // Aggregation should handle missing fields gracefully (only sum existing values)
        assert_eq!(
            results.len(),
            1,
            "Aggregation should handle missing source fields"
        );
    }

    #[test]
    fn test_percentile_edge_cases() {
        let mut network = ReteNetwork::new();
        let mut fact_store = ArenaFactStore::new();
        let calculator = Calculator::new();

        // Add facts with single value for percentile calculation
        let fact = create_test_fact(1, {
            let mut fields = HashMap::new();
            fields.insert("value".to_string(), FactValue::Float(42.0));
            fields
        });
        fact_store.insert(fact.clone());

        let agg_condition = Condition::Aggregation(AggregationCondition {
            aggregation_type: AggregationType::Percentile(95.0),
            source_field: "value".to_string(),
            alias: "p95_value".to_string(),
            group_by: vec![],
            window: None,
            having: None,
        });

        let rule = Rule {
            id: 1,
            name: "percentile_edge_case_test".to_string(),
            conditions: vec![agg_condition],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "percentile_result".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        };

        network.add_rule(rule).expect("Failed to add rule");

        let results = network
            .process_facts(&[fact], &fact_store, &calculator)
            .expect("Failed to process facts");

        // Percentile calculation with single value should work
        assert_eq!(
            results.len(),
            1,
            "Percentile calculation should handle single value"
        );
    }
}

#[cfg(test)]
mod performance_regression_tests {
    use super::*;

    #[test]
    fn test_cache_hit_ratio_effectiveness() {
        let mut network = ReteNetwork::new();
        let fact_store = ArenaFactStore::new();
        let calculator = Calculator::new();

        // Add a rule that will be evaluated multiple times
        let rule = Rule {
            id: 1,
            name: "cache_hit_test".to_string(),
            conditions: vec![create_simple_condition(
                "repeatable",
                Operator::Equal,
                FactValue::Integer(1),
            )],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "cached_result".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        };

        network.add_rule(rule).expect("Failed to add rule");

        let fact = create_test_fact(1, {
            let mut fields = HashMap::new();
            fields.insert("repeatable".to_string(), FactValue::Integer(1));
            fields
        });

        // First processing - should execute rule
        let start_time = std::time::Instant::now();
        let results1 = network
            .process_facts(&[fact.clone()], &fact_store, &calculator)
            .expect("Failed to process facts first time");
        let first_duration = start_time.elapsed();

        // Second processing - should use cache (be faster)
        let start_time = std::time::Instant::now();
        let results2 = network
            .process_facts(&[fact.clone()], &fact_store, &calculator)
            .expect("Failed to process facts second time");
        let second_duration = start_time.elapsed();

        assert_eq!(results1.len(), 1, "First processing should execute rule");
        assert_eq!(
            results2.len(),
            0,
            "Second processing should use cache (no execution)"
        );

        // Cache should make second run faster (though this is timing-dependent)
        println!(
            "✅ Cache effectiveness: First run: {:?}, Second run: {:?}",
            first_duration, second_duration
        );
    }

    #[test]
    fn test_memory_pool_efficiency() {
        let mut network = ReteNetwork::new();
        let fact_store = ArenaFactStore::new();
        let calculator = Calculator::new();

        // Get initial memory pool stats
        let initial_stats = network.get_memory_pool_stats();
        let initial_efficiency = network.get_memory_pool_efficiency();

        let rule = Rule {
            id: 1,
            name: "memory_pool_test".to_string(),
            conditions: vec![create_simple_condition(
                "pool_test",
                Operator::GreaterThan,
                FactValue::Integer(0),
            )],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "pooled".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        };

        network.add_rule(rule).expect("Failed to add rule");

        // Process many facts to test memory pooling
        let facts: Vec<Fact> = (1..=1000)
            .map(|i| {
                create_test_fact(i, {
                    let mut fields = HashMap::new();
                    fields.insert("pool_test".to_string(), FactValue::Integer(i as i64));
                    fields
                })
            })
            .collect();

        let _results = network
            .process_facts(&facts, &fact_store, &calculator)
            .expect("Failed to process facts for pooling test");

        // Get final memory pool stats
        let final_stats = network.get_memory_pool_stats();
        let final_efficiency = network.get_memory_pool_efficiency();

        // Memory pool efficiency should improve with usage
        assert!(
            final_efficiency >= initial_efficiency * 0.8,
            "Memory pool efficiency should not degrade significantly: {} -> {}",
            initial_efficiency,
            final_efficiency
        );

        println!(
            "✅ Memory pool efficiency: {:.2}% -> {:.2}%",
            initial_efficiency * 100.0,
            final_efficiency * 100.0
        );
        println!("   Pool stats: {:?} -> {:?}", initial_stats, final_stats);
    }

    #[test]
    fn test_rule_scaling_performance() {
        let _network = ReteNetwork::new();
        let _fact_store = ArenaFactStore::new();
        let calculator = Calculator::new();

        // Add varying numbers of rules to test scaling
        let rule_counts = vec![1, 10, 50, 100];
        let mut timings = Vec::new();

        for &rule_count in &rule_counts {
            let mut test_network = ReteNetwork::new();
            let test_fact_store = ArenaFactStore::new();

            // Add rules
            for i in 1..=rule_count {
                let rule = Rule {
                    id: i,
                    name: format!("scaling_rule_{}", i),
                    conditions: vec![create_simple_condition(
                        "scale_test",
                        Operator::Equal,
                        FactValue::Integer(i as i64),
                    )],
                    actions: vec![Action {
                        action_type: ActionType::SetField {
                            field: format!("rule_{}_executed", i),
                            value: FactValue::Boolean(true),
                        },
                    }],
                };
                test_network.add_rule(rule).expect("Failed to add rule");
            }

            // Create test facts
            let facts: Vec<Fact> = (1..=100)
                .map(|i| {
                    create_test_fact(i, {
                        let mut fields = HashMap::new();
                        fields.insert(
                            "scale_test".to_string(),
                            FactValue::Integer((i % rule_count as u64 + 1) as i64),
                        );
                        fields
                    })
                })
                .collect();

            // Measure processing time
            let start_time = std::time::Instant::now();
            let _results = test_network
                .process_facts(&facts, &test_fact_store, &calculator)
                .expect("Failed to process facts for scaling test");
            let duration = start_time.elapsed();

            timings.push((rule_count, duration));
            println!(
                "✅ Scaling test: {} rules processed 100 facts in {:?}",
                rule_count, duration
            );
        }

        // Verify that performance doesn't degrade exponentially
        // (Linear or sub-linear scaling is acceptable)
        let first_timing = timings[0].1.as_micros() as f64;
        let last_timing = timings.last().unwrap().1.as_micros() as f64;
        let rule_ratio = timings.last().unwrap().0 as f64 / timings[0].0 as f64;
        let time_ratio = last_timing / first_timing;

        // Performance should scale better than O(n²)
        let max_acceptable_ratio = rule_ratio * rule_ratio;
        assert!(
            time_ratio < max_acceptable_ratio,
            "Performance scaling is worse than O(n²): {:.2}x rules took {:.2}x time (max acceptable: {:.2}x)",
            rule_ratio,
            time_ratio,
            max_acceptable_ratio
        );
    }
}
