use bingo_core::*;
use std::collections::HashMap;

#[test]
fn test_calculator_formula_action_integration() {
    let mut engine = BingoEngine::new().unwrap();

    // Create a rule with a Formula action using calculator DSL
    let rule = Rule {
        id: 1,
        name: "Tax Calculation Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "amount".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Float(0.0),
        }],
        actions: vec![Action {
            action_type: ActionType::Formula {
                target_field: "tax".to_string(),
                expression: "amount * 0.15".to_string(),
                source_calculator: None,
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Create test facts
    let facts = vec![
        Fact {
            id: 1,
            data: FactData {
                fields: {
                    let mut fields = HashMap::new();
                    fields.insert("amount".to_string(), FactValue::Float(100.0));
                    fields.insert("customer_id".to_string(), FactValue::Integer(12345));
                    fields
                },
            },
        },
        Fact {
            id: 2,
            data: FactData {
                fields: {
                    let mut fields = HashMap::new();
                    fields.insert("amount".to_string(), FactValue::Float(250.0));
                    fields.insert("customer_id".to_string(), FactValue::Integer(67890));
                    fields
                },
            },
        },
    ];

    // Process facts through the engine
    let results = engine.process_facts(facts).unwrap();

    println!("Results: {:#?}", results);

    // Validate results
    assert_eq!(results.len(), 2, "Should generate 2 results");

    // Check first result
    let first_result = &results[0];
    assert_eq!(
        first_result.data.fields.get("amount"),
        Some(&FactValue::Float(100.0))
    );
    assert_eq!(
        first_result.data.fields.get("tax"),
        Some(&FactValue::Float(15.0))
    );

    // Check second result
    let second_result = &results[1];
    assert_eq!(
        second_result.data.fields.get("amount"),
        Some(&FactValue::Float(250.0))
    );
    assert_eq!(
        second_result.data.fields.get("tax"),
        Some(&FactValue::Float(37.5))
    );
}

#[test]
fn test_calculator_complex_formula_with_functions() {
    let mut engine = BingoEngine::new().unwrap();

    // Create a rule with complex formula using built-in functions
    let rule = Rule {
        id: 2,
        name: "Discount Calculation Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("premium".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::Formula {
                target_field: "discount".to_string(),
                expression: "max(amount * 0.1, 10)".to_string(),
                source_calculator: None,
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Create test facts with different amounts
    let facts = vec![
        // Small amount - should get minimum discount of 10
        Fact {
            id: 1,
            data: FactData {
                fields: {
                    let mut fields = HashMap::new();
                    fields.insert("amount".to_string(), FactValue::Float(50.0));
                    fields.insert(
                        "status".to_string(),
                        FactValue::String("premium".to_string()),
                    );
                    fields
                },
            },
        },
        // Large amount - should get 10% discount
        Fact {
            id: 2,
            data: FactData {
                fields: {
                    let mut fields = HashMap::new();
                    fields.insert("amount".to_string(), FactValue::Float(500.0));
                    fields.insert(
                        "status".to_string(),
                        FactValue::String("premium".to_string()),
                    );
                    fields
                },
            },
        },
    ];

    let results = engine.process_facts(facts).unwrap();

    println!("Complex formula results: {:#?}", results);

    assert_eq!(results.len(), 2);

    // Small amount should get minimum discount
    let first_result = &results[0];
    assert_eq!(
        first_result.data.fields.get("discount"),
        Some(&FactValue::Integer(10))
    );

    // Large amount should get percentage discount
    let second_result = &results[1];
    assert_eq!(
        second_result.data.fields.get("discount"),
        Some(&FactValue::Float(50.0))
    );
}

#[test]
fn test_calculator_conditional_formula() {
    let mut engine = BingoEngine::new().unwrap();

    // Create a rule with conditional expression
    let rule = Rule {
        id: 3,
        name: "Risk Assessment Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "score".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(0),
        }],
        actions: vec![Action {
            action_type: ActionType::Formula {
                target_field: "risk_level".to_string(),
                expression:
                    "if score > 80 then \"low\" else if score > 50 then \"medium\" else \"high\""
                        .to_string(),
                source_calculator: None,
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Create test facts with different scores
    let facts = vec![
        Fact {
            id: 1,
            data: FactData {
                fields: {
                    let mut fields = HashMap::new();
                    fields.insert("score".to_string(), FactValue::Integer(95));
                    fields.insert("customer_id".to_string(), FactValue::Integer(1));
                    fields
                },
            },
        },
        Fact {
            id: 2,
            data: FactData {
                fields: {
                    let mut fields = HashMap::new();
                    fields.insert("score".to_string(), FactValue::Integer(65));
                    fields.insert("customer_id".to_string(), FactValue::Integer(2));
                    fields
                },
            },
        },
        Fact {
            id: 3,
            data: FactData {
                fields: {
                    let mut fields = HashMap::new();
                    fields.insert("score".to_string(), FactValue::Integer(30));
                    fields.insert("customer_id".to_string(), FactValue::Integer(3));
                    fields
                },
            },
        },
    ];

    let results = engine.process_facts(facts).unwrap();

    println!("Conditional formula results: {:#?}", results);

    assert_eq!(results.len(), 3);

    // High score should be low risk
    assert_eq!(
        results[0].data.fields.get("risk_level"),
        Some(&FactValue::String("low".to_string()))
    );

    // Medium score should be medium risk
    assert_eq!(
        results[1].data.fields.get("risk_level"),
        Some(&FactValue::String("medium".to_string()))
    );

    // Low score should be high risk
    assert_eq!(
        results[2].data.fields.get("risk_level"),
        Some(&FactValue::String("high".to_string()))
    );
}

#[test]
fn test_calculator_error_handling() {
    let mut engine = BingoEngine::new().unwrap();

    // Create a rule with an invalid formula
    let rule = Rule {
        id: 4,
        name: "Error Test Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "test".to_string(),
            operator: Operator::Equal,
            value: FactValue::Boolean(true),
        }],
        actions: vec![Action {
            action_type: ActionType::Formula {
                target_field: "result".to_string(),
                expression: "nonexistent_field * 2".to_string(), // This will fail
                source_calculator: None,
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    let facts = vec![Fact {
        id: 1,
        data: FactData {
            fields: {
                let mut fields = HashMap::new();
                fields.insert("test".to_string(), FactValue::Boolean(true));
                fields
            },
        },
    }];

    // Processing should succeed but not generate results due to formula error
    let results = engine.process_facts(facts).unwrap();

    // Should not crash, but no results should be generated due to the error
    assert_eq!(
        results.len(),
        0,
        "Should not generate results for failed formula"
    );
}
