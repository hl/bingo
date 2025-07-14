use bingo_core::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_performance_profiling() {
    println!("🔍 Starting performance profiling...");

    let start_total = Instant::now();
    let engine = BingoEngine::new().unwrap();
    println!("✅ Engine creation: {:?}", start_total.elapsed());

    // Add a simple rule
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

    engine.add_rule(rule).unwrap();
    println!("✅ Rule addition: {:?}", rule_start.elapsed());

    // Test different fact counts to find the breaking point
    for fact_count in [100, 500, 1000, 2000, 5000] {
        println!("\n📊 Testing with {fact_count} facts...");

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

        let process_start = Instant::now();
        let results = engine.process_facts(facts).unwrap();
        let process_time = process_start.elapsed();

        println!("  ⚡ Processing time: {process_time:?}");
        println!("  📈 Results: {} matches", results.len());
        println!(
            "  🚀 Throughput: {:.0} facts/sec",
            fact_count as f64 / process_time.as_secs_f64()
        );

        // If processing takes more than 5 seconds, stop testing larger sizes
        if process_time.as_secs() > 5 {
            println!("  ⚠️  Processing too slow, stopping here");
            break;
        }
    }

    println!("\n⏱️ Total test time: {:?}", start_total.elapsed());
}
