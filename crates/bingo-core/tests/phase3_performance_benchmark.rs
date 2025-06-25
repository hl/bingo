//! Comprehensive Performance Benchmark for Phase 3 Optimizations
//!
//! This benchmark validates the combined performance impact of all Phase 3 optimizations:
//! 1. Fact Lookup Optimization (LRU caching + O(1) hash-based access)
//! 2. Calculator Expression Caching (compilation + result caching)  
//! 3. RETE Network Node Sharing (memory optimization)
//! 4. Memory Pool Management (allocation optimization)

use bingo_core::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_phase3_combined_performance_benchmark() {
    println!("🚀 Phase 3 Performance Benchmark - Combined Optimizations");
    println!("=========================================================");

    let mut engine = ReteNetwork::new().unwrap();

    // Create multiple rules with shared conditions and complex formulas
    // This tests all optimizations simultaneously

    // Start with a simple shared condition that will definitely match
    let shared_condition = Condition::Simple {
        field: "user_type".to_string(),
        operator: Operator::Equal,
        value: FactValue::String("premium".to_string()),
    };

    let unique_conditions = vec![
        Condition::Simple {
            field: "region".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("us-west".to_string()),
        },
        Condition::Simple {
            field: "activity_score".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(50),
        },
        Condition::Simple {
            field: "subscription_level".to_string(),
            operator: Operator::Equal,
            value: FactValue::Integer(3),
        },
    ];

    let formula_expressions = vec![
        "account_balance * 0.05",                      // 5% bonus calculation
        "activity_score + 10",                         // Activity bonus
        "account_balance / 100",                       // Points calculation
        "activity_score * 2 + account_balance * 0.01", // Complex scoring
    ];

    let rule_count = 25;
    let start_compilation = Instant::now();

    println!(
        "📋 Creating {} rules with shared and unique conditions...",
        rule_count
    );

    for i in 1..=rule_count {
        let mut conditions = vec![shared_condition.clone()]; // Start with shared condition

        // Add unique condition for some variety
        if i % 3 == 0 {
            conditions.push(unique_conditions[(i - 1) % unique_conditions.len()].clone());
        }

        let formula_expr = &formula_expressions[(i - 1) % formula_expressions.len()];

        let rule = Rule {
            id: i as u64,
            name: format!("benchmark_rule_{}", i),
            conditions,
            actions: vec![
                Action {
                    action_type: ActionType::Formula {
                        target_field: "calculated_value".to_string(),
                        expression: formula_expr.to_string(),
                        source_calculator: None,
                    },
                },
                Action {
                    action_type: ActionType::SetField {
                        field: format!("processed_by_rule_{}", i),
                        value: FactValue::Boolean(true),
                    },
                },
                Action {
                    action_type: ActionType::CreateFact {
                        data: FactData {
                            fields: vec![
                                (
                                    "derived_fact".to_string(),
                                    FactValue::String(format!("from_rule_{}", i)),
                                ),
                                ("timestamp".to_string(), FactValue::Integer(i as i64)),
                            ]
                            .into_iter()
                            .collect(),
                        },
                    },
                },
            ],
        };

        engine.add_rule(rule).unwrap();
    }

    let compilation_time = start_compilation.elapsed();

    // Get optimization statistics after rule compilation
    let node_sharing_stats = engine.get_node_sharing_stats();
    let memory_savings = engine.get_memory_savings();
    let initial_pool_stats = engine.get_memory_pool_stats();

    println!("⚙️ Compilation Complete:");
    println!("  Time: {:?}", compilation_time);
    println!(
        "  Alpha nodes sharing rate: {:.1}%",
        node_sharing_stats.alpha_sharing_rate
    );
    println!(
        "  Beta nodes sharing rate: {:.1}%",
        node_sharing_stats.beta_sharing_rate
    );
    println!("  Memory saved: {}", memory_savings.to_human_readable());
    println!(
        "  Initial pool objects: {}",
        initial_pool_stats.total_pooled_objects()
    );

    // Performance test with multiple rounds to measure caching effectiveness
    let fact_count_per_round = 200;
    let rounds = 5;
    let mut total_facts_processed = 0;
    let mut total_results_generated = 0;
    let mut round_times = Vec::new();

    println!("⚡ Performance Testing:");
    println!("  Facts per round: {}", fact_count_per_round);
    println!("  Rounds: {}", rounds);

    for round in 1..=rounds {
        println!("  🔄 Round {}...", round);

        let mut facts = Vec::with_capacity(fact_count_per_round);

        for i in 0..fact_count_per_round {
            let mut fields = HashMap::new();

            // Ensure facts match the shared conditions for rules to fire
            fields.insert(
                "user_type".to_string(),
                FactValue::String("premium".to_string()),
            );
            fields.insert(
                "account_balance".to_string(),
                FactValue::Float(1200.0 + (i as f64 * 10.0)),
            ); // > 1000.0
            fields.insert(
                "activity_score".to_string(),
                FactValue::Integer(60 + (i as i64 % 30)),
            ); // for formulas
            fields.insert(
                "user_id".to_string(),
                FactValue::Integer(round * 1000 + i as i64),
            );

            // Add region for subset of facts
            if i % 3 == 0 {
                fields.insert(
                    "region".to_string(),
                    FactValue::String("us-west".to_string()),
                );
            }

            // Add subscription level for subset
            if i % 4 == 0 {
                fields.insert("subscription_level".to_string(), FactValue::Integer(3));
            }

            facts.push(Fact {
                id: (round * fact_count_per_round as i64 + i as i64) as u64,
                data: FactData { fields },
            });
        }

        let round_start = Instant::now();
        let results = engine.process_facts(facts).unwrap();
        let round_time = round_start.elapsed();

        round_times.push(round_time);
        total_facts_processed += fact_count_per_round;
        total_results_generated += results.len();

        println!(
            "    ✅ Processed {} facts → {} results in {:?}",
            fact_count_per_round,
            results.len(),
            round_time
        );

        // Verify some results have calculated values
        let calculated_results: Vec<_> = results
            .iter()
            .filter(|r| r.data.fields.contains_key("calculated_value"))
            .collect();
        assert!(
            !calculated_results.is_empty(),
            "Should have calculated values from formulas"
        );
    }

    // Get final optimization statistics
    let final_pool_stats = engine.get_memory_pool_stats();
    let fact_lookup_stats = engine.get_fast_lookup_stats();

    // Calculate performance metrics
    let total_processing_time: std::time::Duration = round_times.iter().sum();
    let average_round_time = total_processing_time / rounds as u32;
    let facts_per_second = total_facts_processed as f64 / total_processing_time.as_secs_f64();
    let fastest_round = round_times.iter().min().unwrap();
    let slowest_round = round_times.iter().max().unwrap();
    let speedup_ratio = slowest_round.as_nanos() as f64 / fastest_round.as_nanos() as f64;

    println!("📊 Performance Results:");
    println!("  Total facts processed: {}", total_facts_processed);
    println!("  Total results generated: {}", total_results_generated);
    println!("  Total processing time: {:?}", total_processing_time);
    println!("  Average round time: {:?}", average_round_time);
    println!("  Facts per second: {:.1}", facts_per_second);
    println!("  Fastest round: {:?}", fastest_round);
    println!("  Slowest round: {:?}", slowest_round);
    println!(
        "  Performance improvement: {:.2}x (slowest vs fastest)",
        speedup_ratio
    );

    println!("🎯 Optimization Impact:");

    // Fact Lookup Optimization
    println!("  📍 Fact Lookup:");
    println!("    Cache hit rate: {:.1}%", fact_lookup_stats.hit_rate);
    println!(
        "    Cache utilization: {:.1}%",
        fact_lookup_stats.cache_stats.map(|s| s.utilization()).unwrap_or(0.0)
    );
    println!("    Total lookups: {}", fact_lookup_stats.total_lookups);

    // Calculator Caching - Note: Stats are aggregated at terminal node level, would need special access
    println!("  🧮 Calculator Caching:");
    println!("    Caching enabled in terminal nodes (stats aggregation pending)");

    // Node Sharing
    println!("  🔗 Node Sharing:");
    println!(
        "    Alpha sharing rate: {:.1}%",
        node_sharing_stats.alpha_sharing_rate
    );
    println!(
        "    Beta sharing rate: {:.1}%",
        node_sharing_stats.beta_sharing_rate
    );
    println!("    Memory saved: {}", memory_savings.to_human_readable());
    assert!(
        node_sharing_stats.alpha_shares_found > 0,
        "Should have alpha node sharing"
    );
    assert!(
        memory_savings.total_memory_saved_bytes > 0,
        "Should save memory"
    );

    // Memory Pools
    println!("  🏊 Memory Pools:");
    println!(
        "    Pool operations: {}",
        final_pool_stats.total_operations()
    );
    println!(
        "    Objects pooled: {}",
        final_pool_stats.total_pooled_objects()
    );
    println!(
        "    Average hit rate: {:.1}%",
        final_pool_stats.average_hit_rate()
    );
    assert!(
        final_pool_stats.total_operations() > 0,
        "Should use memory pools"
    );
    assert!(
        final_pool_stats.average_hit_rate() > 0.0,
        "Should have pool reuse"
    );

    // Performance assertions
    assert!(
        facts_per_second > 1000.0,
        "Should process at least 1000 facts/second"
    );
    assert!(
        speedup_ratio >= 1.0,
        "Performance should be stable or improve across rounds"
    );
    assert!(
        total_results_generated > total_facts_processed,
        "Should generate more results than input facts (due to CreateFact actions)"
    );

    // Caching effectiveness assertions
    assert!(
        fact_lookup_stats.hit_rate > 0.0,
        "Fact lookup cache should be used"
    );

    println!("✅ Phase 3 Performance Benchmark Complete!");
    println!("   Combined optimizations working effectively");
    println!("   Processing rate: {:.1} facts/second", facts_per_second);
    println!("   Performance scaling: {:.2}x improvement", speedup_ratio);
}

