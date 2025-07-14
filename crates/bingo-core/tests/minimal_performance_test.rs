use bingo_core::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_minimal_scaling() {
    println!("🔍 Minimal Performance Test");

    // Test different fact counts
    for fact_count in [10, 100, 500, 1000] {
        let engine = BingoEngine::new().unwrap();

        // Add ONE simple rule
        let rule = Rule {
            id: 1,
            name: "Simple Rule".to_string(),
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

        // Generate minimal facts
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

        // If it takes more than 1 second for 1000 facts, something is wrong
        if fact_count <= 1000 && elapsed.as_millis() > 1000 {
            panic!("Performance issue: {} facts took {:?}", fact_count, elapsed);
        }
    }

    println!("✅ Basic scaling test passed");
}
