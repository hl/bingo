use bingo_core::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_isolated_performance_components() {
    println!("🔍 Isolated Performance Component Analysis");
    println!("==========================================");

    // Test fact store operations in isolation
    for fact_count in [1000, 2000, 3000, 4000, 5000] {
        println!("\n📊 Testing {fact_count} facts - Fact Store Only:");

        use bingo_core::fact_store::arena_store::ArenaFactStore;

        let fact_store = ArenaFactStore::new();

        // Generate facts
        let start = Instant::now();
        let facts: Vec<Fact> = (0..fact_count)
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
        let fact_generation_time = start.elapsed();

        // Test fact store insertion
        let start = Instant::now();
        let _fact_ids = fact_store.bulk_insert_slice(&facts);
        let fact_store_time = start.elapsed();

        println!("  Fact generation: {fact_generation_time:?}");
        println!("  Fact store insert: {fact_store_time:?}");

        let facts_per_second = fact_count as f64 / fact_store_time.as_secs_f64();
        println!("  Fact store performance: {facts_per_second:.0} facts/sec");

        // Test fact lookup performance
        let start = Instant::now();
        let mut lookup_count = 0;
        for i in 0..std::cmp::min(fact_count, 100) {
            if let Some(_fact) = fact_store.get_fact(i as u64) {
                lookup_count += 1;
            }
        }
        let lookup_time = start.elapsed();
        println!("  Fact lookup ({lookup_count} lookups): {lookup_time:?}");

        // Test search performance
        let start = Instant::now();
        let search_results =
            fact_store.find_by_field("status", &FactValue::String("active".to_string()));
        let search_time = start.elapsed();
        println!(
            "  Fact search ({} results): {:?}",
            search_results.len(),
            search_time
        );

        if fact_store_time.as_millis() > 1000 {
            println!("  ⚠️ Fact store performance degraded, stopping");
            break;
        }
    }

    // Test RETE network operations in isolation (no fact store)
    println!("\n🧪 Testing RETE Network Performance (isolated):");

    for fact_count in [1000, 2000, 3000] {
        let engine = BingoEngine::new().unwrap();

        // Add rule
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

        // Create minimal facts to isolate RETE processing
        let facts: Vec<Fact> = (0..fact_count)
            .map(|i| {
                let mut fields = HashMap::new();
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

        let start = Instant::now();
        let results = engine.process_facts(facts).unwrap();
        let processing_time = start.elapsed();

        println!(
            "  {} facts: {:?} | {} results",
            fact_count,
            processing_time,
            results.len()
        );

        if processing_time.as_millis() > 2000 {
            println!("  ⚠️ RETE performance degraded, stopping");
            break;
        }
    }

    println!("\n✅ Isolated performance analysis complete");
}
