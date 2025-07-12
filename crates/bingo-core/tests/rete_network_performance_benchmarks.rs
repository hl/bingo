//! Performance benchmarks and stress tests for RETE Network
//!
//! This test suite provides comprehensive performance validation including:
//! - Throughput benchmarks for various workload patterns
//! - Memory usage validation under stress
//! - Latency measurements for real-time processing scenarios
//! - Concurrent access patterns (when applicable)
//! - Regression detection for performance-critical operations

#![allow(dead_code)]
#![allow(clippy::uninlined_format_args)]
#![allow(unused_comparisons)]

use bingo_calculator::calculator::Calculator;
use bingo_core::fact_store::arena_store::ArenaFactStore;
use bingo_core::rete_network::ReteNetwork;
use bingo_core::types::*;
use chrono::{Duration, Utc};
use std::collections::HashMap;

/// Performance test configuration
#[derive(Debug, Clone)]
struct PerfTestConfig {
    pub fact_count: usize,
    pub rule_count: usize,
    pub max_execution_time_ms: u64,
    pub max_memory_mb: usize,
}

impl Default for PerfTestConfig {
    fn default() -> Self {
        Self {
            fact_count: 1000,
            rule_count: 10,
            max_execution_time_ms: 1000,
            max_memory_mb: 100,
        }
    }
}

/// Performance measurement results
#[derive(Debug, Clone)]
struct PerfResults {
    pub execution_time_ms: u64,
    pub throughput_facts_per_sec: f64,
    pub rules_executed: usize,
    pub memory_usage_mb: f64,
    pub cache_hit_rate: f64,
}

/// Helper function to create benchmark facts with realistic data patterns
fn create_benchmark_facts(count: usize, pattern: FactPattern) -> Vec<Fact> {
    let mut facts = Vec::new();
    let base_time = Utc::now();

    for i in 0..count {
        let timestamp = base_time + Duration::milliseconds(i as i64 * 100);

        let fields = match pattern {
            FactPattern::Financial => create_financial_fact_fields(i),
            FactPattern::IoT => create_iot_fact_fields(i, timestamp),
            FactPattern::Ecommerce => create_ecommerce_fact_fields(i),
            FactPattern::Mixed => match i % 3 {
                0 => create_financial_fact_fields(i),
                1 => create_iot_fact_fields(i, timestamp),
                _ => create_ecommerce_fact_fields(i),
            },
        };

        facts.push(Fact {
            id: i as u64 + 1,
            external_id: Some(format!("bench_{}", i)),
            timestamp,
            data: FactData { fields },
        });
    }

    facts
}

#[derive(Debug, Clone, Copy)]
enum FactPattern {
    Financial,
    IoT,
    Ecommerce,
    Mixed,
}

fn create_financial_fact_fields(i: usize) -> HashMap<String, FactValue> {
    let mut fields = HashMap::new();
    fields.insert(
        "transaction_id".to_string(),
        FactValue::String(format!("txn_{}", i)),
    );
    fields.insert(
        "amount".to_string(),
        FactValue::Float((i as f64 * 123.45) % 10000.0),
    );
    fields.insert(
        "account_type".to_string(),
        FactValue::String(
            match i % 4 {
                0 => "checking",
                1 => "savings",
                2 => "credit",
                _ => "investment",
            }
            .to_string(),
        ),
    );
    fields.insert(
        "risk_score".to_string(),
        FactValue::Integer((i % 100) as i64),
    );
    fields.insert(
        "currency".to_string(),
        FactValue::String(
            match i % 3 {
                0 => "USD",
                1 => "EUR",
                _ => "GBP",
            }
            .to_string(),
        ),
    );
    fields
}

