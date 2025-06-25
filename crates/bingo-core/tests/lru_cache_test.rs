//! Integration tests for LRU cache functionality
//!
//! This module tests the LRU cache implementation and its integration
//! with the fact store for improved performance on repeated access patterns.

use bingo_core::*;
use std::collections::HashMap;

fn create_test_fact(id: u64, name: &str, value: i64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert("status".to_string(), FactValue::String(name.to_string()));
    fields.insert("entity_id".to_string(), FactValue::Integer(value));

    Fact { id, data: FactData { fields } }
}

#[test]
fn test_lru_cache_basic_functionality() {
    let mut cache = LruCache::new(3);

    // Test basic operations
    cache.put("key1", "value1");
    cache.put("key2", "value2");
    cache.put("key3", "value3");

    assert_eq!(cache.len(), 3);
    assert_eq!(cache.get(&"key1"), Some(&"value1"));
    assert_eq!(cache.get(&"key2"), Some(&"value2"));
    assert_eq!(cache.get(&"key3"), Some(&"value3"));

    // Test eviction
    cache.put("key4", "value4"); // Should evict key1 (least recently used)

    assert_eq!(cache.len(), 3);
    assert_eq!(cache.get(&"key1"), None); // Evicted
    assert_eq!(cache.get(&"key2"), Some(&"value2"));
    assert_eq!(cache.get(&"key3"), Some(&"value3"));
    assert_eq!(cache.get(&"key4"), Some(&"value4"));
}

#[test]
fn test_lru_cache_access_order() {
    let mut cache = LruCache::new(3);

    cache.put(1, "one");
    cache.put(2, "two");
    cache.put(3, "three");

    // Access item 1 to make it more recent
    cache.get(&1);

    // Add new item - should evict item 2 (oldest unaccessed)
    cache.put(4, "four");

    assert_eq!(cache.get(&1), Some(&"one")); // Still present (recently accessed)
    assert_eq!(cache.get(&2), None); // Evicted
    assert_eq!(cache.get(&3), Some(&"three")); // Still present
    assert_eq!(cache.get(&4), Some(&"four")); // Newly added
}

#[test]
fn test_cached_fact_store_basic_operations() {
    let mut store = CachedFactStore::new(100); // Cache capacity of 100

    let fact1 = create_test_fact(1, "Alice", 100);
    let fact2 = create_test_fact(2, "Bob", 200);
    let fact3 = create_test_fact(3, "Carol", 300);

    let id1 = store.insert(fact1.clone());
    let id2 = store.insert(fact2.clone());
    let id3 = store.insert(fact3.clone());

    assert_eq!(store.len(), 3);

    // Test retrieval
    assert_eq!(store.get(id1).unwrap().id, id1);
    assert_eq!(store.get(id2).unwrap().id, id2);
    assert_eq!(store.get(id3).unwrap().id, id3);

    // Test cache statistics
    let cache_stats = store.cache_stats().unwrap();
    assert_eq!(cache_stats.capacity, 100);
    assert_eq!(cache_stats.size, 3); // All facts should be cached
}

#[test]
fn test_cached_fact_store_with_capacity() {
    let mut store = CachedFactStore::with_capacity(1000, 50); // 1000 facts, 50 cache

    // Add facts beyond cache capacity
    for i in 0..100 {
        let fact = create_test_fact(i, &format!("User{}", i), i as i64 * 10);
        store.insert(fact);
    }

    assert_eq!(store.len(), 100);

    let cache_stats = store.cache_stats().unwrap();
    assert_eq!(cache_stats.capacity, 50);
    assert!(cache_stats.size <= 50); // Cache should not exceed capacity
}

#[test]
fn test_cached_fact_store_field_search() {
    let mut store = CachedFactStore::new(100);

    // Add test facts
    let facts = vec![
        create_test_fact(1, "Alice", 100),
        create_test_fact(2, "Bob", 200),
        create_test_fact(3, "Alice", 150), // Same name, different value
        create_test_fact(4, "Carol", 100), // Same value, different name
    ];

    for fact in facts {
        store.insert(fact);
    }

    // Test finding by status in cached store
    let alice_facts = store.find_by_field("status", &FactValue::String("Alice".to_string()));
    assert_eq!(alice_facts.len(), 2);

    // Test finding by entity_id
    let value_100_facts = store.find_by_field("entity_id", &FactValue::Integer(100));
    assert_eq!(value_100_facts.len(), 2);

    // Test finding by multiple criteria
    let criteria = vec![
        ("status".to_string(), FactValue::String("Alice".to_string())),
        ("entity_id".to_string(), FactValue::Integer(100)),
    ];
    let matching_facts = store.find_by_criteria(&criteria);
    assert_eq!(matching_facts.len(), 1);
}

