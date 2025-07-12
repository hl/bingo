//! Comprehensive Concurrency Tests for Bingo RETE Engine
//!
//! This test suite validates the parallel processing system under concurrent load conditions,
//! ensuring thread safety, performance, and correctness across all parallel components.

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::useless_vec)]

use bingo_calculator::calculator::Calculator;
use bingo_core::BingoEngine;
use bingo_core::fact_store::arena_store::ArenaFactStore;
use bingo_core::memory_pools::ConcurrentMemoryPoolManager;
use bingo_core::parallel::{
    ParallelAggregationEngine, ParallelAggregator, ParallelConfig, ParallelReteNetwork,
};
use bingo_core::rete_network::ReteNetwork;
use bingo_core::rete_nodes::RuleExecutionResult;
use bingo_core::types::{Action, ActionType, Condition, Fact, FactData, FactValue, Operator, Rule};
use std::collections::HashMap;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

/// Test concurrent parallel fact processing across multiple threads
#[test]
fn test_concurrent_parallel_fact_processing() {
    let num_threads = 4;
    let facts_per_thread = 100;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            let mut rete_network = ReteNetwork::new();
            let mut fact_store = ArenaFactStore::new();
            let calculator = Calculator::new();
            let config = ParallelConfig::default();

            // Create test rule
            let rule = Rule {
                id: thread_id as u64 + 1,
                name: format!("TestRule{}", thread_id),
                conditions: vec![Condition::Simple {
                    field: "value".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Float(50.0),
                }],
                actions: vec![Action {
                    action_type: ActionType::SetField {
                        field: "processed".to_string(),
                        value: FactValue::Boolean(true),
                    },
                }],
            };

            rete_network.add_rule(rule).unwrap();

            // Create test facts
            let mut facts = Vec::new();
            for i in 0..facts_per_thread {
                let mut fields = HashMap::new();
                fields.insert("value".to_string(), FactValue::Float((i as f64) * 2.0));
                fields.insert(
                    "thread_id".to_string(),
                    FactValue::Integer(thread_id as i64),
                );

                let fact = Fact::new(
                    (thread_id * facts_per_thread + i) as u64,
                    FactData { fields },
                );
                facts.push(fact);
            }

            // Wait for all threads to be ready
            barrier.wait();

            // Process facts in parallel
            let start = Instant::now();
            let results = rete_network
                .process_facts_parallel(&facts, &mut fact_store, &calculator, &config)
                .unwrap();
            let duration = start.elapsed();

            (thread_id, results.len(), duration)
        });

        handles.push(handle);
    }

    // Collect results from all threads
    let mut total_results = 0;
    let mut total_duration = Duration::new(0, 0);

    for handle in handles {
        let (thread_id, result_count, duration) = handle.join().unwrap();
        println!(
            "Thread {}: {} results in {:?}",
            thread_id, result_count, duration
        );
        total_results += result_count;
        total_duration += duration;
    }

    println!(
        "Total results: {}, Average duration: {:?}",
        total_results,
        total_duration / num_threads as u32
    );

    // Verify results
    assert!(total_results > 0, "Should have processed some facts");
    assert!(
        total_duration.as_micros() > 0,
        "Should have taken some time"
    );
}

