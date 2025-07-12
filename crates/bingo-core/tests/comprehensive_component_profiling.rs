//! Comprehensive component profiling for bottleneck identification
//!
//! This test module systematically profiles all major components of the Bingo engine
//! to identify performance bottlenecks, validate optimization opportunities, and
//! generate detailed performance reports.

use bingo_core::{BingoEngine, types::*};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Instant;

/// Comprehensive profiling suite for all major components
#[test]
fn test_comprehensive_component_profiling() {
    println!("=== STARTING COMPREHENSIVE COMPONENT PROFILING ===");

    // Create engine with profiling enabled
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Enable detailed profiling
    engine.set_profiling_enabled(true);

    // Profile each major component systematically
    profile_fact_store_operations(&mut engine);
    profile_rule_compilation(&mut engine);
    profile_rete_network_processing(&mut engine);
    profile_aggregation_operations(&mut engine);
    profile_calculator_integration(&mut engine);
    profile_memory_management(&mut engine);

    // Generate comprehensive performance report
    generate_performance_analysis(&engine);
}

/// Profile fact store operations for bottlenecks
fn profile_fact_store_operations(engine: &mut BingoEngine) {
    println!("\n--- PROFILING FACT STORE OPERATIONS ---");

    let start_time = Instant::now();

    // Test 1: Large fact insertion performance
    let fact_count = 10000;
    println!("Testing fact insertion performance with {fact_count} facts");

    let insert_start = Instant::now();
    let facts = create_large_fact_dataset(fact_count);
    let creation_time = insert_start.elapsed();

    // Clear engine and time fact processing
    engine.clear();

    let process_start = Instant::now();
    for fact in &facts {
        // This adds to fact store internally
        let _ = engine.process_facts(vec![fact.clone()]);
    }
    let process_time = process_start.elapsed();

    let total_facts = engine.fact_count();

    println!("=== FACT STORE PERFORMANCE RESULTS ===");
    println!("Facts created: {fact_count}");
    println!("Facts stored: {total_facts}");
    println!("Creation time: {creation_time:?}");
    println!("Processing time: {process_time:?}");
    println!(
        "Insertion rate: {:.2} facts/sec",
        fact_count as f64 / process_time.as_secs_f64()
    );

    // Test 2: Fact lookup performance
    println!("\nTesting fact lookup performance...");
    let lookup_start = Instant::now();

    let mut lookup_count = 0;
    for i in 0..1000 {
        let external_id = format!("fact_{i}");
        if engine.lookup_fact_by_id(&external_id).is_some() {
            lookup_count += 1;
        }
    }

    let lookup_time = lookup_start.elapsed();
    println!("Fact lookups: {lookup_count} successful out of 1000");
    println!("Lookup time: {lookup_time:?}");
    println!(
        "Lookup rate: {:.2} lookups/sec",
        1000.0 / lookup_time.as_secs_f64()
    );

    let store_total_time = start_time.elapsed();
    println!("Total fact store profiling time: {store_total_time:?}");
}

