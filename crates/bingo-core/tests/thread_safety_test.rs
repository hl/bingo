//! Comprehensive Thread Safety Tests for Unified BingoEngine
//!
//! This test suite validates the thread safety of the unified BingoEngine implementation,
//! ensuring proper concurrent access, data integrity, and performance under load.

use bingo_core::BingoEngine;
use bingo_core::types::{Action, ActionType, Condition, Fact, FactData, FactValue, Operator, Rule};
use std::collections::HashMap;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

/// Test basic Send + Sync traits for BingoEngine
#[test]
fn test_engine_send_sync_traits() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<BingoEngine>();
    assert_sync::<BingoEngine>();
}

/// Test concurrent rule addition from multiple threads
#[test]
fn test_concurrent_rule_addition() {
    let engine = Arc::new(BingoEngine::new().expect("Failed to create engine"));
    let num_threads = 8;
    let rules_per_thread = 10;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let engine = Arc::clone(&engine);
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier.wait();

            for rule_id in 0..rules_per_thread {
                let rule = create_test_rule(
                    (thread_id * rules_per_thread + rule_id) as u64 + 1,
                    &format!("ThreadRule_{thread_id}_{rule_id}"),
                );

                engine.add_rule(rule).expect("Failed to add rule");
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let stats = engine.get_stats();
    assert_eq!(stats.rule_count, num_threads * rules_per_thread);
}

/// Test concurrent fact processing from multiple threads
#[test]
fn test_concurrent_fact_processing() {
    let engine = Arc::new(BingoEngine::new().expect("Failed to create engine"));

    // Add a test rule first
    let rule = create_test_rule(1, "TestRule");
    engine.add_rule(rule).expect("Failed to add rule");

    let num_threads = 4;
    let facts_per_thread = 50;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let engine = Arc::clone(&engine);
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier.wait();

            let mut total_results = 0;
            for fact_id in 0..facts_per_thread {
                let fact = create_test_fact(
                    (thread_id * facts_per_thread + fact_id) as u64,
                    75.0, // Should trigger the rule
                );

                match engine.add_fact_to_working_memory(fact) {
                    Ok(results) => total_results += results.len(),
                    Err(e) => panic!("Failed to process fact: {e}"),
                }
            }
            total_results
        });

        handles.push(handle);
    }

    let mut total_results = 0;
    for handle in handles {
        total_results += handle.join().expect("Thread panicked");
    }

    assert!(total_results > 0, "Expected some rule executions");

    let stats = engine.get_stats();
    assert_eq!(stats.fact_count, num_threads * facts_per_thread);
}

/// Test concurrent mixed operations (rules + facts) from multiple threads
#[test]
fn test_concurrent_mixed_operations() {
    let engine = Arc::new(BingoEngine::new().expect("Failed to create engine"));
    let num_threads = 6;
    let operations_per_thread = 20;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let engine = Arc::clone(&engine);
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier.wait();

            for op_id in 0..operations_per_thread {
                if op_id % 2 == 0 {
                    // Add rule
                    let rule = create_test_rule(
                        (thread_id * operations_per_thread + op_id) as u64 + 1,
                        &format!("MixedRule_{thread_id}_{op_id}"),
                    );
                    engine.add_rule(rule).expect("Failed to add rule");
                } else {
                    // Add fact
                    let fact =
                        create_test_fact((thread_id * operations_per_thread + op_id) as u64, 60.0);
                    engine.add_fact_to_working_memory(fact).expect("Failed to add fact");
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let stats = engine.get_stats();
    assert!(stats.rule_count > 0);
    assert!(stats.fact_count > 0);
}

/// Test engine statistics consistency under concurrent access
#[test]
fn test_concurrent_statistics_consistency() {
    let engine = Arc::new(BingoEngine::new().expect("Failed to create engine"));
    let num_threads = 4;
    let barrier = Arc::new(Barrier::new(num_threads + 1)); // +1 for stats reader
    let mut handles = Vec::new();

    // Statistics reader thread
    let engine_stats = Arc::clone(&engine);
    let barrier_stats = Arc::clone(&barrier);
    let stats_handle = thread::spawn(move || {
        barrier_stats.wait();

        let mut stats_history = Vec::new();
        for _ in 0..50 {
            stats_history.push(engine_stats.get_stats());
            thread::sleep(Duration::from_millis(10));
        }
        stats_history
    });

    // Worker threads adding rules and facts
    for thread_id in 0..num_threads {
        let engine = Arc::clone(&engine);
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier.wait();

            for i in 0..25 {
                let rule = create_test_rule(
                    (thread_id * 25 + i) as u64 + 1,
                    &format!("StatsRule_{thread_id}_{i}"),
                );
                engine.add_rule(rule).expect("Failed to add rule");

                let fact = create_test_fact((thread_id * 25 + i) as u64, 55.0);
                engine.add_fact_to_working_memory(fact).expect("Failed to add fact");

                thread::sleep(Duration::from_millis(5));
            }
        });

        handles.push(handle);
    }

    // Wait for all workers to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let stats_history = stats_handle.join().expect("Stats thread panicked");

    // Verify statistics are monotonically increasing (no race conditions)
    for i in 1..stats_history.len() {
        assert!(
            stats_history[i].rule_count >= stats_history[i - 1].rule_count,
            "Rule count should be monotonically increasing"
        );
        assert!(
            stats_history[i].fact_count >= stats_history[i - 1].fact_count,
            "Fact count should be monotonically increasing"
        );
    }
}