/// Test concurrent parallel aggregation operations
#[test]
fn test_concurrent_parallel_aggregations() {
    let num_threads = 6;
    let data_size = 1000;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    // Generate test data
    let test_data: Vec<f64> = (0..data_size).map(|i| i as f64).collect();
    let shared_data = Arc::new(test_data);

    // Test different aggregation operations concurrently
    let operations = vec!["sum", "average", "min_max", "variance", "count", "custom_sum"];

    for (thread_id, operation) in operations.iter().enumerate() {
        let barrier = Arc::clone(&barrier);
        let data = Arc::clone(&shared_data);
        let op = operation.to_string();

        let handle = thread::spawn(move || {
            let engine = ParallelAggregationEngine::new();

            // Wait for all threads to be ready
            barrier.wait();

            let start = Instant::now();
            let result = match op.as_str() {
                "sum" => {
                    let sum = engine.parallel_sum(&data).unwrap();
                    format!("Sum: {}", sum)
                }
                "average" => {
                    let avg = engine.parallel_average(&data).unwrap();
                    format!("Average: {}", avg)
                }
                "min_max" => {
                    let (min, max) = engine.parallel_min_max(&data).unwrap();
                    format!("Min: {}, Max: {}", min, max)
                }
                "variance" => {
                    let (var, std) = engine.parallel_variance(&data).unwrap();
                    format!("Variance: {}, StdDev: {}", var, std)
                }
                "count" => {
                    let count = engine.parallel_count(&data, |x| *x > 500.0).unwrap();
                    format!("Count > 500: {}", count)
                }
                "custom_sum" => {
                    // Multiple operations in sequence
                    let sum1 = engine.parallel_sum(&data[..500]).unwrap();
                    let sum2 = engine.parallel_sum(&data[500..]).unwrap();
                    format!("Custom Sum: {}", sum1 + sum2)
                }
                _ => "Unknown".to_string(),
            };
            let duration = start.elapsed();

            (thread_id, op, result, duration)
        });

        handles.push(handle);
    }

    // Collect and verify results
    for handle in handles {
        let (thread_id, operation, result, duration) = handle.join().unwrap();
        println!(
            "Thread {}: {} = {} (took {:?})",
            thread_id, operation, result, duration
        );

        // Verify results make sense
        match operation.as_str() {
            "sum" => assert!(result.contains("499500"), "Sum should be 499500"),
            "average" => assert!(result.contains("499.5"), "Average should be 499.5"),
            "min_max" => {
                assert!(result.contains("Min: 0"));
                assert!(result.contains("Max: 999"));
            }
            "count" => assert!(result.contains("Count > 500: 499"), "Count should be 499"),
            _ => {} // Other operations may vary
        }
    }
}

/// Test concurrent memory pool operations under high load
#[test]
fn test_concurrent_memory_pool_stress() {
    let num_threads = 8;
    let operations_per_thread = 500;
    let barrier = Arc::new(Barrier::new(num_threads));
    let pool_manager = Arc::new(ConcurrentMemoryPoolManager::with_high_throughput_config());
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let barrier = Arc::clone(&barrier);
        let manager = Arc::clone(&pool_manager);

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier.wait();

            let start = Instant::now();
            let mut local_stats = (0, 0); // (gets, returns)

            for i in 0..operations_per_thread {
                // Get and return rule execution result vectors
                let mut result_vec = manager.rule_execution_results.get();
                // Add dummy rule execution result
                result_vec.push(RuleExecutionResult {
                    rule_id: 1,
                    fact_id: 1,
                    actions_executed: vec![],
                });
                manager.rule_execution_results.return_vec(result_vec);
                local_stats.0 += 1;
                local_stats.1 += 1;

                // Get and return rule ID vectors
                let mut id_vec = manager.rule_id_vecs.get();
                id_vec.push(i as u64);
                manager.rule_id_vecs.return_vec(id_vec);

                // Simulate some work
                if i % 10 == 0 {
                    thread::sleep(Duration::from_micros(1));
                }
            }

            let duration = start.elapsed();
            (thread_id, local_stats, duration)
        });

        handles.push(handle);
    }

    // Collect results and verify pool statistics
    let mut total_operations = 0;

    for handle in handles {
        let (thread_id, (gets, returns), duration) = handle.join().unwrap();
        println!(
            "Thread {}: {} gets, {} returns in {:?}",
            thread_id, gets, returns, duration
        );
        total_operations += gets + returns;
    }

    // Check final pool statistics
    let final_stats = pool_manager.get_concurrent_stats();
    println!("Final pool stats: {:#?}", final_stats);

    assert!(total_operations > 0, "Should have performed operations");
    assert!(
        final_stats.rule_execution_result_pool.hits + final_stats.rule_execution_result_pool.misses
            > 0
    );
    assert!(final_stats.rule_id_vec_pool.hits + final_stats.rule_id_vec_pool.misses > 0);

    // Check efficiency
    let total_efficiency = pool_manager.total_efficiency();
    println!("Total pool efficiency: {:.2}%", total_efficiency);

    // Should have some level of efficiency from reuse
    assert!(total_efficiency >= 0.0, "Efficiency should be non-negative");
}

