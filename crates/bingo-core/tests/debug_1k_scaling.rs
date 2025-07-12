use bingo_core::*;
use std::collections::HashMap;

#[test]
fn test_1k_fact_scaling() {
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

    // Generate 1K facts
    let facts: Vec<Fact> = (0..1_000)
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
        "✅ Processed 1K facts in {:?}, generated {} results",
        elapsed,
        results.len()
    );

    let stats = engine.get_stats();
    println!("Final engine stats: {stats:?}");

    // Should be reasonably fast
    assert!(
        elapsed.as_millis() < 100,
        "Should process 1K facts under 100ms"
    );

    // Expected: facts with IDs 0, 3, 6, 9, etc. should match (~333 results)
    assert!(
        results.len() > 300,
        "Should generate results for ~33% of facts (got {})",
        results.len()
    );

    // Let's check some specific results
    let matched_fact_ids: Vec<u64> = results.iter().map(|r| r.fact_id).collect();
    assert!(matched_fact_ids.contains(&0), "Fact 0 should match");
    assert!(matched_fact_ids.contains(&3), "Fact 3 should match");
    assert!(matched_fact_ids.contains(&6), "Fact 6 should match");
    assert!(matched_fact_ids.contains(&9), "Fact 9 should match");

    // Check that we get the expected count
    let expected_count = 1_000 / 3 + if 1_000 % 3 > 0 { 1 } else { 0 };
    assert_eq!(
        results.len(),
        expected_count,
        "Should get exactly {expected_count} results"
    );
}