/// Profile rule compilation performance
fn profile_rule_compilation(engine: &mut BingoEngine) {
    println!("\n--- PROFILING RULE COMPILATION ---");

    let start_time = Instant::now();

    // Test 1: Simple rule compilation
    println!("Testing simple rule compilation...");
    let simple_rules = create_simple_rules(100);

    let compile_start = Instant::now();
    for rule in simple_rules {
        engine.add_rule(rule).expect("Failed to add simple rule");
    }
    let simple_compile_time = compile_start.elapsed();

    println!("=== SIMPLE RULE COMPILATION RESULTS ===");
    println!("Rules compiled: 100");
    println!("Compilation time: {simple_compile_time:?}");
    println!(
        "Rules per second: {:.2}",
        100.0 / simple_compile_time.as_secs_f64()
    );

    // Test 2: Complex rule compilation
    println!("\nTesting complex rule compilation...");
    engine.clear();

    let complex_rules = create_complex_rules(50);
    let complex_compile_start = Instant::now();
    for rule in complex_rules {
        engine.add_rule(rule).expect("Failed to add complex rule");
    }
    let complex_compile_time = complex_compile_start.elapsed();

    println!("=== COMPLEX RULE COMPILATION RESULTS ===");
    println!("Complex rules compiled: 50");
    println!("Compilation time: {complex_compile_time:?}");
    println!(
        "Rules per second: {:.2}",
        50.0 / complex_compile_time.as_secs_f64()
    );

    // Test 3: Rule update performance
    println!("\nTesting rule update performance...");
    let update_start = Instant::now();

    for i in 0..10 {
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
    println!("Rule updates: 10");
    println!("Update time: {update_time:?}");
    println!(
        "Updates per second: {:.2}",
        10.0 / update_time.as_secs_f64()
    );

    let compilation_total_time = start_time.elapsed();
    println!("Total rule compilation profiling time: {compilation_total_time:?}");
}

/// Profile RETE network processing performance
fn profile_rete_network_processing(engine: &mut BingoEngine) {
    println!("\n--- PROFILING RETE NETWORK PROCESSING ---");

    let start_time = Instant::now();

    // Clear and setup for RETE testing
    engine.clear();

    // Add diverse rules for comprehensive RETE testing
    let rete_rules = create_rete_test_rules();
    for rule in rete_rules {
        engine.add_rule(rule).expect("Failed to add RETE test rule");
    }

    // Test 1: Pattern matching performance
    println!("Testing RETE pattern matching performance...");
    let facts = create_pattern_matching_facts(5000);

    let rete_start = Instant::now();
    let results = engine.process_facts(facts).expect("Failed to process facts through RETE");
    let rete_time = rete_start.elapsed();

    println!("=== RETE NETWORK PROCESSING RESULTS ===");
    println!("Facts processed: 5000");
    println!("Rules fired: {}", results.len());
    println!("Processing time: {rete_time:?}");
    println!(
        "Throughput: {:.2} facts/sec",
        5000.0 / rete_time.as_secs_f64()
    );

    // Test 2: Memory efficiency
    let stats = engine.get_stats();
    println!("\n=== RETE MEMORY USAGE ===");
    println!("Node count: {}", stats.node_count);
    println!("Memory usage: {} bytes", stats.memory_usage_bytes);
    println!("Rules loaded: {}", stats.rule_count);
    println!("Facts stored: {}", stats.fact_count);

    // Test 3: Action result pool efficiency
    let (pool_size, active_items, _, _) = engine.get_action_result_pool_stats();
    println!("\n=== ACTION RESULT POOL STATS ===");
    println!("Pool size: {pool_size}");
    println!("Active items: {active_items}");
    println!(
        "Pool utilization: {:.2}%",
        (active_items as f64 / pool_size.max(1) as f64) * 100.0
    );

    let rete_total_time = start_time.elapsed();
    println!("Total RETE network profiling time: {rete_total_time:?}");
}

/// Profile aggregation operations
fn profile_aggregation_operations(engine: &mut BingoEngine) {
    println!("\n--- PROFILING AGGREGATION OPERATIONS ---");

    let start_time = Instant::now();

    // Clear and setup for aggregation testing
    engine.clear();

    // Add aggregation rules
    let aggregation_rules = create_aggregation_rules();
    for rule in aggregation_rules {
        engine.add_rule(rule).expect("Failed to add aggregation rule");
    }

    // Test aggregation with various dataset sizes
    let dataset_sizes = [100, 500, 1000, 2000];

    for &size in &dataset_sizes {
        println!("\nTesting aggregation with {size} facts...");

        let facts = create_aggregation_facts(size);
        let agg_start = Instant::now();
        let results = engine.process_facts(facts).expect("Failed to process aggregation facts");
        let agg_time = agg_start.elapsed();

        println!("Dataset size: {size}");
        println!("Aggregation results: {}", results.len());
        println!("Processing time: {agg_time:?}");
        println!(
            "Throughput: {:.2} facts/sec",
            size as f64 / agg_time.as_secs_f64()
        );

        engine.clear();
        // Re-add rules for next iteration
        let aggregation_rules = create_aggregation_rules();
        for rule in aggregation_rules {
            engine.add_rule(rule).expect("Failed to re-add aggregation rule");
        }
    }

    // Test lazy aggregation statistics
    let lazy_stats = engine.get_lazy_aggregation_stats();
    println!("\n=== LAZY AGGREGATION STATS ===");
    println!("Aggregations created: {}", lazy_stats.aggregations_created);
    println!("Aggregations reused: {}", lazy_stats.aggregations_reused);
    println!("Cache invalidations: {}", lazy_stats.cache_invalidations);
    println!(
        "Total full computations: {}",
        lazy_stats.total_full_computations
    );
    println!(
        "Total early terminations: {}",
        lazy_stats.total_early_terminations
    );

    let aggregation_total_time = start_time.elapsed();
    println!("Total aggregation profiling time: {aggregation_total_time:?}");
}

/// Profile calculator integration performance  
fn profile_calculator_integration(engine: &mut BingoEngine) {
    println!("\n--- PROFILING CALCULATOR INTEGRATION ---");

    let start_time = Instant::now();

    // Clear and setup for calculator testing
    engine.clear();

    // Add calculator-based rules
    let calculator_rules = create_calculator_rules();
    for rule in calculator_rules {
        engine.add_rule(rule).expect("Failed to add calculator rule");
    }

    // Test calculator operations
    println!("Testing calculator integration performance...");
    let calc_facts = create_calculator_facts(1000);

    let calc_start = Instant::now();
    let calc_results =
        engine.process_facts(calc_facts).expect("Failed to process calculator facts");
    let calc_time = calc_start.elapsed();

    println!("=== CALCULATOR INTEGRATION RESULTS ===");
    println!("Facts processed: 1000");
    println!("Calculator results: {}", calc_results.len());
    println!("Processing time: {calc_time:?}");
    println!(
        "Throughput: {:.2} facts/sec",
        1000.0 / calc_time.as_secs_f64()
    );

    let calculator_total_time = start_time.elapsed();
    println!("Total calculator profiling time: {calculator_total_time:?}");
}

/// Profile memory management and pool efficiency
fn profile_memory_management(engine: &mut BingoEngine) {
    println!("\n--- PROFILING MEMORY MANAGEMENT ---");

    let start_time = Instant::now();

    // Test memory pool statistics
    let pool_stats = engine.get_memory_pool_stats();
    println!("=== MEMORY POOL STATISTICS ===");
    println!(
        "Rule execution result pool: {:?}",
        pool_stats.rule_execution_result_pool
    );
    println!("Rule ID vec pool: {:?}", pool_stats.rule_id_vec_pool);
    println!("Fact ID vec pool: {:?}", pool_stats.fact_id_vec_pool);
    println!("Fact field map pool: {:?}", pool_stats.fact_field_map_pool);
    println!("Numeric vec pool: {:?}", pool_stats.numeric_vec_pool);

    // Test memory efficiency
    let efficiency = engine.get_memory_pool_efficiency();
    println!("\n=== MEMORY EFFICIENCY ===");
    println!("Overall pool efficiency: {:.2}%", efficiency * 100.0);

    // Test serialization performance
    let serialization_stats = engine.get_serialization_stats();
    println!("\n=== SERIALIZATION PERFORMANCE ===");
    println!("Cache hits: {}", serialization_stats.cache_hits);
    println!("Cache misses: {}", serialization_stats.cache_misses);
    println!("Buffer hits: {}", serialization_stats.buffer_hits);
    println!("Buffer misses: {}", serialization_stats.buffer_misses);
    println!("Cache size: {}", serialization_stats.cache_size);
    println!("Buffer pool size: {}", serialization_stats.buffer_pool_size);

    let memory_total_time = start_time.elapsed();
    println!("Total memory profiling time: {memory_total_time:?}");
}

/// Generate comprehensive performance analysis and bottleneck report
fn generate_performance_analysis(engine: &BingoEngine) {
    println!("\n=== GENERATING COMPREHENSIVE PERFORMANCE ANALYSIS ===");

    // Get profiler reference and generate report
    let profiler = engine.profiler();
    let all_metrics = profiler.get_all_metrics();

    println!("\n--- OPERATION PERFORMANCE SUMMARY ---");
    for (i, metric) in all_metrics.iter().enumerate().take(10) {
        println!(
            "{}. {} ({}x invocations)",
            i + 1,
            metric.name,
            metric.invocations
        );
        println!(
            "   Avg: {}μs, Min: {}μs, Max: {}μs, P95: {}μs",
            metric.avg_duration_us,
            metric.min_duration_us,
            metric.max_duration_us,
            metric.p95_duration_us
        );
        println!(
            "   Total time: {}μs, Std dev: {}μs",
            metric.total_duration_us, metric.std_deviation_us
        );
    }

    // Analyze bottlenecks
    let bottlenecks = profiler.analyze_bottlenecks();
    println!("\n--- BOTTLENECK ANALYSIS ---");
    if bottlenecks.is_empty() {
        println!("✅ No significant bottlenecks detected!");
    } else {
        for (i, bottleneck) in bottlenecks.iter().enumerate().take(5) {
            println!(
                "{}. {} (Severity: {}/10)",
                i + 1,
                bottleneck.operation,
                bottleneck.severity
            );
            println!("   Description: {}", bottleneck.description);
            println!("   Suggestion: {}", bottleneck.suggestion);
            println!(
                "   Performance impact: {:.1}%",
                bottleneck.performance_impact
            );
        }
    }

    // Generate performance alerts
    let alerts = profiler.generate_alerts();
    println!("\n--- PERFORMANCE ALERTS ---");
    if alerts.is_empty() {
        println!("✅ No performance alerts generated!");
    } else {
        for alert in &alerts {
            println!(
                "🚨 {:?}: {} ({})",
                alert.severity, alert.message, alert.operation
            );
            println!(
                "   Actual: {:.2}ms, Threshold: {:.2}ms",
                alert.actual_value, alert.threshold_value
            );
        }
    }

    // Generate final unified statistics
    let unified_stats = bingo_core::unified_statistics::UnifiedStats::new();
    let report = profiler.generate_report(unified_stats);

    println!("\n--- OVERALL PERFORMANCE SCORE ---");
    println!("🏆 Performance Score: {:.1}/100", report.overall_score);

    if report.overall_score >= 90.0 {
        println!("✅ EXCELLENT: Engine performance is optimal");
    } else if report.overall_score >= 75.0 {
        println!("✅ GOOD: Engine performance is satisfactory");
    } else if report.overall_score >= 60.0 {
        println!("⚠️  FAIR: Engine performance needs attention");
    } else {
        println!("❌ POOR: Engine performance requires optimization");
    }

    // Export profiling data for analysis
    if let Ok(json_data) = profiler.export_json() {
        println!("\n--- PROFILING DATA EXPORT ---");
        println!("JSON export size: {} characters", json_data.len());

        // Optionally write to file for detailed analysis
        // std::fs::write("profiling_results.json", json_data).expect("Failed to write profiling data");
        // println!("Detailed profiling data written to: profiling_results.json");
    }

    println!("\n=== COMPREHENSIVE PROFILING COMPLETE ===");
}

// Helper functions for creating test data

fn create_large_fact_dataset(count: usize) -> Vec<Fact> {
    let mut facts = Vec::with_capacity(count);
    let categories = ["A", "B", "C", "D", "E"];
    let statuses = ["active", "inactive", "pending"];

    for i in 0..count {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FactValue::Integer(i as i64));
        fields.insert(
            "category".to_string(),
            FactValue::String(categories[i % categories.len()].to_string()),
        );
        fields.insert(
            "status".to_string(),
            FactValue::String(statuses[i % statuses.len()].to_string()),
        );
        fields.insert("amount".to_string(), FactValue::Float((i as f64) * 1.5));
        fields.insert("active".to_string(), FactValue::Boolean(i % 2 == 0));

        facts.push(Fact {
            id: i as u64,
            external_id: Some(format!("fact_{i}")),
            timestamp: Utc::now(),
            data: FactData { fields },
        });
    }

    facts
}

