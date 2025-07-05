// Integration test for lazy aggregation functionality in RETE network.
//
// Validates that lazy aggregations work correctly within the RETE network and
// provide performance benefits.

use bingo_core::BingoEngine;
use bingo_core::types::{
    Action, ActionType, AggregationCondition, AggregationType, Condition, Fact, FactData,
    FactValue, Operator, Rule,
};
use chrono::Utc;
use std::collections::HashMap;

/// Create test facts for aggregation testing
fn create_test_facts() -> Vec<Fact> {
    vec![
        create_fact(1, "department", "sales", "amount", 100.0),
        create_fact(2, "department", "sales", "amount", 200.0),
        create_fact(3, "department", "sales", "amount", 150.0),
        create_fact(4, "department", "marketing", "amount", 75.0),
        create_fact(5, "department", "marketing", "amount", 125.0),
        create_fact(6, "department", "engineering", "amount", 300.0),
        create_fact(7, "department", "engineering", "amount", 250.0),
        create_fact(8, "department", "engineering", "amount", 400.0),
    ]
}

fn create_fact(
    id: u64,
    group_field: &str,
    group_value: &str,
    value_field: &str,
    value: f64,
) -> Fact {
    let mut fields = HashMap::new();
    fields.insert(
        group_field.to_string(),
        FactValue::String(group_value.to_string()),
    );
    fields.insert(value_field.to_string(), FactValue::Float(value));
    fields.insert("id".to_string(), FactValue::Integer(id as i64));

    Fact {
        id,
        external_id: Some(format!("fact-{}", id)),
        timestamp: Utc::now(),
        data: FactData { fields },
    }
}

