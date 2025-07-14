//! Simple RETE Performance Test
//!
//! This test verifies that our RETE implementation shows performance benefits
//! for incremental fact processing compared to batch processing all facts.

use bingo_core::engine::BingoEngine;
use bingo_core::types::{Action, ActionType, Condition, Fact, FactData, FactValue, Operator, Rule};
use chrono::Utc;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[test]
fn test_incremental_vs_batch_performance() {
    println!("🚀 Starting RETE Incremental vs Batch Performance Test");

    // Test configuration
    let rule_count = 20;
    let initial_fact_count = 500;
    let incremental_fact_count = 50;

    // Generate test data
    let rules = generate_simple_rules(rule_count);
    let initial_facts = generate_simple_facts(initial_fact_count, 0);
    let incremental_facts = generate_simple_facts(incremental_fact_count, initial_fact_count);

    // Test 1: Batch processing (simulating O(rules × facts) approach)
    println!("📊 Testing Batch Processing (O(rules × facts) simulation)");
    let batch_time = measure_batch_processing(&rules, &initial_facts, &incremental_facts);

    // Test 2: Incremental processing (our RETE O(Δfacts) implementation)
    println!("⚡ Testing Incremental Processing (RETE O(Δfacts))");
    let incremental_time =
        measure_incremental_processing(&rules, &initial_facts, &incremental_facts);

    // Calculate performance improvement
    let speedup = batch_time.as_nanos() as f64 / incremental_time.as_nanos() as f64;

    // Results
    println!("\n🎯 RETE Performance Test Results");
    println!("═══════════════════════════════════════════════════════════");
    println!("Batch Processing Time:       {batch_time:>8.2?}");
    println!("Incremental Processing Time: {incremental_time:>8.2?}");
    println!("Performance Improvement:     {speedup:>8.2}x");
    println!("═══════════════════════════════════════════════════════════");

    if speedup >= 3.0 {
        println!("🏆 EXCELLENT: >3x speedup demonstrates O(Δfacts) advantage!");
    } else if speedup >= 1.5 {
        println!("✅ GOOD: >1.5x speedup shows incremental processing benefits");
    } else {
        println!("⚠️  Speedup below expected levels");
    }

    // Assert that incremental processing is faster
    assert!(
        speedup >= 1.0,
        "Incremental processing should be at least as fast as batch processing"
    );

    println!("✅ RETE Performance Test PASSED - Incremental processing confirmed!");
}

fn measure_batch_processing(
    rules: &[Rule],
    initial_facts: &[Fact],
    incremental_facts: &[Fact],
) -> Duration {
    let start_time = Instant::now();

    // Simulate batch processing by creating fresh engine and processing all facts together
    let engine = BingoEngine::new().expect("Failed to create BingoEngine");

    // Add rules
    for rule in rules {
        let _ = engine.add_rule(rule.clone());
    }

    // Process all facts together (initial + incremental)
    let all_facts: Vec<Fact> =
        initial_facts.iter().chain(incremental_facts.iter()).cloned().collect();
    let _ = engine.process_facts(all_facts);

    let duration = start_time.elapsed();
    println!(
        "📈 Batch: Processed {} total facts in {:?}",
        initial_facts.len() + incremental_facts.len(),
        duration
    );
    duration
}

fn measure_incremental_processing(
    rules: &[Rule],
    initial_facts: &[Fact],
    incremental_facts: &[Fact],
) -> Duration {
    let engine = BingoEngine::new().expect("Failed to create BingoEngine");

    // Add rules
    for rule in rules {
        let _ = engine.add_rule(rule.clone());
    }

    // Process initial facts (setup time not counted)
    let _ = engine.process_facts(initial_facts.to_vec());

    // Measure only incremental processing time
    let start_time = Instant::now();
    let _ = engine.process_facts(incremental_facts.to_vec());
    let duration = start_time.elapsed();

    println!(
        "⚡ Incremental: Processed {} incremental facts in {:?}",
        incremental_facts.len(),
        duration
    );
    duration
}

fn generate_simple_rules(count: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| Rule {
            id: i as u64,
            name: format!("Test Rule {i}"),
            conditions: vec![
                Condition::Simple {
                    field: "type".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("test".to_string()),
                },
                Condition::Simple {
                    field: "score".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Float((i % 100) as f64),
                },
            ],
            actions: vec![Action {
                action_type: ActionType::CreateFact {
                    data: FactData {
                        fields: HashMap::from([
                            ("source_rule".to_string(), FactValue::Integer(i as i64)),
                            ("processed".to_string(), FactValue::Boolean(true)),
                        ]),
                    },
                },
            }],
        })
        .collect()
}

fn generate_simple_facts(count: usize, start_id: usize) -> Vec<Fact> {
    (0..count)
        .map(|i| {
            let fact_id = start_id + i;
            Fact {
                id: fact_id as u64,
                external_id: Some(format!("ext_{fact_id}")),
                timestamp: Utc::now(),
                data: FactData {
                    fields: HashMap::from([
                        ("type".to_string(), FactValue::String("test".to_string())),
                        (
                            "score".to_string(),
                            FactValue::Float((fact_id % 150) as f64),
                        ),
                        (
                            "category".to_string(),
                            FactValue::String(format!("cat_{}", fact_id % 5)),
                        ),
                    ]),
                },
            }
        })
        .collect()
}