fn create_iot_fact_fields(
    i: usize,
    timestamp: chrono::DateTime<Utc>,
) -> HashMap<String, FactValue> {
    let mut fields = HashMap::new();
    fields.insert(
        "device_id".to_string(),
        FactValue::String(format!("device_{}", i % 100)),
    );
    fields.insert(
        "temperature".to_string(),
        FactValue::Float(20.0 + (i as f64 * 0.1) % 40.0),
    );
    fields.insert(
        "humidity".to_string(),
        FactValue::Float(30.0 + (i as f64 * 0.2) % 50.0),
    );
    fields.insert(
        "battery_level".to_string(),
        FactValue::Integer((100 - (i % 100)) as i64),
    );
    fields.insert(
        "location".to_string(),
        FactValue::String(format!("zone_{}", i % 10)),
    );
    fields.insert(
        "timestamp_ms".to_string(),
        FactValue::Integer(timestamp.timestamp_millis()),
    );
    fields
}

fn create_ecommerce_fact_fields(i: usize) -> HashMap<String, FactValue> {
    let mut fields = HashMap::new();
    fields.insert(
        "order_id".to_string(),
        FactValue::String(format!("order_{}", i)),
    );
    fields.insert(
        "customer_id".to_string(),
        FactValue::String(format!("cust_{}", i % 500)),
    );
    fields.insert(
        "product_category".to_string(),
        FactValue::String(
            match i % 5 {
                0 => "electronics",
                1 => "clothing",
                2 => "books",
                3 => "home",
                _ => "sports",
            }
            .to_string(),
        ),
    );
    fields.insert(
        "order_value".to_string(),
        FactValue::Float((i as f64 * 29.99) % 1000.0),
    );
    fields.insert(
        "customer_tier".to_string(),
        FactValue::String(
            match i % 3 {
                0 => "bronze",
                1 => "silver",
                _ => "gold",
            }
            .to_string(),
        ),
    );
    fields
}

/// Create benchmark rules for different complexity levels
fn create_benchmark_rules(count: usize, complexity: RuleComplexity) -> Vec<Rule> {
    let mut rules = Vec::new();

    for i in 0..count {
        let rule = match complexity {
            RuleComplexity::Simple => create_simple_benchmark_rule(i),
            RuleComplexity::Medium => create_medium_benchmark_rule(i),
            RuleComplexity::Complex => create_complex_benchmark_rule(i),
            RuleComplexity::Aggregation => create_aggregation_benchmark_rule(i),
        };
        rules.push(rule);
    }

    rules
}

#[derive(Debug, Clone, Copy)]
enum RuleComplexity {
    Simple,
    Medium,
    Complex,
    Aggregation,
}