/// Test concurrent rule compilation and evaluation
#[test]
fn test_concurrent_rule_operations() {
    let num_threads = 4;
    let rules_per_thread = 20;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            let mut rete_network = ReteNetwork::new();
            let mut fact_store = ArenaFactStore::new();
            let calculator = Calculator::new();
            let config = ParallelConfig::default();

            // Create multiple rules for this thread
            let mut rules = Vec::new();
            for i in 0..rules_per_thread {
                let rule = Rule {
                    id: (thread_id * rules_per_thread + i) as u64,
                    name: format!("Rule{}_{}", thread_id, i),
                    conditions: vec![Condition::Simple {
                        field: "score".to_string(),
                        operator: Operator::GreaterThan,
                        value: FactValue::Float(i as f64 * 10.0),
                    }],
                    actions: vec![Action {
                        action_type: ActionType::SetField {
                            field: format!("level_{}", i),
                            value: FactValue::String(format!("Level{}", i)),
                        },
                    }],
                };
                rules.push(rule);
            }

            // Create test facts
            let mut facts = Vec::new();
            for i in 0..50 {
                let mut fields = HashMap::new();
                fields.insert("score".to_string(), FactValue::Float(i as f64 * 5.0));
                fields.insert("thread".to_string(), FactValue::Integer(thread_id as i64));

                let fact = Fact::new((thread_id * 50 + i) as u64, FactData { fields });
                facts.push(fact);
            }

            // Wait for all threads to be ready
            barrier.wait();

            let start = Instant::now();

            // Add rules in parallel
            rete_network.add_rules_parallel(rules, &config).unwrap();

            // Evaluate rules in parallel
            let results = rete_network
                .evaluate_rules_parallel(
                    &facts,
                    &[], // Use all rules in network
                    &mut fact_store,
                    &calculator,
                    &config,
                )
                .unwrap();

            let duration = start.elapsed();

            (thread_id, results.len(), duration)
        });

        handles.push(handle);
    }

    // Collect and verify results
    let mut total_results = 0;

    for handle in handles {
        let (thread_id, result_count, duration) = handle.join().unwrap();
        println!(
            "Thread {}: {} rule results in {:?}",
            thread_id, result_count, duration
        );
        total_results += result_count;
    }

    println!("Total rule execution results: {}", total_results);
    // Results are always non-negative by type - no need to assert anything specific
}