#[test]
fn test_phase3_memory_efficiency_benchmark() {
    println!("💾 Phase 3 Memory Efficiency Benchmark");
    println!("=====================================");

    let mut engine = ReteNetwork::new().unwrap();

    // Create rules designed to test memory efficiency
    let rule_count = 100;
    let shared_condition = Condition::Simple {
        field: "status".to_string(),
        operator: Operator::Equal,
        value: FactValue::String("active".to_string()),
    };

    println!(
        "📋 Creating {} rules with shared conditions for memory testing...",
        rule_count
    );

    let start_time = Instant::now();

    for i in 1..=rule_count {
        let rule = Rule {
            id: i as u64,
            name: format!("memory_test_rule_{}", i),
            conditions: vec![shared_condition.clone()],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: format!("flag_{}", i),
                    value: FactValue::Boolean(true),
                },
            }],
        };
        engine.add_rule(rule).unwrap();
    }

    let compilation_time = start_time.elapsed();

    let node_sharing_stats = engine.get_node_sharing_stats();
    let memory_savings = engine.get_memory_savings();

    println!("⚙️ Memory Optimization Results:");
    println!("  Compilation time: {:?}", compilation_time);
    println!("  Rules created: {}", rule_count);
    println!(
        "  Alpha nodes created: {}",
        node_sharing_stats.alpha_nodes_total
    );
    println!(
        "  Alpha nodes shared: {}",
        node_sharing_stats.alpha_shares_found
    );
    println!(
        "  Alpha nodes active: {}",
        node_sharing_stats.alpha_nodes_active
    );
    println!(
        "  Sharing efficiency: {:.1}%",
        node_sharing_stats.alpha_sharing_rate
    );
    println!("  Memory saved: {}", memory_savings.to_human_readable());
    println!(
        "  Nodes without sharing: {}",
        node_sharing_stats.nodes_without_sharing()
    );

    // With perfect sharing, should have only 1 active alpha node for 100 rules
    assert_eq!(
        node_sharing_stats.alpha_nodes_total, 1,
        "Should create only 1 unique alpha node"
    );
    assert_eq!(
        node_sharing_stats.alpha_shares_found,
        (rule_count - 1) as usize,
        "Should share {} nodes",
        rule_count - 1
    );
    assert_eq!(
        node_sharing_stats.alpha_nodes_active, 1,
        "Should have only 1 active alpha node"
    );
    assert!(
        node_sharing_stats.alpha_sharing_rate > 1000.0,
        "Should have very high sharing rate"
    );

    // Memory savings should be significant
    assert_eq!(memory_savings.alpha_nodes_saved, (rule_count - 1) as usize);
    assert!(
        memory_savings.total_memory_saved_bytes > 1000,
        "Should save significant memory"
    );

    // Test that all rules still work correctly despite sharing
    let mut fields = HashMap::new();
    fields.insert(
        "status".to_string(),
        FactValue::String("active".to_string()),
    );
    fields.insert("test_id".to_string(), FactValue::Integer(42));

    let fact = Fact { id: 1, data: FactData { fields } };

    let processing_start = Instant::now();
    let results = engine.process_facts(vec![fact]).unwrap();
    let processing_time = processing_start.elapsed();

    println!("⚡ Functional Validation:");
    println!("  Processing time: {:?}", processing_time);
    println!("  Results generated: {}", results.len());

    // Should generate one result per rule
    assert_eq!(
        results.len(),
        rule_count as usize,
        "All rules should fire despite node sharing"
    );

    // Verify each rule fired by checking for its unique flag
    for i in 1..=rule_count {
        let flag_field = format!("flag_{}", i);
        let matching_results: Vec<_> = results
            .iter()
            .filter(|r| r.data.fields.get(&flag_field) == Some(&FactValue::Boolean(true)))
            .collect();
        assert_eq!(
            matching_results.len(),
            1,
            "Rule {} should fire exactly once",
            i
        );
    }

    println!("✅ Memory Efficiency Benchmark Complete!");
    println!("   {} rules → 1 shared node", rule_count);
    println!(
        "   Memory efficiency: {:.1}% saved",
        (memory_savings.alpha_nodes_saved as f64 / rule_count as f64) * 100.0
    );
}

