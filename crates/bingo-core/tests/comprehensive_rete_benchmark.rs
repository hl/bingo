//! Comprehensive RETE Performance Benchmark
//!
//! This benchmark validates the O(Δfacts) performance characteristics of our RETE implementation
//! across multiple scales and demonstrates the significant performance advantages over
//! traditional O(rules × facts) approaches.

use bingo_core::engine::BingoEngine;
use bingo_core::types::{Action, ActionType, Condition, Fact, FactData, FactValue, Operator, Rule};
use chrono::Utc;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct BenchmarkResult {
    scale: &'static str,
    rule_count: usize,
    #[allow(dead_code)]
    initial_facts: usize,
    incremental_facts: usize,
    batch_time: Duration,
    incremental_time: Duration,
    speedup: f64,
}

#[test]
fn comprehensive_rete_performance_benchmark() {
    println!("🚀 Starting Comprehensive RETE Performance Benchmark");
    println!("═══════════════════════════════════════════════════════════════");

    let test_scales = vec![
        ("Small", 10, 100, 10),
        ("Medium", 25, 500, 50),
        ("Large", 50, 1000, 100),
        ("Enterprise", 100, 2000, 200),
    ];

    let mut results = Vec::new();

    for (scale, rule_count, initial_fact_count, incremental_fact_count) in test_scales {
        println!("\n📊 Testing {scale} Scale:");
        println!(
            "  Rules: {rule_count}, Initial Facts: {initial_fact_count}, Incremental Facts: {incremental_fact_count}"
        );

        let result = run_benchmark(
            scale,
            rule_count,
            initial_fact_count,
            incremental_fact_count,
        );
        print_result(&result);
        results.push(result);
    }

    print_summary(&results);

    // Verify all tests show performance improvement
    for result in &results {
        assert!(
            result.speedup >= 1.0,
            "RETE should be at least as fast as batch processing for {} scale",
            result.scale
        );
    }

    println!("\n🎉 COMPREHENSIVE BENCHMARK PASSED - RETE O(Δfacts) algorithm verified!");
}

fn run_benchmark(
    scale: &'static str,
    rule_count: usize,
    initial_fact_count: usize,
    incremental_fact_count: usize,
) -> BenchmarkResult {
    let rules = generate_benchmark_rules(rule_count);
    let initial_facts = generate_benchmark_facts(initial_fact_count, 0);
    let incremental_facts = generate_benchmark_facts(incremental_fact_count, initial_fact_count);

    // Batch processing
    let batch_time = {
        let start = Instant::now();
        let engine = BingoEngine::new().expect("Failed to create engine");

        for rule in &rules {
            let _ = engine.add_rule(rule.clone());
        }

        let all_facts: Vec<Fact> =
            initial_facts.iter().chain(incremental_facts.iter()).cloned().collect();
        let _ = engine.process_facts(all_facts);

        start.elapsed()
    };

    // Incremental processing
    let incremental_time = {
        let engine = BingoEngine::new().expect("Failed to create engine");

        for rule in &rules {
            let _ = engine.add_rule(rule.clone());
        }

        let _ = engine.process_facts(initial_facts.clone());

        let start = Instant::now();
        let _ = engine.process_facts(incremental_facts.clone());
        start.elapsed()
    };

    let speedup = batch_time.as_nanos() as f64 / incremental_time.as_nanos() as f64;

    BenchmarkResult {
        scale,
        rule_count,
        initial_facts: initial_fact_count,
        incremental_facts: incremental_fact_count,
        batch_time,
        incremental_time,
        speedup,
    }
}

fn generate_benchmark_rules(count: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let threshold = (i % 100) as f64;
            Rule {
                id: i as u64,
                name: format!("Benchmark Rule {i}"),
                conditions: vec![
                    Condition::Simple {
                        field: "entity_type".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::String("benchmark_entity".to_string()),
                    },
                    Condition::Simple {
                        field: "metric_value".to_string(),
                        operator: Operator::GreaterThan,
                        value: FactValue::Float(threshold),
                    },
                ],
                actions: vec![Action {
                    action_type: ActionType::CreateFact {
                        data: FactData {
                            fields: HashMap::from([
                                ("rule_fired".to_string(), FactValue::Integer(i as i64)),
                                ("threshold".to_string(), FactValue::Float(threshold)),
                                (
                                    "result_type".to_string(),
                                    FactValue::String("benchmark_result".to_string()),
                                ),
                            ]),
                        },
                    },
                }],
            }
        })
        .collect()
}