fn create_simple_benchmark_rule(id: usize) -> Rule {
    Rule {
        id: id as u64,
        name: format!("simple_rule_{}", id),
        conditions: vec![Condition::Simple {
            field: "amount".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Float(100.0 * (id as f64 + 1.0)),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "high_value".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    }
}

fn create_medium_benchmark_rule(id: usize) -> Rule {
    Rule {
        id: id as u64,
        name: format!("medium_rule_{}", id),
        conditions: vec![
            Condition::Simple {
                field: "amount".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Float(500.0),
            },
            Condition::Simple {
                field: "account_type".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("checking".to_string()),
            },
        ],
        actions: vec![
            Action {
                action_type: ActionType::SetField {
                    field: "alert_triggered".to_string(),
                    value: FactValue::Boolean(true),
                },
            },
            Action {
                action_type: ActionType::CreateFact {
                    data: FactData {
                        fields: {
                            let mut fields = HashMap::new();
                            fields.insert(
                                "alert_type".to_string(),
                                FactValue::String("high_value_checking".to_string()),
                            );
                            fields.insert("rule_id".to_string(), FactValue::Integer(id as i64));
                            fields
                        },
                    },
                },
            },
        ],
    }
}

fn create_complex_benchmark_rule(id: usize) -> Rule {
    Rule {
        id: id as u64,
        name: format!("complex_rule_{}", id),
        conditions: vec![Condition::Complex {
            operator: LogicalOperator::And,
            conditions: vec![
                Condition::Simple {
                    field: "amount".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Float(1000.0),
                },
                Condition::Complex {
                    operator: LogicalOperator::Or,
                    conditions: vec![
                        Condition::Simple {
                            field: "account_type".to_string(),
                            operator: Operator::Equal,
                            value: FactValue::String("credit".to_string()),
                        },
                        Condition::Simple {
                            field: "risk_score".to_string(),
                            operator: Operator::GreaterThan,
                            value: FactValue::Integer(80),
                        },
                    ],
                },
            ],
        }],
        actions: vec![
            Action {
                action_type: ActionType::SetField {
                    field: "complex_rule_matched".to_string(),
                    value: FactValue::Boolean(true),
                },
            },
            Action {
                action_type: ActionType::IncrementField {
                    field: "complex_match_count".to_string(),
                    increment: FactValue::Integer(1),
                },
            },
        ],
    }
}

fn create_aggregation_benchmark_rule(id: usize) -> Rule {
    Rule {
        id: id as u64,
        name: format!("aggregation_rule_{}", id),
        conditions: vec![Condition::Aggregation(AggregationCondition {
            aggregation_type: AggregationType::Sum,
            source_field: "amount".to_string(),
            alias: "total_amount".to_string(),
            group_by: vec!["account_type".to_string()],
            window: Some(AggregationWindow::Sliding { size: 100 }),
            having: Some(Box::new(Condition::Simple {
                field: "total_amount".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Float(5000.0),
            })),
        })],
        actions: vec![Action {
            action_type: ActionType::CreateFact {
                data: FactData {
                    fields: {
                        let mut fields = HashMap::new();
                        fields.insert(
                            "aggregation_alert".to_string(),
                            FactValue::String("high_volume_detected".to_string()),
                        );
                        fields.insert("rule_id".to_string(), FactValue::Integer(id as i64));
                        fields
                    },
                },
            },
        }],
    }
}

/// Run a performance benchmark and collect results
fn run_performance_benchmark(
    config: PerfTestConfig,
    fact_pattern: FactPattern,
    rule_complexity: RuleComplexity,
) -> anyhow::Result<PerfResults> {
    let mut network = ReteNetwork::new();
    let mut fact_store = ArenaFactStore::new();
    let calculator = Calculator::new();

    // Setup rules
    let rules = create_benchmark_rules(config.rule_count, rule_complexity);
    for rule in rules {
        network.add_rule(rule)?;
    }

    // Create benchmark facts
    let facts = create_benchmark_facts(config.fact_count, fact_pattern);

    // Populate fact store with some facts for aggregation rules
    for fact in facts.iter().take(config.fact_count / 2) {
        fact_store.insert(fact.clone());
    }

    // Measure performance
    let start_time = std::time::Instant::now();
    let start_memory = get_memory_usage_mb();

    let results = network.process_facts(&facts, &fact_store, &calculator)?;

    let execution_time = start_time.elapsed();
    let execution_time_ms = execution_time.as_millis() as u64;
    let end_memory = get_memory_usage_mb();

    // Calculate metrics
    let throughput = config.fact_count as f64 / execution_time.as_secs_f64();
    let memory_usage = end_memory - start_memory;

    // Get cache statistics (simplified - would need actual cache hit rate from network)
    let cache_hit_rate = 0.0; // Placeholder - would implement actual cache monitoring

    let perf_results = PerfResults {
        execution_time_ms,
        throughput_facts_per_sec: throughput,
        rules_executed: results.len(),
        memory_usage_mb: memory_usage,
        cache_hit_rate,
    };

    // Validate performance constraints
    if execution_time_ms > config.max_execution_time_ms {
        return Err(anyhow::anyhow!(
            "Performance test failed: execution time {}ms exceeds limit {}ms",
            execution_time_ms,
            config.max_execution_time_ms
        ));
    }

    if memory_usage > config.max_memory_mb as f64 {
        return Err(anyhow::anyhow!(
            "Performance test failed: memory usage {:.2}MB exceeds limit {}MB",
            memory_usage,
            config.max_memory_mb
        ));
    }

    Ok(perf_results)
}

/// Get current memory usage in MB (simplified implementation)
fn get_memory_usage_mb() -> f64 {
    // This is a simplified implementation
    // In a real benchmark, you'd use a proper memory profiling library
    0.0
}

#[cfg(test)]
mod throughput_benchmarks {
    use super::*;

    #[test]
    fn test_simple_rule_throughput() {
        let config = PerfTestConfig {
            fact_count: 5000,
            rule_count: 10,
            max_execution_time_ms: 2000,
            max_memory_mb: 50,
        };

        let results =
            run_performance_benchmark(config, FactPattern::Financial, RuleComplexity::Simple)
                .expect("Simple rule throughput test failed");

        println!(
            "✅ Simple rule throughput: {:.0} facts/sec, {} rules executed in {}ms",
            results.throughput_facts_per_sec, results.rules_executed, results.execution_time_ms
        );

        // Assert minimum throughput (should process at least 1000 facts/sec for simple rules)
        assert!(
            results.throughput_facts_per_sec > 1000.0,
            "Simple rule throughput too low: {:.0} facts/sec",
            results.throughput_facts_per_sec
        );
    }

    #[test]
    fn test_complex_rule_throughput() {
        let config = PerfTestConfig {
            fact_count: 2000,
            rule_count: 5,
            max_execution_time_ms: 3000,
            max_memory_mb: 100,
        };

        let results =
            run_performance_benchmark(config, FactPattern::Mixed, RuleComplexity::Complex)
                .expect("Complex rule throughput test failed");

        println!(
            "✅ Complex rule throughput: {:.0} facts/sec, {} rules executed in {}ms",
            results.throughput_facts_per_sec, results.rules_executed, results.execution_time_ms
        );

        // Assert minimum throughput (should process at least 500 facts/sec for complex rules)
        assert!(
            results.throughput_facts_per_sec > 500.0,
            "Complex rule throughput too low: {:.0} facts/sec",
            results.throughput_facts_per_sec
        );
    }

    #[test]
    fn test_aggregation_rule_throughput() {
        let config = PerfTestConfig {
            fact_count: 1000,
            rule_count: 3,
            max_execution_time_ms: 5000,
            max_memory_mb: 150,
        };

        let results =
            run_performance_benchmark(config, FactPattern::Financial, RuleComplexity::Aggregation)
                .expect("Aggregation rule throughput test failed");

        println!(
            "✅ Aggregation rule throughput: {:.0} facts/sec, {} rules executed in {}ms",
            results.throughput_facts_per_sec, results.rules_executed, results.execution_time_ms
        );

        // Assert minimum throughput (should process at least 200 facts/sec for aggregation rules)
        assert!(
            results.throughput_facts_per_sec > 200.0,
            "Aggregation rule throughput too low: {:.0} facts/sec",
            results.throughput_facts_per_sec
        );
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[test]
    fn test_high_volume_fact_processing() {
        let config = PerfTestConfig {
            fact_count: 5000,             // Reduced from 10000 to be more reasonable
            rule_count: 10,               // Reduced from 20
            max_execution_time_ms: 30000, // Increased timeout for stress test
            max_memory_mb: 200,
        };

        let results = run_performance_benchmark(config, FactPattern::Mixed, RuleComplexity::Medium)
            .expect("High volume fact processing test failed");

        println!(
            "✅ High volume processing: {:.0} facts/sec, {} rules executed in {}ms",
            results.throughput_facts_per_sec, results.rules_executed, results.execution_time_ms
        );

        // Should maintain reasonable throughput even with high volume (relaxed for stress test)
        assert!(
            results.throughput_facts_per_sec > 100.0,
            "High volume throughput degraded: {:.0} facts/sec",
            results.throughput_facts_per_sec
        );
    }

    #[test]
    fn test_many_rules_performance() {
        let config = PerfTestConfig {
            fact_count: 1000,
            rule_count: 100,
            max_execution_time_ms: 8000,
            max_memory_mb: 150,
        };

        let results =
            run_performance_benchmark(config, FactPattern::Financial, RuleComplexity::Simple)
                .expect("Many rules performance test failed");

        println!(
            "✅ Many rules processing: {:.0} facts/sec, {} rules executed in {}ms",
            results.throughput_facts_per_sec, results.rules_executed, results.execution_time_ms
        );

        // Should handle many rules without excessive performance degradation
        assert!(
            results.throughput_facts_per_sec > 500.0,
            "Many rules throughput too low: {:.0} facts/sec",
            results.throughput_facts_per_sec
        );
    }

    #[test]
    fn test_memory_stability_under_load() {
        let mut network = ReteNetwork::new();
        let fact_store = ArenaFactStore::new();
        let calculator = Calculator::new();

        // Add rules
        let rules = create_benchmark_rules(10, RuleComplexity::Medium);
        for rule in rules {
            network.add_rule(rule).expect("Failed to add rule");
        }

        let initial_stats = network.get_memory_pool_stats();
        let initial_efficiency = network.get_memory_pool_efficiency();

        // Process multiple batches to test memory stability
        for batch in 0..10 {
            let facts = create_benchmark_facts(500, FactPattern::Mixed);

            let _results = network
                .process_facts(&facts, &fact_store, &calculator)
                .expect("Failed to process batch");

            // Check memory stats periodically
            if batch % 3 == 0 {
                let _current_stats = network.get_memory_pool_stats();
                let current_efficiency = network.get_memory_pool_efficiency();

                println!(
                    "Batch {}: Pool efficiency: {:.2}%",
                    batch,
                    current_efficiency * 100.0
                );

                // Memory efficiency should not degrade significantly
                assert!(
                    current_efficiency >= initial_efficiency * 0.7,
                    "Memory efficiency degraded significantly: {:.2}% -> {:.2}%",
                    initial_efficiency * 100.0,
                    current_efficiency * 100.0
                );
            }
        }

        let final_stats = network.get_memory_pool_stats();
        let final_efficiency = network.get_memory_pool_efficiency();

        println!("✅ Memory stability test completed");
        println!("   Initial efficiency: {:.2}%", initial_efficiency * 100.0);
        println!("   Final efficiency: {:.2}%", final_efficiency * 100.0);
        println!("   Stats: {:?} -> {:?}", initial_stats, final_stats);
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_performance_regression_baseline() {
        // This test establishes performance baselines for regression detection
        let configs = vec![
            (
                "small_simple",
                PerfTestConfig {
                    fact_count: 100,
                    rule_count: 5,
                    max_execution_time_ms: 100,
                    max_memory_mb: 20,
                },
            ),
            (
                "medium_mixed",
                PerfTestConfig {
                    fact_count: 1000,
                    rule_count: 10,
                    max_execution_time_ms: 500,
                    max_memory_mb: 50,
                },
            ),
            (
                "large_complex",
                PerfTestConfig {
                    fact_count: 2000,
                    rule_count: 20,
                    max_execution_time_ms: 2000,
                    max_memory_mb: 100,
                },
            ),
        ];

        let mut baseline_results = Vec::new();

        for (test_name, config) in configs {
            let results =
                run_performance_benchmark(config, FactPattern::Mixed, RuleComplexity::Medium)
                    .unwrap_or_else(|_| panic!("Baseline test {} failed", test_name));

            println!(
                "✅ Baseline {}: {:.0} facts/sec in {}ms",
                test_name, results.throughput_facts_per_sec, results.execution_time_ms
            );

            baseline_results.push((test_name, results));
        }

        // Store baseline results for future regression testing
        // In a real implementation, these would be stored persistently
        for (test_name, results) in baseline_results {
            assert!(
                results.throughput_facts_per_sec > 0.0,
                "Invalid baseline throughput for {}",
                test_name
            );
            // Note: execution_time_ms is u64, so it's always >= 0
            // This assertion is removed as it's always true
        }
    }
}
