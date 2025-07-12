//! Simple profiling report for component performance
//!
//! This test provides basic performance metrics for major engine components.

use bingo_core::{BingoEngine, types::*};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_simple_profiling_report() {
    println!("=== SIMPLE PROFILING REPORT ===");

    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Run basic performance tests
    test_basic_performance(&mut engine);

    // Analyze current state
    analyze_engine_state(&engine);

    println!("\n=== PROFILING REPORT COMPLETE ===");
}

fn test_basic_performance(engine: &mut BingoEngine) {
    println!("\n--- BASIC PERFORMANCE TESTS ---");

    // Test 1: Fact processing throughput
    println!("Testing fact processing throughput...");
    let facts = create_test_facts(2000);

    let start = Instant::now();
    for fact in facts {
        let _ = engine.process_facts(vec![fact]);
    }
    let processing_time = start.elapsed();

    let throughput = 2000.0 / processing_time.as_secs_f64();
    println!("✅ Fact Processing: {throughput:.0} facts/sec ({processing_time:?})");

    // Test 2: Rule compilation
    println!("\nTesting rule compilation...");
    engine.clear();

    let start = Instant::now();
    for i in 0..100 {
        let rule = create_test_rule(i);
        engine.add_rule(rule).expect("Failed to add rule");
    }
    let compilation_time = start.elapsed();

    let rule_rate = 100.0 / compilation_time.as_secs_f64();
    println!("✅ Rule Compilation: {rule_rate:.0} rules/sec ({compilation_time:?})");

    // Test 3: RETE network processing
    println!("\nTesting RETE network processing...");
    let facts = create_test_facts(3000);

    let start = Instant::now();
    let results = engine.process_facts(facts).expect("Failed to process facts");
    let rete_time = start.elapsed();

    let rete_throughput = 3000.0 / rete_time.as_secs_f64();
    println!("✅ RETE Processing: {rete_throughput:.0} facts/sec ({rete_time:?})");
    println!("   Rules fired: {}", results.len());

    // Test 4: Fact lookup performance
    println!("\nTesting fact lookup performance...");
    let start = Instant::now();
    let mut found_count = 0;

    for i in 0..500 {
        if engine.lookup_fact_by_id(&format!("test_{i}")).is_some() {
            found_count += 1;
        }
    }
    let lookup_time = start.elapsed();

    let lookup_rate = 500.0 / lookup_time.as_secs_f64();
    println!("✅ Fact Lookup: {lookup_rate:.0} lookups/sec (found {found_count}/500)");
}

