use bingo_core::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn test_concurrent_engine_basic() {
    println!("🧪 Testing Basic Concurrent Engine Functionality");

    let engine = Arc::new(BingoEngine::new().unwrap());

    // Add a rule first
    let rule = Rule {
        id: 1,
        name: "Status Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("active".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "processed".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Test basic functionality
    let facts = vec![Fact {
        id: 1,
        external_id: None,
        timestamp: chrono::Utc::now(),
        data: FactData {
            fields: HashMap::from([(
                "status".to_string(),
                FactValue::String("active".to_string()),
            )]),
        },
    }];

    let results = engine.process_facts(facts).unwrap();
    println!("✅ Basic processing: {} results", results.len());
    assert_eq!(results.len(), 1);

    let stats = engine.get_stats();
    println!(
        "✅ Stats: {} rules, {} facts",
        stats.rule_count, stats.fact_count
    );
    assert_eq!(stats.rule_count, 1);
}

#[test]
fn test_concurrent_fact_processing() {
    println!("🧪 Testing Concurrent Fact Processing");

    let engine = Arc::new(BingoEngine::new().unwrap());

    // Add a rule first
    let rule = Rule {
        id: 1,
        name: "Status Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("active".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "processed".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    println!("🚀 Starting concurrent fact processing with 4 threads...");

    let mut handles = vec![];
    let start_time = Instant::now();

    // Spawn 4 concurrent threads to process facts
    for thread_id in 0..4 {
        let engine_clone = Arc::clone(&engine);

        let handle = thread::spawn(move || {
            let thread_start = Instant::now();

            // Each thread processes 500 facts
            let facts: Vec<Fact> = (0..500)
                .map(|i| {
                    let mut fields = HashMap::new();
                    fields.insert("thread_id".to_string(), FactValue::Integer(thread_id));
                    fields.insert("fact_id".to_string(), FactValue::Integer(i));
                    fields.insert(
                        "status".to_string(),
                        FactValue::String(
                            if i % 2 == 0 { "active" } else { "inactive" }.to_string(),
                        ),
                    );

                    Fact {
                        id: (thread_id * 1000 + i) as u64,
                        external_id: None,
                        timestamp: chrono::Utc::now(),
                        data: FactData { fields },
                    }
                })
                .collect();

            let results = engine_clone.process_facts(facts).unwrap();
            let thread_elapsed = thread_start.elapsed();

            println!(
                "  Thread {}: {} results in {:?}",
                thread_id,
                results.len(),
                thread_elapsed
            );

            (thread_id, results.len(), thread_elapsed)
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    let mut total_results = 0;
    for handle in handles {
        let (thread_id, result_count, thread_time) = handle.join().unwrap();
        total_results += result_count;
        println!("✅ Thread {thread_id} completed: {result_count} results in {thread_time:?}");
    }

    let total_elapsed = start_time.elapsed();

    println!("🎯 Concurrent Results:");
    println!("  Total results: {total_results}");
    println!("  Total time: {total_elapsed:?}");
    println!(
        "  Concurrent throughput: {:.0} facts/sec",
        2000.0 / total_elapsed.as_secs_f64()
    );

    let final_stats = engine.get_stats();
    println!(
        "  Final stats: {} rules, {} facts",
        final_stats.rule_count, final_stats.fact_count
    );

    // Verify concurrency worked correctly
    assert!(total_results > 0, "Should have processed some facts");
    assert_eq!(final_stats.rule_count, 1, "Should still have 1 rule");
    assert!(
        final_stats.fact_count >= 2000,
        "Should have processed 2000+ facts"
    );

    // Concurrent processing should be faster than sequential
    // Even with overhead, 4 threads should provide some speedup
    assert!(
        total_elapsed.as_millis() < 10000,
        "Concurrent processing should be reasonably fast"
    );

    println!("✅ Concurrent fact processing test passed!");
}

#[test]
fn test_concurrent_rule_access() {
    println!("🧪 Testing Concurrent Rule Access");

    let engine = Arc::new(BingoEngine::new().unwrap());

    // One thread adds rules
    let engine_writer = Arc::clone(&engine);
    let writer_handle = thread::spawn(move || {
        for i in 0..10 {
            let rule = Rule {
                id: i + 1,
                name: format!("Rule {}", i + 1),
                conditions: vec![Condition::Simple {
                    field: "value".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::Integer(i as i64),
                }],
                actions: vec![Action {
                    action_type: ActionType::SetField {
                        field: "processed".to_string(),
                        value: FactValue::Boolean(true),
                    },
                }],
            };

            engine_writer.add_rule(rule).unwrap();
            thread::sleep(Duration::from_millis(10)); // Small delay
        }

        println!("  Writer thread: Added 10 rules");
    });

    // Multiple threads read statistics concurrently
    let mut reader_handles = vec![];
    for thread_id in 0..3 {
        let engine_reader = Arc::clone(&engine);

        let reader_handle = thread::spawn(move || {
            let mut max_rules_seen = 0;

            for _ in 0..20 {
                let stats = engine_reader.get_stats();
                max_rules_seen = max_rules_seen.max(stats.rule_count);

                if thread_id == 0 && stats.rule_count > 0 {
                    println!("  Reader {}: Saw {} rules", thread_id, stats.rule_count);
                }

                thread::sleep(Duration::from_millis(5));
            }

            println!("  Reader thread {thread_id}: Max rules seen: {max_rules_seen}");
            max_rules_seen
        });

        reader_handles.push(reader_handle);
    }

    // Wait for writer thread
    writer_handle.join().unwrap();

    // Wait for reader threads and collect results
    let mut results = vec![];
    for reader_handle in reader_handles {
        let max_rules_seen = reader_handle.join().unwrap();
        results.push(max_rules_seen);
    }

    let final_stats = engine.get_stats();
    println!("✅ Final rule count: {}", final_stats.rule_count);

    // Verify concurrent access worked
    assert_eq!(final_stats.rule_count, 10, "Should have 10 rules");

    // Readers should have seen rules being added concurrently
    for &max_seen in &results {
        assert!(max_seen > 0, "Readers should have seen rules being added");
    }

    println!("✅ Concurrent rule access test passed!");
}

#[test]
fn test_concurrent_performance_comparison() {
    println!("🧪 Testing Concurrent vs Sequential Performance");

    let engine = Arc::new(BingoEngine::new().unwrap());

    // Add a rule
    let rule = Rule {
        id: 1,
        name: "Performance Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("active".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "processed".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    let fact_count = 1000;

    // Test 1: Sequential processing
    println!("🔄 Sequential processing {fact_count} facts...");
    let start_sequential = Instant::now();

    let facts: Vec<Fact> = (0..fact_count)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("id".to_string(), FactValue::Integer(i as i64));
            fields.insert(
                "status".to_string(),
                FactValue::String(if i % 2 == 0 { "active" } else { "inactive" }.to_string()),
            );

            Fact {
                id: i as u64,
                external_id: None,
                timestamp: chrono::Utc::now(),
                data: FactData { fields },
            }
        })
        .collect();

    let sequential_results = engine.process_facts(facts).unwrap();
    let sequential_time = start_sequential.elapsed();

    // Clear for next test
    engine.clear_facts();

    // Test 2: Concurrent processing (2 threads, 500 facts each)
    println!("⚡ Concurrent processing {fact_count} facts...");
    let start_concurrent = Instant::now();

    let mut handles = vec![];

    for thread_id in 0..2 {
        let engine_clone = Arc::clone(&engine);

        let handle = thread::spawn(move || {
            let facts: Vec<Fact> = (0..500)
                .map(|i| {
                    let mut fields = HashMap::new();
                    fields.insert(
                        "id".to_string(),
                        FactValue::Integer((thread_id * 500 + i) as i64),
                    );
                    fields.insert(
                        "status".to_string(),
                        FactValue::String(
                            if i % 2 == 0 { "active" } else { "inactive" }.to_string(),
                        ),
                    );

                    Fact {
                        id: (thread_id * 500 + i) as u64,
                        external_id: None,
                        timestamp: chrono::Utc::now(),
                        data: FactData { fields },
                    }
                })
                .collect();

            engine_clone.process_facts(facts).unwrap()
        });

        handles.push(handle);
    }

    let mut concurrent_results = 0;
    for handle in handles {
        concurrent_results += handle.join().unwrap().len();
    }

    let concurrent_time = start_concurrent.elapsed();

    // Results
    println!("📊 Performance Comparison:");
    println!(
        "  Sequential: {} results in {:?} ({:.0} facts/sec)",
        sequential_results.len(),
        sequential_time,
        fact_count as f64 / sequential_time.as_secs_f64()
    );
    println!(
        "  Concurrent: {} results in {:?} ({:.0} facts/sec)",
        concurrent_results,
        concurrent_time,
        fact_count as f64 / concurrent_time.as_secs_f64()
    );

    let speedup = sequential_time.as_secs_f64() / concurrent_time.as_secs_f64();
    println!("  Speedup: {speedup:.2}x");

    // Verify correctness
    assert_eq!(
        sequential_results.len(),
        concurrent_results,
        "Both approaches should produce same number of results"
    );

    // Concurrent should be at least somewhat competitive (allowing for overhead)
    // We're not requiring speedup since there might be contention, but it shouldn't be much slower
    let efficiency = concurrent_time.as_secs_f64() / sequential_time.as_secs_f64();
    assert!(
        efficiency < 3.0,
        "Concurrent shouldn't be more than 3x slower than sequential"
    );

    println!("✅ Performance comparison test passed!");
}
