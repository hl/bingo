use bingo_core::*;
use std::collections::HashMap;

#[test]
fn test_debug_rule_matching() {
    println!("Testing rule matching with small dataset...");

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
    println!("Rule added successfully");

    // Generate small test set
    let facts: Vec<Fact> = (0..10)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("entity_id".to_string(), FactValue::Integer(i as i64));
            fields.insert(
                "status".to_string(),
                FactValue::String(if i % 3 == 0 { "active" } else { "inactive" }.to_string()),
            );

            Fact {
                id: i as u64,
                external_id: None,
                timestamp: chrono::Utc::now(),
                data: FactData { fields },
            }
        })
        .collect();

    println!("Generated {} facts", facts.len());

    // Print first few facts to debug
    for (i, fact) in facts.iter().take(5).enumerate() {
        println!("Fact {}: {:?}", i, fact.data.fields);
    }

    let start = std::time::Instant::now();
    let results = engine.process_facts(facts).unwrap();
    let elapsed = start.elapsed();

    println!(
        "Processed 10 facts in {:?}, generated {} results",
        elapsed,
        results.len()
    );

    // Expected: facts with IDs 0, 3, 6, 9 should match (4 results)
    println!("Expected ~4 results (facts 0, 3, 6, 9 have status=active)");

    if !results.is_empty() {
        println!("First few results:");
        for (i, result) in results.iter().take(5).enumerate() {
            println!(
                "Result {}: rule_id={}, fact_id={}",
                i, result.rule_id, result.fact_id
            );
        }
    }

    // Assert we get the expected results
    assert_eq!(
        results.len(),
        4,
        "Should get 4 results for facts 0, 3, 6, 9"
    );

    // Check that the correct facts matched
    let matched_fact_ids: Vec<u64> = results.iter().map(|r| r.fact_id).collect();
    assert!(matched_fact_ids.contains(&0), "Fact 0 should match");
    assert!(matched_fact_ids.contains(&3), "Fact 3 should match");
    assert!(matched_fact_ids.contains(&6), "Fact 6 should match");
    assert!(matched_fact_ids.contains(&9), "Fact 9 should match");
}
