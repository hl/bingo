use bingo_core::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn debug_performance_scaling() {
    println!("🔍 Performance Debugging Analysis");
    println!("=================================");

    // Test 1: Engine creation
    let start = Instant::now();
    let engine = BingoEngine::new().unwrap();
    println!("✅ Engine creation: {:?}", start.elapsed());

    // Test 2: Rule addition
    let start = Instant::now();
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
    println!("✅ Rule addition: {:?}", start.elapsed());

    // Test 3: Fact generation and processing (different sizes)
    for fact_count in [100, 500, 1000, 2000, 5000] {
        let start = Instant::now();
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
        let fact_gen_time = start.elapsed();

        // Test fact processing
        let start = Instant::now();
        let results = engine.process_facts(facts).unwrap();
        let processing_time = start.elapsed();

        let stats = engine.get_stats();

        println!(
            "📊 {} facts: gen={:?}, process={:?}, results={}, total_facts={}",
            fact_count,
            fact_gen_time,
            processing_time,
            results.len(),
            stats.fact_count
        );

        // If processing is getting slow, stop here
        if processing_time.as_millis() > 5000 {
            println!(
                "⚠️ Processing is getting slow, stopping at {} facts",
                fact_count
            );
            return; // Stop the test here
        }
    }

    println!("\n🎯 Performance looks good up to 5K facts");
}
