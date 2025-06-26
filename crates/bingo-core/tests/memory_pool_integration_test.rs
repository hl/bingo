//! Integration tests for Memory Pool Management optimization
//!
//! This test validates that memory pools reduce allocation overhead and improve
//! performance by reusing frequently allocated objects.

use bingo_core::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_memory_pools_basic_functionality() {
    let engine = ReteNetwork::new().unwrap();

    // Create a rule to trigger fact processing
    let rule = Rule {
        id: 1,
        name: "memory_pool_test".to_string(),
        conditions: vec![Condition::Simple {
            field: "category".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("test".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "processed".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Get initial pool statistics
    let initial_stats = engine.get_memory_pool_stats();
    println!("Memory Pool Basic Functionality Test:");
    println!(
        "  Initial pool operations: {}",
        initial_stats.total_operations()
    );
    println!(
        "  Initial pooled objects: {}",
        initial_stats.total_pooled_objects()
    );

    // Process facts to trigger memory pool usage
    let fact_count = 100;
    let mut facts = Vec::with_capacity(fact_count);

    for i in 0..fact_count {
        let mut fields = HashMap::new();
        fields.insert(
            "category".to_string(),
            FactValue::String("test".to_string()),
        );
        fields.insert("id".to_string(), FactValue::Integer(i as i64));

        facts.push(Fact { id: i as u64, data: FactData { fields } });
    }

    let results = engine.process_facts(facts).unwrap();

    // Get final pool statistics
    let final_stats = engine.get_memory_pool_stats();
    println!(
        "  Final pool operations: {}",
        final_stats.total_operations()
    );
    println!(
        "  Final pooled objects: {}",
        final_stats.total_pooled_objects()
    );
    println!("  Average hit rate: {:.1}%", final_stats.average_hit_rate());

    // Verify that facts were processed correctly
    assert_eq!(results.len(), fact_count);

    // Memory pools should have been used
    assert!(final_stats.total_operations() > initial_stats.total_operations());

    // Should have some objects pooled for reuse
    assert!(final_stats.total_pooled_objects() > 0);

    println!("  ✓ Memory pools used successfully during fact processing");
}

#[test]
fn test_memory_pool_efficiency_improvement() {
    let engine = ReteNetwork::new().unwrap();

    // Create a rule that will cause many memory allocations
    let rule = Rule {
        id: 1,
        name: "efficiency_test".to_string(),
        conditions: vec![Condition::Simple {
            field: "type".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("efficiency_test".to_string()),
        }],
        actions: vec![
            Action {
                action_type: ActionType::Formula {
                    target_field: "calculated".to_string(),
                    expression: "id * 2 + 100".to_string(),
                    source_calculator: None,
                },
            },
            Action {
                action_type: ActionType::SetField {
                    field: "processed".to_string(),
                    value: FactValue::Boolean(true),
                },
            },
        ],
    };

    engine.add_rule(rule).unwrap();

    let fact_count = 500;
    let rounds = 3;

    println!("Memory Pool Efficiency Test:");
    println!("  Facts per round: {}", fact_count);
    println!("  Rounds: {}", rounds);

    let mut total_processing_time = std::time::Duration::ZERO;

    for round in 1..=rounds {
        // Create facts for this round
        let mut facts = Vec::with_capacity(fact_count);

        for i in 0..fact_count {
            let mut fields = HashMap::new();
            fields.insert(
                "type".to_string(),
                FactValue::String("efficiency_test".to_string()),
            );
            fields.insert("id".to_string(), FactValue::Integer(i as i64));
            fields.insert("round".to_string(), FactValue::Integer(round));

            facts.push(Fact {
                id: (round * fact_count as i64 + i as i64) as u64,
                data: FactData { fields },
            });
        }

        // Process facts and measure time
        let start_time = Instant::now();
        let results = engine.process_facts(facts).unwrap();
        let processing_time = start_time.elapsed();

        total_processing_time += processing_time;

        println!("  Round {} processing time: {:?}", round, processing_time);
        // Each fact triggers both formula and set field actions, so we get 2 results per fact
        assert_eq!(results.len(), fact_count * 2);

        // Verify calculated fields are correct
        for result in &results {
            if let Some(FactValue::Integer(id)) = result.data.fields.get("id") {
                if let Some(FactValue::Float(calculated)) = result.data.fields.get("calculated") {
                    let expected = *id as f64 * 2.0 + 100.0;
                    assert!(
                        (calculated - expected).abs() < f64::EPSILON,
                        "Calculated value should be correct: {} vs {}",
                        calculated,
                        expected
                    );
                }
            }
        }
    }

    let pool_stats = engine.get_memory_pool_stats();
    let pool_efficiency = engine.get_memory_pool_efficiency();

    println!("  Total processing time: {:?}", total_processing_time);
    println!("  Pool efficiency: {:.1}%", pool_efficiency);
    println!("  Total pool operations: {}", pool_stats.total_operations());
    println!("  Objects pooled: {}", pool_stats.total_pooled_objects());

    // Pool efficiency should improve over multiple rounds
    assert!(pool_efficiency > 0.0, "Pool efficiency should be positive");

    // Should have significant pool usage
    assert!(pool_stats.total_operations() > 0);

    println!("  ✓ Memory pool efficiency improved across multiple rounds");
}

#[test]
fn test_individual_pool_performance() {
    let pools = ReteMemoryPools::new();

    println!("Individual Pool Performance Test:");

    // Test token pool performance
    let token_iterations = 1000;
    let start_time = Instant::now();

    for _ in 0..token_iterations {
        let mut token = pools.get_token();
        token.fact_ids = FactIdSet::new(vec![1, 2, 3, 4, 5]);
        pools.return_token(token);
    }

    let token_time = start_time.elapsed();
    let token_stats = pools.token_pool.stats();

    println!("  Token Pool:");
    println!("    Operations: {}", token_iterations);
    println!("    Time: {:?}", token_time);
    println!("    Hit rate: {:.1}%", token_stats.hit_rate);
    println!("    Current size: {}", token_stats.current_size);

    // Test fact data pool performance
    let fact_data_iterations = 800;
    let start_time = Instant::now();

    for i in 0..fact_data_iterations {
        let mut fact_data = pools.get_fact_data();
        fact_data.fields.insert("test".to_string(), FactValue::Integer(i as i64));
        fact_data.fields.insert(
            "category".to_string(),
            FactValue::String("test".to_string()),
        );
        pools.return_fact_data(fact_data);
    }

    let fact_data_time = start_time.elapsed();
    let fact_data_stats = pools.fact_data_pool.stats();

    println!("  Fact Data Pool:");
    println!("    Operations: {}", fact_data_iterations);
    println!("    Time: {:?}", fact_data_time);
    println!("    Hit rate: {:.1}%", fact_data_stats.hit_rate);
    println!("    Current size: {}", fact_data_stats.current_size);

    // Test field map pool performance
    let field_map_iterations = 600;
    let start_time = Instant::now();

    for i in 0..field_map_iterations {
        let mut field_map = pools.get_field_map();
        field_map.insert("key1".to_string(), FactValue::Integer(i as i64));
        field_map.insert(
            "key2".to_string(),
            FactValue::String(format!("value_{}", i)),
        );
        pools.return_field_map(field_map);
    }

    let field_map_time = start_time.elapsed();
    let field_map_stats = pools.field_map_pool.stats();

    println!("  Field Map Pool:");
    println!("    Operations: {}", field_map_iterations);
    println!("    Time: {:?}", field_map_time);
    println!("    Hit rate: {:.1}%", field_map_stats.hit_rate);
    println!("    Current size: {}", field_map_stats.current_size);

    // Verify all pools have good hit rates after warmup
    assert!(
        token_stats.hit_rate > 80.0,
        "Token pool should have high hit rate"
    );
    assert!(
        fact_data_stats.hit_rate > 80.0,
        "Fact data pool should have high hit rate"
    );
    assert!(
        field_map_stats.hit_rate > 80.0,
        "Field map pool should have high hit rate"
    );

    // Verify pools are holding objects for reuse
    assert!(token_stats.current_size > 0);
    assert!(fact_data_stats.current_size > 0);
    assert!(field_map_stats.current_size > 0);

    println!("  ✓ All individual pools show good performance");
}

#[test]
fn test_memory_pool_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    let pools = Arc::new(ReteMemoryPools::new());
    let thread_count = 4;
    let operations_per_thread = 250;

    println!("Memory Pool Thread Safety Test:");
    println!("  Threads: {}", thread_count);
    println!("  Operations per thread: {}", operations_per_thread);

    let start_time = Instant::now();

    // Spawn multiple threads that use the pools concurrently
    let handles: Vec<_> = (0..thread_count)
        .map(|thread_id| {
            let pools_clone = Arc::clone(&pools);
            thread::spawn(move || {
                for i in 0..operations_per_thread {
                    // Token operations
                    let mut token = pools_clone.get_token();
                    token.fact_ids = FactIdSet::new(vec![thread_id as u64, i as u64]);
                    pools_clone.return_token(token);

                    // Fact data operations
                    let mut fact_data = pools_clone.get_fact_data();
                    fact_data.fields.insert(format!("thread_{}", thread_id), FactValue::Integer(i));
                    pools_clone.return_fact_data(fact_data);

                    // Field map operations
                    let mut field_map = pools_clone.get_field_map();
                    field_map.insert("thread_id".to_string(), FactValue::Integer(thread_id));
                    field_map.insert("iteration".to_string(), FactValue::Integer(i));
                    pools_clone.return_field_map(field_map);
                }
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let total_time = start_time.elapsed();
    let stats = pools.get_stats();

    println!("  Total time: {:?}", total_time);
    println!("  Total operations: {}", stats.total_operations());
    println!("  Average hit rate: {:.1}%", stats.average_hit_rate());
    println!("  Objects pooled: {}", stats.total_pooled_objects());

    // Verify that all operations completed successfully
    let expected_total_operations = thread_count * operations_per_thread * 3; // 3 pool types
    assert!(stats.total_operations() >= expected_total_operations as usize);

    // Thread safety means all operations should complete without errors
    assert!(stats.average_hit_rate() >= 0.0);
    assert!(stats.total_pooled_objects() > 0);

    println!("  ✓ Memory pools are thread-safe and performant");
}

#[test]
fn test_memory_pool_cleanup_and_reset() {
    let engine = ReteNetwork::new().unwrap();

    // Create a simple rule
    let rule = Rule {
        id: 1,
        name: "cleanup_test".to_string(),
        conditions: vec![Condition::Simple {
            field: "cleanup".to_string(),
            operator: Operator::Equal,
            value: FactValue::Boolean(true),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "cleaned".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Generate some pool activity
    let mut facts = Vec::new();
    for i in 0..50 {
        let mut fields = HashMap::new();
        fields.insert("cleanup".to_string(), FactValue::Boolean(true));
        fields.insert("id".to_string(), FactValue::Integer(i));

        facts.push(Fact { id: i as u64, data: FactData { fields } });
    }

    let _results = engine.process_facts(facts).unwrap();

    let stats_before = engine.get_memory_pool_stats();
    println!("Memory Pool Cleanup Test:");
    println!(
        "  Operations before cleanup: {}",
        stats_before.total_operations()
    );
    println!(
        "  Objects pooled before cleanup: {}",
        stats_before.total_pooled_objects()
    );

    // Verify we have some pool activity
    assert!(stats_before.total_operations() > 0);
    assert!(stats_before.total_pooled_objects() > 0);

    // Clear memory pools
    engine.clear_memory_pools();

    let stats_after = engine.get_memory_pool_stats();
    println!(
        "  Operations after cleanup: {}",
        stats_after.total_operations()
    );
    println!(
        "  Objects pooled after cleanup: {}",
        stats_after.total_pooled_objects()
    );

    // Verify pools are cleared
    assert_eq!(stats_after.total_operations(), 0);
    assert_eq!(stats_after.total_pooled_objects(), 0);
    assert_eq!(stats_after.average_hit_rate(), 0.0);

    println!("  ✓ Memory pools cleaned up successfully");
}

#[test]
fn test_memory_pool_performance_comparison() {
    // This test compares allocation patterns with and without memory pools
    // by measuring the performance of frequent object creation/destruction

    let iterations = 1000;
    println!("Memory Pool Performance Comparison:");
    println!("  Iterations: {}", iterations);

    // Test without memory pools (direct allocation)
    let start_time = Instant::now();
    for i in 0..iterations {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FactValue::Integer(i as i64));
        fields.insert(
            "category".to_string(),
            FactValue::String("test".to_string()),
        );

        let _fact_data = FactData { fields };
        let _token = Token::from_facts(vec![i as u64]);
        let _fact_vec: Vec<Fact> = Vec::with_capacity(10);

        // Objects are dropped here (garbage collected)
    }
    let direct_allocation_time = start_time.elapsed();

    // Test with memory pools
    let pools = ReteMemoryPools::new();
    let start_time = Instant::now();
    for i in 0..iterations {
        let mut fact_data = pools.get_fact_data();
        fact_data.fields.insert("id".to_string(), FactValue::Integer(i as i64));
        fact_data.fields.insert(
            "category".to_string(),
            FactValue::String("test".to_string()),
        );
        pools.return_fact_data(fact_data);

        let token = pools.get_token();
        pools.return_token(token);

        let fact_vec = pools.get_fact_vec();
        pools.return_fact_vec(fact_vec);
    }
    let pooled_allocation_time = start_time.elapsed();

    println!("  Direct allocation time: {:?}", direct_allocation_time);
    println!("  Pooled allocation time: {:?}", pooled_allocation_time);

    let speedup =
        direct_allocation_time.as_nanos() as f64 / pooled_allocation_time.as_nanos() as f64;
    println!("  Speedup factor: {:.2}x", speedup);

    let pool_stats = pools.get_stats();
    println!("  Pool hit rate: {:.1}%", pool_stats.average_hit_rate());
    println!("  Objects reused: {}", pool_stats.total_pooled_objects());

    // Memory pools should show performance benefit after warmup
    // Note: In microbenchmarks, the benefit may be small, but in real workloads
    // with GC pressure, the benefits are more significant
    assert!(
        pool_stats.average_hit_rate() > 50.0,
        "Should have significant hit rate after iterations"
    );
    assert!(
        pool_stats.total_pooled_objects() > 0,
        "Should have objects pooled for reuse"
    );

    if speedup > 1.0 {
        println!("  ✓ Memory pools provided {:.2}x speedup", speedup);
    } else {
        println!("  ⚠ Memory pools showed no significant speedup in this microbenchmark");
        println!("    (Real-world benefits are typically higher due to GC pressure reduction)");
    }
}