fn create_simple_rules(count: usize) -> Vec<Rule> {
    let mut rules = Vec::with_capacity(count);

    for i in 0..count {
        rules.push(Rule {
            id: i as u64,
            name: format!("Simple Rule {i}"),
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
        });
    }

    rules
}

fn create_complex_rules(count: usize) -> Vec<Rule> {
    let mut rules = Vec::with_capacity(count);

    for i in 0..count {
        rules.push(Rule {
            id: i as u64,
            name: format!("Complex Rule {i}"),
            conditions: vec![Condition::Complex {
                operator: LogicalOperator::And,
                conditions: vec![
                    Condition::Simple {
                        field: "amount".to_string(),
                        operator: Operator::GreaterThan,
                        value: FactValue::Float(100.0 + (i as f64)),
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
            actions: vec![
                Action {
                    action_type: ActionType::SetField {
                        field: "complex_processed".to_string(),
                        value: FactValue::Boolean(true),
                    },
                },
                Action {
                    action_type: ActionType::IncrementField {
                        field: "process_count".to_string(),
                        increment: FactValue::Integer(1),
                    },
                },
            ],
        });
    }

    rules
}

fn create_rete_test_rules() -> Vec<Rule> {
    vec![
        Rule {
            id: 1,
            name: "RETE Pattern Test".to_string(),
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
            name: "RETE Multi-Condition".to_string(),
            conditions: vec![
                Condition::Simple {
                    field: "value".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Float(50.0),
                },
                Condition::Simple {
                    field: "status".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("ready".to_string()),
                },
            ],
            actions: vec![Action {
                action_type: ActionType::Log { message: "RETE multi-condition fired".to_string() },
            }],
        },
    ]
}

fn create_pattern_matching_facts(count: usize) -> Vec<Fact> {
    let mut facts = Vec::with_capacity(count);

    for i in 0..count {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FactValue::Integer(i as i64));
        fields.insert("type".to_string(), FactValue::String("test".to_string()));
        fields.insert("value".to_string(), FactValue::Float((i % 100) as f64));
        fields.insert("status".to_string(), FactValue::String("ready".to_string()));

        facts.push(Fact {
            id: i as u64,
            external_id: Some(format!("pattern_{i}")),
            timestamp: Utc::now(),
            data: FactData { fields },
        });
    }

    facts
}

fn create_aggregation_rules() -> Vec<Rule> {
    vec![Rule {
        id: 1,
        name: "Sum Aggregation Test".to_string(),
        conditions: vec![Condition::Simple {
            field: "amount".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Float(50.0),
        }],
        actions: vec![Action {
            action_type: ActionType::Log { message: "High sum detected".to_string() },
        }],
    }]
}

fn create_aggregation_facts(count: usize) -> Vec<Fact> {
    let mut facts = Vec::with_capacity(count);
    let categories = ["cat1", "cat2", "cat3"];

    for i in 0..count {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FactValue::Integer(i as i64));
        fields.insert(
            "amount".to_string(),
            FactValue::Float((i % 100) as f64 + 10.0),
        );
        fields.insert(
            "category".to_string(),
            FactValue::String(categories[i % categories.len()].to_string()),
        );

        facts.push(Fact {
            id: i as u64,
            external_id: Some(format!("agg_{i}")),
            timestamp: Utc::now(),
            data: FactData { fields },
        });
    }

    facts
}

fn create_calculator_rules() -> Vec<Rule> {
    vec![Rule {
        id: 1,
        name: "Calculator Formula".to_string(),
        conditions: vec![Condition::Simple {
            field: "amount".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Float(0.0),
        }],
        actions: vec![Action {
            action_type: ActionType::Formula {
                expression: "amount * 1.1".to_string(),
                output_field: "adjusted_amount".to_string(),
            },
        }],
    }]
}

fn create_calculator_facts(count: usize) -> Vec<Fact> {
    let mut facts = Vec::with_capacity(count);

    for i in 0..count {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FactValue::Integer(i as i64));
        fields.insert("amount".to_string(), FactValue::Float((i as f64) * 2.5));

        facts.push(Fact {
            id: i as u64,
            external_id: Some(format!("calc_{i}")),
            timestamp: Utc::now(),
            data: FactData { fields },
        });
    }

    facts
}