/// Test high contention scenario with many short-lived threads
#[test]
fn test_high_contention_scenario() {
    let engine = Arc::new(BingoEngine::new().expect("Failed to create engine"));
    let num_batches = 10;
    let threads_per_batch = 8;

    for batch in 0..num_batches {
        let mut handles = Vec::new();

        for thread_id in 0..threads_per_batch {
            let engine = Arc::clone(&engine);

            let handle = thread::spawn(move || {
                let global_id = batch * threads_per_batch + thread_id;

                // Add rule
                let rule = create_test_rule(
                    global_id as u64 + 1,
                    &format!("HighContentionRule_{global_id}"),
                );
                engine.add_rule(rule).expect("Failed to add rule");

                // Add multiple facts quickly
                for fact_id in 0..5 {
                    let fact = create_test_fact((global_id * 5 + fact_id) as u64, 65.0);
                    engine.add_fact_to_working_memory(fact).expect("Failed to add fact");
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    let stats = engine.get_stats();
    assert_eq!(stats.rule_count, num_batches * threads_per_batch);
    assert_eq!(stats.fact_count, num_batches * threads_per_batch * 5);
}

/// Test that multiple engines can be used concurrently
#[test]
fn test_multiple_engines_concurrent() {
    let num_engines = 4;
    let threads_per_engine = 3;
    let operations_per_thread = 15;
    let mut engine_handles = Vec::new();

    for engine_id in 0..num_engines {
        let handle = thread::spawn(move || {
            let engine = Arc::new(BingoEngine::new().expect("Failed to create engine"));
            let mut thread_handles = Vec::new();

            for thread_id in 0..threads_per_engine {
                let engine = Arc::clone(&engine);

                let thread_handle = thread::spawn(move || {
                    for op_id in 0..operations_per_thread {
                        let rule = create_test_rule(
                            (thread_id * operations_per_thread + op_id) as u64 + 1,
                            &format!("Engine{engine_id}_Thread{thread_id}_Rule{op_id}"),
                        );
                        engine.add_rule(rule).expect("Failed to add rule");

                        let fact = create_test_fact(
                            (thread_id * operations_per_thread + op_id) as u64,
                            70.0,
                        );
                        engine.add_fact_to_working_memory(fact).expect("Failed to add fact");
                    }
                });

                thread_handles.push(thread_handle);
            }

            for thread_handle in thread_handles {
                thread_handle.join().expect("Thread panicked");
            }

            engine.get_stats()
        });

        engine_handles.push(handle);
    }

    for handle in engine_handles {
        let stats = handle.join().expect("Engine thread panicked");
        assert_eq!(stats.rule_count, threads_per_engine * operations_per_thread);
        assert_eq!(stats.fact_count, threads_per_engine * operations_per_thread);
    }
}

/// Test stress scenario with rapid rule/fact additions
#[test]
fn test_stress_rapid_operations() {
    let engine = Arc::new(BingoEngine::new().expect("Failed to create engine"));
    let duration = Duration::from_millis(500);
    let num_threads = 6;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let engine = Arc::clone(&engine);
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier.wait();
            let start = Instant::now();
            let mut operations = 0;

            while start.elapsed() < duration {
                let rule = create_test_rule(
                    (thread_id * 10000 + operations) as u64 + 1,
                    &format!("StressRule_{thread_id}_{operations}"),
                );
                engine.add_rule(rule).expect("Failed to add rule");

                let fact = create_test_fact((thread_id * 10000 + operations) as u64, 80.0);
                engine.add_fact_to_working_memory(fact).expect("Failed to add fact");

                operations += 1;
            }

            operations
        });

        handles.push(handle);
    }

    let mut total_operations = 0;
    for handle in handles {
        total_operations += handle.join().expect("Thread panicked");
    }

    println!("Completed {total_operations} operations in 500ms across {num_threads} threads");
    assert!(total_operations > 0);

    let stats = engine.get_stats();
    assert_eq!(stats.rule_count, total_operations);
    assert_eq!(stats.fact_count, total_operations);
}

// Helper functions

fn create_test_rule(id: u64, name: &str) -> Rule {
    let mut result_fields = HashMap::new();
    result_fields.insert(
        "result".to_string(),
        FactValue::String("triggered".to_string()),
    );
    result_fields.insert("rule_id".to_string(), FactValue::Integer(id as i64));

    Rule {
        id,
        name: name.to_string(),
        conditions: vec![Condition::Simple {
            field: "value".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Float(50.0),
        }],
        actions: vec![Action {
            action_type: ActionType::CreateFact { data: FactData { fields: result_fields } },
        }],
    }
}

fn create_test_fact(id: u64, value: f64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert("value".to_string(), FactValue::Float(value));
    fields.insert("id".to_string(), FactValue::Integer(id as i64));

    Fact {
        id,
        external_id: Some(format!("test_fact_{id}")),
        timestamp: chrono::Utc::now(),
        data: FactData { fields },
    }
}
