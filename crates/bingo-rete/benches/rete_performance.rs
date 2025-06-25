use bingo_core::types::{Action, ActionType, Condition, Fact, FactData, FactValue, Operator, Rule};
use bingo_rete::ReteNetwork;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::collections::HashMap;

fn create_sample_fact(id: u64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert("employee_id".to_string(), FactValue::Integer(id as i64));
    fields.insert("hours_worked".to_string(), FactValue::Float(40.0));
    fields.insert(
        "department".to_string(),
        FactValue::String("Engineering".to_string()),
    );

    Fact { id, data: FactData { fields } }
}

fn create_sample_rule(id: u64) -> Rule {
    Rule {
        id,
        name: format!("Rule {}", id),
        conditions: vec![Condition::Simple {
            field: "hours_worked".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Float(35.0),
        }],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Employee worked overtime".to_string() },
        }],
    }
}

fn bench_fact_processing(c: &mut Criterion) {
    let mut network = ReteNetwork::new().unwrap();

    // Add a sample rule
    let rule = create_sample_rule(1);
    network.add_rule(rule).unwrap();

    c.bench_function("process_single_fact", |b| {
        b.iter(|| {
            let fact = create_sample_fact(1);
            black_box(network.process_facts(vec![fact]).unwrap());
        });
    });
}

fn bench_bulk_processing(c: &mut Criterion) {
    let mut network = ReteNetwork::new().unwrap();

    // Add sample rules
    for i in 1..=10 {
        let rule = create_sample_rule(i);
        network.add_rule(rule).unwrap();
    }

    c.bench_function("process_1000_facts", |b| {
        b.iter(|| {
            let facts: Vec<Fact> = (1..=1000).map(create_sample_fact).collect();
            black_box(network.process_facts(facts).unwrap());
        });
    });
}

criterion_group!(benches, bench_fact_processing, bench_bulk_processing);
criterion_main!(benches);
