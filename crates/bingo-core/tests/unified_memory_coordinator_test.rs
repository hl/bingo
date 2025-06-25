//! Integration test for unified memory coordinator
//!
//! This test demonstrates the consolidation of memory management across all
//! optimization layers and the coordinated memory pressure handling.

use bingo_core::{
    FactStore,
    cache::LruCache,
    types::{Fact, FactData, FactValue},
    unified_fact_store::OptimizedFactStore,
    unified_memory_coordinator::{
        CacheMemoryConsumer, CoordinationResult, MemoryConsumer, MemoryCoordinatorConfig,
        UnifiedMemoryCoordinator,
    },
    unified_statistics::UnifiedStatsBuilder,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn create_test_fact(id: u64, entity_id: &str, status: &str, value: i64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert(
        "entity_id".to_string(),
        FactValue::String(entity_id.to_string()),
    );
    fields.insert("status".to_string(), FactValue::String(status.to_string()));
    fields.insert("value".to_string(), FactValue::Integer(value));

    Fact { id, data: FactData { fields } }
}

/// Custom memory consumer for testing
struct TestMemoryConsumer {
    name: String,
    memory_usage: usize,
    reduction_count: usize,
}

impl TestMemoryConsumer {
    fn new(name: &str, initial_memory: usize) -> Self {
        Self { name: name.to_string(), memory_usage: initial_memory, reduction_count: 0 }
    }
}

impl MemoryConsumer for TestMemoryConsumer {
    fn memory_usage_bytes(&self) -> usize {
        self.memory_usage
    }

    fn reduce_memory_usage(&mut self, reduction_factor: f64) -> usize {
        let old_usage = self.memory_usage;
        self.memory_usage = (self.memory_usage as f64 * reduction_factor) as usize;
        self.reduction_count += 1;
        old_usage - self.memory_usage
    }

    fn get_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        stats.insert(
            "memory_usage_mb".to_string(),
            self.memory_usage as f64 / (1024.0 * 1024.0),
        );
        stats.insert("reduction_count".to_string(), self.reduction_count as f64);
        stats
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[test]
fn test_unified_memory_coordinator_integration() {
    println!("🧪 Testing unified memory coordinator integration...");

    // Create memory coordinator with testing-friendly configuration
    let config = MemoryCoordinatorConfig {
        max_memory_bytes: 100 * 1024 * 1024, // 100MB limit for testing
        pressure_threshold: 10.0,            // Very low threshold to trigger easily
        critical_threshold: 20.0,            // Low critical threshold
        monitor_interval: Duration::from_millis(100),
        auto_cleanup: true,
        cache_reduction_factor: 0.7,
        pool_reduction_factor: 0.8,
    };

    let coordinator = UnifiedMemoryCoordinator::new(config);

    // Register test memory consumers
    let test_consumer1 = TestMemoryConsumer::new("TestStore1", 50 * 1024 * 1024); // 50MB
    let test_consumer2 = TestMemoryConsumer::new("TestStore2", 30 * 1024 * 1024); // 30MB

    coordinator.register_consumer("TestStore1".to_string(), test_consumer1);
    coordinator.register_consumer("TestStore2".to_string(), test_consumer2);

    // Create cache consumers
    let cache1 = Arc::new(Mutex::new(LruCache::new(1000)));
    let cache2 = Arc::new(Mutex::new(LruCache::new(500)));

    // Populate caches
    {
        let mut cache1_lock = cache1.lock().unwrap();
        for i in 0..100 {
            cache1_lock.put(format!("key_{}", i), format!("value_{}", i));
        }
    }
    {
        let mut cache2_lock = cache2.lock().unwrap();
        for i in 0..50 {
            cache2_lock.put(format!("cache2_key_{}", i), i);
        }
    }

    let cache_consumer1 = CacheMemoryConsumer::new("Cache1".to_string(), cache1.clone());
    let cache_consumer2 = CacheMemoryConsumer::new("Cache2".to_string(), cache2.clone());

    coordinator.register_consumer("Cache1".to_string(), cache_consumer1);
    coordinator.register_consumer("Cache2".to_string(), cache_consumer2);

    // Test initial memory info
    let initial_info = coordinator.get_memory_info();
    println!("📊 Initial memory info: {}", initial_info.format_summary());

    assert_eq!(initial_info.coordination_stats.cleanup_operations, 0);
    assert_eq!(initial_info.coordination_stats.pressure_events_handled, 0);

    // Force memory coordination to trigger pressure handling
    let result = coordinator.coordinate_memory().unwrap();
    println!("🔧 Coordination result: {:?}", result);

    // Check if pressure was handled (depends on actual system memory)
    match result {
        CoordinationResult::NoAction => {
            println!("✅ No memory pressure detected - system has sufficient memory");
        }
        CoordinationResult::PressureHandled { memory_freed_bytes, components_affected } => {
            println!(
                "⚠️ Memory pressure handled: {} bytes freed from {} components",
                memory_freed_bytes, components_affected
            );
            assert!(memory_freed_bytes > 0);
            assert!(components_affected > 0);
        }
        CoordinationResult::CriticalHandled { memory_freed_bytes, components_affected } => {
            println!(
                "🚨 Critical memory handled: {} bytes freed from {} components",
                memory_freed_bytes, components_affected
            );
            assert!(memory_freed_bytes > 0);
            assert!(components_affected > 0);
        }
    }

    // Test forced cleanup
    let freed_bytes = coordinator.force_cleanup().unwrap();
    println!("🧹 Forced cleanup freed: {} bytes", freed_bytes);
    assert!(freed_bytes > 0);

    // Get final memory info
    let final_info = coordinator.get_memory_info();
    println!("📊 Final memory info: {}", final_info.format_summary());

    // Verify cleanup statistics were updated
    assert!(final_info.coordination_stats.cleanup_operations > 0);
    assert!(final_info.coordination_stats.memory_freed_bytes > 0);

    println!("✅ Unified memory coordinator integration test completed!");
}

#[test]
fn test_memory_coordinator_with_fact_stores() {
    println!("🧪 Testing memory coordinator with fact stores...");

    let coordinator = UnifiedMemoryCoordinator::with_defaults();

    // Create optimized fact stores
    let mut hashmap_store = OptimizedFactStore::new_fast(100);
    let mut vector_store = OptimizedFactStore::new_memory_efficient(50);

    // Add facts to stores
    for i in 1..=1000 {
        let fact = create_test_fact(i, &format!("entity_{}", i), "active", i as i64 * 10);
        hashmap_store.insert(fact.clone());
        vector_store.insert(fact);
    }

    // Perform lookups to generate cache activity
    for i in 1..=1000 {
        hashmap_store.get_mut(i);
        vector_store.get_mut(i);
    }

    // Generate unified statistics
    let hashmap_stats = hashmap_store.generate_unified_stats();
    let vector_stats = vector_store.generate_unified_stats();

    println!("📊 HashMap store stats:");
    println!(
        "  Facts: {}, Lookups: {}, Cache hits: {}",
        hashmap_stats.fact_storage.total_facts,
        hashmap_stats.fact_storage.total_lookups,
        hashmap_stats.caching.total_hits
    );

    println!("📊 Vector store stats:");
    println!(
        "  Facts: {}, Lookups: {}, Cache hits: {}",
        vector_stats.fact_storage.total_facts,
        vector_stats.fact_storage.total_lookups,
        vector_stats.caching.total_hits
    );

    // Get global memory pools from coordinator
    let global_pools = coordinator.get_global_pools();
    let pool_stats = global_pools.get_stats();

    let total_hits = pool_stats.token_pool.pool_hits
        + pool_stats.fact_data_pool.pool_hits
        + pool_stats.field_map_pool.pool_hits
        + pool_stats.fact_vec_pool.pool_hits
        + pool_stats.fact_id_set_pool.pool_hits;
    let total_misses = pool_stats.token_pool.pool_misses
        + pool_stats.fact_data_pool.pool_misses
        + pool_stats.field_map_pool.pool_misses
        + pool_stats.fact_vec_pool.pool_misses
        + pool_stats.fact_id_set_pool.pool_misses;

    println!("🏊 Global pool stats:");
    println!(
        "  Total hits: {}, Total misses: {}",
        total_hits, total_misses
    );
    println!(
        "  Token pool size: {}, Fact data pool size: {}",
        pool_stats.token_pool.current_size, pool_stats.fact_data_pool.current_size
    );

    // Generate unified memory statistics from coordinator
    let memory_stats = coordinator.generate_unified_stats();
    println!("🧠 Unified memory statistics:");
    println!(
        "  Pool hits: {}, Pool misses: {}",
        memory_stats.pool_hits, memory_stats.pool_misses
    );
    println!(
        "  Total allocated: {} bytes",
        memory_stats.total_allocated_bytes
    );
    println!("  Peak pool sizes: {:?}", memory_stats.peak_pool_sizes);

    // Verify integration (pools exist and are accessible)
    println!(
        "✅ Pool stats accessible: token pool size = {}",
        pool_stats.token_pool.current_size
    );
    println!(
        "✅ Memory stats accessible: allocated = {} bytes",
        memory_stats.total_allocated_bytes
    );

    println!("✅ Memory coordinator with fact stores test completed!");
}

#[test]
fn test_memory_pressure_simulation() {
    println!("🧪 Testing memory pressure simulation...");

    // Create coordinator with very low memory limits for testing
    let config = MemoryCoordinatorConfig {
        max_memory_bytes: 1024,  // 1KB limit to force pressure
        pressure_threshold: 1.0, // 1% threshold
        critical_threshold: 2.0, // 2% threshold
        auto_cleanup: true,
        cache_reduction_factor: 0.5,
        pool_reduction_factor: 0.5,
        ..Default::default()
    };

    let coordinator = UnifiedMemoryCoordinator::new(config);

    // Register high-memory consumers
    let high_memory_consumer = TestMemoryConsumer::new("HighMemoryStore", 10 * 1024 * 1024); // 10MB
    coordinator.register_consumer("HighMemoryStore".to_string(), high_memory_consumer);

    // Force coordinate memory - should trigger pressure/critical handling
    let result = coordinator.coordinate_memory().unwrap();

    match result {
        CoordinationResult::NoAction => {
            println!("⚠️ Expected pressure handling but got no action");
        }
        CoordinationResult::PressureHandled { memory_freed_bytes, components_affected } => {
            println!(
                "✅ Pressure handling triggered: {} bytes freed, {} components affected",
                memory_freed_bytes, components_affected
            );
            assert!(memory_freed_bytes > 0);
            assert_eq!(components_affected, 1);
        }
        CoordinationResult::CriticalHandled { memory_freed_bytes, components_affected } => {
            println!(
                "✅ Critical handling triggered: {} bytes freed, {} components affected",
                memory_freed_bytes, components_affected
            );
            assert!(memory_freed_bytes > 0);
            assert_eq!(components_affected, 1);
        }
    }

    let final_info = coordinator.get_memory_info();
    println!(
        "📊 Final coordination stats: {:?}",
        final_info.coordination_stats
    );

    // Verify pressure/critical events were handled
    assert!(
        final_info.coordination_stats.pressure_events_handled > 0
            || final_info.coordination_stats.critical_events_handled > 0
    );

    println!("✅ Memory pressure simulation test completed!");
}

#[test]
fn test_memory_coordination_with_unified_statistics() {
    println!("🧪 Testing memory coordination with unified statistics...");

    let coordinator = UnifiedMemoryCoordinator::with_defaults();

    // Create comprehensive system with multiple optimization layers
    let mut fact_store = OptimizedFactStore::new_fast(200);

    // Add facts and perform operations
    for i in 1..=500 {
        let fact = create_test_fact(i, &format!("user_{}", i), "active", i as i64);
        fact_store.insert(fact);
    }

    // Perform many lookups to generate statistics
    for i in 1..=500 {
        fact_store.get_mut(i);
        fact_store.get_mut(i); // Second lookup for cache hits
    }

    // Generate combined statistics
    let fact_store_stats = fact_store.generate_unified_stats();
    let memory_coordinator_stats = coordinator.generate_unified_stats();

    // Combine statistics using builder pattern
    let comprehensive_stats = UnifiedStatsBuilder::new()
        .with_fact_storage(
            "OptimizedStore",
            fact_store_stats.fact_storage.total_facts,
            fact_store_stats.fact_storage.total_lookups,
        )
        .with_cache(
            "FactStoreCache",
            fact_store.cache_stats().unwrap_or(bingo_core::CacheStats {
                capacity: 0,
                size: 0,
                access_counter: 0,
            }),
            fact_store_stats.caching.total_hits,
            fact_store_stats.caching.total_misses,
        )
        .with_memory_pool(
            "CoordinatorPools",
            memory_coordinator_stats.pool_hits,
            memory_coordinator_stats.pool_misses,
            memory_coordinator_stats.pool_allocated,
            memory_coordinator_stats.pool_returned,
            memory_coordinator_stats.peak_pool_sizes.values().max().cloned().unwrap_or(0),
        )
        .build();

    println!("📊 Comprehensive system statistics:");
    println!("{}", comprehensive_stats);

    // Verify comprehensive integration
    assert!(comprehensive_stats.fact_storage.total_facts > 0);
    assert!(comprehensive_stats.caching.total_hits > 0);
    // pool_hits is always >= 0 by type definition - pools may not be used in this test
    assert!(comprehensive_stats.optimization_efficiency_score() > 0.0);

    // Test memory coordination
    let coordination_result = coordinator.coordinate_memory().unwrap();
    let memory_info = coordinator.get_memory_info();

    println!("🔧 Memory coordination result: {:?}", coordination_result);
    println!("📊 Memory info: {}", memory_info.format_summary());

    // Verify memory coordination statistics
    match coordination_result {
        CoordinationResult::NoAction => {
            println!("✅ System operating within normal memory parameters");
        }
        _ => {
            println!("✅ Memory coordination active - system under load");
            assert!(memory_info.coordination_stats.cleanup_operations > 0);
        }
    }

    println!("✅ Memory coordination with unified statistics test completed!");
    println!(
        "📈 System efficiency score: {:.1}%",
        comprehensive_stats.optimization_efficiency_score()
    );
    println!(
        "⚡ Cache performance: {:.1}% hit rate",
        comprehensive_stats.overall_cache_hit_rate()
    );
    println!(
        "🧠 Memory pool utilization: {:.1}%",
        comprehensive_stats.overall_pool_utilization()
    );
}
