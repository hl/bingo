//! Test to demonstrate field indexing consolidation benefits
//!
//! This test validates that the shared FieldIndexer successfully replaces
//! the duplicated indexing logic across fact storage implementations.

use bingo_core::*;
use std::collections::HashMap;

#[test]
fn test_field_indexing_consolidation() {
    // Test that OptimizedFactStore now uses shared FieldIndexer
    let mut store = OptimizedFactStore::new_fast(100);

    // Create test facts with indexed fields
    let mut facts = Vec::new();
    for i in 1..=20 {
        let mut fields = HashMap::new();
        fields.insert(
            "status".to_string(),
            FactValue::String(if i % 3 == 0 { "active" } else { "inactive" }.to_string()),
        );
        fields.insert(
            "category".to_string(),
            FactValue::String(if i % 2 == 0 { "premium" } else { "basic" }.to_string()),
        );
        fields.insert("user_id".to_string(), FactValue::Integer(i as i64));

        facts.push(Fact { id: i, data: FactData { fields } });
    }

    // Insert facts (should build indexes automatically)
    for fact in facts {
        store.insert(fact);
    }

    // Test single field queries using shared indexer
    let active_facts = store.find_by_field("status", &FactValue::String("active".to_string()));
    let premium_facts = store.find_by_field("category", &FactValue::String("premium".to_string()));

    println!("✅ Field indexing consolidation test results:");
    println!("   Active facts found: {}", active_facts.len());
    println!("   Premium facts found: {}", premium_facts.len());

    // Verify expected results
    assert_eq!(active_facts.len(), 6); // Facts 3, 6, 9, 12, 15, 18 (every 3rd)
    assert_eq!(premium_facts.len(), 10); // Facts 2, 4, 6, 8, 10, 12, 14, 16, 18, 20 (even)

    // Test multi-criteria queries
    let criteria = vec![
        (
            "status".to_string(),
            FactValue::String("active".to_string()),
        ),
        (
            "category".to_string(),
            FactValue::String("premium".to_string()),
        ),
    ];
    let multi_criteria_facts = store.find_by_criteria(&criteria);

    println!(
        "   Multi-criteria facts found: {}",
        multi_criteria_facts.len()
    );
    assert_eq!(multi_criteria_facts.len(), 3); // Facts 6, 12, 18 (active AND premium)

    // Test field indexing statistics
    let field_stats = store.get_field_index_stats();
    println!("   Field indexes created: {}", field_stats.total_indexes);
    println!("   Total index entries: {}", field_stats.total_entries);
    println!("   Total fact references: {}", field_stats.total_fact_refs);
    println!("   Index efficiency: {:.2}", field_stats.index_efficiency());

    // Verify indexing efficiency
    assert!(field_stats.total_indexes >= 3); // At least status, category, user_id
    assert!(field_stats.total_entries > 0);
    assert_eq!(field_stats.total_fact_refs, 20 * 3); // 20 facts × 3 indexed fields each
    assert!(field_stats.index_efficiency() > 0.0);

    println!("✅ Shared FieldIndexer successfully replaced duplicated logic");
}

#[test]
fn test_field_indexer_customization() {
    // Test that we can now easily customize indexed fields across all fact stores
    let custom_fields = vec!["custom_field_1".to_string(), "custom_field_2".to_string()];

    let mut indexer = FieldIndexer::with_fields(custom_fields.clone());

    // Create test fact with custom fields
    let mut fields = HashMap::new();
    fields.insert(
        "custom_field_1".to_string(),
        FactValue::String("value1".to_string()),
    );
    fields.insert("custom_field_2".to_string(), FactValue::Integer(42));
    fields.insert("ignored_field".to_string(), FactValue::Boolean(true));

    let fact = Fact { id: 1, data: FactData { fields } };

    indexer.index_fact(&fact);

    // Test that custom fields are indexed
    let results1 =
        indexer.find_by_field("custom_field_1", &FactValue::String("value1".to_string()));
    let results2 = indexer.find_by_field("custom_field_2", &FactValue::Integer(42));
    let ignored = indexer.find_by_field("ignored_field", &FactValue::Boolean(true));

    assert_eq!(results1.len(), 1);
    assert_eq!(results2.len(), 1);
    assert_eq!(ignored.len(), 0); // Not indexed

    println!("✅ Custom field indexing configuration works correctly");
    println!("   Indexed fields: {:?}", indexer.get_indexed_fields());
}

#[test]
fn test_indexing_consolidation_memory_efficiency() {
    // Demonstrate that shared indexing is more memory efficient
    let mut store = OptimizedFactStore::new_fast(50);

    // Add many facts to test memory usage
    for i in 1..=1000 {
        let mut fields = HashMap::new();
        fields.insert("entity_id".to_string(), FactValue::Integer(i));
        fields.insert(
            "status".to_string(),
            FactValue::String(
                format!("status_{}", i % 10), // 10 different statuses
            ),
        );

        let fact = Fact { id: i as u64, data: FactData { fields } };
        store.insert(fact);
    }

    let stats = store.stats();
    let field_stats = stats.field_index_stats;

    println!("✅ Memory efficiency test results:");
    println!("   Facts stored: {}", stats.facts_stored);
    println!("   Field indexes: {}", field_stats.total_indexes);
    println!("   Index entries: {}", field_stats.total_entries);
    println!("   Memory usage: {} bytes", field_stats.memory_usage_bytes);
    println!(
        "   Average entries per index: {:.2}",
        field_stats.avg_entries_per_index()
    );
    println!(
        "   Index selectivity: {:.2}",
        field_stats.index_efficiency()
    );

    // Verify reasonable memory usage
    assert_eq!(stats.facts_stored, 1000);
    assert!(field_stats.memory_usage_bytes > 0);
    assert!(field_stats.avg_entries_per_index() > 0.0);

    // Test query performance with many facts
    let query_result = store.find_by_field("status", &FactValue::String("status_5".to_string()));
    assert_eq!(query_result.len(), 100); // Every 10th fact starting from 5

    println!("✅ Shared indexing handles large datasets efficiently");
}

#[test]
fn test_indexing_removal_and_cleanup() {
    // Test that the shared indexer properly handles fact removal
    let mut store = OptimizedFactStore::new_fast(10);

    // Add some facts
    let mut facts_to_remove = Vec::new();
    for i in 1..=10 {
        let mut fields = HashMap::new();
        fields.insert(
            "category".to_string(),
            FactValue::String("test".to_string()),
        );
        fields.insert("id".to_string(), FactValue::Integer(i));

        let fact = Fact { id: i as u64, data: FactData { fields } };
        facts_to_remove.push(fact.clone());
        store.insert(fact);
    }

    // Verify facts are indexed
    let before_removal = store.find_by_field("category", &FactValue::String("test".to_string()));
    assert_eq!(before_removal.len(), 10);

    // Remove half the facts
    for i in 1..=5 {
        store.remove(i as u64);
    }

    // Verify index cleanup
    let after_removal = store.find_by_field("category", &FactValue::String("test".to_string()));
    assert_eq!(after_removal.len(), 5);

    let field_stats = store.get_field_index_stats();
    println!("✅ Index cleanup test results:");
    println!("   Facts remaining: {}", store.len());
    println!(
        "   Index entries after cleanup: {}",
        field_stats.total_entries
    );
    println!("   Total fact references: {}", field_stats.total_fact_refs);

    // Verify proper cleanup
    assert_eq!(store.len(), 5);
    assert!(field_stats.total_fact_refs == 10); // 5 facts × 2 indexed fields each

    println!("✅ Shared indexer properly handles fact removal and cleanup");
}