fn analyze_engine_state(engine: &BingoEngine) {
    println!("\n--- ENGINE STATE ANALYSIS ---");

    // Basic engine statistics
    let stats = engine.get_stats();
    println!("Engine Statistics:");
    println!("  Rules loaded: {}", stats.rule_count);
    println!("  Facts stored: {}", stats.fact_count);
    println!("  RETE nodes: {}", stats.node_count);
    println!("  Memory usage: {} bytes", stats.memory_usage_bytes);

    // Memory pool statistics
    println!("\nMemory Pool Analysis:");
    let pool_stats = engine.get_memory_pool_stats();

    let rule_pool = &pool_stats.rule_execution_result_pool;
    let total_rule_ops = rule_pool.hits + rule_pool.misses;
    if total_rule_ops > 0 {
        println!(
            "  Rule execution pool: {:.1}% hit rate ({} ops)",
            rule_pool.hit_rate, total_rule_ops
        );
    }

    let rule_id_pool = &pool_stats.rule_id_vec_pool;
    let total_id_ops = rule_id_pool.hits + rule_id_pool.misses;
    if total_id_ops > 0 {
        println!(
            "  Rule ID pool: {:.1}% hit rate ({} ops)",
            rule_id_pool.hit_rate, total_id_ops
        );
    }

    let efficiency = engine.get_memory_pool_efficiency();
    println!("  Overall efficiency: {:.1}%", efficiency * 100.0);

    // Lazy aggregation statistics
    println!("\nAggregation Analysis:");
    let lazy_stats = engine.get_lazy_aggregation_stats();
    println!(
        "  Aggregations created: {}",
        lazy_stats.aggregations_created
    );
    println!("  Aggregations reused: {}", lazy_stats.aggregations_reused);
    println!("  Cache invalidations: {}", lazy_stats.cache_invalidations);
    println!(
        "  Full computations: {}",
        lazy_stats.total_full_computations
    );
    println!(
        "  Early terminations: {}",
        lazy_stats.total_early_terminations
    );

    // Serialization statistics
    println!("\nSerialization Analysis:");
    let ser_stats = engine.get_serialization_stats();

    let total_cache_ops = ser_stats.cache_hits + ser_stats.cache_misses;
    if total_cache_ops > 0 {
        let cache_hit_rate = (ser_stats.cache_hits as f64 / total_cache_ops as f64) * 100.0;
        println!("  Cache hit rate: {cache_hit_rate:.1}% ({total_cache_ops} total ops)");
    } else {
        println!("  No serialization cache activity");
    }

    let total_buffer_ops = ser_stats.buffer_hits + ser_stats.buffer_misses;
    if total_buffer_ops > 0 {
        let buffer_hit_rate = (ser_stats.buffer_hits as f64 / total_buffer_ops as f64) * 100.0;
        println!("  Buffer hit rate: {buffer_hit_rate:.1}% ({total_buffer_ops} total ops)");
    } else {
        println!("  No buffer activity");
    }

    println!("  Cache size: {} entries", ser_stats.cache_size);
    println!("  Buffer pool size: {} entries", ser_stats.buffer_pool_size);

    // Performance assessment
    println!("\n--- PERFORMANCE ASSESSMENT ---");

    // Assess based on memory usage
    if stats.memory_usage_bytes > 10_000_000 {
        // 10MB
        println!(
            "⚠️  HIGH MEMORY: {} bytes (>10MB)",
            stats.memory_usage_bytes
        );
    } else if stats.memory_usage_bytes > 1_000_000 {
        // 1MB
        println!(
            "⚠️  MODERATE MEMORY: {} bytes (>1MB)",
            stats.memory_usage_bytes
        );
    } else {
        println!("✅ LOW MEMORY: {} bytes (<1MB)", stats.memory_usage_bytes);
    }

    // Assess memory pool efficiency
    if efficiency < 0.5 {
        // 50%
        println!("⚠️  LOW POOL EFFICIENCY: {:.1}%", efficiency * 100.0);
    } else if efficiency < 0.8 {
        // 80%
        println!("✅ MODERATE POOL EFFICIENCY: {:.1}%", efficiency * 100.0);
    } else {
        println!("✅ HIGH POOL EFFICIENCY: {:.1}%", efficiency * 100.0);
    }

    // Assess aggregation reuse
    if lazy_stats.aggregations_created > 0 {
        let reuse_rate =
            lazy_stats.aggregations_reused as f64 / lazy_stats.aggregations_created as f64;
        if reuse_rate < 0.3 {
            println!("⚠️  LOW AGGREGATION REUSE: {:.1}%", reuse_rate * 100.0);
        } else {
            println!("✅ GOOD AGGREGATION REUSE: {:.1}%", reuse_rate * 100.0);
        }
    }

    // Overall node efficiency
    let nodes_per_rule = if stats.rule_count > 0 {
        stats.node_count as f64 / stats.rule_count as f64
    } else {
        0.0
    };

    if nodes_per_rule > 10.0 {
        println!("⚠️  HIGH NODE DENSITY: {nodes_per_rule:.1} nodes/rule");
    } else {
        println!("✅ EFFICIENT NODE DENSITY: {nodes_per_rule:.1} nodes/rule");
    }

    println!("\n--- RECOMMENDATIONS ---");

    if stats.memory_usage_bytes > 5_000_000 {
        println!(
            "• Consider optimizing memory usage - current: {} bytes",
            stats.memory_usage_bytes
        );
    }

    if efficiency < 0.7 {
        println!(
            "• Memory pool efficiency could be improved: {:.1}%",
            efficiency * 100.0
        );
    }

    if nodes_per_rule > 8.0 {
        println!("• Consider rule optimization - high node density: {nodes_per_rule:.1}");
    }

    if total_rule_ops > 0 && rule_pool.hit_rate < 70.0 {
        println!(
            "• Rule execution pool hit rate low: {:.1}%",
            rule_pool.hit_rate
        );
    }

    if total_cache_ops > 0 && (ser_stats.cache_hits as f64 / total_cache_ops as f64) < 0.6 {
        println!("• Serialization cache hit rate could be improved");
    }

    println!("✅ Performance profiling completed successfully");
}

// Helper functions

fn create_test_facts(count: usize) -> Vec<Fact> {
    let mut facts = Vec::with_capacity(count);
    let statuses = ["active", "inactive", "pending"];
    let categories = ["A", "B", "C", "D"];

    for i in 0..count {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FactValue::Integer(i as i64));
        fields.insert(
            "status".to_string(),
            FactValue::String(statuses[i % statuses.len()].to_string()),
        );
        fields.insert(
            "category".to_string(),
            FactValue::String(categories[i % categories.len()].to_string()),
        );
        fields.insert("amount".to_string(), FactValue::Float(i as f64 * 1.5));
        fields.insert("active".to_string(), FactValue::Boolean(i % 2 == 0));

        facts.push(Fact {
            id: i as u64,
            external_id: Some(format!("test_{i}")),
            timestamp: Utc::now(),
            data: FactData { fields },
        });
    }

    facts
}

fn create_test_rule(id: usize) -> Rule {
    Rule {
        id: id as u64,
        name: format!("Test Rule {id}"),
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
    }
}
