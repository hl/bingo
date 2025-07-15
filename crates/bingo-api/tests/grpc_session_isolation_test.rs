use bingo_api::AppState;
use std::sync::Arc;

/// Tests for gRPC service session-based client isolation
///
/// This test verifies that the gRPC service properly isolates clients
/// through session management and prevents data mixing.

#[tokio::test]
async fn test_app_state_session_isolation() {
    println!("🔒 Testing AppState Session Isolation");

    let app_state = Arc::new(AppState::new().await.unwrap());

    // Create engines for different sessions
    let session_a_engine = app_state.get_or_create_engine("session_a");
    let session_b_engine = app_state.get_or_create_engine("session_b");
    let session_c_engine = app_state.get_or_create_engine("session_c");

    // Verify different engines are created
    assert!(
        !Arc::ptr_eq(&session_a_engine, &session_b_engine),
        "Session A and B should have different engines"
    );
    assert!(
        !Arc::ptr_eq(&session_b_engine, &session_c_engine),
        "Session B and C should have different engines"
    );
    assert!(
        !Arc::ptr_eq(&session_a_engine, &session_c_engine),
        "Session A and C should have different engines"
    );

    // Verify session tracking
    assert_eq!(
        app_state.active_sessions(),
        3,
        "Should have 3 active sessions"
    );

    // Verify same session returns same engine
    let session_a_engine_2 = app_state.get_or_create_engine("session_a");
    assert!(
        Arc::ptr_eq(&session_a_engine, &session_a_engine_2),
        "Same session should return same engine"
    );

    // Should still have 3 sessions (no new one created)
    assert_eq!(
        app_state.active_sessions(),
        3,
        "Should still have 3 active sessions"
    );

    println!("✅ AppState session isolation verified!");
}

#[tokio::test]
async fn test_concurrent_session_creation() {
    println!("⚡ Testing Concurrent Session Creation");

    let app_state = Arc::new(AppState::new().await.unwrap());
    let mut handles = vec![];

    // Create 10 concurrent threads trying to create sessions
    for i in 0..10 {
        let app_state_clone = Arc::clone(&app_state);
        let session_id = format!("concurrent_session_{i}");

        let handle = tokio::spawn(async move {
            let engine = app_state_clone.get_or_create_engine(&session_id);

            // Add a rule specific to this session
            let rule = bingo_core::Rule {
                id: (i + 1) as u64,
                name: format!("Rule for {session_id}"),
                conditions: vec![bingo_core::Condition::Simple {
                    field: "session_id".to_string(),
                    operator: bingo_core::Operator::Equal,
                    value: bingo_core::FactValue::String(session_id.clone()),
                }],
                actions: vec![bingo_core::Action {
                    action_type: bingo_core::ActionType::SetField {
                        field: "processed".to_string(),
                        value: bingo_core::FactValue::Boolean(true),
                    },
                }],
            };

            engine.add_rule(rule).unwrap();

            // Return session info
            (session_id, engine.get_stats())
        });

        handles.push(handle);
    }

    // Wait for all sessions to be created
    let mut results = vec![];
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    // Verify all sessions were created correctly
    assert_eq!(
        app_state.active_sessions(),
        10,
        "Should have 10 active sessions"
    );

    // Verify each session has its own rule
    for (session_id, stats) in results {
        assert_eq!(
            stats.rule_count, 1,
            "Session {session_id} should have 1 rule"
        );
        assert_eq!(
            stats.fact_count, 0,
            "Session {session_id} should have 0 facts initially"
        );
    }

    println!("✅ Concurrent session creation verified!");
}

