use bingo_rete::ReteNetwork;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use serde_json::json;

fn create_sample_fact(id: u64) -> serde_json::Value {
    json!({
        "id": id,
        "data": {
            "fields": {
                "employee_id": id,
                "hours_worked": 40.0,
                "department": "Engineering"
            }
        }
    })
}

fn create_sample_rule(id: u64) -> serde_json::Value {
    json!({
        "id": id,
        "name": format!("Rule {}", id),
        "conditions": [{
            "field": "hours_worked",
            "operator": "GreaterThan",
            "value": 35.0
        }],
        "actions": [{
            "action_type": {
                "Log": {
                    "message": "Employee worked overtime"
                }
            }
        }]
    })
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
            let facts: Vec<serde_json::Value> = (1..=1000).map(create_sample_fact).collect();
            black_box(network.process_facts(facts).unwrap());
        });
    });
}

criterion_group!(benches, bench_fact_processing, bench_bulk_processing);
criterion_main!(benches);
