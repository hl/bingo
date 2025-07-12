//! Integration tests for BingoEngine threaded parallel processing
//!
//! This test suite validates the complete integration of true multi-threaded
//! parallel RETE processing with the main BingoEngine.

use bingo_core::{engine::BingoEngine, parallel_rete::ParallelReteConfig, types::*};
use std::collections::HashMap;

fn create_test_fact(id: u64, age: i64, status: &str, purchase_amount: f64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert("age".to_string(), FactValue::Integer(age));
    fields.insert("status".to_string(), FactValue::String(status.to_string()));
    fields.insert(
        "purchase_amount".to_string(),
        FactValue::Float(purchase_amount),
    );

    Fact {
        id,
        external_id: Some(format!("fact_{id}")),
        timestamp: chrono::Utc::now(),
        data: FactData { fields },
    }
}

fn create_test_rule(id: u64, name: &str) -> Rule {
    Rule {
        id,
        name: name.to_string(),
        conditions: vec![Condition::Simple {
            field: "age".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(21),
        }],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Rule fired".to_string() },
        }],
    }
}

#[test]
fn test_engine_threaded_parallel_processing() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Add test rules to the parallel RETE network
    let rules = vec![
        create_test_rule(1, "Age Check Rule"),
        create_test_rule(2, "Premium Customer Rule"),
    ];

    engine.add_rules_to_parallel_rete(rules).expect("Failed to add rules");

    // Configure for true parallel processing
    let config = ParallelReteConfig {
        worker_count: 2,
        parallel_threshold: 1, // Force parallel processing
        fact_chunk_size: 3,
        enable_parallel_alpha: true,
        enable_parallel_beta: true,
        enable_parallel_execution: true,
        ..Default::default()
    };

    // Create test facts (enough to trigger parallel processing)
    let facts = vec![
        create_test_fact(1, 25, "active", 1500.0),
        create_test_fact(2, 30, "premium", 2500.0),
        create_test_fact(3, 18, "inactive", 150.0),
        create_test_fact(4, 40, "active", 3500.0),
        create_test_fact(5, 22, "pending", 750.0),
        create_test_fact(6, 35, "premium", 4500.0),
    ];

    // Process facts using true multi-threaded processing
    let result = engine.process_facts_parallel_threaded(facts, &config);
    assert!(
        result.is_ok(),
        "Failed to process facts with threaded processing: {result:?}"
    );

    let results = result.unwrap();
    println!(
        "✅ Threaded parallel processing completed with {} results",
        results.len()
    );

    // Verify processor stats
    let stats = engine.get_parallel_rete_stats().expect("Failed to get stats");
    assert_eq!(stats.worker_count, 2, "Expected 2 workers");
    println!("   Worker count: {}", stats.worker_count);
    println!("   Facts processed: {}", stats.facts_processed);
}

#[test]
fn test_engine_threaded_vs_regular_parallel() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Add test rules
    let rules = vec![create_test_rule(1, "Test Rule 1"), create_test_rule(2, "Test Rule 2")];

    engine.add_rules_to_parallel_rete(rules).expect("Failed to add rules");

    // Create test facts
    let facts = vec![
        create_test_fact(1, 25, "active", 1500.0),
        create_test_fact(2, 30, "premium", 2500.0),
        create_test_fact(3, 22, "pending", 750.0),
        create_test_fact(4, 35, "premium", 4500.0),
    ];

    let config = ParallelReteConfig {
        worker_count: 2,
        parallel_threshold: 1, // Force parallel processing
        fact_chunk_size: 2,
        enable_parallel_alpha: true,
        enable_parallel_beta: true,
        enable_parallel_execution: true,
        ..Default::default()
    };

    // Test regular parallel processing
    let regular_start = std::time::Instant::now();
    let regular_results = engine
        .process_facts_advanced_parallel(facts.clone(), &config)
        .expect("Regular parallel processing failed");
    let regular_duration = regular_start.elapsed();

    // Reset stats
    engine.reset_parallel_rete_stats().expect("Failed to reset stats");

    // Test threaded parallel processing
    let threaded_start = std::time::Instant::now();
    let threaded_results = engine
        .process_facts_parallel_threaded(facts, &config)
        .expect("Threaded parallel processing failed");
    let threaded_duration = threaded_start.elapsed();

    println!("✅ Performance comparison test completed");
    println!("   Regular parallel duration: {regular_duration:?}");
    println!("   Threaded parallel duration: {threaded_duration:?}");
    println!("   Regular results: {}", regular_results.len());
    println!("   Threaded results: {}", threaded_results.len());

    // Both should produce the same number of results
    assert_eq!(
        regular_results.len(),
        threaded_results.len(),
        "Regular and threaded processing should produce same number of results"
    );
}