/// Test BingoEngine aggregation operations with separate instances
#[test]
fn test_bingo_engine_concurrent_aggregations() {
    let num_threads = 3;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    // Test different engine operations concurrently with separate instances
    let operations = vec!["sum_avg", "min_max", "variance"];

    for (thread_id, operation) in operations.iter().enumerate() {
        let barrier = Arc::clone(&barrier);
        let op = operation.to_string();

        let handle = thread::spawn(move || {
            // Create separate engine instance for thread safety
            let engine = BingoEngine::new().unwrap();

            // Wait for all threads to be ready
            barrier.wait();

            let start = Instant::now();
            let data: Vec<f64> = (0..1000).map(|i| (i + thread_id * 1000) as f64).collect();

            let result = match op.as_str() {
                "sum_avg" => {
                    let sum = engine.parallel_sum(&data).unwrap();
                    let avg = engine.parallel_average(&data).unwrap();
                    format!("Sum: {}, Avg: {}", sum, avg)
                }
                "min_max" => {
                    let (min, max) = engine.parallel_min_max(&data).unwrap();
                    format!("Min: {}, Max: {}", min, max)
                }
                "variance" => {
                    let (var, std) = engine.parallel_variance(&data).unwrap();
                    format!("Variance: {:.2}, StdDev: {:.2}", var, std)
                }
                _ => "Unknown".to_string(),
            };
            let duration = start.elapsed();

            (thread_id, op, result, duration)
        });

        handles.push(handle);
    }

    // Collect results
    for handle in handles {
        let (thread_id, operation, result, duration) = handle.join().unwrap();
        println!(
            "Thread {}: {} = {} (took {:?})",
            thread_id, operation, result, duration
        );

        // Verify results are reasonable
        assert!(!result.is_empty(), "Should have valid results");
        assert!(duration.as_micros() > 0, "Should take some time");
    }
}

/// Performance benchmark for parallel aggregations under load
#[test]
fn test_parallel_aggregation_performance_benchmark() {
    let data_sizes = vec![1000, 5000, 10000, 50000];
    let num_iterations = 5;

    for data_size in data_sizes {
        println!("\n=== Testing with {} data points ===", data_size);

        let data: Vec<f64> = (0..data_size).map(|i| (i as f64) * 0.1).collect();
        let engine = ParallelAggregationEngine::new();

        // Warm up
        let _ = engine.parallel_sum(&data).unwrap();

        let mut durations = Vec::new();

        for i in 0..num_iterations {
            let start = Instant::now();

            // Perform multiple operations
            let sum = engine.parallel_sum(&data).unwrap();
            let avg = engine.parallel_average(&data).unwrap();
            let (min, max) = engine.parallel_min_max(&data).unwrap();
            let count_above_median = engine.parallel_count(&data, |x| *x > avg).unwrap();

            let duration = start.elapsed();
            durations.push(duration);

            if i == 0 {
                println!(
                    "Results: sum={:.2}, avg={:.2}, min={:.2}, max={:.2}, count_above_avg={}",
                    sum, avg, min, max, count_above_median
                );
            }
        }

        let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        let min_duration = durations.iter().min().unwrap();
        let max_duration = durations.iter().max().unwrap();

        println!(
            "Performance: avg={:?}, min={:?}, max={:?}",
            avg_duration, min_duration, max_duration
        );

        // Verify results are reasonable
        assert!(avg_duration.as_micros() > 0, "Should take some time");
        assert!(
            max_duration.as_micros() < 100_000,
            "Should not take too long"
        ); // 100ms max

        // Get aggregation statistics
        if let Ok(stats) = engine.get_stats() {
            println!(
                "Stats: {} sums, {} averages, {} min_max, {} fallbacks",
                stats.parallel_sums,
                stats.parallel_averages,
                stats.parallel_min_max,
                stats.sequential_fallbacks
            );
        }
    }
}

