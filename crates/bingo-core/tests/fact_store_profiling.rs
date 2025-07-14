use bingo_core::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_fact_store_insertion_performance() {
    println!("🔍 Testing fact store insertion performance...");

    // Test different fact counts for insertion
    for fact_count in [100, 500, 1000, 2000, 5000] {
        println!("\n📊 Testing fact store with {fact_count} facts...");

        let fact_store = bingo_core::fact_store::arena_store::ArenaFactStore::new();

        let fact_gen_start = Instant::now();
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
        println!("  📝 Fact generation: {:?}", fact_gen_start.elapsed());

        let insert_start = Instant::now();
        for fact in facts {
            fact_store.insert(fact);
        }
        let insert_time = insert_start.elapsed();

        println!("  💾 Fact store insertion: {insert_time:?}");
        println!(
            "  🚀 Insertion throughput: {:.0} facts/sec",
            fact_count as f64 / insert_time.as_secs_f64()
        );

        // If insertion takes more than 2 seconds, stop testing larger sizes
        if insert_time.as_secs() > 2 {
            println!("  ⚠️  Insertion too slow, stopping here");
            break;
        }
    }
}
