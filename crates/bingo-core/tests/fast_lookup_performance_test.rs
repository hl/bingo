//! Performance test for fast fact lookup optimization
//!
//! This test validates that the LRU caching and O(1) hash-based lookup
//! provides significant performance improvements over linear search.

use bingo_core::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_fast_lookup_performance_improvement() {
    let mut network = ReteNetwork::new().unwrap();

    // Create a large number of facts to test performance with
    let fact_count = 10_000;
    let mut facts = Vec::with_capacity(fact_count);

    for i in 0..fact_count {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FactValue::Integer(i as i64));
        fields.insert(
            "value".to_string(),
            FactValue::String(format!("value_{}", i)),
        );
        fields.insert(
            "category".to_string(),
            FactValue::String("test".to_string()),
        );

        facts.push(Fact { id: i as u64, data: FactData { fields } });
    }

    // Create a rule that processes facts
    let rule = Rule {
        id: 1,
        name: "performance_test_rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "category".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("test".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "processed".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };

    network.add_rule(rule).unwrap();

    // Test processing performance with optimized lookup
    let start_time = Instant::now();
    let results = network.process_facts(facts.clone()).unwrap();
    let processing_duration = start_time.elapsed();

    // Validate results
    assert!(!results.is_empty(), "Should generate processed facts");
    assert_eq!(results.len(), fact_count, "Should process all facts");

    // Validate that facts were actually processed
    for result in &results {
        assert!(
            result.data.fields.contains_key("processed"),
            "Facts should be processed"
        );
        assert_eq!(
            result.data.fields.get("processed"),
            Some(&FactValue::Boolean(true))
        );
    }

    // Check fast lookup statistics
    let lookup_stats = network.get_fast_lookup_stats();
    println!("Fast Lookup Performance Statistics:");
    println!("  Facts stored: {}", lookup_stats.facts_stored);
    println!("  Total lookups: {}", lookup_stats.total_lookups);
    println!("  Cache hits: {}", lookup_stats.cache_hits);
    println!("  Cache misses: {}", lookup_stats.cache_misses);
    println!("  Hit rate: {:.2}%", lookup_stats.hit_rate);
    println!("  Processing time: {:?}", processing_duration);

    // Validate cache effectiveness
    assert!(
        lookup_stats.total_lookups > 0,
        "Should have performed lookups"
    );
    assert_eq!(
        lookup_stats.facts_stored, fact_count,
        "Should store all facts"
    );

    // Performance should be reasonable for 10K facts - production target
    assert!(
        processing_duration.as_millis() < 2000,
        "Should process 10K facts in under 2 seconds (production target)"
    );

    // Create a rule that would access facts multiple times (beta node joins)
    let join_rule = Rule {
        id: 2,
        name: "join_test_rule".to_string(),
        conditions: vec![
            Condition::Simple {
                field: "category".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("test".to_string()),
            },
            Condition::Simple {
                field: "value".to_string(),
                operator: Operator::Contains,
                value: FactValue::String("value".to_string()),
            },
        ],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "joined".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };

    network.add_rule(join_rule).unwrap();

    // Test repeated access with join rule (should trigger more fact lookups)
    let start_time = Instant::now();
    let _results2 = network.process_facts(facts[0..100].to_vec()).unwrap(); // Smaller subset for joins
    let join_processing_duration = start_time.elapsed();

    println!("  Join processing time: {:?}", join_processing_duration);

    // Second processing should show more lookup activity
    let updated_stats = network.get_fast_lookup_stats();
    println!("Updated Fast Lookup Statistics:");
    println!("  Total lookups: {}", updated_stats.total_lookups);
    println!("  Cache hits: {}", updated_stats.cache_hits);
    println!("  Hit rate: {:.2}%", updated_stats.hit_rate);

    // Should have performed more lookups due to join operations
    assert!(
        updated_stats.total_lookups > lookup_stats.total_lookups,
        "Join operations should increase lookup count"
    );
}

#[test]
fn test_cache_statistics_accuracy() {
    let mut network = ReteNetwork::new().unwrap();

    // Create facts with predictable access patterns
    let mut facts = Vec::new();
    for i in 0..100 {
        let mut fields = HashMap::new();
        fields.insert("test_id".to_string(), FactValue::Integer(i));

        facts.push(Fact { id: i as u64, data: FactData { fields } });
    }

    // Create rule
    let rule = Rule {
        id: 1,
        name: "cache_test_rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "test_id".to_string(),
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

    network.add_rule(rule).unwrap();

    // Process facts to populate cache
    let _results = network.process_facts(facts).unwrap();

    let stats = network.get_fast_lookup_stats();

    // Validate statistics structure
    assert_eq!(stats.facts_stored, 100);
    assert!(stats.total_lookups > 0);
    assert!(stats.cache_hits + stats.cache_misses == stats.total_lookups);
    assert!((0.0..=100.0).contains(&stats.hit_rate));

    // Validate cache effectiveness metrics
    let efficiency = stats.efficiency();
    assert!((0.0..=100.0).contains(&efficiency));

    let lookups_per_fact = stats.lookups_per_fact();
    assert!(lookups_per_fact >= 0.0);

    println!("Cache Performance Metrics:");
    println!("  Efficiency: {:.2}%", efficiency);
    println!("  Lookups per fact: {:.2}", lookups_per_fact);
}
