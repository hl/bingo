//! Performance bottleneck analysis for major components
//!
//! This test systematically identifies performance bottlenecks in core engine components.

use bingo_core::{BingoEngine, types::*};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_component_performance_bottlenecks() {
    println!("=== PERFORMANCE BOTTLENECK ANALYSIS ===");

    let mut engine = BingoEngine::new().expect("Failed to create engine");
    engine.set_profiling_enabled(true);

    // 1. Fact Store Performance Analysis
    analyze_fact_store_bottlenecks(&mut engine);

    // 2. Rule Compilation Performance Analysis
    analyze_rule_compilation_bottlenecks(&mut engine);

    // 3. RETE Network Performance Analysis
    analyze_rete_network_bottlenecks(&mut engine);

    // 4. Memory Management Analysis
    analyze_memory_bottlenecks(&engine);

    // 5. Overall Performance Summary
    generate_bottleneck_summary(&engine);
}

fn analyze_fact_store_bottlenecks(engine: &mut BingoEngine) {
    println!("\n--- FACT STORE BOTTLENECK ANALYSIS ---");

    // Test 1: Large dataset insertion performance
    let fact_counts = [1000, 5000, 10000];

    for &count in &fact_counts {
        let facts = create_test_facts(count);

        let start = Instant::now();
        for fact in facts {
            let _ = engine.process_facts(vec![fact]);
        }
        let time = start.elapsed();

        let throughput = count as f64 / time.as_secs_f64();

        println!("Dataset size: {count} | Time: {time:?} | Throughput: {throughput:.0} facts/sec");

        // Identify bottleneck threshold
        if throughput < 50_000.0 {
            println!("🚨 BOTTLENECK: Fact insertion performance below 50k facts/sec");
        }

        engine.clear();
    }

    // Test 2: Fact lookup performance
    println!("\nFact lookup performance test...");
    let facts = create_test_facts(5000);
    for fact in facts {
        let _ = engine.process_facts(vec![fact]);
    }

    let lookup_start = Instant::now();
    let mut found_count = 0;
    for i in 0..1000 {
        if engine.lookup_fact_by_id(&format!("test_{i}")).is_some() {
            found_count += 1;
        }
    }
    let lookup_time = lookup_start.elapsed();

    let lookup_rate = 1000.0 / lookup_time.as_secs_f64();
    println!("Lookup rate: {lookup_rate:.0} lookups/sec | Found: {found_count}/1000");

    if lookup_rate < 100_000.0 {
        println!("🚨 BOTTLENECK: Fact lookup performance below 100k lookups/sec");
    }
}