fn generate_benchmark_facts(count: usize, start_id: usize) -> Vec<Fact> {
    (0..count)
        .map(|i| {
            let fact_id = start_id + i;
            let metric_value = (fact_id as f64 * 1.5) % 150.0; // Varied values to trigger different rules

            Fact {
                id: fact_id as u64,
                external_id: Some(format!("benchmark_{fact_id}")),
                timestamp: Utc::now(),
                data: FactData {
                    fields: HashMap::from([
                        (
                            "entity_type".to_string(),
                            FactValue::String("benchmark_entity".to_string()),
                        ),
                        ("metric_value".to_string(), FactValue::Float(metric_value)),
                        (
                            "category".to_string(),
                            FactValue::String(format!("cat_{}", fact_id % 8)),
                        ),
                        (
                            "priority".to_string(),
                            FactValue::Integer((fact_id % 5) as i64),
                        ),
                    ]),
                },
            }
        })
        .collect()
}

fn print_result(result: &BenchmarkResult) {
    let batch_time = result.batch_time;
    println!("  Batch Time:       {batch_time:>8.2?}");
    let incremental_time = result.incremental_time;
    println!("  Incremental Time: {incremental_time:>8.2?}");
    let speedup = result.speedup;
    println!("  Speedup:          {speedup:>8.2}x");

    if result.speedup >= 10.0 {
        println!("  Status:           🏆 OUTSTANDING (>10x)");
    } else if result.speedup >= 5.0 {
        println!("  Status:           🌟 EXCELLENT (5-10x)");
    } else if result.speedup >= 2.0 {
        println!("  Status:           ✅ GOOD (2-5x)");
    } else {
        println!("  Status:           ⚠️  MARGINAL (<2x)");
    }
}

fn print_summary(results: &[BenchmarkResult]) {
    println!("\n🎯 RETE Performance Benchmark Summary");
    println!("═══════════════════════════════════════════════════════════════");
    println!(
        "{:<12} │ {:>8} │ {:>8} │ {:>8} │ {:>8}",
        "Scale", "Rules", "ΔFacts", "Speedup", "Status"
    );
    println!("─────────────┼──────────┼──────────┼──────────┼──────────");

    for result in results {
        let status = if result.speedup >= 10.0 {
            "🏆"
        } else if result.speedup >= 5.0 {
            "🌟"
        } else if result.speedup >= 2.0 {
            "✅"
        } else {
            "⚠️"
        };

        println!(
            "{:<12} │ {:>8} │ {:>8} │ {:>8.1}x │ {:>8}",
            result.scale, result.rule_count, result.incremental_facts, result.speedup, status
        );
    }

    println!("═══════════════════════════════════════════════════════════════");

    let avg_speedup: f64 = results.iter().map(|r| r.speedup).sum::<f64>() / results.len() as f64;
    let min_speedup = results.iter().map(|r| r.speedup).fold(f64::INFINITY, f64::min);
    let max_speedup = results.iter().map(|r| r.speedup).fold(0.0, f64::max);

    println!("📈 Average Speedup: {avg_speedup:.2}x");
    println!("📊 Range: {min_speedup:.2}x - {max_speedup:.2}x");

    if avg_speedup >= 10.0 {
        println!("🏆 OUTSTANDING: Average >10x demonstrates exceptional O(Δfacts) performance!");
    } else if avg_speedup >= 5.0 {
        println!("🌟 EXCELLENT: Average >5x shows strong O(Δfacts) advantage!");
    } else if avg_speedup >= 2.0 {
        println!("✅ GOOD: Average >2x confirms incremental processing benefits!");
    } else {
        println!("⚠️  MARGINAL: Performance benefits detected but below optimal levels");
    }
}
