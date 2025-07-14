use bingo_core::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Comprehensive test suite for multi-client isolation
///
/// This test validates that multiple clients can operate simultaneously
/// without their facts, rules, or processing results being mixed together.

#[test]
fn test_multiple_client_complete_isolation() {
    println!("🔒 Testing Complete Multi-Client Isolation");

    // Create 3 separate "clients" with their own engines
    let client_a_engine = Arc::new(BingoEngine::new().unwrap());
    let client_b_engine = Arc::new(BingoEngine::new().unwrap());
    let client_c_engine = Arc::new(BingoEngine::new().unwrap());

    // Each client has different rules
    let client_a_rule = Rule {
        id: 1,
        name: "Client A Status Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "client".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("client_a".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "processed_by".to_string(),
                value: FactValue::String("client_a_engine".to_string()),
            },
        }],
    };

    let client_b_rule = Rule {
        id: 2,
        name: "Client B Status Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "client".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("client_b".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "processed_by".to_string(),
                value: FactValue::String("client_b_engine".to_string()),
            },
        }],
    };

    let client_c_rule = Rule {
        id: 3,
        name: "Client C Status Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "client".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("client_c".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "processed_by".to_string(),
                value: FactValue::String("client_c_engine".to_string()),
            },
        }],
    };

    // Add rules to respective engines
    client_a_engine.add_rule(client_a_rule).unwrap();
    client_b_engine.add_rule(client_b_rule).unwrap();
    client_c_engine.add_rule(client_c_rule).unwrap();

    // Each client processes different facts simultaneously
    let mut handles = vec![];

    // Client A thread
    let client_a_clone = Arc::clone(&client_a_engine);
    let handle_a = thread::spawn(move || {
        let facts: Vec<Fact> = (0..100)
            .map(|i| {
                let mut fields = HashMap::new();
                fields.insert(
                    "client".to_string(),
                    FactValue::String("client_a".to_string()),
                );
                fields.insert("data_id".to_string(), FactValue::Integer(i));

                Fact {
                    id: i as u64,
                    external_id: Some(format!("client_a_fact_{}", i)),
                    timestamp: chrono::Utc::now(),
                    data: FactData { fields },
                }
            })
            .collect();

        let results = client_a_clone.process_facts(facts).unwrap();
        println!(
            "Client A processed {} facts, got {} results",
            100,
            results.len()
        );
        (client_a_clone.get_stats(), results.len())
    });

    // Client B thread
    let client_b_clone = Arc::clone(&client_b_engine);
    let handle_b = thread::spawn(move || {
        let facts: Vec<Fact> = (100..200)
            .map(|i| {
                let mut fields = HashMap::new();
                fields.insert(
                    "client".to_string(),
                    FactValue::String("client_b".to_string()),
                );
                fields.insert("data_id".to_string(), FactValue::Integer(i));

                Fact {
                    id: i as u64,
                    external_id: Some(format!("client_b_fact_{}", i)),
                    timestamp: chrono::Utc::now(),
                    data: FactData { fields },
                }
            })
            .collect();

        let results = client_b_clone.process_facts(facts).unwrap();
        println!(
            "Client B processed {} facts, got {} results",
            100,
            results.len()
        );
        (client_b_clone.get_stats(), results.len())
    });

    // Client C thread
    let client_c_clone = Arc::clone(&client_c_engine);
    let handle_c = thread::spawn(move || {
        let facts: Vec<Fact> = (200..300)
            .map(|i| {
                let mut fields = HashMap::new();
                fields.insert(
                    "client".to_string(),
                    FactValue::String("client_c".to_string()),
                );
                fields.insert("data_id".to_string(), FactValue::Integer(i));

                Fact {
                    id: i as u64,
                    external_id: Some(format!("client_c_fact_{}", i)),
                    timestamp: chrono::Utc::now(),
                    data: FactData { fields },
                }
            })
            .collect();

        let results = client_c_clone.process_facts(facts).unwrap();
        println!(
            "Client C processed {} facts, got {} results",
            100,
            results.len()
        );
        (client_c_clone.get_stats(), results.len())
    });

    handles.push(handle_a);
    handles.push(handle_b);
    handles.push(handle_c);

    // Collect results
    let mut client_results = vec![];
    for handle in handles {
        client_results.push(handle.join().unwrap());
    }

    // Verify complete isolation
    println!("🔍 Verifying Client Isolation:");

    // Each client should have exactly 1 rule
    assert_eq!(
        client_results[0].0.rule_count, 1,
        "Client A should have 1 rule"
    );
    assert_eq!(
        client_results[1].0.rule_count, 1,
        "Client B should have 1 rule"
    );
    assert_eq!(
        client_results[2].0.rule_count, 1,
        "Client C should have 1 rule"
    );

    // Each client should have exactly 100 facts
    assert_eq!(
        client_results[0].0.fact_count, 100,
        "Client A should have 100 facts"
    );
    assert_eq!(
        client_results[1].0.fact_count, 100,
        "Client B should have 100 facts"
    );
    assert_eq!(
        client_results[2].0.fact_count, 100,
        "Client C should have 100 facts"
    );

    // Each client should have produced exactly 100 results (1 per fact)
    assert_eq!(client_results[0].1, 100, "Client A should have 100 results");
    assert_eq!(client_results[1].1, 100, "Client B should have 100 results");
    assert_eq!(client_results[2].1, 100, "Client C should have 100 results");

    println!("✅ Complete client isolation verified!");
    println!("  - Each client maintained separate rule sets");
    println!("  - Each client maintained separate fact stores");
    println!("  - Each client produced isolated results");
}