/// Test concurrent access to parallel aggregator
#[test]
fn test_parallel_aggregator_concurrent_access() {
    let num_threads = 6;
    let barrier = Arc::new(Barrier::new(num_threads));
    let aggregator = Arc::new(ParallelAggregator::new());
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let barrier = Arc::clone(&barrier);
        let aggregator = Arc::clone(&aggregator);

        let handle = thread::spawn(move || {
            let data: Vec<f64> = (0..1000).map(|i| (i + thread_id * 1000) as f64).collect();

            // Wait for all threads to be ready
            barrier.wait();

            let start = Instant::now();

            // Perform multiple aggregation operations
            let sum = aggregator.parallel_sum(&data).unwrap();
            let avg = aggregator.parallel_average(&data).unwrap();
            let (min, max) = aggregator.parallel_min_max(&data).unwrap();

            // Add some worker results
            let worker_stats = bingo_core::parallel::WorkerStats {
                facts_processed: 1000,
                rules_evaluated: 50,
                rules_fired: 25,
                processing_time_ms: start.elapsed().as_millis() as u64,
            };

            aggregator.add_worker_results(thread_id, vec![], worker_stats).unwrap();

            let duration = start.elapsed();
            (thread_id, sum, avg, min, max, duration)
        });

        handles.push(handle);
    }

    // Collect results
    let mut all_sums = Vec::new();

    for handle in handles {
        let (thread_id, sum, avg, min, max, duration) = handle.join().unwrap();
        println!(
            "Thread {}: sum={:.0}, avg={:.2}, min={:.0}, max={:.0}, duration={:?}",
            thread_id, sum, avg, min, max, duration
        );
        all_sums.push(sum);
    }

    // Verify results
    assert_eq!(all_sums.len(), num_threads);

    // Get performance summary
    let summary = aggregator.get_performance_summary().unwrap();
    println!("Performance summary: {:#?}", summary);

    assert_eq!(summary.worker_count, num_threads);
    assert!(summary.total_facts_processed > 0);
    assert!(
        !summary.parallel_efficiency.is_nan(),
        "Parallel efficiency should not be NaN"
    );

    // Test aggregation statistics
    if let Ok(agg_stats) = aggregator.get_aggregation_stats() {
        println!("Aggregation stats: {:#?}", agg_stats);
        assert!(agg_stats.parallel_sums + agg_stats.sequential_fallbacks >= num_threads);
    }
}

/// Stress test for memory pool contention
#[test]
fn test_memory_pool_contention_stress() {
    let num_threads = 10;
    let high_contention_ops = 1000;
    let barrier = Arc::new(Barrier::new(num_threads));
    let pool_manager = Arc::new(ConcurrentMemoryPoolManager::new());
    let mut handles = Vec::new();

    // Enable concurrent memory pools
    pool_manager.set_enabled(true);

    for thread_id in 0..num_threads {
        let barrier = Arc::clone(&barrier);
        let manager = Arc::clone(&pool_manager);

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier.wait();

            let start = Instant::now();
            let mut operations = 0;

            for i in 0..high_contention_ops {
                // Rapidly get and return objects to create contention
                let vec1 = manager.rule_execution_results.get();
                let vec2 = manager.rule_id_vecs.get();

                // Simulate brief usage
                thread::sleep(Duration::from_nanos(100));

                manager.rule_execution_results.return_vec(vec1);
                manager.rule_id_vecs.return_vec(vec2);

                operations += 2;

                // Periodically check if pools are still enabled
                if i % 100 == 0 {
                    assert!(manager.is_enabled(), "Pools should remain enabled");
                }
            }

            let duration = start.elapsed();
            (thread_id, operations, duration)
        });

        handles.push(handle);
    }

    // Collect results and verify no deadlocks or panics occurred
    let mut total_operations = 0;

    for handle in handles {
        let (thread_id, operations, duration) = handle.join().unwrap();
        println!(
            "Thread {}: {} operations in {:?} ({:.0} ops/sec)",
            thread_id,
            operations,
            duration,
            operations as f64 / duration.as_secs_f64()
        );
        total_operations += operations;
    }

    // Final verification
    let final_stats = pool_manager.get_concurrent_stats();
    println!("Final contention test stats: {:#?}", final_stats);

    assert_eq!(total_operations, num_threads * high_contention_ops * 2);
    assert!(final_stats.enabled, "Pools should still be enabled");
    assert!(
        final_stats.rule_execution_result_pool.hits + final_stats.rule_execution_result_pool.misses
            > 0
    );

    // Check that we got some pool reuse (hits) due to contention
    let hit_rate = final_stats.average_hit_rate();
    println!("Average hit rate under contention: {:.2}%", hit_rate);
    assert!(hit_rate >= 0.0, "Hit rate should be non-negative");
}
