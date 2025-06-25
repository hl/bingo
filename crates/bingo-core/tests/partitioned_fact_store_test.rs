//! Tests for partitioned fact store functionality
//!
//! This module tests the partitioned fact store implementation for handling
//! large datasets efficiently across multiple memory partitions.

use bingo_core::{
    Fact, FactData, FactStore, FactStoreFactory, FactValue, PartitionedFactStore,
    memory::MemoryStats,
};
use std::collections::HashMap;

fn create_test_fact(id: u64, category: &str, value: i64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert(
        "category".to_string(),
        FactValue::String(category.to_string()),
    );
    fields.insert("value".to_string(), FactValue::Integer(value));
    fields.insert("entity_id".to_string(), FactValue::Integer(value % 100)); // Groups facts

    Fact { id, data: FactData { fields } }
}

#[test]
fn test_partitioned_fact_store_basic_operations() {
    let mut store = PartitionedFactStore::new(4); // 4 partitions

    // Insert facts
    let fact1 = create_test_fact(1, "A", 100);
    let fact2 = create_test_fact(2, "B", 200);
    let fact3 = create_test_fact(3, "C", 300);
    let fact4 = create_test_fact(4, "D", 400);

    let id1 = store.insert(fact1.clone());
    let id2 = store.insert(fact2.clone());
    let id3 = store.insert(fact3.clone());
    let id4 = store.insert(fact4.clone());

    // Verify basic operations
    assert_eq!(store.len(), 4);
    assert!(!store.is_empty());

    // Verify retrieval
    println!("Inserted IDs: {}, {}, {}, {}", id1, id2, id3, id4);

    println!("Getting fact {}: {:?}", id1, store.get(id1).map(|f| f.id));
    println!("Getting fact {}: {:?}", id2, store.get(id2).map(|f| f.id));
    println!("Getting fact {}: {:?}", id3, store.get(id3).map(|f| f.id));
    println!("Getting fact {}: {:?}", id4, store.get(id4).map(|f| f.id));

    assert_eq!(store.get(id1).unwrap().id, id1);
    assert_eq!(store.get(id2).unwrap().id, id2);
    assert_eq!(store.get(id3).unwrap().id, id3);
    assert_eq!(store.get(id4).unwrap().id, id4);

    // Verify content
    assert_eq!(
        store.get(id1).unwrap().data.fields.get("category").unwrap(),
        &FactValue::String("A".to_string())
    );
    assert_eq!(
        store.get(id2).unwrap().data.fields.get("value").unwrap(),
        &FactValue::Integer(200)
    );
}

#[test]
fn test_partitioned_fact_store_with_capacity() {
    let store = PartitionedFactStore::with_capacity(3, 100); // 3 partitions, 100 capacity each

    assert_eq!(store.len(), 0);
    assert!(store.is_empty());

    // Check partition stats
    let stats = store.partition_stats();
    assert_eq!(stats.len(), 3);
    for (_, count) in stats {
        assert_eq!(count, 0); // All partitions should be empty initially
    }
}

#[test]
fn test_partitioned_fact_store_distribution() {
    let mut store = PartitionedFactStore::new(4);

    // Insert many facts to see distribution
    for i in 0..20 {
        let fact = create_test_fact(i, &format!("Cat{}", i % 5), i as i64);
        store.insert(fact);
    }

    assert_eq!(store.len(), 20);

    // Check partition distribution
    let stats = store.partition_stats();
    assert_eq!(stats.len(), 4);

    let total_in_partitions: usize = stats.iter().map(|(_, count)| count).sum();
    assert_eq!(total_in_partitions, 20);

    // Each partition should have some facts (may not be perfectly balanced)
    let non_empty_partitions = stats.iter().filter(|(_, count)| *count > 0).count();
    assert!(
        non_empty_partitions >= 2,
        "Should have facts distributed across partitions"
    );

    println!("Partition distribution: {:?}", stats);
}

#[test]
fn test_partitioned_fact_store_field_search() {
    let mut store = PartitionedFactStore::new(3);

    // Insert facts with known patterns
    let facts = vec![
        create_test_fact(1, "active", 1),   // entity_id = 1
        create_test_fact(2, "inactive", 2), // entity_id = 2
        create_test_fact(3, "active", 3),   // entity_id = 3
        create_test_fact(4, "pending", 4),  // entity_id = 4
        create_test_fact(5, "active", 5),   // entity_id = 5
    ];

    for fact in facts {
        store.insert(fact);
    }

    // Debug: Print partition stats and total facts
    let stats = store.partition_stats();
    println!("Partition distribution: {:?}", stats);
    println!("Total facts in store: {}", store.len());

    // Search by category
    let active_facts = store.find_by_field("category", &FactValue::String("active".to_string()));
    println!("Active facts found: {}", active_facts.len());
    for fact in &active_facts {
        println!(
            "  Fact ID: {}, category: {:?}",
            fact.id,
            fact.data.fields.get("category")
        );
    }
    assert_eq!(active_facts.len(), 3);

    let inactive_facts =
        store.find_by_field("category", &FactValue::String("inactive".to_string()));
    println!("Inactive facts found: {}", inactive_facts.len());
    assert_eq!(inactive_facts.len(), 1);

    let pending_facts = store.find_by_field("category", &FactValue::String("pending".to_string()));
    println!("Pending facts found: {}", pending_facts.len());
    assert_eq!(pending_facts.len(), 1);

    // Search by entity_id (which should be indexed)
    let entity_id_1_facts = store.find_by_field("entity_id", &FactValue::Integer(1));
    println!("Entity ID 1 facts found: {}", entity_id_1_facts.len());
    assert_eq!(entity_id_1_facts.len(), 1); // Only fact with value 1 has entity_id = 1
}

