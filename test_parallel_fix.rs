use bingo_core::{engine::BingoEngine, parallel_rete::ParallelReteConfig, types::*};
use std::collections::HashMap;

fn main() {
    println!("🧪 Testing parallel processing fix");
    
    let mut engine = BingoEngine::new().expect("Failed to create engine");
    
    // Create a simple rule: age > 21
    let rule = Rule {
        id: 1,
        name: "Age Check".to_string(),
        conditions: vec![Condition::Simple {
            field: "age".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(21),
        }],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Rule fired".to_string() },
        }],
    };
    
    // Create a simple fact: age = 25
    let mut fields = HashMap::new();
    fields.insert("age".to_string(), FactValue::Integer(25));
    let fact = Fact {
        id: 1,
        external_id: Some("fact_1".to_string()),
        timestamp: chrono::Utc::now(),
        data: FactData { fields },
    };
    
    println!("📋 Rule: age > 21");
    println!("📄 Fact: age = 25");
    
    // Add rule to parallel RETE
    engine.add_rules_to_parallel_rete(vec![rule]).expect("Failed to add rule");
    println!("✅ Rule added to parallel RETE");
    
    // Test threaded processing with fix
    let config = ParallelReteConfig {
        worker_count: 1,
        parallel_threshold: 1, // Force parallel processing
        ..Default::default()
    };
    
    let threaded_results = engine
        .process_facts_parallel_threaded(vec![fact], &config)
        .expect("Threaded processing failed");
    
    println!("🧵 Threaded processing results: {}", threaded_results.len());
    
    if threaded_results.len() > 0 {
        println!("🎉 SUCCESS! Parallel processing now generates results!");
        for result in &threaded_results {
            println!("   Rule {} fired for fact {}", result.rule_id, result.fact_id);
        }
    } else {
        println!("❌ FAILED: Still no results from parallel processing");
    }
}