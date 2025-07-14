// Quick debug test to check if basic rule conditions work
use bingo_core::BingoEngine;
use bingo_core::types::{Action, ActionType, Condition, Fact, FactData, FactValue, Operator, Rule};
use chrono::Utc;
use std::collections::HashMap;

fn create_simple_fact(id: u64, department: &str, amount: f64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert("department".to_string(), FactValue::String(department.to_string()));
    fields.insert("amount".to_string(), FactValue::Float(amount));

    Fact {
        id,
        external_id: Some(format!("fact-{id}")),
        timestamp: Utc::now(),
        data: FactData { fields },
    }
}

fn main() {
    println!("🧪 Testing Basic Rule Functionality");

    let engine = BingoEngine::new().unwrap();

    // Create a simple rule with a basic condition
    let simple_rule = Rule {
        id: 1,
        name: "Simple Department Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "department".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("sales".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::Log {
                message: "Sales department fact detected".to_string(),
            },
        }],
    };

    println!("Adding simple rule...");
    engine.add_rule(simple_rule).unwrap();

    // Create test facts
    let facts = vec![
        create_simple_fact(1, "sales", 100.0),
        create_simple_fact(2, "marketing", 200.0),
        create_simple_fact(3, "sales", 150.0),
    ];

    println!("Processing {} facts...", facts.len());
    let results = engine.process_facts(facts).unwrap();

    println!("🎯 Rule execution results: {} rules fired", results.len());

    // We expect 2 rule fires (2 sales facts)
    if results.len() >= 2 {
        println!("✅ Basic rule functionality works!");
    } else {
        println!("❌ Basic rule functionality failed!");
    }

    let stats = engine.get_stats();
    println!("📊 Engine stats: {:?}", stats);
}