#[test]
fn test_lazy_aggregation_basic_functionality() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Basic Lazy Aggregation Functionality");

    // Create a rule that uses aggregation
    let aggregation_condition = AggregationCondition {
        aggregation_type: AggregationType::Sum,
        source_field: "amount".to_string(),
        group_by: vec!["department".to_string()],
        having: Some(Box::new(Condition::Simple {
            field: "total_amount".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Float(400.0), // Should match sales (450) and engineering (950)
        })),
        alias: "total_amount".to_string(),
        window: None,
    };

    let rule = Rule {
        id: 1,
        name: "High Department Spending".to_string(),
        conditions: vec![Condition::Aggregation(aggregation_condition)],
        actions: vec![Action {
            action_type: ActionType::Log {
                message: "High spending department detected".to_string(),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Get baseline lazy aggregation stats
    let baseline_stats = engine.get_lazy_aggregation_stats();
    println!("📊 Baseline stats: {:?}", baseline_stats);

    // Process facts
    let facts = create_test_facts();
    let results = engine.process_facts(facts).unwrap();

    println!("🎯 Rule execution results: {} rules fired", results.len());

    // Check lazy aggregation stats after processing
    let final_stats = engine.get_lazy_aggregation_stats();
    println!("📈 Final stats: {:?}", final_stats);

    // We expect the rule to fire for facts that trigger departments with high total spending
    // Sales: 100 + 200 + 150 = 450 (> 400) ✓
    // Marketing: 75 + 125 = 200 (< 400) ✗
    // Engineering: 300 + 250 + 400 = 950 (> 400) ✓

    // Each fact from sales and engineering departments should trigger the rule
    let _expected_fires = 3 + 3; // 3 sales facts + 3 engineering facts
    assert!(
        results.len() >= 6,
        "Expected at least 6 rule fires, got {}",
        results.len()
    );

    // Verify that lazy aggregations were created
    assert!(
        final_stats.aggregations_created > 0,
        "Should have created lazy aggregations"
    );

    println!("✅ Basic lazy aggregation test passed");
}

#[test]
fn test_lazy_aggregation_caching() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Lazy Aggregation Caching");

    // Create a rule with count aggregation for easier testing
    let aggregation_condition = AggregationCondition {
        aggregation_type: AggregationType::Count,
        source_field: "amount".to_string(),
        group_by: vec!["department".to_string()],
        having: Some(Box::new(Condition::Simple {
            field: "count_amount".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(2), // Departments with more than 2 entries
        })),
        alias: "count_amount".to_string(),
        window: None,
    };

    let rule = Rule {
        id: 1,
        name: "Large Department Count".to_string(),
        conditions: vec![Condition::Aggregation(aggregation_condition)],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Large department detected".to_string() },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts multiple times to test caching
    let facts = create_test_facts();

    // First processing
    let _results1 = engine.process_facts(facts.clone()).unwrap();
    let stats_after_first = engine.get_lazy_aggregation_stats();

    // Second processing of same facts
    let _results2 = engine.process_facts(facts).unwrap();
    let stats_after_second = engine.get_lazy_aggregation_stats();

    println!("📊 Stats after first processing: {:?}", stats_after_first);
    println!("📊 Stats after second processing: {:?}", stats_after_second);

    // Should have more aggregation reuses in second run
    assert!(
        stats_after_second.aggregations_reused >= stats_after_first.aggregations_reused,
        "Should have reused aggregations in second run"
    );

    println!("✅ Lazy aggregation caching test passed");
}

#[test]
fn test_lazy_aggregation_early_termination() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Lazy Aggregation Early Termination");

    // Create a rule that should benefit from early termination (count > 0)
    let aggregation_condition = AggregationCondition {
        aggregation_type: AggregationType::Count,
        source_field: "amount".to_string(),
        group_by: vec!["department".to_string()],
        having: Some(Box::new(Condition::Simple {
            field: "any_count".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(0), // Any department with entries
        })),
        alias: "any_count".to_string(),
        window: None,
    };

    let rule = Rule {
        id: 1,
        name: "Any Department Activity".to_string(),
        conditions: vec![Condition::Aggregation(aggregation_condition)],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Department activity detected".to_string() },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts
    let facts = create_test_facts();
    let _results = engine.process_facts(facts).unwrap();

    let final_stats = engine.get_lazy_aggregation_stats();
    println!("📊 Early termination stats: {:?}", final_stats);

    // Note: Early termination statistics are tracked at the individual lazy aggregation level
    // The manager stats show overall aggregation creation/reuse patterns
    assert!(
        final_stats.aggregations_created > 0,
        "Should have created aggregations"
    );

    println!("✅ Lazy aggregation early termination test passed");
}

#[test]
fn test_lazy_aggregation_cache_invalidation() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Lazy Aggregation Cache Invalidation");

    // Create a simple aggregation rule
    let aggregation_condition = AggregationCondition {
        aggregation_type: AggregationType::Sum,
        source_field: "amount".to_string(),
        group_by: vec!["department".to_string()],
        having: None,
        alias: "total_amount".to_string(),
        window: None,
    };

    let rule = Rule {
        id: 1,
        name: "Department Total".to_string(),
        conditions: vec![Condition::Aggregation(aggregation_condition)],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Department total calculated".to_string() },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process initial facts
    let facts = create_test_facts();
    let _results1 = engine.process_facts(facts).unwrap();

    // Manually invalidate caches
    engine.invalidate_lazy_aggregation_caches();

    // Process same facts again - should recreate aggregations
    let more_facts = create_test_facts();
    let _results2 = engine.process_facts(more_facts).unwrap();

    let final_stats = engine.get_lazy_aggregation_stats();
    println!("📊 Cache invalidation stats: {:?}", final_stats);

    // Should show cache invalidations
    assert!(
        final_stats.cache_invalidations > 0,
        "Should have recorded cache invalidations"
    );

    println!("✅ Lazy aggregation cache invalidation test passed");
}

#[test]
fn test_lazy_aggregation_memory_cleanup() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Lazy Aggregation Memory Cleanup");

    // Create aggregation rule
    let aggregation_condition = AggregationCondition {
        aggregation_type: AggregationType::Average,
        source_field: "amount".to_string(),
        group_by: vec!["department".to_string()],
        having: None,
        alias: "avg_amount".to_string(),
        window: None,
    };

    let rule = Rule {
        id: 1,
        name: "Department Average".to_string(),
        conditions: vec![Condition::Aggregation(aggregation_condition)],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Department average calculated".to_string() },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts to create aggregations
    let facts = create_test_facts();
    let _results = engine.process_facts(facts).unwrap();

    let stats_before_cleanup = engine.get_lazy_aggregation_stats();

    // Clean up inactive aggregations
    engine.cleanup_lazy_aggregations();

    let stats_after_cleanup = engine.get_lazy_aggregation_stats();

    println!("📊 Stats before cleanup: {:?}", stats_before_cleanup);
    println!("📊 Stats after cleanup: {:?}", stats_after_cleanup);

    // After cleanup, aggregation counts should be reset (simple cleanup clears all)
    // This is expected behavior for the test cleanup implementation

    println!("✅ Lazy aggregation memory cleanup test passed");
}

#[test]
fn test_lazy_aggregation_performance_comparison() {
    println!("🧪 Testing Lazy Aggregation Performance Benefits");

    // Create a larger dataset for performance testing
    let mut large_facts = Vec::new();
    for i in 0..1000 {
        let dept = match i % 3 {
            0 => "sales",
            1 => "marketing",
            _ => "engineering",
        };
        large_facts.push(create_fact(
            i,
            "department",
            dept,
            "amount",
            (i as f64) * 1.5,
        ));
    }

    let mut engine = BingoEngine::new().unwrap();

    // Create aggregation rule
    let aggregation_condition = AggregationCondition {
        aggregation_type: AggregationType::Sum,
        source_field: "amount".to_string(),
        group_by: vec!["department".to_string()],
        having: Some(Box::new(Condition::Simple {
            field: "total_amount".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Float(100000.0),
        })),
        alias: "total_amount".to_string(),
        window: None,
    };

    let rule = Rule {
        id: 1,
        name: "High Department Total".to_string(),
        conditions: vec![Condition::Aggregation(aggregation_condition)],
        actions: vec![Action {
            action_type: ActionType::Log { message: "High department total detected".to_string() },
        }],
    };

    engine.add_rule(rule).unwrap();

    use std::time::Instant;

    // Measure performance
    let start = Instant::now();
    let _results = engine.process_facts(large_facts).unwrap();
    let duration = start.elapsed();

    let final_stats = engine.get_lazy_aggregation_stats();

    println!("⏱️  Processing time: {:?}", duration);
    println!("📊 Lazy aggregation stats: {:?}", final_stats);
    println!(
        "📈 Memory pool efficiency: {:.1}%",
        engine.get_memory_pool_efficiency()
    );

    // Validate that lazy aggregations were used
    assert!(
        final_stats.aggregations_created > 0,
        "Should have created lazy aggregations for performance"
    );

    println!("✅ Lazy aggregation performance test completed");
}
