//! Integration test for unified statistics system
//!
//! This test demonstrates the consolidation of cache statistics across all
//! optimization layers and the unified reporting capabilities.

use bingo_core::{
    cache::CacheStats,
    field_indexing::FieldIndexer,
    types::{Fact, FactData, FactValue},
    unified_fact_store::OptimizedFactStore,
    unified_statistics::{ComponentStats, UnifiedStats, UnifiedStatsBuilder},
};
use std::collections::HashMap;

fn create_test_fact(id: u64, entity_id: &str, status: &str, category: &str) -> Fact {
    let mut fields = HashMap::new();
    fields.insert(
        "entity_id".to_string(),
        FactValue::String(entity_id.to_string()),
    );
    fields.insert("status".to_string(), FactValue::String(status.to_string()));
    fields.insert(
        "category".to_string(),
        FactValue::String(category.to_string()),
    );
    fields.insert("value".to_string(), FactValue::Integer(id as i64 * 10));

    Fact { id, data: FactData { fields } }
}

#[test]
fn test_unified_statistics_consolidation() {
    println!("🧪 Testing unified statistics consolidation across optimization layers...");

    // Create multiple OptimizedFactStores with different configurations
    let mut hashmap_store = OptimizedFactStore::new_fast(50);
    let mut vector_store = OptimizedFactStore::new_memory_efficient(30);

    // Add facts to both stores
    let facts = vec![
        create_test_fact(1, "user_1", "active", "premium"),
        create_test_fact(2, "user_2", "inactive", "basic"),
        create_test_fact(3, "user_3", "active", "premium"),
        create_test_fact(4, "user_4", "pending", "basic"),
        create_test_fact(5, "user_5", "active", "enterprise"),
    ];

    for fact in facts.clone() {
        hashmap_store.insert(fact.clone());
        vector_store.insert(fact);
    }

    // Perform lookups to generate cache statistics
    for i in 1..=5 {
        // Multiple lookups to create cache hits and misses
        hashmap_store.get_mut(i);
        hashmap_store.get_mut(i); // Cache hit
        hashmap_store.get_mut(i); // Cache hit

        vector_store.get_mut(i);
        vector_store.get_mut(i); // Cache hit
    }

    // Perform field-based queries to exercise indexing
    hashmap_store.find_by_field("status", &FactValue::String("active".to_string()));
    hashmap_store.find_by_field("category", &FactValue::String("premium".to_string()));
    vector_store.find_by_field("status", &FactValue::String("active".to_string()));

    // Generate unified statistics from both stores
    let hashmap_stats = hashmap_store.generate_unified_stats();
    let vector_stats = vector_store.generate_unified_stats();

    // Create consolidated system statistics
    let mut consolidated_stats = UnifiedStats::new();
    consolidated_stats.merge(&hashmap_stats);
    consolidated_stats.merge(&vector_stats);

    // Add simulated memory pool statistics
    consolidated_stats.register_memory_pool("Token", 800, 200, 500, 450, 75);
    consolidated_stats.register_memory_pool("FactIdSet", 600, 100, 300, 280, 50);

    // Add simulated calculator statistics
    consolidated_stats.register_calculator(700, 300, 850, 150, 1000);

    // Add custom component statistics
    let mut custom_component = ComponentStats {
        operations: 1000,
        successes: 950,
        failures: 50,
        avg_time_us: 25.5,
        ..Default::default()
    };
    custom_component.custom_metrics.insert("throughput".to_string(), 95.0);
    consolidated_stats.register_component("ReteNetwork", custom_component);

    // Print comprehensive statistics report
    println!("\n{}", consolidated_stats);

    // Verify consolidation worked correctly
    assert_eq!(consolidated_stats.fact_storage.total_facts, 10); // 5 facts in each store
    assert_eq!(consolidated_stats.fact_storage.facts_by_backend.len(), 2); // HashMap + Vector
    assert!(consolidated_stats.caching.total_hits > 0);
    assert!(consolidated_stats.caching.total_misses > 0);
    assert!(consolidated_stats.memory.pool_hits > 0);
    assert!(consolidated_stats.calculator.compilation_hits > 0);
    assert_eq!(consolidated_stats.indexing.indexed_fields, 12); // 6 fields from each store merged
    assert_eq!(consolidated_stats.component_counters.len(), 1);

    // Verify efficiency calculations
    let overall_efficiency = consolidated_stats.optimization_efficiency_score();
    println!(
        "📊 Overall optimization efficiency: {:.1}%",
        overall_efficiency
    );
    assert!(overall_efficiency > 50.0); // Should be reasonably high

    let cache_hit_rate = consolidated_stats.overall_cache_hit_rate();
    println!("⚡ Overall cache hit rate: {:.1}%", cache_hit_rate);
    assert!(cache_hit_rate > 50.0); // Should have good cache performance

    let pool_utilization = consolidated_stats.overall_pool_utilization();
    println!("🧠 Memory pool utilization: {:.1}%", pool_utilization);
    assert!(pool_utilization > 70.0); // Simulated high pool utilization

    // Test builder pattern with complex scenario
    let complex_stats = UnifiedStatsBuilder::new()
        .with_fact_storage("HashMap", 5000, 15000)
        .with_fact_storage("Vector", 3000, 8000)
        .with_cache(
            "PrimaryCache",
            CacheStats { capacity: 1000, size: 800, access_counter: 5000 },
            4000,
            1000,
        )
        .with_cache(
            "SecondaryCache",
            CacheStats { capacity: 500, size: 300, access_counter: 2000 },
            1500,
            500,
        )
        .with_memory_pool("TokenPool", 9000, 1000, 5000, 4800, 200)
        .with_memory_pool("FactPool", 7500, 500, 3000, 2900, 150)
        .build();

    println!("\n=== Complex System Statistics ===");
    println!("{}", complex_stats);

    assert_eq!(complex_stats.fact_storage.total_facts, 8000);
    assert_eq!(complex_stats.fact_storage.total_lookups, 23000);
    assert_eq!(complex_stats.caching.total_hits, 5500);
    assert_eq!(complex_stats.caching.total_misses, 1500);
    assert_eq!(complex_stats.memory.pool_hits, 16500);
    assert_eq!(complex_stats.memory.pool_misses, 1500);

    let complex_efficiency = complex_stats.optimization_efficiency_score();
    println!("🎯 Complex system efficiency: {:.1}%", complex_efficiency);
    assert!(complex_efficiency > 50.0); // Should be reasonable efficiency (no calculator/indexing stats)

    println!("✅ Unified statistics consolidation test completed successfully!");
    println!(
        "📈 Cache statistics unified: {} components",
        consolidated_stats.caching.cache_by_component.len()
    );
    println!(
        "🏪 Memory pools tracked: {} pools",
        consolidated_stats.memory.peak_pool_sizes.len()
    );
    println!(
        "🔧 Custom components: {} components",
        consolidated_stats.component_counters.len()
    );
}

