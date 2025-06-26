use bingo_core::{BingoEngine, Fact, FactData, FactStore, FactValue, MemoryTracker, VecFactStore};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::collections::HashMap;
use std::time::Duration;

fn generate_test_facts(count: usize) -> Vec<Fact> {
    (0..count)
        .map(|i| Fact {
            id: i as u64,
            data: FactData {
                fields: {
                    let mut map = HashMap::new();
                    map.insert("entity_id".to_string(), FactValue::Integer(i as i64));
                    map.insert("value".to_string(), FactValue::Float(i as f64 * 1.5));
                    map.insert(
                        "status".to_string(),
                        FactValue::String("active".to_string()),
                    );
                    map.insert(
                        "category".to_string(),
                        FactValue::String(format!("cat_{}", i % 10)),
                    );
                    map.insert("score".to_string(), FactValue::Float((i % 100) as f64));
                    map
                },
            },
        })
        .collect()
}

fn bench_fact_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("fact_processing");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    for size in [1_000, 10_000, 100_000].iter() {
        group.bench_with_input(BenchmarkId::new("process_facts", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let facts = generate_test_facts(size);
                    let engine = BingoEngine::new().unwrap();
                    (facts, engine)
                },
                |(facts, engine)| black_box(engine.process_facts(facts).unwrap()),
                criterion::BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    for size in [10_000, 50_000, 100_000].iter() {
        group.bench_function(BenchmarkId::new("memory_rss", size), |b| {
            b.iter_custom(|iters| {
                let mut total_time = Duration::ZERO;

                for _ in 0..iters {
                    let memory_tracker = MemoryTracker::start().unwrap();
                    let facts = generate_test_facts(*size);
                    let engine = BingoEngine::new().unwrap();

                    let start = std::time::Instant::now();
                    let _result = black_box(engine.process_facts(facts).unwrap());
                    let elapsed = start.elapsed();

                    let (start_stats, end_stats, delta) = memory_tracker.finish().unwrap();

                    // Print memory usage for manual verification
                    println!(
                        "Size: {}, Memory: {} -> {}, Delta: {} bytes ({:.2} MB)",
                        *size,
                        start_stats.format_rss(),
                        end_stats.format_rss(),
                        delta,
                        delta as f64 / (1024.0 * 1024.0)
                    );

                    total_time += elapsed;
                }

                total_time
            });
        });
    }
    group.finish();
}

fn bench_fact_store_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("fact_store");

    for size in [1_000, 10_000, 100_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("vec_fact_store_insert", size),
            size,
            |b, &size| {
                b.iter_batched(
                    || {
                        let facts = generate_test_facts(size);
                        let store = VecFactStore::new();
                        (facts, store)
                    },
                    |(facts, mut store)| {
                        for fact in facts {
                            black_box(store.insert(fact));
                        }
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_engine_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_stats");

    // Pre-populate engine with facts
    let engine = BingoEngine::new().unwrap();
    let facts = generate_test_facts(100_000);
    engine.process_facts(facts).unwrap();

    group.bench_function("get_stats_100k", |b| {
        b.iter(|| black_box(engine.get_stats()))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_fact_processing,
    bench_memory_usage,
    bench_fact_store_operations,
    bench_engine_stats
);
criterion_main!(benches);
