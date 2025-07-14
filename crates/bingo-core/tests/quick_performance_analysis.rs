//! Quick performance analysis for major components
//!
//! This test provides a fast overview of component performance and bottlenecks.

use bingo_core::{BingoEngine, types::*};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_quick_performance_analysis() {
    println!("=== QUICK PERFORMANCE ANALYSIS ===");

    let mut engine = BingoEngine::new().expect("Failed to create engine");
    engine.set_profiling_enabled(true);

    // 1. Basic throughput tests
    test_fact_processing_throughput(&mut engine);
    test_rule_compilation_performance(&mut engine);
    test_rete_network_performance(&mut engine);

    // 2. Memory and resource analysis
    analyze_resource_usage(&engine);

    // 3. Simple bottleneck detection
    detect_simple_bottlenecks(&engine);

    println!("\n=== QUICK PERFORMANCE ANALYSIS COMPLETE ===");
}

fn test_fact_processing_throughput(engine: &mut BingoEngine) {
    println!("\n--- FACT PROCESSING THROUGHPUT ---");

    // Test small batch
    let facts = create_test_facts(1000);
    let start = Instant::now();
    for fact in facts {
        let _ = engine.process_facts(vec![fact]);
    }
    let time = start.elapsed();
    let throughput = 1000.0 / time.as_secs_f64();

    println!("1000 facts processed in {time:?} - {throughput:.0} facts/sec");

    if throughput < 50_000.0 {
        println!("⚠️  Fact processing throughput below expected threshold");
    } else {
        println!("✅ Fact processing performance: GOOD");
    }

    // Test lookup performance
    engine.clear();
    let facts = create_test_facts(5000);
    for fact in &facts[0..5000] {
        let _ = engine.process_facts(vec![fact.clone()]);
    }

    let lookup_start = Instant::now();
    let mut found = 0;
    for i in 0..100 {
        if engine.lookup_fact_by_id(&format!("test_{i}")).is_some() {
            found += 1;
        }
    }
    let lookup_time = lookup_start.elapsed();

    let lookup_rate = 100.0 / lookup_time.as_secs_f64();
    println!("Fact lookups: {lookup_rate:.0} lookups/sec (found {found}/100)");

    if lookup_rate < 10_000.0 {
        println!("⚠️  Fact lookup performance below threshold");
    } else {
        println!("✅ Fact lookup performance: GOOD");
    }
}

