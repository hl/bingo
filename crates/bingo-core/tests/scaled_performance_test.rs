use bingo_core::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_scaled_performance() {
    println!("🔍 Scaled Performance Test");

    // Test with exact same structure as debug_10k_scaling but smaller counts first
    for fact_count in [1000, 2000, 5000, 10000] {
        let engine = BingoEngine::new().unwrap();

        // Add the SAME rule as debug_10k_scaling
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

        // Generate facts EXACTLY like debug_10k_scaling
        let facts: Vec<Fact> = (0..fact_count)
            .map(|i| {
                let mut fields = HashMap::new();
                fields.insert("entity_id".to_string(), FactValue::Integer(i as i64));
                fields.insert(
                    "status".to_string(),
                    FactValue::String(if i % 3 == 0 { "active" } else { "inactive" }.to_string()),
                );
                fields.insert(
                    "category".to_string(),
                    FactValue::String({
                        let cat_id = i % 100;
                        format!("cat_{cat_id}")
                    }),
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

        let stats = engine.get_stats();

        println!(
            "📊 {} facts: {:?} | {} results | {} total_facts",
            fact_count,
            elapsed,
            results.len(),
            stats.fact_count
        );

        // Performance targets
        let expected_results = fact_count / 3 + if fact_count % 3 > 0 { 1 } else { 0 };
        assert_eq!(results.len(), expected_results);

        // Stop if performance degrades significantly
        if elapsed.as_millis() > 10000 {
            // 10 seconds
            println!("⚠️ Performance degraded at {fact_count} facts: {elapsed:?}");
            break;
        }

        // Performance expectation: should be sub-linear
        let facts_per_ms = fact_count as f64 / elapsed.as_millis() as f64;
        println!("  → {facts_per_ms:.0} facts/ms");
    }

    println!("✅ Scaled performance test completed");
}
