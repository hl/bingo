use bingo_core::{
    BingoEngine,
    types::{Action, ActionType, Condition, Fact, FactData, FactValue, Operator, Rule},
};
use chrono::Utc;
use std::collections::HashMap;

fn create_fact(id: u64, name: &str) -> Fact {
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), FactValue::Integer(id as i64));
    fields.insert("name".to_string(), FactValue::String(name.to_string()));
    Fact { id, external_id: None, timestamp: Utc::now(), data: FactData { fields } }
}

#[test]
fn delete_fact_rule_test() {
    let mut engine = BingoEngine::new().unwrap();

    let rule = Rule {
        id: 1,
        name: "delete bob".to_string(),
        conditions: vec![Condition::Simple {
            field: "name".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("Bob".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::DeleteFact { fact_id_field: "user_id".to_string() },
        }],
    };
    engine.add_rule(rule).unwrap();

    let facts = vec![create_fact(2, "Bob")];
    let results = engine.process_facts(facts).unwrap();
    println!("results len {}", results.len());
    assert_eq!(results.len(), 1);
}
