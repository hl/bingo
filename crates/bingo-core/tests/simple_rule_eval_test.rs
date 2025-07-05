use bingo_core::{
    Action, ActionType, BingoEngine, Condition, Fact, FactData, FactValue, Operator, Rule,
};

#[test]
fn simple_greater_than_rule_fires() {
    let mut engine = BingoEngine::new().unwrap();

    // Create rule
    let rule = Rule {
        id: 2424450237894045744,
        name: "overtime".to_string(),
        conditions: vec![Condition::Simple {
            field: "hours_worked".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(40),
        }],
        actions: vec![Action {
            action_type: ActionType::DeleteFact { fact_id_field: "hours_worked".to_string() },
        }],
    };
    engine.add_rule(rule).unwrap();

    // Create fact
    let mut fields = std::collections::HashMap::new();
    fields.insert("hours_worked".to_string(), FactValue::Float(45.0));
    let fact = Fact::new(9494918793295902219, FactData { fields });

    let results = engine.process_facts(vec![fact]).unwrap();
    assert_eq!(results.len(), 1);
}
