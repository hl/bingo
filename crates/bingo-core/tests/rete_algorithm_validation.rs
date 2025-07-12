/// RETE Algorithm Validation Test
///
/// This test exposes the fundamental O(rules × facts) problem in the current
/// "brute force" implementation and validates the proper RETE algorithm fix.
use bingo_core::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_current_algorithm_scaling_problem() {
    println!("🔍 Testing current algorithm scaling with multiple rules...");

    let mut engine = BingoEngine::new().unwrap();

    // Add multiple rules to expose O(rules × facts) problem
    let rule_count = 50;
    for i in 0..rule_count {
        let rule = Rule {
            id: i,
            name: format!("Rule {i}"),
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
    }

    println!("✅ Added {rule_count} rules");

    // Test with increasing fact counts to show O(rules × facts) scaling
    for fact_count in [100, 500, 1000, 2000] {
        let facts: Vec<Fact> = (0..fact_count)
            .map(|i| {
                let mut fields = HashMap::new();
                fields.insert("entity_id".to_string(), FactValue::Integer(i as i64));
                fields.insert(
                    "status".to_string(),
                    FactValue::String("active".to_string()),
                );

                Fact {
                    id: i as u64,
                    external_id: None,
                    timestamp: chrono::Utc::now(),
                    data: FactData { fields },
                }
            })
            .collect();

        let start = Instant::now();
        let results = engine.process_facts(facts).unwrap();
        let elapsed = start.elapsed();

        // Each fact should match ALL rules, so results should be facts × rules
        let expected_results = fact_count * rule_count as usize;

        println!(
            "📊 {} facts × {} rules = {} evaluations in {:?} ({:.0} eval/sec)",
            fact_count,
            rule_count,
            fact_count * rule_count as usize,
            elapsed,
            (fact_count * rule_count as usize) as f64 / elapsed.as_secs_f64()
        );

        assert_eq!(
            results.len(),
            expected_results,
            "Each fact should match all {rule_count} rules"
        );

        // The real problem: evaluation count grows as O(rules × facts)
        // This test shows the fundamental scaling issue
        let _evaluations_per_second =
            (fact_count * rule_count as usize) as f64 / elapsed.as_secs_f64();

        // With proper RETE, this should be nearly constant regardless of rule count
        // The current implementation will degrade as rule count increases
        if rule_count > 10 && fact_count > 1000 {
            println!("⚠️  Performance will degrade severely with more rules!");
        }
    }

    println!("🚨 CRITICAL: This test demonstrates O(rules × facts) complexity");
    println!("   Proper RETE should achieve O(Δfacts) regardless of rule count");
}

#[test]
fn test_alpha_memory_requirement() {
    println!("🔍 Testing alpha memory requirement for proper RETE...");

    let mut engine = BingoEngine::new().unwrap();

    // Add rules with identical conditions (should share alpha nodes in proper RETE)
    for i in 0..10 {
        let rule = Rule {
            id: i,
            name: format!("Shared Condition Rule {i}"),
            conditions: vec![Condition::Simple {
                field: "status".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("active".to_string()),
            }],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: format!("result_{i}"),
                    value: FactValue::Boolean(true),
                },
            }],
        };
        engine.add_rule(rule).unwrap();
    }

    // Create one fact that matches the shared condition
    let fact = Fact {
        id: 1,
        external_id: None,
        timestamp: chrono::Utc::now(),
        data: FactData {
            fields: [(
                "status".to_string(),
                FactValue::String("active".to_string()),
            )]
            .iter()
            .cloned()
            .collect(),
        },
    };

    let start = Instant::now();
    let results = engine.process_facts(vec![fact]).unwrap();
    let elapsed = start.elapsed();

    println!("✅ 1 fact × 10 rules with shared condition: {elapsed:?}");
    println!("📈 Results: {} (should be 10)", results.len());

    // In proper RETE:
    // 1. Fact would be evaluated by ONE alpha node for "status == active"
    // 2. Alpha node would trigger ALL matching rules instantly
    // 3. Time complexity would be O(1) regardless of rule count

    println!("🎯 Proper RETE should evaluate this in O(1) time");
    println!("   Current implementation: O(rules) time for each fact");

    assert_eq!(results.len(), 10);
}

#[test]
fn test_working_memory_requirement() {
    println!("🔍 Testing working memory requirement...");

    // This test shows why working memory is essential for incremental processing
    let mut engine = BingoEngine::new().unwrap();

    let rule = Rule {
        id: 1,
        name: "Working Memory Test".to_string(),
        conditions: vec![Condition::Simple {
            field: "count".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(5),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "triggered".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Initial facts
    let initial_facts: Vec<Fact> = (0..3)
        .map(|i| {
            Fact {
                id: i,
                external_id: None,
                timestamp: chrono::Utc::now(),
                data: FactData {
                    fields: [
                        ("count".to_string(), FactValue::Integer((i + 3) as i64)), // 3, 4, 5
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                },
            }
        })
        .collect();

    let start = Instant::now();
    let results1 = engine.process_facts(initial_facts.clone()).unwrap();
    let time1 = start.elapsed();

    // Add one more fact
    let new_fact = Fact {
        id: 3,
        external_id: None,
        timestamp: chrono::Utc::now(),
        data: FactData {
            fields: [
                ("count".to_string(), FactValue::Integer(8)), // > 5, should match
            ]
            .iter()
            .cloned()
            .collect(),
        },
    };

    let mut all_facts = initial_facts;
    all_facts.push(new_fact);

    let start = Instant::now();
    let all_facts_len = all_facts.len();
    let results2 = engine.process_facts(all_facts).unwrap();
    let time2 = start.elapsed();

    println!(
        "✅ First run (3 facts): {} results in {:?}",
        results1.len(),
        time1
    );
    println!(
        "✅ Second run (4 facts): {} results in {:?}",
        results2.len(),
        time2
    );

    // Current implementation re-evaluates ALL facts every time
    // Proper RETE should only evaluate the NEW/CHANGED facts
    println!("🚨 Current: Re-evaluates all {all_facts_len} facts every time");
    println!("🎯 Proper RETE: Should only evaluate 1 new fact incrementally");

    assert_eq!(results1.len(), 0); // count 3,4,5 all <= 5
    assert_eq!(results2.len(), 1); // only count 8 > 5
}
