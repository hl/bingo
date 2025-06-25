//! Test to demonstrate optimization layer consolidation benefits
//!
//! This test validates that the OptimizedFactStore successfully replaces
//! both FastFactLookup and CachedFactStore with equivalent functionality.

use bingo_core::*;
use std::collections::HashMap;

#[test]
fn test_consolidation_replaces_fast_fact_lookup() {
    // Test that OptimizedFactStore provides all FastFactLookup functionality
    let mut optimized_store = OptimizedFactStore::new_fast(100);

    // Create test facts
    let mut fields1 = HashMap::new();
    fields1.insert("name".to_string(), FactValue::String("Alice".to_string()));
    fields1.insert("age".to_string(), FactValue::Integer(30));

    let mut fields2 = HashMap::new();
    fields2.insert("name".to_string(), FactValue::String("Bob".to_string()));
    fields2.insert("age".to_string(), FactValue::Integer(25));

    let fact1 = Fact { id: 1, data: FactData { fields: fields1 } };
    let fact2 = Fact { id: 2, data: FactData { fields: fields2 } };

    // Test insertion and retrieval (equivalent to FastFactLookup)
    optimized_store.insert(fact1.clone());
    optimized_store.insert(fact2.clone());

    assert_eq!(optimized_store.len(), 2);
    assert!(!optimized_store.is_empty());

    // Test mutable access with caching
    let retrieved = optimized_store.get_mut(1).unwrap();
    assert_eq!(retrieved.id, 1);
    assert_eq!(
        retrieved.data.fields.get("name").unwrap(),
        &FactValue::String("Alice".to_string())
    );

    // Test batch operations
    let fact_ids = vec![1, 2];
    let batch_results = optimized_store.get_many(&fact_ids);
    assert_eq!(batch_results.len(), 2);

    // Test cache performance tracking
    let stats = optimized_store.stats();
    assert!(stats.total_lookups > 0);
    assert_eq!(stats.backend_type, "HashMap");

    println!("✅ OptimizedFactStore successfully replaces FastFactLookup functionality");
}

#[test]
fn test_consolidation_supports_fact_store_trait() {
    // Test that OptimizedFactStore implements FactStore trait (CachedFactStore replacement)
    let mut optimized_store = OptimizedFactStore::new_memory_efficient(50);

    // Test FactStore trait methods
    let mut fields = HashMap::new();
    fields.insert(
        "category".to_string(),
        FactValue::String("test".to_string()),
    );
    fields.insert("value".to_string(), FactValue::Integer(42));

    let fact = Fact { id: 10, data: FactData { fields } };

    // Use trait methods (like CachedFactStore)
    let stored_id = FactStore::insert(&mut optimized_store, fact.clone());
    let retrieved = FactStore::get(&optimized_store, stored_id);

    assert!(retrieved.is_some());
    assert_eq!(
        retrieved.unwrap().data.fields.get("value").unwrap(),
        &FactValue::Integer(42)
    );

    // Test cache statistics through trait
    let cache_stats = optimized_store.cache_stats();
    assert!(cache_stats.is_some());

    // Test backend switching
    let stats = optimized_store.stats();
    assert_eq!(stats.backend_type, "Vector");

    println!("✅ OptimizedFactStore successfully implements FactStore trait");
}

#[test]
fn test_consolidation_memory_efficiency_modes() {
    // Test different memory efficiency configurations

    // Fast mode (HashMap backend) - equivalent to FastFactLookup
    let mut fast_store = OptimizedFactStore::new_fast(10);

    // Memory efficient mode (Vector backend) - equivalent to VecFactStore with cache
    let mut memory_store = OptimizedFactStore::new_memory_efficient(10);

    // No cache mode for minimal memory usage
    let mut minimal_store = OptimizedFactStore::without_cache(true);

    // Insert same data in all stores
    let fact = Fact {
        id: 1,
        data: FactData {
            fields: {
                let mut fields = HashMap::new();
                fields.insert("test".to_string(), FactValue::Boolean(true));
                fields
            },
        },
    };

    let fast_id = fast_store.insert(fact.clone());
    let memory_id = memory_store.insert(fact.clone());
    let minimal_id = minimal_store.insert(fact.clone());

    // All should store the fact successfully (may have different IDs)
    assert!(fast_store.contains(fast_id));
    assert!(memory_store.contains(memory_id));
    assert!(minimal_store.contains(minimal_id));

    // Verify different backend types
    assert_eq!(fast_store.stats().backend_type, "HashMap");
    assert_eq!(memory_store.stats().backend_type, "Vector");
    assert_eq!(minimal_store.stats().backend_type, "HashMap");

    // Verify cache behavior
    assert!(fast_store.stats().cache_stats.is_some());
    assert!(memory_store.stats().cache_stats.is_some());
    assert!(minimal_store.stats().cache_stats.is_none());

    println!("✅ OptimizedFactStore provides configurable memory efficiency modes");
}

#[test]
fn test_consolidation_reduces_complexity() {
    // Demonstrate that one OptimizedFactStore replaces multiple implementations

    // Before: Would need FastFactLookup for RETE + CachedFactStore for general use
    // After: One OptimizedFactStore for both use cases

    let mut unified_store = OptimizedFactStore::with_capacity(1000, 100, true);

    // Test RETE-style usage (high-frequency lookups)
    for i in 1..=50 {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FactValue::Integer(i));
        let fact = Fact { id: i as u64, data: FactData { fields } };
        unified_store.insert(fact);
    }

    // Simulate RETE network access patterns
    for _ in 0..5 {
        for id in 1..=10 {
            unified_store.get_mut(id); // High-frequency access
        }
    }

    // Test general fact store usage (indexed queries)
    let search_value = FactValue::Integer(5);
    let matching_ids = unified_store.find_by_field("id", &search_value);
    assert_eq!(matching_ids.len(), 1);
    assert_eq!(matching_ids[0], 5);

    let stats = unified_store.stats();

    // Should show good cache performance from repeated access
    assert!(stats.hit_rate > 0.0);
    assert_eq!(stats.facts_stored, 50);
    assert!(stats.total_lookups >= 50); // From repeated access pattern (at least 50 lookups)

    println!("✅ Single OptimizedFactStore handles both RETE and general use cases");
    println!("   Cache hit rate: {:.1}%", stats.hit_rate);
    println!("   Total lookups: {}", stats.total_lookups);
    println!("   Facts stored: {}", stats.facts_stored);
}

#[test]
fn test_integration_with_rete_network() {
    // Test that RETE network works correctly with OptimizedFactStore
    let mut engine = BingoEngine::new().unwrap();

    // Create a simple rule
    let rule = Rule {
        id: 1,
        name: "Integration Test".to_string(),
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

    // Create test facts
    let mut fields = HashMap::new();
    fields.insert(
        "status".to_string(),
        FactValue::String("active".to_string()),
    );
    fields.insert(
        "name".to_string(),
        FactValue::String("test_fact".to_string()),
    );

    let facts = vec![Fact { id: 1, data: FactData { fields } }];

    // Process through RETE network (now using OptimizedFactStore internally)
    let _results = engine.process_facts(facts).unwrap();

    // Verify the engine works with the unified fact store
    let stats = engine.get_stats();
    assert_eq!(stats.rule_count, 1);
    assert_eq!(stats.fact_count, 1);

    println!("✅ RETE network integration successful with OptimizedFactStore");
    println!(
        "   Rules: {}, Facts: {}, Nodes: {}",
        stats.rule_count, stats.fact_count, stats.node_count
    );
}
