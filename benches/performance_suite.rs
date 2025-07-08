use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bingo_core::*;
use std::collections::HashMap;

fn multi_condition_performance_benchmark(c: &mut Criterion) {
    c.bench_function("multi_condition_performance", |b| {
        let mut engine = BingoEngine::with_capacity(10_000).unwrap();

        // Add multiple multi-condition rules
        for i in 0..20 {
            let rule = Rule {
                id: 6000 + i,
                name: format!("Multi-Condition Rule {i}"),
                conditions: vec![
                    Condition::Simple {
                        field: "department".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::String("engineering".to_string()),
                    },
                    Condition::Simple {
                        field: "level".to_string(),
                        operator: Operator::GreaterThan,
                        value: FactValue::Integer(2),
                    },
                    Condition::Simple {
                        field: "active".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::Boolean(true),
                    },
                ],
                actions: vec![Action {
                    action_type: ActionType::Log {
                        message: format!("Multi-condition rule {i} triggered"),
                    },
                }],
            };
            engine.add_rule(rule).unwrap();
        }

        // Generate 10K facts with varying patterns
        let facts: Vec<Fact> = (0..10_000)
            .map(|i| {
                let mut fields = HashMap::new();
                fields.insert("employee_id".to_string(), FactValue::Integer(i));
                fields.insert(
                    "department".to_string(),
                    FactValue::String(if i % 3 == 0 { "engineering" } else { "sales" }.to_string()),
                );
                fields.insert("level".to_string(), FactValue::Integer(i % 6));
                fields.insert("active".to_string(), FactValue::Boolean(i % 4 != 0));

                Fact {
                    id: i as u64,
                    external_id: None,
                    timestamp: chrono::Utc::now(),
                    data: FactData { fields },
                }
            })
            .collect();

        b.iter(|| {
            engine.process_facts(black_box(facts.clone())).unwrap();
        });
    });
}

criterion_group!(benches, multi_condition_performance_benchmark);
criterion_main!(benches);
