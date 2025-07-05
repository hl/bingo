use bingo_core::{
    Action, ActionType, BingoEngine, Condition, Fact, FactData, FactValue, Operator, Rule,
};

#[test]
fn test_rule() {
    let mut engine = BingoEngine::new().unwrap();
    let rule = Rule {
        id: 1,
        name: "test".to_string(),
        conditions: vec![Condition::Simple {
            field: "hours_worked".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Float(40.0),
        }],
        actions: vec![Action {
            action_type: ActionType::DeleteFact { fact_id_field: "hours_worked".to_string() },
        }],
    };
    engine.add_rule(rule).unwrap();
    let fact = Fact::new(
        1,
        FactData {
            fields: vec![("hours_worked".to_string(), FactValue::Float(45.0))]
                .into_iter()
                .collect(),
        },
    );
    let results = engine.process_facts(vec![fact]).unwrap();
    assert_eq!(results.len(), 1);
}