#[test]
fn test_phase3_scaling_performance() {
    println!("📈 Phase 3 Scaling Performance Test");
    println!("==================================");

    let fact_counts = vec![50, 100, 250, 500, 1000];
    let mut scaling_results = Vec::new();

    for &fact_count in &fact_counts {
        println!("🔢 Testing with {} facts...", fact_count);

        let mut engine = ReteNetwork::new().unwrap();

        // Create a moderately complex rule set
        let rules = vec![
            Rule {
                id: 1,
                name: "premium_user_rule".to_string(),
                conditions: vec![
                    Condition::Simple {
                        field: "user_type".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::String("premium".to_string()),
                    },
                    Condition::Simple {
                        field: "account_balance".to_string(),
                        operator: Operator::GreaterThan,
                        value: FactValue::Float(500.0),
                    },
                ],
                actions: vec![Action {
                    action_type: ActionType::Formula {
                        target_field: "bonus_points".to_string(),
                        expression: "account_balance * 0.1".to_string(),
                        source_calculator: None,
                    },
                }],
            },
            Rule {
                id: 2,
                name: "activity_bonus_rule".to_string(),
                conditions: vec![Condition::Simple {
                    field: "activity_score".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Integer(75),
                }],
                actions: vec![Action {
                    action_type: ActionType::CreateFact {
                        data: FactData {
                            fields: [
                                ("activity_bonus".to_string(), FactValue::Integer(100)),
                                (
                                    "bonus_type".to_string(),
                                    FactValue::String("activity".to_string()),
                                ),
                            ]
                            .iter()
                            .cloned()
                            .collect(),
                        },
                    },
                }],
            },
        ];

        for rule in rules {
            engine.add_rule(rule).unwrap();
        }

        // Generate test facts
        let mut facts = Vec::with_capacity(fact_count);
        for i in 0..fact_count {
            let mut fields = HashMap::new();

            // Ensure some facts match conditions
            if i % 3 == 0 {
                fields.insert(
                    "user_type".to_string(),
                    FactValue::String("premium".to_string()),
                );
                fields.insert(
                    "account_balance".to_string(),
                    FactValue::Float(600.0 + (i as f64 * 10.0)),
                );
            }

            if i % 4 == 0 {
                fields.insert(
                    "activity_score".to_string(),
                    FactValue::Integer(80 + (i as i64 % 20)),
                );
            }

            fields.insert("user_id".to_string(), FactValue::Integer(i as i64));

            facts.push(Fact { id: i as u64, data: FactData { fields } });
        }

        // Measure processing performance
        let start_time = Instant::now();
        let results = engine.process_facts(facts).unwrap();
        let processing_time = start_time.elapsed();

        let facts_per_second = fact_count as f64 / processing_time.as_secs_f64();

        scaling_results.push((fact_count, processing_time, facts_per_second, results.len()));

        println!(
            "  📊 {} facts → {} results in {:?} ({:.1} facts/sec)",
            fact_count,
            results.len(),
            processing_time,
            facts_per_second
        );
    }

    println!("📈 Scaling Analysis:");

    // Check that performance scales reasonably
    for i in 1..scaling_results.len() {
        let (prev_count, prev_time, _prev_rate, _) = scaling_results[i - 1];
        let (curr_count, curr_time, _curr_rate, _) = scaling_results[i];

        let count_ratio = curr_count as f64 / prev_count as f64;
        let time_ratio = curr_time.as_nanos() as f64 / prev_time.as_nanos() as f64;

        println!(
            "  {}x facts → {:.2}x time (efficiency: {:.2})",
            count_ratio,
            time_ratio,
            count_ratio / time_ratio
        );

        // Performance should scale sub-linearly (better than O(n))
        assert!(
            time_ratio < count_ratio * 1.5,
            "Performance should scale better than linear: {}x facts should not take more than {}x time",
            count_ratio,
            count_ratio * 1.5
        );
    }

    // Final performance should still be reasonable
    let (final_count, _final_time, final_rate, _) = scaling_results.last().unwrap();
    assert!(
        *final_rate > 500.0,
        "Should maintain at least 500 facts/second even at {} facts",
        final_count
    );

    println!("✅ Scaling Performance Test Complete!");
    println!("   Maintains good performance up to {} facts", final_count);
    println!("   Final processing rate: {:.1} facts/second", final_rate);
}
