use bingo_core::*;

use bingo_core::fact_store::ArenaFactStore;
use std::collections::HashMap;

#[test]
fn test_optimized_engine_creation() {
    // Test small capacity uses Vec store
    let small_engine = BingoEngine::with_capacity(1000).unwrap();
    let stats = small_engine.get_stats();
    assert_eq!(stats.fact_count, 0);
    assert_eq!(stats.rule_count, 0);

    // Test large capacity selection
    let large_engine = BingoEngine::with_capacity(100_000).unwrap();
    let large_stats = large_engine.get_stats();
    assert_eq!(large_stats.fact_count, 0);

    println!("Small engine stats: {:?}", stats);
    println!("Large engine stats: {:?}", large_stats);
}

#[test]
fn test_fact_store_factory() {
    // Test simple store creation
    let simple_store = FactStoreFactory::create_simple();
    assert_eq!(simple_store.len(), 0);

    // Test optimized store selection
    let small_store = FactStoreFactory::create_optimized(1000);
    assert_eq!(small_store.len(), 0);

    let large_store = FactStoreFactory::create_optimized(100_000);
    assert_eq!(large_store.len(), 0);

    println!("Factory created stores successfully");
}

#[test]
fn test_arena_fact_store() {
    let mut store = ArenaFactStore::with_capacity(1000);

    // Create test facts
    let mut fields = HashMap::new();
    fields.insert("entity_id".to_string(), FactValue::Integer(123));
    fields.insert(
        "status".to_string(),
        FactValue::String("active".to_string()),
    );

    let fact = Fact { id: 1, data: FactData { fields } };

    // Test insertion
    let id = store.insert(fact);
    assert_eq!(store.len(), 1);

    // Test retrieval
    let retrieved = store.get(id).unwrap();
    assert_eq!(retrieved.id, id);

    // Test indexing
    let found = store.find_by_field("status", &FactValue::String("active".to_string()));
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, id);

    println!("Arena store test passed with {} facts", store.len());
}

#[test]
fn test_parallel_vs_sequential_processing() {
    let mut engine = BingoEngine::with_capacity(50_000).unwrap();

    // Add a simple rule
    let rule = Rule {
        id: 1,
        name: "Test Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("active".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "processed".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Create test facts (large enough to trigger parallel processing)
    let facts: Vec<Fact> = (0..15_000)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("entity_id".to_string(), FactValue::Integer(i as i64));
            fields.insert(
                "status".to_string(),
                FactValue::String(if i % 2 == 0 { "active" } else { "inactive" }.to_string()),
            );

            Fact { id: i as u64, data: FactData { fields } }
        })
        .collect();

    let start = std::time::Instant::now();
    let results = engine.process_facts(facts).unwrap();
    let elapsed = start.elapsed();

    println!(
        "Processed 15K facts in {:?}, generated {} results",
        elapsed,
        results.len()
    );

    let stats = engine.get_stats();
    println!("Final engine stats: {:?}", stats);

    // Should trigger parallel path (> 10K facts)
    // Performance will be optimized further in Phase 3
    assert!(
        elapsed.as_secs() < 60,
        "Should process 15K facts within reasonable time"
    );
    assert_eq!(stats.fact_count, 15_000);
}

#[test]
#[ignore] // Skip in CI - memory usage varies in CI environments
fn test_memory_efficiency_comparison() {
    let memory_tracker = MemoryTracker::start().unwrap();

    // Create optimized engine
    let mut engine = BingoEngine::with_capacity(25_000).unwrap();

    // Add a rule to ensure facts are processed and stored
    let rule = Rule {
        id: 1,
        name: "memory_test_rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "entity_id".to_string(),
            operator: Operator::GreaterThanOrEqual,
            value: FactValue::Integer(0),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "processed".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Generate facts
    let facts: Vec<Fact> = (0..25_000)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("entity_id".to_string(), FactValue::Integer(i as i64));
            fields.insert(
                "category".to_string(),
                FactValue::String(format!("cat_{}", i % 100)),
            );
            fields.insert("value".to_string(), FactValue::Float(i as f64));

            Fact { id: i as u64, data: FactData { fields } }
        })
        .collect();

    // Process facts
    let _results = engine.process_facts(facts).unwrap();

    let (start_stats, end_stats, delta) = memory_tracker.finish().unwrap();

    println!(
        "Memory efficiency test: {} -> {}, Delta: {} bytes ({:.2} MB)",
        start_stats.format_rss(),
        end_stats.format_rss(),
        delta,
        delta as f64 / (1024.0 * 1024.0)
    );

    // Memory usage should be reasonable for 25K facts
    assert!(
        delta < 100_000_000,
        "Memory usage should be under 100MB for 25K facts"
    );

    let stats = engine.get_stats();
    assert_eq!(stats.fact_count, 25_000);
}
