use bingo_core::*;
use std::collections::HashMap;
fn main(){
    let rule = types::Rule {
        id:1,
        name:"Test".into(),
        conditions: vec![types::Condition::Simple{ field:"entity_type".into(), operator: types::Operator::Equal, value: types::FactValue::String("data_source".into())}],
        actions: vec![ types::Action { action_type: types::ActionType::Log{ message:"ok".into()} } ],
    };
    let fact = types::Fact { id:3, external_id:None, timestamp: chrono::Utc::now(), data: types::FactData{fields: HashMap::from([("entity_type".to_string(), types::FactValue::String("data_source".into()))])}};
    let mut engine = engine::BingoEngine::new().unwrap();
    engine.add_rule(rule).unwrap();
    let results = engine.process_facts(vec![fact]).unwrap();
    println!("Results {}", results.len());
}