#[test]
fn test_concurrent_client_rule_isolation() {
    println!("🧪 Testing Concurrent Client Rule Isolation");

    // Create engines for different client sessions
    let session_1_engine = Arc::new(BingoEngine::new().unwrap());
    let session_2_engine = Arc::new(BingoEngine::new().unwrap());

    let mut handles = vec![];

    // Session 1: Add rules for user management
    let session_1_clone = Arc::clone(&session_1_engine);
    let handle_1 = thread::spawn(move || {
        for i in 1..=5 {
            let rule = Rule {
                id: i,
                name: format!("User Rule {}", i),
                conditions: vec![Condition::Simple {
                    field: "user_type".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("premium".to_string()),
                }],
                actions: vec![Action {
                    action_type: ActionType::SetField {
                        field: "discount".to_string(),
                        value: FactValue::Integer(i as i64 * 10),
                    },
                }],
            };
            session_1_clone.add_rule(rule).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
        session_1_clone.get_stats()
    });

    // Session 2: Add rules for order processing (different domain)
    let session_2_clone = Arc::clone(&session_2_engine);
    let handle_2 = thread::spawn(move || {
        for i in 1..=3 {
            let rule = Rule {
                id: i + 100, // Different ID range
                name: format!("Order Rule {}", i),
                conditions: vec![Condition::Simple {
                    field: "order_status".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("pending".to_string()),
                }],
                actions: vec![Action {
                    action_type: ActionType::SetField {
                        field: "priority".to_string(),
                        value: FactValue::String("high".to_string()),
                    },
                }],
            };
            session_2_clone.add_rule(rule).unwrap();
            thread::sleep(Duration::from_millis(15));
        }
        session_2_clone.get_stats()
    });

    handles.push(handle_1);
    handles.push(handle_2);

    // Wait for all rule additions
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Verify rule isolation
    println!("📊 Rule Isolation Results:");
    println!("  Session 1: {} rules", results[0].rule_count);
    println!("  Session 2: {} rules", results[1].rule_count);

    assert_eq!(results[0].rule_count, 5, "Session 1 should have 5 rules");
    assert_eq!(results[1].rule_count, 3, "Session 2 should have 3 rules");

    // Verify engines are completely independent
    assert_eq!(
        results[0].fact_count, 0,
        "Session 1 should have no facts initially"
    );
    assert_eq!(
        results[1].fact_count, 0,
        "Session 2 should have no facts initially"
    );

    println!("✅ Concurrent rule isolation verified!");
}

#[test]
fn test_high_concurrency_client_isolation() {
    println!("⚡ Testing High Concurrency Client Isolation (10 clients)");

    let start_time = Instant::now();
    let client_count = 10;
    let facts_per_client = 50;

    // Create multiple engines (simulating different client sessions)
    let engines: Vec<Arc<BingoEngine>> =
        (0..client_count).map(|_| Arc::new(BingoEngine::new().unwrap())).collect();

    // Each client gets a unique rule
    for (i, engine) in engines.iter().enumerate() {
        let rule = Rule {
            id: (i + 1) as u64,
            name: format!("Client {} Rule", i + 1),
            conditions: vec![Condition::Simple {
                field: "client_id".to_string(),
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
        engine.add_rule(rule).unwrap();
    }

    // Spawn concurrent client threads
    let mut handles = vec![];

    for (client_id, engine) in engines.into_iter().enumerate() {
        let handle = thread::spawn(move || {
            let client_start = Instant::now();

            // Each client processes facts with their unique client_id
            let facts: Vec<Fact> = (0..facts_per_client)
                .map(|fact_id| {
                    let mut fields = HashMap::new();
                    fields.insert(
                        "client_id".to_string(),
                        FactValue::Integer(client_id as i64),
                    );
                    fields.insert("fact_id".to_string(), FactValue::Integer(fact_id as i64));
                    fields.insert(
                        "timestamp".to_string(),
                        FactValue::Integer(chrono::Utc::now().timestamp()),
                    );

                    Fact {
                        id: (client_id * 1000 + fact_id) as u64,
                        external_id: Some(format!("client_{}_fact_{}", client_id, fact_id)),
                        timestamp: chrono::Utc::now(),
                        data: FactData { fields },
                    }
                })
                .collect();

            let results = engine.process_facts(facts).unwrap();
            let client_time = client_start.elapsed();
            let stats = engine.get_stats();

            println!(
                "  Client {}: {} results in {:?}",
                client_id,
                results.len(),
                client_time
            );

            (client_id, stats, results.len(), client_time)
        });

        handles.push(handle);
    }

    // Collect all results
    let mut all_results = vec![];
    for handle in handles {
        all_results.push(handle.join().unwrap());
    }

    let total_time = start_time.elapsed();

    // Verify isolation and performance
    println!("🎯 High Concurrency Results:");
    println!("  Total time: {:?}", total_time);
    println!("  Clients: {}", client_count);
    println!("  Facts per client: {}", facts_per_client);

    let mut total_results = 0;

    for (client_id, stats, result_count, _client_time) in all_results {
        // Each client should have exactly 1 rule
        assert_eq!(
            stats.rule_count, 1,
            "Client {} should have 1 rule",
            client_id
        );

        // Each client should have exactly facts_per_client facts
        assert_eq!(
            stats.fact_count, facts_per_client,
            "Client {} should have {} facts",
            client_id, facts_per_client
        );

        // Each client should produce exactly facts_per_client results
        assert_eq!(
            result_count, facts_per_client,
            "Client {} should have {} results",
            client_id, facts_per_client
        );

        total_results += result_count;
    }

    // Overall verification
    assert_eq!(
        total_results,
        client_count * facts_per_client,
        "Total results should match"
    );

    // Performance check (should be reasonably fast with proper isolation)
    assert!(
        total_time.as_millis() < 5000,
        "10 concurrent clients should complete in under 5 seconds"
    );

    println!("✅ High concurrency isolation verified!");
    println!(
        "  - {} clients processed {} facts each",
        client_count, facts_per_client
    );
    println!("  - Total: {} results in {:?}", total_results, total_time);
    println!(
        "  - Throughput: {:.0} facts/sec",
        (total_results as f64) / total_time.as_secs_f64()
    );
}

#[test]
fn test_session_cleanup_isolation() {
    println!("🧹 Testing Session Cleanup and Isolation");

    // Create multiple engines (simulating client sessions)
    let session_engines: Vec<Arc<BingoEngine>> =
        (0..3).map(|_| Arc::new(BingoEngine::new().unwrap())).collect();

    // Each session adds different data
    for (i, engine) in session_engines.iter().enumerate() {
        // Add rules
        let rule = Rule {
            id: (i + 1) as u64,
            name: format!("Session {} Rule", i + 1),
            conditions: vec![Condition::Simple {
                field: "session".to_string(),
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
        engine.add_rule(rule).unwrap();

        // Add facts
        let facts: Vec<Fact> = (0..10)
            .map(|j| {
                let mut fields = HashMap::new();
                fields.insert("session".to_string(), FactValue::Integer(i as i64));
                fields.insert("data".to_string(), FactValue::Integer(j as i64));

                Fact {
                    id: (i * 100 + j) as u64,
                    external_id: Some(format!("session_{}_fact_{}", i, j)),
                    timestamp: chrono::Utc::now(),
                    data: FactData { fields },
                }
            })
            .collect();

        engine.process_facts(facts).unwrap();
    }

    // Verify initial state
    println!("📊 Initial Session States:");
    for (i, engine) in session_engines.iter().enumerate() {
        let stats = engine.get_stats();
        println!(
            "  Session {}: {} rules, {} facts",
            i, stats.rule_count, stats.fact_count
        );
        assert_eq!(stats.rule_count, 1, "Session {} should have 1 rule", i);
        assert_eq!(stats.fact_count, 10, "Session {} should have 10 facts", i);
    }

    // Clear one session
    println!("🧹 Clearing Session 1...");
    session_engines[1].clear();

    // Verify isolation after cleanup
    println!("📊 States After Session 1 Cleanup:");
    for (i, engine) in session_engines.iter().enumerate() {
        let stats = engine.get_stats();
        println!(
            "  Session {}: {} rules, {} facts",
            i, stats.rule_count, stats.fact_count
        );

        if i == 1 {
            // Cleared session should have no data
            assert_eq!(stats.rule_count, 0, "Cleared session should have 0 rules");
            assert_eq!(stats.fact_count, 0, "Cleared session should have 0 facts");
        } else {
            // Other sessions should be unaffected
            assert_eq!(
                stats.rule_count, 1,
                "Unaffected session {} should still have 1 rule",
                i
            );
            assert_eq!(
                stats.fact_count, 10,
                "Unaffected session {} should still have 10 facts",
                i
            );
        }
    }

    // Clear facts only from another session
    println!("🧹 Clearing facts only from Session 0...");
    session_engines[0].clear_facts();

    // Verify facts-only cleanup isolation
    println!("📊 States After Session 0 Facts Cleanup:");
    let stats_0 = session_engines[0].get_stats();
    let stats_2 = session_engines[2].get_stats();

    // Session 0 should have rules but no facts
    assert_eq!(stats_0.rule_count, 1, "Session 0 should still have 1 rule");
    assert_eq!(
        stats_0.fact_count, 0,
        "Session 0 should have 0 facts after clear_facts"
    );

    // Session 2 should be completely unaffected
    assert_eq!(stats_2.rule_count, 1, "Session 2 should still have 1 rule");
    assert_eq!(
        stats_2.fact_count, 10,
        "Session 2 should still have 10 facts"
    );

    println!("✅ Session cleanup isolation verified!");
    println!("  - Clearing one session doesn't affect others");
    println!("  - Facts-only cleanup preserves rules");
    println!("  - Complete isolation maintained during cleanup");
}