#[test]
fn test_partitioned_fact_store_criteria_search() {
    let mut store = PartitionedFactStore::new(2);

    // Insert facts
    let facts = vec![
        create_test_fact(1, "active", 10),   // entity_id = 10
        create_test_fact(2, "active", 20),   // entity_id = 20
        create_test_fact(3, "inactive", 10), // entity_id = 10
        create_test_fact(4, "active", 30),   // entity_id = 30
    ];

    for fact in facts {
        store.insert(fact);
    }

    // Search by multiple criteria
    let criteria = vec![
        (
            "category".to_string(),
            FactValue::String("active".to_string()),
        ),
        ("entity_id".to_string(), FactValue::Integer(10)),
    ];

    let matching_facts = store.find_by_criteria(&criteria);
    assert_eq!(matching_facts.len(), 1);

    let found_fact = matching_facts[0];
    assert_eq!(
        found_fact.data.fields.get("category").unwrap(),
        &FactValue::String("active".to_string())
    );
    assert_eq!(
        found_fact.data.fields.get("entity_id").unwrap(),
        &FactValue::Integer(10)
    );
}

#[test]
fn test_partitioned_fact_store_bulk_operations() {
    let mut store = PartitionedFactStore::new(4);

    // Create bulk facts
    let bulk_facts: Vec<_> = (0..100)
        .map(|i| create_test_fact(i, &format!("bulk_{}", i % 10), i as i64))
        .collect();

    // Test bulk extend
    store.extend_from_vec(bulk_facts);
    assert_eq!(store.len(), 100);

    // Verify distribution across partitions
    let stats = store.partition_stats();
    let total_in_partitions: usize = stats.iter().map(|(_, count)| count).sum();
    assert_eq!(total_in_partitions, 100);

    // Each partition should have some facts
    for (partition_id, count) in stats {
        assert!(
            count > 0,
            "Partition {} should have some facts",
            partition_id
        );
        println!("Partition {}: {} facts", partition_id, count);
    }

    // Test clear
    store.clear();
    assert_eq!(store.len(), 0);
    assert!(store.is_empty());

    let cleared_stats = store.partition_stats();
    for (_, count) in cleared_stats {
        assert_eq!(count, 0);
    }
}

#[test]
fn test_fact_store_factory_partitioned() {
    // Test factory creation of partitioned store
    let store = FactStoreFactory::create_partitioned(4, 1000);
    assert_eq!(store.len(), 0);

    // Test large dataset factory method
    let large_store = FactStoreFactory::create_for_large_dataset(2_000_000);
    assert_eq!(large_store.len(), 0);

    let medium_store = FactStoreFactory::create_for_large_dataset(50_000);
    assert_eq!(medium_store.len(), 0);

    let small_store = FactStoreFactory::create_for_large_dataset(1_000);
    assert_eq!(small_store.len(), 0);
}

#[test]
fn test_partitioned_fact_store_memory_efficiency() {
    // Test memory efficiency with large dataset
    let mut store = PartitionedFactStore::new(8); // 8 partitions

    let start_memory = MemoryStats::current().unwrap();

    // Insert 10K facts
    for i in 0..10_000 {
        let fact = create_test_fact(i, &format!("cat_{}", i % 50), i as i64);
        store.insert(fact);
    }

    let end_memory = MemoryStats::current().unwrap();
    let memory_delta = end_memory.delta_from(&start_memory);

    assert_eq!(store.len(), 10_000);

    // Check distribution
    let stats = store.partition_stats();
    println!("Memory usage for 10K facts across 8 partitions:");
    println!(
        "  Total memory delta: {:.2} MB",
        memory_delta as f64 / (1024.0 * 1024.0)
    );

    for (partition_id, count) in stats {
        println!("  Partition {}: {} facts", partition_id, count);
    }

    // Memory usage should be reasonable
    assert!(
        memory_delta < 100 * 1024 * 1024,
        "Memory usage should be under 100MB for 10K facts"
    );

    // Test retrieval performance by accessing random facts
    let mut found_count = 0;
    for i in (0..10_000).step_by(100) {
        if store.get(i).is_some() {
            found_count += 1;
        }
    }

    assert_eq!(
        found_count, 100,
        "Should be able to retrieve all sampled facts"
    );
}

#[test]
fn test_partitioned_fact_store_edge_cases() {
    // Test with single partition (should work like regular store)
    let mut single_partition_store = PartitionedFactStore::new(1);

    for i in 0..10 {
        let fact = create_test_fact(i, "test", i as i64);
        single_partition_store.insert(fact);
    }

    assert_eq!(single_partition_store.len(), 10);
    let stats = single_partition_store.partition_stats();
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].1, 10); // All facts in single partition

    // Test with many partitions (more than facts)
    let mut many_partitions_store = PartitionedFactStore::new(20);

    for i in 0..5 {
        let fact = create_test_fact(i, "test", i as i64);
        many_partitions_store.insert(fact);
    }

    assert_eq!(many_partitions_store.len(), 5);
    let stats = many_partitions_store.partition_stats();
    assert_eq!(stats.len(), 20);

    let non_empty_partitions = stats.iter().filter(|(_, count)| *count > 0).count();
    assert!(
        non_empty_partitions <= 5,
        "Should have at most 5 non-empty partitions"
    );
}
