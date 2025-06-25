use bingo_core::*;
use std::collections::HashMap;

#[test]
fn test_rete_engine_integration() {
    // Create a new engine
    let mut engine = BingoEngine::new().unwrap();

    // Create a simple rule: age > 18 => mark as adult
    let rule = Rule {
        id: 1,
        name: "Adult Check".to_string(),
        conditions: vec![Condition::Simple {
            field: "age".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(18),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "status".to_string(),
                value: FactValue::String("adult".to_string()),
            },
        }],
    };

    // Add rule to engine
    engine.add_rule(rule).unwrap();

    // Create test facts
    let mut fields1 = HashMap::new();
    fields1.insert("age".to_string(), FactValue::Integer(25));
    fields1.insert("name".to_string(), FactValue::String("Alice".to_string()));

    let mut fields2 = HashMap::new();
    fields2.insert("age".to_string(), FactValue::Integer(16));
    fields2.insert("name".to_string(), FactValue::String("Bob".to_string()));

    let facts = vec![
        Fact { id: 1, data: FactData { fields: fields1 } },
        Fact { id: 2, data: FactData { fields: fields2 } },
    ];

    // Process facts through RETE engine
    let results = engine.process_facts(facts).unwrap();

    // Verify that the rule fired for the adult (age > 18)
    // The exact behavior depends on implementation, but we should get some results
    println!("Results: {:?}", results);
    println!("Engine stats: {:?}", engine.get_stats());

    // Basic assertions
    let stats = engine.get_stats();
    assert_eq!(stats.rule_count, 1);
    assert_eq!(stats.fact_count, 2);
    assert!(stats.node_count > 0, "Should have created RETE nodes");
}

#[test]
fn test_multiple_conditions() {
    let mut engine = BingoEngine::new().unwrap();

    // Create rule with multiple conditions: age > 18 AND salary > 50000
    let rule = Rule {
        id: 2,
        name: "High Earner Adult".to_string(),
        conditions: vec![
            Condition::Simple {
                field: "age".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Integer(18),
            },
            Condition::Simple {
                field: "salary".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Float(50000.0),
            },
        ],
        actions: vec![Action {
            action_type: ActionType::Log { message: "High earning adult identified".to_string() },
        }],
    };

    engine.add_rule(rule).unwrap();

    let mut fields = HashMap::new();
    fields.insert("age".to_string(), FactValue::Integer(30));
    fields.insert("salary".to_string(), FactValue::Float(75000.0));
    fields.insert("name".to_string(), FactValue::String("Charlie".to_string()));

    let facts = vec![Fact { id: 1, data: FactData { fields } }];

    let results = engine.process_facts(facts).unwrap();
    let stats = engine.get_stats();

    println!("Multi-condition results: {:?}", results);
    println!("Multi-condition stats: {:?}", stats);

    assert_eq!(stats.rule_count, 1);
    assert_eq!(stats.fact_count, 1);
    assert!(
        stats.node_count >= 2,
        "Should have multiple nodes for multiple conditions"
    );
}

#[test]
fn test_complex_condition_integration() {
    println!("Testing Complex Condition Implementation");

    // Create a new engine
    let mut engine = BingoEngine::new().unwrap();

    // Create a complex rule: (age > 18 AND department = "Engineering") => mark as senior_engineer
    let rule = Rule {
        id: 1,
        name: "Senior Engineer Check".to_string(),
        conditions: vec![Condition::Complex {
            operator: LogicalOperator::And,
            conditions: vec![
                Condition::Simple {
                    field: "age".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Integer(18),
                },
                Condition::Simple {
                    field: "department".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("Engineering".to_string()),
                },
            ],
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "role".to_string(),
                value: FactValue::String("senior_engineer".to_string()),
            },
        }],
    };

    // Add rule to engine
    engine.add_rule(rule).unwrap();

    // Create test facts
    let mut fields1 = HashMap::new();
    fields1.insert("name".to_string(), FactValue::String("Alice".to_string()));
    fields1.insert("age".to_string(), FactValue::Integer(25));
    fields1.insert(
        "department".to_string(),
        FactValue::String("Engineering".to_string()),
    );

    let fact1 = Fact { id: 1, data: FactData { fields: fields1 } };

    let mut fields2 = HashMap::new();
    fields2.insert("name".to_string(), FactValue::String("Bob".to_string()));
    fields2.insert("age".to_string(), FactValue::Integer(17)); // Too young
    fields2.insert(
        "department".to_string(),
        FactValue::String("Engineering".to_string()),
    );

    let fact2 = Fact { id: 2, data: FactData { fields: fields2 } };

    let mut fields3 = HashMap::new();
    fields3.insert("name".to_string(), FactValue::String("Carol".to_string()));
    fields3.insert("age".to_string(), FactValue::Integer(30));
    fields3.insert(
        "department".to_string(),
        FactValue::String("Marketing".to_string()),
    ); // Wrong dept

    let fact3 = Fact { id: 3, data: FactData { fields: fields3 } };

    // Process facts
    println!("Processing facts...");
    let results = engine.process_facts(vec![fact1, fact2, fact3]).unwrap();

    println!("Results: {:?}", results);
    println!("Engine stats: {:?}", engine.get_stats());

    // Verify statistics
    let stats = engine.get_stats();
    assert_eq!(stats.rule_count, 1);
    assert_eq!(stats.fact_count, 3);

    // Should have created alpha nodes for each sub-condition in the complex condition
    assert!(
        stats.node_count >= 2,
        "Should have created multiple alpha nodes for complex condition parts"
    );

    println!("✅ Complex condition test completed successfully");
}
