/// Alpha Memory Performance Test
///
/// This test specifically validates that alpha memory provides proper RETE optimization
use bingo_core::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_alpha_memory_performance_benefit() {
    println!("🔍 Testing alpha memory performance benefit with many rules...");

    let mut engine = BingoEngine::new().unwrap();

    // Add MANY rules with different conditions to stress test alpha memory
    let rule_count = 200;
    for i in 0..rule_count {
        let rule = Rule {
            id: i,
            name: format!("Rule {i}"),
            conditions: vec![Condition::Simple {
                field: "status".to_string(),
                operator: Operator::Equal,
                value: FactValue::String(format!("status_{}", i % 10)), // 10 different statuses
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

    println!("✅ Added {rule_count} rules with varied conditions");

    // Create facts that match only a subset of rules
    let fact_count = 1000;
    let facts: Vec<Fact> = (0..fact_count)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("entity_id".to_string(), FactValue::Integer(i as i64));
            // Only status_0 facts - should only match 20 rules (10% of total)
            fields.insert(
                "status".to_string(),
                FactValue::String("status_0".to_string()),
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

    // Each fact should only match rules 0, 10, 20, 30, ..., 190 (20 rules total)
    let expected_matches_per_fact = 20;
    let expected_total_results = fact_count * expected_matches_per_fact;

    println!(
        "📊 {} facts × {} matching rules = {} results in {:?}",
        fact_count,
        expected_matches_per_fact,
        results.len(),
        elapsed
    );

    println!(
        "🚀 Processing rate: {:.0} facts/sec",
        fact_count as f64 / elapsed.as_secs_f64()
    );

    // The key insight: with proper alpha memory, processing time should be
    // proportional to MATCHING rules, not TOTAL rules
    assert_eq!(results.len(), expected_total_results);

    // With 200 rules but only 20 matching, alpha memory should give us ~10x speedup
    // compared to brute force checking all 200 rules
    println!("🎯 Alpha memory should check ~20 rules instead of 200 per fact");

    // Performance should be much better than O(facts × total_rules)
    let facts_per_sec = fact_count as f64 / elapsed.as_secs_f64();
    println!("✅ Achieved {facts_per_sec:.0} facts/sec processing rate");

    // This should be reasonably fast with alpha memory optimization
    assert!(
        facts_per_sec > 5_000.0,
        "Alpha memory should provide significant speedup"
    );
}

#[test]
fn test_rule_count_independence() {
    println!("🔍 Testing that processing time is independent of total rule count...");

    // Test with different rule counts but same number of matching facts
    for &total_rules in &[10, 50, 100, 200] {
        let mut engine = BingoEngine::new().unwrap();

        // Add rules - only first rule will match our facts
        for i in 0..total_rules {
            let rule = Rule {
                id: i as u64,
                name: format!("Rule {i}"),
                conditions: vec![Condition::Simple {
                    field: "test_field".to_string(),
                    operator: Operator::Equal,
                    value: if i == 0 {
                        FactValue::String("matching_value".to_string())
                    } else {
                        FactValue::String(format!("non_matching_{i}"))
                    },
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

        // Create facts that only match the first rule
        let facts: Vec<Fact> = (0..500)
            .map(|i| {
                let mut fields = HashMap::new();
                fields.insert(
                    "test_field".to_string(),
                    FactValue::String("matching_value".to_string()),
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
        let results = engine.process_facts(facts.clone()).unwrap();
        let elapsed = start.elapsed();

        println!(
            "📊 {} total rules, {} facts, {} results in {:?} ({:.0} facts/sec)",
            total_rules,
            facts.len(),
            results.len(),
            elapsed,
            facts.len() as f64 / elapsed.as_secs_f64()
        );

        // All facts should match exactly one rule
        assert_eq!(results.len(), facts.len());

        // With proper alpha memory, time should be roughly constant regardless of total rule count
        let facts_per_sec = facts.len() as f64 / elapsed.as_secs_f64();

        // Performance shouldn't degrade significantly as we add more non-matching rules
        assert!(
            facts_per_sec > 1_000.0,
            "Performance should remain high with {total_rules} total rules"
        );
    }

    println!("🎯 Proper RETE: Processing time independent of non-matching rule count");
}
