//! Debug test for threaded processing

use bingo_core::{engine::BingoEngine, parallel_rete::ParallelReteConfig, types::*};
use std::collections::HashMap;

fn create_debug_fact(id: u64, age: i64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert("age".to_string(), FactValue::Integer(age));
    fields.insert(
        "test_field".to_string(),
        FactValue::String(format!("value_{id}")),
    );

    Fact {
        id,
        external_id: Some(format!("debug_fact_{id}")),
        timestamp: chrono::Utc::now(),
        data: FactData { fields },
    }
}

fn create_debug_rule(id: u64) -> Rule {
    Rule {
        id,
        name: format!("Debug Rule {id}"),
        conditions: vec![Condition::Simple {
            field: "age".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(0), // Match all facts
        }],
        actions: vec![Action {
            action_type: ActionType::Log { message: format!("Debug rule {id} fired") },
        }],
    }
}

#[test]
fn test_debug_threaded_processing() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Add multiple rules to increase processing work
    let rules = vec![create_debug_rule(1), create_debug_rule(2), create_debug_rule(3)];

    engine.add_rules_to_parallel_rete(rules).expect("Failed to add rules");

    // Create many facts to ensure work distribution
    let facts: Vec<Fact> = (1..=20).map(|i| create_debug_fact(i, (i * 5) as i64)).collect();

    // Configure for guaranteed parallel processing
    let config = ParallelReteConfig {
        worker_count: 2,
        parallel_threshold: 1, // Very low threshold
        fact_chunk_size: 2,    // Small chunks
        enable_parallel_alpha: true,
        enable_parallel_beta: true,
        enable_parallel_execution: true,
        work_queue_capacity: 100,
        ..Default::default()
    };

    println!("=== Debug Threaded Processing Test ===");
    println!("Facts to process: {}", facts.len());
    println!("Worker count: {}", config.worker_count);
    println!("Chunk size: {}", config.fact_chunk_size);
    println!("Parallel threshold: {}", config.parallel_threshold);

    // Reset stats before processing
    let _ = engine.reset_parallel_rete_stats();

    // Process facts using threaded processing
    let start_time = std::time::Instant::now();
    println!("About to call process_facts_parallel_threaded...");
    let result = engine.process_facts_parallel_threaded(facts, &config);
    let duration = start_time.elapsed();
    println!("Returned from process_facts_parallel_threaded");

    match result {
        Ok(results) => {
            println!("✅ Processing completed in {duration:?}");
            println!("   Results generated: {}", results.len());

            // Get detailed stats
            if let Ok(stats) = engine.get_parallel_rete_stats() {
                println!("=== Detailed Statistics ===");
                println!("   Worker count reported: {}", stats.worker_count);
                println!("   Facts processed: {}", stats.facts_processed);
                println!("   Tokens processed: {}", stats.tokens_processed);
                println!("   Rules executed: {}", stats.rules_executed);
                println!(
                    "   Total processing time: {}ms",
                    stats.total_processing_time_ms
                );
                println!("   Work items stolen: {}", stats.work_items_stolen);
                println!("   Queue overflows: {}", stats.queue_overflows);
                println!(
                    "   Worker utilization: {:.1}%",
                    stats.worker_utilization * 100.0
                );

                // The worker count being set to config value indicates the method ran
                // But other stats being 0 suggests workers didn't process anything
                if stats.worker_count > 0 && stats.facts_processed == 0 {
                    println!("⚠️  Workers were configured but didn't process facts");
                    println!("    This suggests work distribution or worker processing issues");
                }
            }

            // Check engine stats
            let engine_stats = engine.get_stats();
            println!("=== Engine Statistics ===");
            println!("   Total rules: {}", engine_stats.rule_count);
            println!("   Total facts: {}", engine_stats.fact_count);
        }
        Err(e) => {
            println!("❌ Processing failed: {e:?}");
            panic!("Failed to process facts: {e:?}");
        }
    }
}