#[tokio::test]
async fn test_session_cleanup_and_isolation() {
    println!("🧹 Testing Session Cleanup and Isolation");

    let app_state = Arc::new(AppState::new().await.unwrap());

    // Create multiple sessions
    let session_1_engine = app_state.get_or_create_engine("cleanup_session_1");
    let session_2_engine = app_state.get_or_create_engine("cleanup_session_2");
    let session_3_engine = app_state.get_or_create_engine("cleanup_session_3");

    // Add different rules to each session
    for (i, engine) in [&session_1_engine, &session_2_engine, &session_3_engine].iter().enumerate()
    {
        let rule = bingo_core::Rule {
            id: (i + 1) as u64,
            name: format!("Rule {}", i + 1),
            conditions: vec![bingo_core::Condition::Simple {
                field: "type".to_string(),
                operator: bingo_core::Operator::Equal,
                value: bingo_core::FactValue::String(format!("type_{}", i + 1)),
            }],
            actions: vec![bingo_core::Action {
                action_type: bingo_core::ActionType::SetField {
                    field: "processed".to_string(),
                    value: bingo_core::FactValue::Boolean(true),
                },
            }],
        };

        engine.add_rule(rule).unwrap();
    }

    // Verify initial state
    assert_eq!(
        app_state.active_sessions(),
        3,
        "Should have 3 active sessions"
    );
    assert_eq!(
        session_1_engine.get_stats().rule_count,
        1,
        "Session 1 should have 1 rule"
    );
    assert_eq!(
        session_2_engine.get_stats().rule_count,
        1,
        "Session 2 should have 1 rule"
    );
    assert_eq!(
        session_3_engine.get_stats().rule_count,
        1,
        "Session 3 should have 1 rule"
    );

    // Remove one session
    let removed_engine = app_state.remove_engine("cleanup_session_2");
    assert!(removed_engine.is_some(), "Should return the removed engine");

    // Verify cleanup
    assert_eq!(
        app_state.active_sessions(),
        2,
        "Should have 2 active sessions after removal"
    );

    // Verify other sessions are unaffected
    assert_eq!(
        session_1_engine.get_stats().rule_count,
        1,
        "Session 1 should still have 1 rule"
    );
    assert_eq!(
        session_3_engine.get_stats().rule_count,
        1,
        "Session 3 should still have 1 rule"
    );

    // Verify removed session can't be retrieved anymore
    let new_session_2_engine = app_state.get_or_create_engine("cleanup_session_2");
    assert!(
        !Arc::ptr_eq(&session_2_engine, &new_session_2_engine),
        "New session 2 should be a different engine"
    );
    assert_eq!(
        new_session_2_engine.get_stats().rule_count,
        0,
        "New session 2 should have 0 rules"
    );

    println!("✅ Session cleanup and isolation verified!");
}

