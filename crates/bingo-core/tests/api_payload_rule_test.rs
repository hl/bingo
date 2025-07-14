use bingo_core::{
    Action, ActionType, BingoEngine, Condition, Fact, FactData, FactValue, Operator, Rule,
};

#[test]
fn api_style_rule_fires() {
    let engine = BingoEngine::new().unwrap();

    // Simulate the API-sent rule (id as string hashed later)
    let rule = Rule {
        id: 123, // We'll just pick a number directly
        name: "Overtime Detection".to_string(),
        conditions: vec![Condition::Simple {
            field: "hours_worked".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Float(40.0),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "overtime".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };
    engine.add_rule(rule).unwrap();

    let mut fields = std::collections::HashMap::new();
    fields.insert("hours_worked".to_string(), FactValue::Float(45.0));
    let fact = Fact::new(1, FactData { fields });

    let results = engine.process_facts(vec![fact]).unwrap();
    assert_eq!(results.len(), 1);
}
