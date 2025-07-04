use bingo_core::*;
use std::collections::HashMap;

#[test]
fn test_simplified_api_workflow() {
    // ✅ YOUR API: Create engine
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // ✅ YOUR API: Define rules with predefined calculators
    let mut rule_fields = HashMap::new();
    rule_fields.insert("hours_worked".to_string(), FactValue::Integer(45));

    let rule = Rule {
        id: 1,
        name: "Test Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "hours_worked".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(40),
        }],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Hours exceeded!".to_string() },
        }],
    };

    // ✅ YOUR API: Define facts
    let mut fact_fields = HashMap::new();
    fact_fields.insert("hours_worked".to_string(), FactValue::Integer(45));
    fact_fields.insert("employee_id".to_string(), FactValue::Integer(12345));

    let fact = Fact {
        id: 1,
        external_id: None,
        timestamp: chrono::Utc::now(),
        data: FactData { fields: fact_fields },
    };

    // ✅ YOUR API: Process rules + facts → get response
    let results = engine.evaluate(vec![rule], vec![fact]).expect("Failed to evaluate rules");

    // ✅ YOUR API: Check response
    assert!(!results.is_empty(), "Should have executed rules");
    assert_eq!(results[0].rule_id, 1);
    assert_eq!(results[0].fact_id, 1);

    println!("✅ Simplified API Test Passed!");
    println!("   Rules processed: 1");
    println!("   Facts processed: 1");
    println!("   Results: {}", results.len());
}

#[test]
fn test_engine_stats() {
    let engine = BingoEngine::new().expect("Failed to create engine");
    let stats = engine.get_stats();

    assert_eq!(stats.rule_count, 0);
    assert_eq!(stats.fact_count, 0);
    // Node count should be valid (non-negative by type constraint)
    assert!(stats.node_count == 0); // Empty engine has no nodes

    println!("✅ Engine Stats Test Passed!");
    println!("   Initial rules: {}", stats.rule_count);
    println!("   Initial facts: {}", stats.fact_count);
    println!("   Network nodes: {}", stats.node_count);
}