#[tokio::test]
async fn test_high_concurrency_session_isolation() {
    println!("🚀 Testing High Concurrency Session Isolation");

    let app_state = Arc::new(AppState::new().await.unwrap());
    let session_count = 20;
    let operations_per_session = 50;

    // Create many concurrent sessions with operations
    let mut handles = vec![];

    for session_idx in 0..session_count {
        let app_state_clone = Arc::clone(&app_state);

        let handle = tokio::spawn(async move {
            let session_id = format!("high_concurrency_session_{session_idx}");
            let engine = app_state_clone.get_or_create_engine(&session_id);

            // Add session-specific rule
            let rule = bingo_core::Rule {
                id: 1,
                name: format!("High Concurrency Rule {session_idx}"),
                conditions: vec![bingo_core::Condition::Simple {
                    field: "session_index".to_string(),
                    operator: bingo_core::Operator::Equal,
                    value: bingo_core::FactValue::Integer(session_idx as i64),
                }],
                actions: vec![bingo_core::Action {
                    action_type: bingo_core::ActionType::SetField {
                        field: "processed".to_string(),
                        value: bingo_core::FactValue::Boolean(true),
                    },
                }],
            };

            engine.add_rule(rule).unwrap();

            // Process many facts
            let facts: Vec<bingo_core::Fact> = (0..operations_per_session)
                .map(|fact_idx| {
                    let mut fields = std::collections::HashMap::new();
                    fields.insert(
                        "session_index".to_string(),
                        bingo_core::FactValue::Integer(session_idx as i64),
                    );
                    fields.insert(
                        "fact_index".to_string(),
                        bingo_core::FactValue::Integer(fact_idx as i64),
                    );

                    bingo_core::Fact {
                        id: (session_idx * 1000 + fact_idx) as u64,
                        external_id: Some(format!("{session_id}_{fact_idx}")),
                        timestamp: chrono::Utc::now(),
                        data: bingo_core::FactData { fields },
                    }
                })
                .collect();

            let results = engine.process_facts(facts).unwrap();

            (session_idx, engine.get_stats(), results.len())
        });

        handles.push(handle);
    }

    // Wait for all sessions to complete
    let mut all_results = vec![];
    for handle in handles {
        all_results.push(handle.await.unwrap());
    }

    // Verify results
    assert_eq!(
        app_state.active_sessions(),
        session_count,
        "Should have {session_count} active sessions"
    );

    let mut total_facts_processed = 0;
    let mut total_results = 0;

    for (session_idx, stats, result_count) in all_results {
        // Each session should have exactly 1 rule
        assert_eq!(
            stats.rule_count, 1,
            "Session {session_idx} should have 1 rule"
        );

        // Each session should have exactly operations_per_session facts
        assert_eq!(
            stats.fact_count, operations_per_session,
            "Session {session_idx} should have {operations_per_session} facts"
        );

        // Each session should produce exactly operations_per_session results
        assert_eq!(
            result_count, operations_per_session,
            "Session {session_idx} should have {operations_per_session} results"
        );

        total_facts_processed += stats.fact_count;
        total_results += result_count;
    }

    // Verify totals
    let expected_total = session_count * operations_per_session;
    assert_eq!(
        total_facts_processed, expected_total,
        "Total facts should be {expected_total}"
    );
    assert_eq!(
        total_results, expected_total,
        "Total results should be {expected_total}"
    );

    println!("✅ High concurrency session isolation verified!");
    println!("  - {session_count} sessions created and operated concurrently");
    println!("  - {operations_per_session} facts processed per session");
    println!("  - Total: {total_facts_processed} facts processed with perfect isolation");
}

#[test]
fn test_default_engine_isolation() {
    println!("🔧 Testing Default Engine Isolation");

    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let app_state = Arc::new(AppState::new().await.unwrap());

        // Get default engine
        let default_engine = app_state.get_default_engine();

        // Create a session engine
        let session_engine = app_state.get_or_create_engine("test_session");

        // Verify they are different engines
        assert!(
            !Arc::ptr_eq(&default_engine, &session_engine),
            "Default and session engines should be different"
        );

        // Add rule to default engine
        let default_rule = bingo_core::Rule {
            id: 1,
            name: "Default Engine Rule".to_string(),
            conditions: vec![bingo_core::Condition::Simple {
                field: "target".to_string(),
                operator: bingo_core::Operator::Equal,
                value: bingo_core::FactValue::String("default".to_string()),
            }],
            actions: vec![bingo_core::Action {
                action_type: bingo_core::ActionType::SetField {
                    field: "processed_by".to_string(),
                    value: bingo_core::FactValue::String("default_engine".to_string()),
                },
            }],
        };

        default_engine.add_rule(default_rule).unwrap();

        // Add different rule to session engine
        let session_rule = bingo_core::Rule {
            id: 2,
            name: "Session Engine Rule".to_string(),
            conditions: vec![bingo_core::Condition::Simple {
                field: "target".to_string(),
                operator: bingo_core::Operator::Equal,
                value: bingo_core::FactValue::String("session".to_string()),
            }],
            actions: vec![bingo_core::Action {
                action_type: bingo_core::ActionType::SetField {
                    field: "processed_by".to_string(),
                    value: bingo_core::FactValue::String("session_engine".to_string()),
                },
            }],
        };

        session_engine.add_rule(session_rule).unwrap();

        // Verify isolation
        let default_stats = default_engine.get_stats();
        let session_stats = session_engine.get_stats();

        assert_eq!(
            default_stats.rule_count, 1,
            "Default engine should have 1 rule"
        );
        assert_eq!(
            session_stats.rule_count, 1,
            "Session engine should have 1 rule"
        );

        // Session count should only include session engines, not default
        assert_eq!(
            app_state.active_sessions(),
            1,
            "Should have 1 active session (default not counted)"
        );

        println!("✅ Default engine isolation verified!");
    });
}
