use bingo_core::*;

use bingo_core::fact_store::ArenaFactStore;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::collections::HashMap;
use std::time::Duration;

fn generate_large_fact_set(count: usize) -> Vec<Fact> {
    (0..count)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert(
                "entity_id".to_string(),
                FactValue::Integer((i % 100_000) as i64),
            );
            fields.insert(
                "category".to_string(),
                FactValue::String(format!("cat_{}", i % 1000)),
            );
            fields.insert("value".to_string(), FactValue::Float(i as f64 * 1.5));
            fields.insert(
                "status".to_string(),
                FactValue::String(
                    match i % 4 {
                        0 => "active",
                        1 => "pending",
                        2 => "inactive",
                        _ => "archived",
                    }
                    .to_string(),
                ),
            );
            fields.insert("score".to_string(), FactValue::Float((i % 1000) as f64));
            fields.insert(
                "region".to_string(),
                FactValue::String(format!("region_{}", i % 50)),
            );

            Fact { id: i as u64, data: FactData { fields } }
        })
        .collect()
}

fn bench_million_fact_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("million_fact_scaling");
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(10);

    // Test scaling from 100K to 1M facts
    for size in [100_000, 500_000, 1_000_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("process_facts_optimized", size),
            size,
            |b, &size| {
                b.iter_batched(
                    || {
                        let facts = generate_large_fact_set(size);
                        let engine = BingoEngine::with_capacity(size).unwrap();
                        (facts, engine)
                    },
                    |(facts, mut engine)| black_box(engine.process_facts(facts).unwrap()),
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_arena_vs_vec_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_vs_vec");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(5);

    let fact_count = 500_000;

    // Benchmark Vec-based fact store
    group.bench_function("vec_fact_store_500k", |b| {
        b.iter_batched(
            || {
                let facts = generate_large_fact_set(fact_count);
                let store = VecFactStore::with_capacity(fact_count);
                (facts, store)
            },
            |(facts, mut store)| {
                for fact in facts {
                    black_box(store.insert(fact));
                }
            },
            criterion::BatchSize::LargeInput,
        );
    });

    // Benchmark Arena-based fact store (if available)
    #[cfg(feature = "arena-alloc")]
    group.bench_function("arena_fact_store_500k", |b| {
        b.iter_batched(
            || {
                let facts = generate_large_fact_set(fact_count);
                let store = ArenaFactStore::with_capacity(fact_count);
                (facts, store)
            },
            |(facts, mut store)| {
                for fact in facts {
                    black_box(store.insert(fact));
                }
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

fn bench_large_fact_indexing(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_fact_indexing");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(5);

    let fact_count = 1_000_000;

    // Pre-populate store with 1M facts
    let mut store = VecFactStore::with_capacity(fact_count);
    let facts = generate_large_fact_set(fact_count);

    for fact in facts {
        store.insert(fact);
    }

    group.bench_function("indexed_lookup_1m_facts", |b| {
        b.iter(|| {
            // Perform 1000 random indexed lookups
            for i in 0..1000 {
                let category = format!("cat_{}", i % 1000);
                let results = store.find_by_field("category", &FactValue::String(category));
                black_box(results);
            }
        });
    });

    group.bench_function("multi_criteria_lookup_1m_facts", |b| {
        b.iter(|| {
            // Perform 100 multi-criteria lookups
            for i in 0..100 {
                let criteria = vec![
                    (
                        "category".to_string(),
                        FactValue::String(format!("cat_{}", i % 1000)),
                    ),
                    (
                        "status".to_string(),
                        FactValue::String("active".to_string()),
                    ),
                ];
                let results = store.find_by_criteria(&criteria);
                black_box(results);
            }
        });
    });

    group.finish();
}

fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(3);

    for size in [100_000, 500_000, 1_000_000].iter() {
        group.bench_function(BenchmarkId::new("memory_usage_rss", size), |b| {
            b.iter_custom(|iters| {
                let mut total_time = Duration::ZERO;

                for _ in 0..iters {
                    let memory_tracker = MemoryTracker::start().unwrap();

                    let start = std::time::Instant::now();

                    // Create optimized engine and process facts
                    let mut engine = BingoEngine::with_capacity(*size).unwrap();
                    let facts = generate_large_fact_set(*size);
                    let _results = black_box(engine.process_facts(facts).unwrap());

                    let elapsed = start.elapsed();

                    let (start_stats, end_stats, delta) = memory_tracker.finish().unwrap();

                    println!(
                        "Size: {}, Memory: {} -> {}, Delta: {} bytes ({:.2} MB), Time: {:?}",
                        size,
                        start_stats.format_rss(),
                        end_stats.format_rss(),
                        delta,
                        delta as f64 / (1024.0 * 1024.0),
                        elapsed
                    );

                    total_time += elapsed;
                }

                total_time
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_million_fact_scaling,
    bench_arena_vs_vec_performance,
    bench_large_fact_indexing,
    bench_memory_efficiency
);
criterion_main!(benches);
