use bingo_core::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn profile_performance_bottleneck() {
    println!("🔍 Performance Bottleneck Analysis");
    println!("==================================");

    // Test scaling with detailed timing of individual operations
    for fact_count in [1000, 2000, 3000, 4000, 5000] {
        println!("\n📊 Testing {fact_count} facts:");

        let engine = BingoEngine::new().unwrap();

        // Add rule
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
        println!("  Rule addition: {:?}", start.elapsed());

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
        let fact_generation_time = start.elapsed();
        println!("  Fact generation: {fact_generation_time:?}");

        // Process facts
        let start = Instant::now();
        let results = engine.process_facts(facts).unwrap();
        let processing_time = start.elapsed();
        println!("  Fact processing: {processing_time:?}");

        let stats = engine.get_stats();
        println!(
            "  Results: {} | Total facts: {}",
            results.len(),
            stats.fact_count
        );

        // Calculate scaling metrics
        let facts_per_second = fact_count as f64 / processing_time.as_secs_f64();
        let time_per_fact_us = processing_time.as_micros() as f64 / fact_count as f64;

        println!(
            "  Performance: {facts_per_second:.0} facts/sec | {time_per_fact_us:.2}µs per fact"
        );

        // Break if performance is getting too slow
        if processing_time.as_millis() > 5000 {
            println!("  ⚠️ Performance degraded significantly, stopping analysis");
            break;
        }

        // Check for quadratic scaling
        if fact_count > 1000 {
            let expected_linear_time = processing_time.as_micros() * 1000 / fact_count as u128;
            let actual_time_per_1k = processing_time.as_micros();
            let scaling_factor = actual_time_per_1k as f64 / expected_linear_time as f64;

            if scaling_factor > 2.0 {
                println!(
                    "  ❌ SCALING ISSUE: {scaling_factor}x worse than linear (expected linear)"
                );
            } else {
                println!("  ✅ Scaling: {scaling_factor:.2}x linear");
            }
        }
    }

    println!("\n🎯 Profiling complete");
}
