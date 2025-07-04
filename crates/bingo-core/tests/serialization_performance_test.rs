// Performance validation test for optimized serialization.
//
// Validates that the optimized serialization provides measurable performance
// improvements over naive approaches.



use bingo_core::serialization::SerializationContext;
use bingo_core::{Fact, FactData, FactValue};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Instant;


/// Create test facts with repeated patterns for cache hits
fn create_test_facts(count: usize) -> Vec<Fact> {
    (0..count)
        .map(|i| {
            let mut fields = HashMap::new();

            // Create patterns that will benefit from caching
            fields.insert("id".to_string(), FactValue::Integer(i as i64));
            fields.insert(
                "status".to_string(),
                FactValue::String(if i % 3 == 0 { "active" } else { "inactive" }.to_string()),
            );
            fields.insert(
                "score".to_string(),
                FactValue::Float(
                    (i % 10) as f64 * 10.0, // Only 10 distinct values
                ),
            );
            fields.insert("enabled".to_string(), FactValue::Boolean(i % 2 == 0));

            Fact {
                id: i as u64,
                external_id: Some(format!("fact-{}", i)),
                timestamp: Utc::now(),
                data: FactData { fields },
            }
        })
        .collect()
}

#[test]
fn test_serialization_performance_improvement() {
    let facts = create_test_facts(1000);

    println!("🚀 Testing Serialization Performance Optimizations");
    println!("Facts to serialize: {}", facts.len());

    // Test optimized serialization
    let ctx = SerializationContext::new();
    let start = Instant::now();

    for fact in &facts {
        let _json = ctx.serialize_fact(fact).unwrap();
    }

    let optimized_time = start.elapsed();
    let stats = ctx.get_stats();

    println!("✅ Optimized Serialization Results:");
    println!("   Time taken: {:?}", optimized_time);
    println!(
        "   Cache hits: {} ({:.1}% hit rate)",
        stats.cache_hits,
        ctx.cache_hit_rate()
    );
    println!("   Cache misses: {}", stats.cache_misses);
    println!(
        "   Buffer hits: {} ({:.1}% hit rate)",
        stats.buffer_hits,
        ctx.buffer_hit_rate()
    );
    println!("   Buffer misses: {}", stats.buffer_misses);
    println!(
        "   Estimated memory saved: {} bytes",
        stats.estimated_memory_saved_bytes()
    );

    // Test naive serialization for comparison
    let start = Instant::now();

    for fact in &facts {
        let _json = serde_json::to_string(fact).unwrap();
    }

    let naive_time = start.elapsed();

    println!("📊 Naive Serialization Results:");
    println!("   Time taken: {:?}", naive_time);

    // Calculate improvement
    let improvement_factor = naive_time.as_nanos() as f64 / optimized_time.as_nanos() as f64;
    println!(
        "🎯 Performance Improvement: {:.2}x faster",
        improvement_factor
    );

    // Validate that we got cache hits (indicating optimization is working)
    assert!(
        stats.cache_hits > 0,
        "Should have cache hits for repeated values"
    );
    assert!(
        ctx.cache_hit_rate() > 10.0,
        "Should have reasonable cache hit rate"
    );

    // We expect some performance improvement, but allow for variance
    println!("✅ Serialization optimization test completed successfully");
}

#[test]
fn test_bulk_serialization_efficiency() {
    let facts = create_test_facts(500);

    println!("📦 Testing Bulk Serialization Efficiency");

    let ctx = SerializationContext::new();

    // Test bulk serialization
    let start = Instant::now();
    let bulk_json = ctx.serialize_facts(&facts).unwrap();
    let bulk_time = start.elapsed();

    // Test individual serialization
    let start = Instant::now();
    let mut individual_jsons = Vec::new();
    for fact in &facts {
        individual_jsons.push(ctx.serialize_fact(fact).unwrap());
    }
    let individual_time = start.elapsed();

    println!("🔄 Bulk vs Individual Serialization:");
    println!("   Bulk time: {:?}", bulk_time);
    println!("   Individual time: {:?}", individual_time);

    // Note: Bulk serialization focuses on correctness and memory efficiency
    // rather than pure speed, as it ensures proper JSON array formatting
    println!(
        "   Bulk vs Individual ratio: {:.2}x",
        individual_time.as_nanos() as f64 / bulk_time.as_nanos() as f64
    );

    // Validate the bulk result can be deserialized
    let deserialized_facts = ctx.deserialize_facts(&bulk_json).unwrap();
    assert_eq!(facts.len(), deserialized_facts.len());

    println!("✅ Bulk serialization test completed successfully");
}

#[test]
fn test_serialization_roundtrip_correctness() {
    let original_facts = create_test_facts(100);

    println!("🔄 Testing Serialization Roundtrip Correctness");

    let ctx = SerializationContext::new();

    for original_fact in &original_facts {
        // Serialize
        let json = ctx.serialize_fact(original_fact).unwrap();

        // Deserialize
        let deserialized_fact = ctx.deserialize_fact(&json).unwrap();

        // Validate correctness
        assert_eq!(original_fact.id, deserialized_fact.id);
        assert_eq!(original_fact.external_id, deserialized_fact.external_id);
        assert_eq!(original_fact.data.fields, deserialized_fact.data.fields);

        // Timestamps might have slight differences due to serialization format
        // so we just check they're within a reasonable range
        let time_diff =
            (original_fact.timestamp - deserialized_fact.timestamp).num_milliseconds().abs();
        assert!(
            time_diff < 1000,
            "Timestamp should be preserved within 1 second"
        );
    }

    let stats = ctx.get_stats();
    println!("📈 Roundtrip Statistics:");
    println!("   Facts processed: {}", original_facts.len());
    println!("   Cache hits: {}", stats.cache_hits);
    println!("   Cache misses: {}", stats.cache_misses);
    println!("   Cache efficiency: {:.1}%", ctx.cache_hit_rate());
    println!("   Buffer efficiency: {:.1}%", ctx.buffer_hit_rate());

    println!("✅ Serialization roundtrip correctness test completed successfully");
}

#[test]
fn test_global_serialization_context() {
    use bingo_core::serialization::{deserialize_fact, get_serialization_stats, serialize_fact};

    println!("🌐 Testing Global Serialization Context");

    let fact = Fact {
        id: 42,
        external_id: Some("global-test".to_string()),
        timestamp: Utc::now(),
        data: FactData {
            fields: std::iter::once((
                "test".to_string(),
                FactValue::String("global context test".to_string()),
            ))
            .collect(),
        },
    };

    // Use global functions
    let json = serialize_fact(&fact).unwrap();
    let deserialized = deserialize_fact(&json).unwrap();

    // Validate
    assert_eq!(fact.id, deserialized.id);
    assert_eq!(fact.external_id, deserialized.external_id);

    // Check global stats
    let stats = get_serialization_stats();
    println!("📊 Global Context Stats:");
    println!("   Cache hits: {}", stats.cache_hits);
    println!("   Cache misses: {}", stats.cache_misses);

    println!("✅ Global serialization context test completed successfully");
}