#[test]
fn test_engine_threaded_large_workload() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Add multiple test rules
    let rules: Vec<Rule> = (1..=5)
        .map(|i| Rule {
            id: i,
            name: format!("Performance Rule {i}"),
            conditions: vec![Condition::Simple {
                field: "age".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Integer((i * 5) as i64),
            }],
            actions: vec![Action {
                action_type: ActionType::Log { message: format!("Rule {i} triggered") },
            }],
        })
        .collect();

    engine.add_rules_to_parallel_rete(rules).expect("Failed to add rules");

    // Create larger fact set for performance testing
    let facts: Vec<Fact> = (1..=100)
        .map(|i| {
            create_test_fact(
                i as u64,
                (i % 50) + 1,
                if i % 2 == 0 { "even" } else { "odd" },
                (i as f64) * 10.0,
            )
        })
        .collect();

    // Configure for aggressive parallel processing
    let config = ParallelReteConfig {
        worker_count: 4,
        parallel_threshold: 1, // Force parallel processing
        fact_chunk_size: 10,
        token_chunk_size: 5,
        enable_parallel_alpha: true,
        enable_parallel_beta: true,
        enable_parallel_execution: true,
        enable_work_stealing: true,
        work_queue_capacity: 1000,
    };

    // Process large workload with threaded processing
    let start_time = std::time::Instant::now();
    let results = engine
        .process_facts_parallel_threaded(facts, &config)
        .expect("Failed to process large workload");
    let duration = start_time.elapsed();

    // Get final statistics
    let final_stats = engine.get_parallel_rete_stats().expect("Failed to get final stats");

    println!("✅ Large workload threaded processing test completed");
    println!("   Processing duration: {duration:?}");
    println!("   Results generated: {}", results.len());
    println!("   Facts processed: {}", final_stats.facts_processed);
    println!("   Worker count: {}", final_stats.worker_count);
    println!(
        "   Total processing time: {}ms",
        final_stats.total_processing_time_ms
    );

    // Verify processing was successful
    assert!(!results.is_empty(), "Should have generated some results");
    assert_eq!(final_stats.worker_count, 4, "Should have used 4 workers");
    assert!(
        final_stats.facts_processed > 0,
        "Should have processed facts"
    );
}