#[test]
fn test_statistics_merge_accuracy() {
    println!("🧪 Testing statistics merge accuracy...");

    // Create baseline statistics
    let mut stats1 = UnifiedStats::new();
    stats1.register_fact_storage("HashMap", 1000, 5000);
    stats1.register_memory_pool("Tokens", 800, 200, 400, 380, 50);

    let mut stats2 = UnifiedStats::new();
    stats2.register_fact_storage("Vector", 2000, 3000);
    stats2.register_memory_pool("Facts", 600, 100, 300, 290, 40);

    // Merge statistics
    stats1.merge(&stats2);

    // Verify merge accuracy
    assert_eq!(stats1.fact_storage.total_facts, 3000);
    assert_eq!(stats1.fact_storage.total_lookups, 8000);
    assert_eq!(stats1.memory.pool_hits, 1400);
    assert_eq!(stats1.memory.pool_misses, 300);
    assert_eq!(stats1.memory.peak_pool_sizes.len(), 2);

    println!("✅ Statistics merge accuracy verified!");
}

#[test]
fn test_optimization_layer_visibility() {
    println!("🧪 Testing optimization layer visibility...");

    let mut unified_stats = UnifiedStats::new();

    // Register statistics from different optimization layers
    unified_stats.register_fact_storage("OptimizedStore", 5000, 12000);

    let cache_stats = CacheStats { capacity: 1000, size: 750, access_counter: 8000 };
    unified_stats.register_cache("FactCache", cache_stats, 7000, 1000);

    unified_stats.register_memory_pool("TokenPool", 9500, 500, 4000, 3800, 200);
    unified_stats.register_calculator(800, 200, 900, 100, 1000);

    // Create field indexer stats
    let mut field_indexer = FieldIndexer::new();

    // Add test facts to indexer (will use default indexed fields)
    for i in 1..=100 {
        let fact = create_test_fact(i, &format!("entity_{}", i), "active", "test");
        field_indexer.index_fact(&fact);
    }

    unified_stats.register_indexing(field_indexer.stats());

    // Verify all layers are visible
    assert!(unified_stats.fact_storage.total_facts > 0);
    assert!(unified_stats.caching.total_hits > 0);
    assert!(unified_stats.memory.pool_hits > 0);
    assert!(unified_stats.calculator.compilation_hits > 0);
    assert!(unified_stats.indexing.indexed_fields > 0);

    println!("📊 All optimization layers visible in unified statistics:");
    println!(
        "  💾 Fact Storage: {} facts, {} lookups",
        unified_stats.fact_storage.total_facts, unified_stats.fact_storage.total_lookups
    );
    println!(
        "  ⚡ Caching: {:.1}% hit rate",
        unified_stats.overall_cache_hit_rate()
    );
    println!(
        "  🧠 Memory Pools: {:.1}% utilization",
        unified_stats.overall_pool_utilization()
    );
    println!(
        "  🧮 Calculator: {:.1}% compilation efficiency",
        unified_stats.calculator_compilation_efficiency()
    );
    println!(
        "  🗂️ Indexing: {} fields, {} entries",
        unified_stats.indexing.indexed_fields, unified_stats.indexing.total_index_entries
    );

    println!("✅ All optimization layers successfully unified!");
}