fn analyze_rule_compilation_bottlenecks(engine: &mut BingoEngine) {
    println!("\n--- RULE COMPILATION BOTTLENECK ANALYSIS ---");

    engine.clear();

    // Test 1: Simple rule compilation scalability
    let rule_counts = [10, 50, 100, 200];

    for &count in &rule_counts {
        let start = Instant::now();

        for i in 0..count {
            let rule = Rule {
                id: i as u64,
                name: format!("Rule {i}"),
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
        let rules_per_sec = count as f64 / compilation_time.as_secs_f64();

        println!(
            "Rules: {count} | Time: {compilation_time:?} | Rate: {rules_per_sec:.0} rules/sec"
        );

        if rules_per_sec < 10_000.0 {
            println!("🚨 BOTTLENECK: Rule compilation below 10k rules/sec");
        }

        engine.clear();
    }

    // Test 2: Complex rule compilation
    println!("\nComplex rule compilation test...");
    let start = Instant::now();

    for i in 0..20 {
        let rule = Rule {
            id: i as u64,
            name: format!("Complex Rule {i}"),
            conditions: vec![Condition::Complex {
                operator: LogicalOperator::And,
                conditions: vec![
                    Condition::Simple {
                        field: "amount".to_string(),
                        operator: Operator::GreaterThan,
                        value: FactValue::Float(100.0),
                    },
                    Condition::Complex {
                        operator: LogicalOperator::Or,
                        conditions: vec![
                            Condition::Simple {
                                field: "category".to_string(),
                                operator: Operator::Equal,
                                value: FactValue::String("A".to_string()),
                            },
                            Condition::Simple {
                                field: "status".to_string(),
                                operator: Operator::Equal,
                                value: FactValue::String("active".to_string()),
                            },
                        ],
                    },
                ],
            }],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "complex_processed".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        };

        engine.add_rule(rule).expect("Failed to add complex rule");
    }

    let complex_time = start.elapsed();
    let complex_rate = 20.0 / complex_time.as_secs_f64();

    println!("Complex rules: 20 | Time: {complex_time:?} | Rate: {complex_rate:.0} rules/sec");

    if complex_rate < 1_000.0 {
        println!("🚨 BOTTLENECK: Complex rule compilation below 1k rules/sec");
    }
}

fn analyze_rete_network_bottlenecks(engine: &mut BingoEngine) {
    println!("\n--- RETE NETWORK BOTTLENECK ANALYSIS ---");

    engine.clear();

    // Setup test rules
    let test_rules = vec![
        Rule {
            id: 1,
            name: "Performance Test Rule".to_string(),
            conditions: vec![Condition::Simple {
                field: "type".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("test".to_string()),
            }],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "processed".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        },
        Rule {
            id: 2,
            name: "Multi-Condition Rule".to_string(),
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

    for rule in test_rules {
        engine.add_rule(rule).expect("Failed to add test rule");
    }

    // Test RETE processing with varying fact loads
    let fact_counts = [500, 2000, 5000, 10000];

    for &count in &fact_counts {
        let facts = create_rete_test_facts(count);

        let start = Instant::now();
        let results = engine.process_facts(facts).expect("Failed to process facts");
        let processing_time = start.elapsed();

        let throughput = count as f64 / processing_time.as_secs_f64();

        println!(
            "Facts: {} | Rules fired: {} | Time: {:?} | Throughput: {:.0} facts/sec",
            count,
            results.len(),
            processing_time,
            throughput
        );

        if throughput < 10_000.0 {
            println!("🚨 BOTTLENECK: RETE processing below 10k facts/sec");
        }

        engine.clear();
        // Re-add rules
        for rule in &[
            Rule {
                id: 1,
                name: "Performance Test Rule".to_string(),
                conditions: vec![Condition::Simple {
                    field: "type".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("test".to_string()),
                }],
                actions: vec![Action {
                    action_type: ActionType::SetField {
                        field: "processed".to_string(),
                        value: FactValue::Boolean(true),
                    },
                }],
            },
            Rule {
                id: 2,
                name: "Multi-Condition Rule".to_string(),
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
        ] {
            engine.add_rule(rule.clone()).expect("Failed to re-add rule");
        }
    }

    // Test memory usage
    let stats = engine.get_stats();
    println!("\nRete Network Memory Analysis:");
    println!(
        "Node count: {} | Memory usage: {} bytes",
        stats.node_count, stats.memory_usage_bytes
    );

    if stats.memory_usage_bytes > 10_000_000 {
        // 10MB
        println!("🚨 BOTTLENECK: RETE memory usage exceeds 10MB");
    }
}

fn analyze_memory_bottlenecks(engine: &BingoEngine) {
    println!("\n--- MEMORY BOTTLENECK ANALYSIS ---");

    // Memory pool analysis
    let pool_stats = engine.get_memory_pool_stats();
    println!("=== MEMORY POOL PERFORMANCE ===");

    // Rule execution result pool
    let rule_pool = &pool_stats.rule_execution_result_pool;
    println!(
        "Rule execution result pool: {} hits, {} misses, {:.1}% hit rate",
        rule_pool.hits, rule_pool.misses, rule_pool.hit_rate
    );

    if rule_pool.hit_rate < 80.0 {
        println!("🚨 BOTTLENECK: Rule execution result pool hit rate below 80%");
    }

    // Rule ID vec pool
    let rule_id_pool = &pool_stats.rule_id_vec_pool;
    println!(
        "Rule ID vec pool: {} hits, {} misses, {:.1}% hit rate",
        rule_id_pool.hits, rule_id_pool.misses, rule_id_pool.hit_rate
    );

    if rule_id_pool.hit_rate < 90.0 {
        println!("🚨 BOTTLENECK: Rule ID vec pool hit rate below 90%");
    }

    // Overall memory efficiency
    let efficiency = engine.get_memory_pool_efficiency();
    println!("Overall memory pool efficiency: {:.1}%", efficiency * 100.0);

    if efficiency < 0.7 {
        // 70%
        println!("🚨 BOTTLENECK: Memory pool efficiency below 70%");
    }

    // Serialization performance
    let ser_stats = engine.get_serialization_stats();
    println!("\n=== SERIALIZATION PERFORMANCE ===");

    let total_cache_operations = ser_stats.cache_hits + ser_stats.cache_misses;
    let cache_hit_rate = if total_cache_operations > 0 {
        (ser_stats.cache_hits as f64 / total_cache_operations as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "Cache operations: {} hits, {} misses, {:.1}% hit rate",
        ser_stats.cache_hits, ser_stats.cache_misses, cache_hit_rate
    );

    if total_cache_operations > 0 && cache_hit_rate < 70.0 {
        println!("🚨 BOTTLENECK: Serialization cache hit rate below 70%");
    }

    let buffer_operations = ser_stats.buffer_hits + ser_stats.buffer_misses;
    let buffer_hit_rate = if buffer_operations > 0 {
        (ser_stats.buffer_hits as f64 / buffer_operations as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "Buffer operations: {} hits, {} misses, {:.1}% hit rate",
        ser_stats.buffer_hits, ser_stats.buffer_misses, buffer_hit_rate
    );
}

fn generate_bottleneck_summary(engine: &BingoEngine) {
    println!("\n--- BOTTLENECK SUMMARY & RECOMMENDATIONS ---");

    // Get profiler metrics
    let profiler = engine.profiler();
    let metrics = profiler.get_all_metrics();

    if metrics.is_empty() {
        println!("No profiling data available");
        return;
    }

    println!("=== TOP PERFORMANCE OPERATIONS ===");
    for (i, metric) in metrics.iter().take(5).enumerate() {
        println!(
            "{}. {} - {} invocations",
            i + 1,
            metric.name,
            metric.invocations
        );
        println!(
            "   Avg: {}μs | Max: {}μs | Total: {}μs",
            metric.avg_duration_us, metric.max_duration_us, metric.total_duration_us
        );

        // Identify potential bottlenecks
        if metric.avg_duration_us > 100_000 {
            // 100ms
            println!("   🚨 HIGH LATENCY: Average execution exceeds 100ms");
        }

        if metric.std_deviation_us > 50_000 {
            // 50ms
            println!("   ⚠️  HIGH VARIABILITY: Inconsistent performance");
        }
    }

    // Analyze bottlenecks
    let bottlenecks = profiler.analyze_bottlenecks();
    if !bottlenecks.is_empty() {
        println!("\n=== IDENTIFIED BOTTLENECKS ===");
        for bottleneck in bottlenecks.iter().take(3) {
            println!(
                "Operation: {} (Severity: {}/10)",
                bottleneck.operation, bottleneck.severity
            );
            println!("Description: {}", bottleneck.description);
            println!("Suggestion: {}", bottleneck.suggestion);
            println!("Performance impact: {:.1}%", bottleneck.performance_impact);
            println!();
        }
    }

    // Generate performance alerts
    let alerts = profiler.generate_alerts();
    if !alerts.is_empty() {
        println!("=== PERFORMANCE ALERTS ===");
        for alert in &alerts {
            println!(
                "{:?}: {} ({})",
                alert.severity, alert.message, alert.operation
            );
            println!(
                "   Actual: {:.2}ms | Threshold: {:.2}ms",
                alert.actual_value, alert.threshold_value
            );
        }
    } else {
        println!("✅ No performance alerts generated - all operations within thresholds");
    }

    println!("\n=== OPTIMIZATION RECOMMENDATIONS ===");
    println!("1. Monitor operations with >100ms average latency");
    println!("2. Investigate high variability operations (>50ms std dev)");
    println!("3. Optimize memory pool hit rates to >90%");
    println!("4. Maintain RETE throughput >10k facts/sec");
    println!("5. Keep rule compilation rate >1k rules/sec");

    println!("\n=== PERFORMANCE BOTTLENECK ANALYSIS COMPLETE ===");
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
        fields.insert("amount".to_string(), FactValue::Float(i as f64 * 1.5));

        facts.push(Fact {
            id: i as u64,
            external_id: Some(format!("test_{i}")),
            timestamp: Utc::now(),
            data: FactData { fields },
        });
    }

    facts
}

fn create_rete_test_facts(count: usize) -> Vec<Fact> {
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
