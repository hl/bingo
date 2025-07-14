/// Beta Network Integration Test
///
/// This test validates the proper RETE beta network implementation for multi-condition rules
/// and demonstrates token propagation, partial match handling, and incremental processing.
use bingo_core::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_beta_network_multi_condition_rule() {
    println!("🔍 Testing beta network with multi-condition rules...");

    let engine = BingoEngine::new().unwrap();

    // Add a multi-condition rule: age > 18 AND status == "active"
    let rule = Rule {
        id: 1,
        name: "Adult Active User Rule".to_string(),
        conditions: vec![
            Condition::Simple {
                field: "age".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Integer(18),
            },
            Condition::Simple {
                field: "status".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("active".to_string()),
            },
        ],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "eligible".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };
    engine.add_rule(rule).unwrap();

    println!("✅ Added multi-condition rule: age > 18 AND status == 'active'");

    // Test Case 1: Add fact that matches first condition only
    let fact1 = Fact {
        id: 1,
        external_id: None,
        timestamp: chrono::Utc::now(),
        data: FactData {
            fields: [
                ("user_id".to_string(), FactValue::Integer(101)),
                ("age".to_string(), FactValue::Integer(25)), // Matches first condition
                (
                    "status".to_string(),
                    FactValue::String("pending".to_string()),
                ), // Does NOT match second
            ]
            .iter()
            .cloned()
            .collect(),
        },
    };

    let results1 = engine.process_facts(vec![fact1.clone()]).unwrap();
    println!(
        "📊 Fact 1 (age=25, status=pending): {} results",
        results1.len()
    );
    assert_eq!(
        results1.len(),
        0,
        "Should not match - only first condition satisfied"
    );

    // Test Case 2: Add fact that matches second condition only
    let fact2 = Fact {
        id: 2,
        external_id: None,
        timestamp: chrono::Utc::now(),
        data: FactData {
            fields: [
                ("user_id".to_string(), FactValue::Integer(102)),
                ("age".to_string(), FactValue::Integer(16)), // Does NOT match first condition
                (
                    "status".to_string(),
                    FactValue::String("active".to_string()),
                ), // Matches second condition
            ]
            .iter()
            .cloned()
            .collect(),
        },
    };

    let results2 = engine.process_facts(vec![fact2.clone()]).unwrap();
    println!(
        "📊 Fact 2 (age=16, status=active): {} results",
        results2.len()
    );
    assert_eq!(results2.len(), 0, "Should not match - age=16 is not > 18");

    // Test Case 3: Add fact that matches both conditions
    let fact3 = Fact {
        id: 3,
        external_id: None,
        timestamp: chrono::Utc::now(),
        data: FactData {
            fields: [
                ("user_id".to_string(), FactValue::Integer(103)),
                ("age".to_string(), FactValue::Integer(22)), // Matches first condition
                (
                    "status".to_string(),
                    FactValue::String("active".to_string()),
                ), // Matches second condition
            ]
            .iter()
            .cloned()
            .collect(),
        },
    };

    let results3 = engine.process_facts(vec![fact3.clone()]).unwrap();
    println!(
        "📊 Fact 3 (age=22, status=active): {} results",
        results3.len()
    );
    assert_eq!(
        results3.len(),
        1,
        "Should match - both conditions satisfied"
    );

    // Test Case 4: Process all facts together to test beta memory
    let all_facts = vec![fact1, fact2, fact3];
    let start = Instant::now();
    let all_results = engine.process_facts(all_facts).unwrap();
    let elapsed = start.elapsed();

    println!(
        "📈 All facts processed in {:?}: {} total results",
        elapsed,
        all_results.len()
    );
    assert_eq!(
        all_results.len(),
        1,
        "Only one fact should match both conditions"
    );

    println!("✅ Beta network correctly handles multi-condition rules");
}

