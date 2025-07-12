use bingo_core::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_rete_direct_performance() {
    println!("🔍 Testing RETE network performance directly (bypassing engine)...");

    let mut rete_network = bingo_core::rete_network::ReteNetwork::new();
    let fact_store = bingo_core::fact_store::arena_store::ArenaFactStore::new();
    let calculator = bingo_calculator::calculator::Calculator::new();

    // Add a simple rule directly to RETE network
    let rule_start = Instant::now();
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

    rete_network.add_rule(rule).unwrap();
    println!("✅ RETE rule addition: {:?}", rule_start.elapsed());

    // Test different fact counts directly against RETE network
    for fact_count in [100, 500, 1000, 2000, 5000, 10000] {
        println!("\n📊 Testing RETE directly with {fact_count} facts...");

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

        let rete_start = Instant::now();
        let results = rete_network.process_facts(&facts, &fact_store, &calculator).unwrap();
        let rete_time = rete_start.elapsed();

        println!("  ⚡ RETE processing time: {rete_time:?}");
        println!("  📈 Results: {} matches", results.len());
        println!(
            "  🚀 Throughput: {:.0} facts/sec",
            fact_count as f64 / rete_time.as_secs_f64()
        );

        // If RETE processing takes more than 1 second, stop testing larger sizes
        if rete_time.as_secs() > 1 {
            println!("  ⚠️  RETE processing too slow, stopping here");
            break;
        }
    }
}