#[test]
fn test_cached_fact_store_clear_operations() {
    let mut store = CachedFactStore::new(50);

    // Add some facts
    for i in 0..10 {
        let fact = create_test_fact(i, &format!("User{}", i), i as i64);
        store.insert(fact);
    }

    assert_eq!(store.len(), 10);
    let cache_stats = store.cache_stats().unwrap();
    assert_eq!(cache_stats.size, 10);

    // Test cache clear
    store.clear_cache();
    let cache_stats = store.cache_stats().unwrap();
    assert_eq!(cache_stats.size, 0);
    assert_eq!(store.len(), 10); // Facts should still be there

    // Test full clear
    store.clear();
    assert_eq!(store.len(), 0);
    let cache_stats = store.cache_stats().unwrap();
    assert_eq!(cache_stats.size, 0);
}

#[test]
fn test_cached_fact_store_bulk_operations() {
    let mut store = CachedFactStore::new(100);

    // Add initial facts
    for i in 0..5 {
        let fact = create_test_fact(i, &format!("Initial{}", i), i as i64);
        store.insert(fact);
    }

    let initial_cache_stats = store.cache_stats().unwrap();
    assert_eq!(initial_cache_stats.size, 5);

    // Test bulk extend - should clear cache
    let bulk_facts = vec![
        create_test_fact(10, "Bulk1", 100),
        create_test_fact(11, "Bulk2", 200),
        create_test_fact(12, "Bulk3", 300),
    ];

    store.extend_from_vec(bulk_facts);

    assert_eq!(store.len(), 8); // 5 initial + 3 bulk
    let cache_stats = store.cache_stats().unwrap();
    assert_eq!(cache_stats.size, 0); // Cache should be cleared after bulk operations
}

#[test]
fn test_fact_store_factory_cached_creation() {
    let store = FactStoreFactory::create_cached(1000, 100);

    // Verify it supports cache operations
    assert!(store.cache_stats().is_some());
}

#[test]
fn test_cache_statistics() {
    let mut cache = LruCache::new(5);

    let initial_stats = cache.stats();
    assert_eq!(initial_stats.capacity, 5);
    assert_eq!(initial_stats.size, 0);
    assert_eq!(initial_stats.utilization(), 0.0);

    // Add some items
    cache.put("a", 1);
    cache.put("b", 2);
    cache.put("c", 3);

    let stats = cache.stats();
    assert_eq!(stats.capacity, 5);
    assert_eq!(stats.size, 3);
    assert_eq!(stats.utilization(), 60.0);

    // Fill to capacity
    cache.put("d", 4);
    cache.put("e", 5);

    let stats = cache.stats();
    assert_eq!(stats.capacity, 5);
    assert_eq!(stats.size, 5);
    assert_eq!(stats.utilization(), 100.0);

    // Add one more to trigger eviction
    cache.put("f", 6);

    let stats = cache.stats();
    assert_eq!(stats.capacity, 5);
    assert_eq!(stats.size, 5);
    assert_eq!(stats.utilization(), 100.0);
}

#[test]
fn test_performance_comparison_pattern() {
    // This test demonstrates how caching could improve performance
    // in scenarios with repeated fact access patterns

    let mut uncached_store = VecFactStore::new();
    let mut cached_store = CachedFactStore::new(100);

    // Add the same facts to both stores
    let facts: Vec<_> = (0..1000)
        .map(|i| create_test_fact(i, &format!("User{}", i), i as i64))
        .collect();

    for fact in &facts {
        uncached_store.insert(fact.clone());
        cached_store.insert(fact.clone());
    }

    // Both should have the same number of facts
    assert_eq!(uncached_store.len(), cached_store.len());

    // Access pattern: repeatedly access the first 50 facts
    // In a real scenario, the cached store would show performance benefits
    for _ in 0..10 {
        for i in 0..50 {
            let uncached_fact = uncached_store.get(i);
            let cached_fact = cached_store.get(i);

            assert_eq!(uncached_fact.is_some(), cached_fact.is_some());
            if let (Some(u), Some(c)) = (uncached_fact, cached_fact) {
                assert_eq!(u.id, c.id);
            }
        }
    }

    // Verify cache utilization
    let cache_stats = cached_store.cache_stats().unwrap();
    println!("Cache utilization: {:.1}%", cache_stats.utilization());
    assert!(cache_stats.size > 0);
}