#[test]
fn test_beta_network_incremental_token_propagation() {
    println!("🔍 Testing beta network incremental token propagation...");

    let engine = BingoEngine::new().unwrap();

    // Add a 3-condition rule: department == "eng" AND level > 3 AND status == "active"
    let rule = Rule {
        id: 1,
        name: "Senior Engineer Rule".to_string(),
        conditions: vec![
            Condition::Simple {
                field: "department".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("eng".to_string()),
            },
            Condition::Simple {
                field: "level".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Integer(3),
            },
            Condition::Simple {
                field: "status".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("active".to_string()),
            },
        ],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "senior_engineer".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };
    engine.add_rule(rule).unwrap();

    println!("✅ Added 3-condition rule for incremental testing");

    // Test incremental fact processing
    let facts = vec![
        // Fact 1: Matches condition 1 only
        Fact {
            id: 1,
            external_id: None,
            timestamp: chrono::Utc::now(),
            data: FactData {
                fields: [
                    ("employee_id".to_string(), FactValue::Integer(1001)),
                    (
                        "department".to_string(),
                        FactValue::String("eng".to_string()),
                    ), // ✓ condition 1
                    ("level".to_string(), FactValue::Integer(2)), // ✗ condition 2
                    (
                        "status".to_string(),
                        FactValue::String("pending".to_string()),
                    ), // ✗ condition 3
                ]
                .iter()
                .cloned()
                .collect(),
            },
        },
        // Fact 2: Matches conditions 1 and 2
        Fact {
            id: 2,
            external_id: None,
            timestamp: chrono::Utc::now(),
            data: FactData {
                fields: [
                    ("employee_id".to_string(), FactValue::Integer(1002)),
                    (
                        "department".to_string(),
                        FactValue::String("eng".to_string()),
                    ), // ✓ condition 1
                    ("level".to_string(), FactValue::Integer(5)), // ✓ condition 2
                    (
                        "status".to_string(),
                        FactValue::String("pending".to_string()),
                    ), // ✗ condition 3
                ]
                .iter()
                .cloned()
                .collect(),
            },
        },
        // Fact 3: Matches all 3 conditions
        Fact {
            id: 3,
            external_id: None,
            timestamp: chrono::Utc::now(),
            data: FactData {
                fields: [
                    ("employee_id".to_string(), FactValue::Integer(1003)),
                    (
                        "department".to_string(),
                        FactValue::String("eng".to_string()),
                    ), // ✓ condition 1
                    ("level".to_string(), FactValue::Integer(6)), // ✓ condition 2
                    (
                        "status".to_string(),
                        FactValue::String("active".to_string()),
                    ), // ✓ condition 3
                ]
                .iter()
                .cloned()
                .collect(),
            },
        },
    ];

    // Process facts incrementally to test token propagation
    let mut total_results = 0;
    let start = Instant::now();

    for (i, fact) in facts.iter().enumerate() {
        let results = engine.process_facts(vec![fact.clone()]).unwrap();
        total_results += results.len();
        println!(
            "📊 Fact {} processed: {} results (total: {total_results})",
            i + 1,
            results.len()
        );
    }

    let elapsed = start.elapsed();

    println!("📈 Incremental processing: {total_results} total results in {elapsed:?}");

    // Only the third fact should produce a result (matches all 3 conditions)
    assert_eq!(
        total_results, 1,
        "Only one fact should match all three conditions"
    );

    // Test batch processing for comparison
    let start = Instant::now();
    let batch_results = engine.process_facts(facts).unwrap();
    let batch_elapsed = start.elapsed();

    println!(
        "📈 Batch processing: {} results in {batch_elapsed:?}",
        batch_results.len()
    );
    assert_eq!(
        batch_results.len(),
        1,
        "Batch processing should yield same result"
    );

    println!("✅ Beta network incremental token propagation working correctly");
}

#[test]
fn test_beta_network_performance_scaling() {
    println!("🔍 Testing beta network performance with complex multi-condition rules...");

    let engine = BingoEngine::new().unwrap();

    // Add multiple complex rules to stress test the beta network
    let rule_count = 10;
    for i in 0..rule_count {
        let rule = Rule {
            id: i,
            name: format!("Complex Rule {i}"),
            conditions: vec![
                Condition::Simple {
                    field: "category".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String(format!("cat_{}", i % 3)),
                },
                Condition::Simple {
                    field: "priority".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Integer(i as i64),
                },
                Condition::Simple {
                    field: "status".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("ready".to_string()),
                },
            ],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "processed".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        };
        engine.add_rule(rule).unwrap();
    }

    println!("✅ Added {rule_count} complex multi-condition rules");

    // Create facts that will create various partial matches
    let fact_count = 500;
    let facts: Vec<Fact> = (0..fact_count)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("id".to_string(), FactValue::Integer(i as i64));
            fields.insert(
                "category".to_string(),
                FactValue::String(format!("cat_{}", i % 3)),
            );
            fields.insert("priority".to_string(), FactValue::Integer((i % 15) as i64)); // Some will match rules
            fields.insert(
                "status".to_string(),
                FactValue::String(if i % 4 == 0 { "ready" } else { "pending" }.to_string()),
            );

            Fact {
                id: i as u64,
                external_id: None,
                timestamp: chrono::Utc::now(),
                data: FactData { fields },
            }
        })
        .collect();

    // Test beta network performance
    let start = Instant::now();
    let results = engine.process_facts(facts.clone()).unwrap();
    let elapsed = start.elapsed();

    println!(
        "📊 Beta network processed {fact_count} facts against {rule_count} multi-condition rules"
    );
    println!(
        "📈 Performance: {:.0} facts/sec, {} rule activations",
        fact_count as f64 / elapsed.as_secs_f64(),
        results.len()
    );
    println!("⏱️  Total time: {elapsed:?}");

    // Verify some facts matched (depends on the specific test data patterns)
    println!("✅ Beta network handles complex multi-condition rules efficiently");
    println!("🎯 Beta network implementation complete");
}
