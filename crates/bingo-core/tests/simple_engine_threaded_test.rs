//! Simple test to verify BingoEngine threaded integration

use bingo_core::{engine::BingoEngine, parallel_rete::ParallelReteConfig, types::*};
use std::collections::HashMap;

fn create_simple_fact(id: u64, age: i64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert("age".to_string(), FactValue::Integer(age));

    Fact {
        id,
        external_id: Some(format!("fact_{id}")),
        timestamp: chrono::Utc::now(),
        data: FactData { fields },
    }
}

fn create_simple_rule(id: u64) -> Rule {
    Rule {
        id,
        name: format!("Test Rule {id}"),
        conditions: vec![Condition::Simple {
            field: "age".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(18),
        }],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Rule fired".to_string() },
        }],
    }
}

#[test]
fn test_simple_engine_threaded_integration() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Add a simple rule
    let rule = create_simple_rule(1);
    engine.add_rules_to_parallel_rete(vec![rule]).expect("Failed to add rule");

    // Create simple facts
    let facts = vec![create_simple_fact(1, 25), create_simple_fact(2, 30)];

    // Configure for minimal parallel processing
    let config = ParallelReteConfig {
        worker_count: 1,
        parallel_threshold: 1, // Force parallel processing
        fact_chunk_size: 1,
        enable_parallel_alpha: true,
        enable_parallel_beta: true,
        enable_parallel_execution: true,
        ..Default::default()
    };

    println!("Testing engine threaded processing...");

    // Process facts using threaded processing
    let result = engine.process_facts_parallel_threaded(facts, &config);
    match result {
        Ok(results) => {
            println!("✅ Threaded processing completed successfully");
            println!("   Results: {}", results.len());

            // Get stats
            if let Ok(stats) = engine.get_parallel_rete_stats() {
                println!("   Worker count: {}", stats.worker_count);
                println!("   Facts processed: {}", stats.facts_processed);
            }
        }
        Err(e) => {
            println!("❌ Threaded processing failed: {e:?}");
            panic!("Failed to process facts with threaded processing: {e:?}");
        }
    }
}

#[test]
fn test_method_exists() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");
    let config = ParallelReteConfig::default();
    let facts = vec![];

    // Just verify the method exists and can be called
    let result = engine.process_facts_parallel_threaded(facts, &config);
    assert!(result.is_ok(), "Method should exist and handle empty facts");
}