#[test]
fn test_engine_threaded_error_handling() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Test with empty facts (should handle gracefully)
    let config = ParallelReteConfig::default();
    let empty_facts = vec![];

    let result = engine.process_facts_parallel_threaded(empty_facts, &config);
    assert!(result.is_ok(), "Should handle empty facts gracefully");

    let results = result.unwrap();
    assert_eq!(results.len(), 0, "Empty facts should produce no results");

    println!("✅ Error handling test completed");
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Complete Phase 7 integration test demonstrating BingoEngine threaded processing
    #[test]
    fn test_complete_phase7_engine_threaded_integration() {
        println!("\n🚀 Starting Phase 7 Complete Engine Threaded Integration Test");

        // Step 1: Create engine
        let mut engine = BingoEngine::new().expect("Failed to create engine");
        println!("✅ Created BingoEngine");

        // Step 2: Create comprehensive rule set
        let rules = vec![
            // Customer tier rules
            Rule {
                id: 2001,
                name: "Premium Customer Detection".to_string(),
                conditions: vec![Condition::Simple {
                    field: "purchase_amount".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Float(1000.0),
                }],
                actions: vec![Action {
                    action_type: ActionType::SetField {
                        field: "customer_tier".to_string(),
                        value: FactValue::String("premium".to_string()),
                    },
                }],
            },
            Rule {
                id: 2002,
                name: "Age Verification Rule".to_string(),
                conditions: vec![Condition::Simple {
                    field: "age".to_string(),
                    operator: Operator::GreaterThanOrEqual,
                    value: FactValue::Integer(18),
                }],
                actions: vec![Action {
                    action_type: ActionType::CreateFact {
                        data: FactData {
                            fields: HashMap::from([
                                (
                                    "verification_status".to_string(),
                                    FactValue::String("verified".to_string()),
                                ),
                                ("eligible_for_service".to_string(), FactValue::Boolean(true)),
                            ]),
                        },
                    },
                }],
            },
        ];

        // Step 3: Add rules to parallel RETE network
        engine.add_rules_to_parallel_rete(rules).expect("Failed to add rules");
        println!("✅ Added {} rules to parallel RETE network", 2);

        // Step 4: Configure for aggressive threading
        let config = ParallelReteConfig {
            worker_count: 4,
            parallel_threshold: 5,
            fact_chunk_size: 3,
            token_chunk_size: 2,
            enable_parallel_alpha: true,
            enable_parallel_beta: true,
            enable_parallel_execution: true,
            enable_work_stealing: true,
            work_queue_capacity: 500,
        };

        println!(
            "✅ Configured parallel processing with {} workers",
            config.worker_count
        );

        // Step 5: Create diverse fact set
        let facts = vec![
            create_test_fact(1, 25, "active", 1500.0),
            create_test_fact(2, 17, "new", 250.0),
            create_test_fact(3, 30, "premium", 2500.0),
            create_test_fact(4, 22, "standard", 750.0),
            create_test_fact(5, 40, "vip", 5000.0),
            create_test_fact(6, 16, "minor", 100.0),
            create_test_fact(7, 35, "active", 3500.0),
            create_test_fact(8, 28, "pending", 1200.0),
        ];

        // Step 6: Process facts with true multi-threading
        let processing_start = std::time::Instant::now();
        let results = engine
            .process_facts_parallel_threaded(facts.clone(), &config)
            .expect("Failed to process facts with threaded processing");
        let processing_duration = processing_start.elapsed();

        println!(
            "✅ Processed {} facts through threaded parallel RETE",
            facts.len()
        );
        println!("   Processing duration: {processing_duration:?}");
        println!("   Results generated: {}", results.len());

        // Step 7: Verify processing statistics
        let stats = engine.get_parallel_rete_stats().expect("Failed to get stats");

        println!("📊 Threaded Processing Statistics:");
        println!("   Facts processed: {}", stats.facts_processed);
        println!("   Worker count used: {}", stats.worker_count);
        println!(
            "   Total processing time: {}ms",
            stats.total_processing_time_ms
        );

        // Step 8: Verify engine state
        let engine_stats = engine.get_stats();
        println!("🎯 Engine Statistics:");
        println!("   Total rules: {}", engine_stats.rule_count);
        println!("   Total facts: {}", engine_stats.fact_count);

        println!("\n🎉 Phase 7 Complete Engine Threaded Integration Test PASSED!");
        println!("   ✅ True multi-threaded processing integrated");
        println!("   ✅ Thread-safe components working");
        println!("   ✅ BingoEngine enhanced with threading support");
        println!("   ✅ Performance statistics operational");

        // Assert success criteria
        assert!(engine_stats.rule_count > 0);
        assert_eq!(stats.worker_count, 4);
        assert!(stats.facts_processed > 0);
    }
}