fn test_rule_compilation_performance(engine: &mut BingoEngine) {
    println!("\n--- RULE COMPILATION PERFORMANCE ---");

    engine.clear();

    // Test rule compilation rate
    let start = Instant::now();
    for i in 0..50 {
        let rule = Rule {
            id: i as u64,
            name: format!("Test Rule {i}"),
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

        engine.add_rule(rule).expect("Failed to add rule");
    }
    let compilation_time = start.elapsed();

    let rules_per_sec = 50.0 / compilation_time.as_secs_f64();
    println!("50 rules compiled in {compilation_time:?} - {rules_per_sec:.0} rules/sec");

    if rules_per_sec < 1_000.0 {
        println!("⚠️  Rule compilation rate below threshold");
    } else {
        println!("✅ Rule compilation performance: GOOD");
    }

    // Test rule update performance
    let update_start = Instant::now();
    for i in 0..5 {
        let updated_rule = Rule {
            id: i as u64,
            name: format!("Updated Rule {i}"),
            conditions: vec![Condition::Simple {
                field: "status".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("updated".to_string()),
            }],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "updated".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        };

        engine.update_rule(updated_rule).expect("Failed to update rule");
    }
    let update_time = update_start.elapsed();

    let updates_per_sec = 5.0 / update_time.as_secs_f64();
    println!("Rule updates: {updates_per_sec:.0} updates/sec");
}

fn test_rete_network_performance(engine: &mut BingoEngine) {
    println!("\n--- RETE NETWORK PERFORMANCE ---");

    engine.clear();

    // Add test rules
    let rules = vec![
        Rule {
            id: 1,
            name: "RETE Test Rule".to_string(),
            conditions: vec![Condition::Simple {
                field: "type".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("test".to_string()),
            }],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "rete_processed".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        },
        Rule {
            id: 2,
            name: "Multi-condition Rule".to_string(),
            conditions: vec![
                Condition::Simple {
                    field: "amount".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Float(50.0),
                },
                Condition::Simple {
                    field: "status".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("active".to_string()),
                },
            ],
            actions: vec![Action {
                action_type: ActionType::Log { message: "Multi-condition fired".to_string() },
            }],
        },
    ];

    for rule in rules {
        engine.add_rule(rule).expect("Failed to add rule");
    }

    // Test RETE processing performance
    let facts = create_rete_facts(2000);
    let start = Instant::now();
    let results = engine.process_facts(facts).expect("Failed to process facts");
    let processing_time = start.elapsed();

    let throughput = 2000.0 / processing_time.as_secs_f64();

    println!("2000 facts processed through RETE in {processing_time:?}");
    println!(
        "Rules fired: {} | Throughput: {:.0} facts/sec",
        results.len(),
        throughput
    );

    if throughput < 10_000.0 {
        println!("⚠️  RETE processing throughput below threshold");
    } else {
        println!("✅ RETE network performance: GOOD");
    }

    // Memory usage analysis
    let stats = engine.get_stats();
    println!(
        "RETE memory: {} bytes | Nodes: {}",
        stats.memory_usage_bytes, stats.node_count
    );

    if stats.memory_usage_bytes > 1_000_000 {
        // 1MB
        println!(
            "⚠️  RETE memory usage high: {} bytes",
            stats.memory_usage_bytes
        );
    }
}

fn analyze_resource_usage(engine: &BingoEngine) {
    println!("\n--- RESOURCE USAGE ANALYSIS ---");

    // Memory pool analysis
    let pool_stats = engine.get_memory_pool_stats();

    println!(
        "Memory Pool Efficiency: {:.1}%",
        engine.get_memory_pool_efficiency() * 100.0
    );

    // Rule execution result pool
    let rule_pool = &pool_stats.rule_execution_result_pool;
    if rule_pool.hits + rule_pool.misses > 0 {
        println!("Rule result pool hit rate: {:.1}%", rule_pool.hit_rate());
    }

    // Rule ID vec pool
    let rule_id_pool = &pool_stats.rule_id_vec_pool;
    if rule_id_pool.hits + rule_id_pool.misses > 0 {
        println!("Rule ID pool hit rate: {:.1}%", rule_id_pool.hit_rate());
    }

    // Serialization stats
    let ser_stats = engine.get_serialization_stats();
    let total_cache_ops = ser_stats.cache_hits + ser_stats.cache_misses;

    if total_cache_ops > 0 {
        let cache_hit_rate = (ser_stats.cache_hits as f64 / total_cache_ops as f64) * 100.0;
        println!("Serialization cache hit rate: {cache_hit_rate:.1}%");
    }

    // Lazy aggregation stats
    let lazy_stats = engine.get_lazy_aggregation_stats();
    println!("Aggregations created: {}", lazy_stats.aggregations_created);
    println!("Aggregations reused: {}", lazy_stats.aggregations_reused);

    if lazy_stats.aggregations_created > 0 {
        let reuse_rate = (lazy_stats.aggregations_reused as f64
            / lazy_stats.aggregations_created as f64)
            * 100.0;
        println!("Aggregation reuse rate: {reuse_rate:.1}%");
    }
}

fn detect_simple_bottlenecks(engine: &BingoEngine) {
    println!("\n--- BOTTLENECK DETECTION ---");

    // Get profiler metrics
    let profiler = engine.profiler();
    let metrics = profiler.get_all_metrics();

    if metrics.is_empty() {
        println!("No profiling data available");
        return;
    }

    println!("Top operations by total time:");
    for (i, metric) in metrics.iter().take(3).enumerate() {
        println!(
            "{}. {} - {}μs total ({} calls)",
            i + 1,
            metric.name,
            metric.total_duration_us,
            metric.invocations
        );

        if metric.avg_duration_us > 100_000 {
            // 100ms
            println!(
                "   🚨 HIGH LATENCY: Avg {}μs exceeds 100ms",
                metric.avg_duration_us
            );
        }

        if metric.std_deviation_us > 50_000 {
            // 50ms
            println!(
                "   ⚠️  HIGH VARIABILITY: StdDev {}μs",
                metric.std_deviation_us
            );
        }
    }

    // Check for bottlenecks using profiler analysis
    let bottlenecks = profiler.analyze_bottlenecks();

    if bottlenecks.is_empty() {
        println!("✅ No significant bottlenecks detected");
    } else {
        println!("\nBottlenecks identified:");
        for bottleneck in bottlenecks.iter().take(2) {
            println!(
                "- {}: {} (Impact: {:.1}%)",
                bottleneck.operation, bottleneck.description, bottleneck.performance_impact
            );
        }
    }

    // Generate alerts
    let alerts = profiler.generate_alerts();
    if !alerts.is_empty() {
        println!("\nPerformance alerts:");
        for alert in alerts.iter().take(2) {
            println!("- {:?}: {}", alert.severity, alert.message);
        }
    }
}

// Helper functions for creating test data

fn create_test_facts(count: usize) -> Vec<Fact> {
    let mut facts = Vec::with_capacity(count);

    for i in 0..count {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FactValue::Integer(i as i64));
        fields.insert(
            "status".to_string(),
            FactValue::String("active".to_string()),
        );
        fields.insert("amount".to_string(), FactValue::Float(i as f64));

        facts.push(Fact {
            id: i as u64,
            external_id: Some(format!("test_{i}")),
            timestamp: Utc::now(),
            data: FactData { fields },
        });
    }

    facts
}

fn create_rete_facts(count: usize) -> Vec<Fact> {
    let mut facts = Vec::with_capacity(count);

    for i in 0..count {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FactValue::Integer(i as i64));
        fields.insert("type".to_string(), FactValue::String("test".to_string()));
        fields.insert("amount".to_string(), FactValue::Float((i % 100) as f64));
        fields.insert(
            "status".to_string(),
            FactValue::String("active".to_string()),
        );

        facts.push(Fact {
            id: i as u64,
            external_id: Some(format!("rete_{i}")),
            timestamp: Utc::now(),
            data: FactData { fields },
        });
    }

    facts
}
