use bingo_core::*;
use std::collections::HashMap;

#[test]
fn test_10k_fact_scaling() {
    let mut engine = BingoEngine::new().unwrap();

    // Add a simple rule
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

    // Generate 10K facts
    let facts: Vec<Fact> = (0..10_000)
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

    let start = std::time::Instant::now();
    let results = engine.process_facts(facts).unwrap();
    let elapsed = start.elapsed();

    println!(
        "✅ Processed 10K facts in {:?}, generated {} results",
        elapsed,
        results.len()
    );

    let stats = engine.get_stats();
    println!("Final engine stats: {stats:?}");

    // Should be reasonably fast
    assert!(
        elapsed.as_millis() < 1000,
        "Should process 10K facts under 1s"
    );

    // Expected: facts with IDs 0, 3, 6, 9, etc. should match (~3333 results)
    assert!(
        results.len() > 3000,
        "Should generate results for ~33% of facts (got {})",
        results.len()
    );

    // Check that we get the expected count
    let expected_count = 10_000 / 3 + if 10_000 % 3 > 0 { 1 } else { 0 };
    assert_eq!(
        results.len(),
        expected_count,
        "Should get exactly {expected_count} results"
    );
}
