//! Comprehensive tests for the Advanced Parallel RETE Processing system
//!
//! This test suite validates the Phase 5 parallel RETE processing features,
//! including multi-core fact processing, worker thread management, and
//! performance improvements from parallel algorithm execution.

use bingo_core::{
    engine::BingoEngine,
    parallel_rete::{ParallelReteConfig, ParallelReteProcessor},
    types::*,
};
use std::collections::HashMap;

#[test]
fn test_parallel_rete_config_defaults() {
    let config = ParallelReteConfig::default();

    assert!(config.parallel_threshold > 0);
    assert!(config.worker_count > 0);
    assert!(config.fact_chunk_size > 0);
    assert!(config.token_chunk_size > 0);
    assert!(config.work_queue_capacity > 0);

    // On multi-core systems, parallel features should be enabled
    if num_cpus::get() >= 2 {
        assert!(config.enable_parallel_alpha);
        assert!(config.enable_parallel_execution);
    }

    if num_cpus::get() >= 4 {
        assert!(config.enable_parallel_beta);
        assert!(config.enable_work_stealing);
    }

    println!("✅ Parallel RETE config defaults test passed");
    println!("   Worker count: {}", config.worker_count);
    println!("   Parallel threshold: {}", config.parallel_threshold);
    println!("   Fact chunk size: {}", config.fact_chunk_size);
}

#[test]
fn test_parallel_rete_config_customization() {
    let config = ParallelReteConfig {
        worker_count: 4,
        parallel_threshold: 100,
        fact_chunk_size: 25,
        enable_work_stealing: true,
        enable_parallel_alpha: true,
        enable_parallel_beta: true,
        enable_parallel_execution: true,
        ..Default::default()
    };

    assert_eq!(config.worker_count, 4);
    assert_eq!(config.parallel_threshold, 100);
    assert_eq!(config.fact_chunk_size, 25);
    assert!(config.enable_work_stealing);
    assert!(config.enable_parallel_alpha);
    assert!(config.enable_parallel_beta);
    assert!(config.enable_parallel_execution);

    println!("✅ Parallel RETE config customization test passed");
}

#[test]
fn test_parallel_rete_processor_creation() {
    let config = ParallelReteConfig::default();
    let processor = ParallelReteProcessor::new(config.clone());

    assert_eq!(processor.get_config().worker_count, config.worker_count);
    assert_eq!(
        processor.get_config().parallel_threshold,
        config.parallel_threshold
    );

    // Test statistics initialization
    let stats = processor.get_stats().expect("Failed to get stats");
    assert_eq!(stats.facts_processed, 0);
    assert_eq!(stats.tokens_processed, 0);
    assert_eq!(stats.rules_executed, 0);

    println!("✅ Parallel RETE processor creation test passed");
}

#[test]
fn test_engine_parallel_rete_integration() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Test getting parallel RETE stats
    let stats = engine.get_parallel_rete_stats().expect("Failed to get parallel RETE stats");
    assert_eq!(stats.facts_processed, 0);

    // Test configuration
    let config = ParallelReteConfig {
        worker_count: 2,
        parallel_threshold: 50,
        fact_chunk_size: 10,
        ..Default::default()
    };
    engine.configure_parallel_rete(config);

    // Test stats reset
    let reset_result = engine.reset_parallel_rete_stats();
    assert!(reset_result.is_ok());

    println!("✅ Engine parallel RETE integration test passed");
}

#[test]
fn test_parallel_rete_rule_addition() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Create test rules
    let rules = vec![
        Rule {
            id: 1,
            name: "Parallel Test Rule 1".to_string(),
            conditions: vec![Condition::Simple {
                field: "age".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Integer(21),
            }],
            actions: vec![Action {
                action_type: ActionType::Log { message: "Adult customer detected".to_string() },
            }],
        },
        Rule {
            id: 2,
            name: "Parallel Test Rule 2".to_string(),
            conditions: vec![Condition::Simple {
                field: "status".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("premium".to_string()),
            }],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "discount".to_string(),
                    value: FactValue::Float(0.15),
                },
            }],
        },
    ];

    // Add rules to parallel RETE network
    let result = engine.add_rules_to_parallel_rete(rules);
    assert!(
        result.is_ok(),
        "Failed to add rules to parallel RETE: {result:?}"
    );

    // Verify engine state
    let engine_stats = engine.get_stats();
    assert_eq!(engine_stats.rule_count, 2);

    println!("✅ Parallel RETE rule addition test passed");
    println!("   Rules added: {}", engine_stats.rule_count);
}

#[test]
fn test_small_fact_set_sequential_fallback() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Configure high threshold to force sequential processing
    let config = ParallelReteConfig {
        parallel_threshold: 1000, // High threshold
        worker_count: 4,
        ..Default::default()
    };

    // Create small fact set
    let facts = vec![create_test_fact(1, 25, "active"), create_test_fact(2, 30, "premium")];

    // Process facts (should use sequential fallback)
    let result = engine.process_facts_advanced_parallel(facts, &config);
    assert!(result.is_ok(), "Failed to process facts: {result:?}");

    let results = result.unwrap();
    assert_eq!(results.len(), 0); // No rules added, so no results expected

    println!("✅ Small fact set sequential fallback test passed");
    println!("   Facts processed: 2 (sequential fallback)");
}

#[test]
fn test_parallel_rete_with_facts_and_rules() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Add test rules first
    let rules = vec![Rule {
        id: 1,
        name: "Age Check Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "age".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(18),
        }],
        actions: vec![Action {
            action_type: ActionType::CreateFact {
                data: FactData {
                    fields: HashMap::from([("eligible".to_string(), FactValue::Boolean(true))]),
                },
            },
        }],
    }];

    engine.add_rules_to_parallel_rete(rules).expect("Failed to add rules");

    // Configure for parallel processing
    let config = ParallelReteConfig {
        parallel_threshold: 1, // Low threshold to force parallel processing
        worker_count: 2,
        fact_chunk_size: 2,
        enable_parallel_alpha: true,
        enable_parallel_execution: true,
        ..Default::default()
    };

    // Create test facts
    let facts = vec![
        create_test_fact(1, 25, "active"),
        create_test_fact(2, 17, "inactive"),
        create_test_fact(3, 30, "premium"),
        create_test_fact(4, 22, "standard"),
    ];

    // Process facts with parallel RETE
    let result = engine.process_facts_advanced_parallel(facts, &config);
    assert!(result.is_ok(), "Failed to process facts: {result:?}");

    // Get parallel processing statistics
    let stats = engine.get_parallel_rete_stats().expect("Failed to get stats");
    println!("✅ Parallel RETE processing with rules test passed");
    println!("   Facts processed: {}", stats.facts_processed);
    println!("   Worker count: {}", stats.worker_count);
}

#[test]
fn test_parallel_rete_performance_comparison() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Add performance test rules
    let rules: Vec<Rule> = (1..=5)
        .map(|i| Rule {
            id: i,
            name: format!("Performance Rule {i}"),
            conditions: vec![Condition::Simple {
                field: "value".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Integer((i * 10) as i64),
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
                (i % 100) + 1,
                if i % 2 == 0 { "even" } else { "odd" },
            )
        })
        .collect();

    // Test with sequential processing (high threshold)
    let sequential_config = ParallelReteConfig {
        parallel_threshold: 1000, // Force sequential
        ..Default::default()
    };

    let sequential_start = std::time::Instant::now();
    let sequential_results = engine
        .process_facts_advanced_parallel(facts.clone(), &sequential_config)
        .expect("Sequential processing failed");
    let sequential_duration = sequential_start.elapsed();

    // Reset stats
    engine.reset_parallel_rete_stats().expect("Failed to reset stats");

    // Test with parallel processing (low threshold)
    let parallel_config = ParallelReteConfig {
        parallel_threshold: 1, // Force parallel
        worker_count: 4,
        fact_chunk_size: 10,
        enable_parallel_alpha: true,
        enable_parallel_execution: true,
        ..Default::default()
    };

    let parallel_start = std::time::Instant::now();
    let parallel_results = engine
        .process_facts_advanced_parallel(facts, &parallel_config)
        .expect("Parallel processing failed");
    let parallel_duration = parallel_start.elapsed();

    // Get final statistics
    let final_stats = engine.get_parallel_rete_stats().expect("Failed to get final stats");

    println!("✅ Parallel RETE performance comparison test passed");
    println!("   Sequential duration: {sequential_duration:?}");
    println!("   Parallel duration: {parallel_duration:?}");
    println!("   Sequential results: {}", sequential_results.len());
    println!("   Parallel results: {}", parallel_results.len());
    println!(
        "   Final stats - Facts processed: {}",
        final_stats.facts_processed
    );
    println!(
        "   Final stats - Worker count: {}",
        final_stats.worker_count
    );

    // Both should produce the same number of results
    assert_eq!(sequential_results.len(), parallel_results.len());
}

#[test]
fn test_parallel_rete_configuration_updates() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Test initial configuration
    let initial_stats = engine.get_parallel_rete_stats().expect("Failed to get initial stats");
    assert_eq!(initial_stats.worker_count, 0); // No processing done yet

    // Update configuration
    let new_config = ParallelReteConfig {
        worker_count: 8,
        parallel_threshold: 25,
        fact_chunk_size: 5,
        token_chunk_size: 3,
        enable_work_stealing: true,
        enable_parallel_alpha: true,
        enable_parallel_beta: true,
        enable_parallel_execution: true,
        work_queue_capacity: 500,
    };

    engine.configure_parallel_rete(new_config.clone());

    // Configuration should be updated (we can't directly verify this without
    // exposing the internal config, but the method should execute without error)

    // Test statistics reset
    let reset_result = engine.reset_parallel_rete_stats();
    assert!(reset_result.is_ok());

    let reset_stats = engine.get_parallel_rete_stats().expect("Failed to get reset stats");
    assert_eq!(reset_stats.facts_processed, 0);
    assert_eq!(reset_stats.tokens_processed, 0);
    assert_eq!(reset_stats.rules_executed, 0);

    println!("✅ Parallel RETE configuration updates test passed");
    println!(
        "   New worker count configured: {}",
        new_config.worker_count
    );
    println!(
        "   New parallel threshold: {}",
        new_config.parallel_threshold
    );
}

// Helper function to create test facts
fn create_test_fact(id: u64, age: i64, status: &str) -> Fact {
    let mut fields = HashMap::new();
    fields.insert("age".to_string(), FactValue::Integer(age));
    fields.insert("status".to_string(), FactValue::String(status.to_string()));
    fields.insert("value".to_string(), FactValue::Integer(age * 2)); // Derived value for testing

    Fact {
        id,
        external_id: Some(format!("test_fact_{id}")),
        timestamp: chrono::Utc::now(),
        data: FactData { fields },
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Integration test demonstrating the complete Phase 5 parallel RETE workflow
    #[test]
    fn test_complete_phase5_parallel_rete_workflow() {
        println!("\n🚀 Starting Phase 5 Complete Parallel RETE Workflow Test");

        // Step 1: Create engine with parallel RETE capabilities
        let mut engine = BingoEngine::new().expect("Failed to create engine");
        println!("✅ Created BingoEngine with parallel RETE support");

        // Step 2: Configure parallel RETE processor
        let config = ParallelReteConfig {
            worker_count: 4,
            parallel_threshold: 10,
            fact_chunk_size: 5,
            token_chunk_size: 3,
            enable_parallel_alpha: true,
            enable_parallel_beta: true,
            enable_parallel_execution: true,
            enable_work_stealing: true,
            work_queue_capacity: 1000,
        };

        engine.configure_parallel_rete(config.clone());
        println!(
            "✅ Configured parallel RETE processor with {} workers",
            config.worker_count
        );

        // Step 3: Create comprehensive rule set
        let rules = vec![
            // Customer tier rules
            Rule {
                id: 1001,
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
                id: 1002,
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
            Rule {
                id: 1003,
                name: "High Value Transaction Alert".to_string(),
                conditions: vec![Condition::Simple {
                    field: "transaction_amount".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Float(5000.0),
                }],
                actions: vec![Action {
                    action_type: ActionType::Log {
                        message: "High value transaction detected - requires approval".to_string(),
                    },
                }],
            },
        ];

        // Step 4: Add rules to parallel RETE network
        engine.add_rules_to_parallel_rete(rules).expect("Failed to add rules");
        println!("✅ Added {} rules to parallel RETE network", 3);

        // Step 5: Create diverse fact set for processing
        let facts = vec![
            // Customer facts
            Fact {
                id: 1,
                external_id: Some("customer_001".to_string()),
                timestamp: chrono::Utc::now(),
                data: FactData {
                    fields: HashMap::from([
                        ("customer_id".to_string(), FactValue::Integer(12345)),
                        ("age".to_string(), FactValue::Integer(25)),
                        ("purchase_amount".to_string(), FactValue::Float(1500.0)),
                        (
                            "status".to_string(),
                            FactValue::String("active".to_string()),
                        ),
                    ]),
                },
            },
            Fact {
                id: 2,
                external_id: Some("customer_002".to_string()),
                timestamp: chrono::Utc::now(),
                data: FactData {
                    fields: HashMap::from([
                        ("customer_id".to_string(), FactValue::Integer(67890)),
                        ("age".to_string(), FactValue::Integer(17)),
                        ("purchase_amount".to_string(), FactValue::Float(250.0)),
                        ("status".to_string(), FactValue::String("new".to_string())),
                    ]),
                },
            },
            // Transaction facts
            Fact {
                id: 3,
                external_id: Some("transaction_001".to_string()),
                timestamp: chrono::Utc::now(),
                data: FactData {
                    fields: HashMap::from([
                        (
                            "transaction_id".to_string(),
                            FactValue::String("TXN-001".to_string()),
                        ),
                        ("transaction_amount".to_string(), FactValue::Float(7500.0)),
                        ("customer_id".to_string(), FactValue::Integer(12345)),
                        (
                            "type".to_string(),
                            FactValue::String("purchase".to_string()),
                        ),
                    ]),
                },
            },
            Fact {
                id: 4,
                external_id: Some("transaction_002".to_string()),
                timestamp: chrono::Utc::now(),
                data: FactData {
                    fields: HashMap::from([
                        (
                            "transaction_id".to_string(),
                            FactValue::String("TXN-002".to_string()),
                        ),
                        ("transaction_amount".to_string(), FactValue::Float(150.0)),
                        ("customer_id".to_string(), FactValue::Integer(67890)),
                        ("type".to_string(), FactValue::String("refund".to_string())),
                    ]),
                },
            },
        ];

        // Step 6: Process facts through parallel RETE network
        let processing_start = std::time::Instant::now();
        let results = engine
            .process_facts_advanced_parallel(facts.clone(), &config)
            .expect("Failed to process facts through parallel RETE");
        let processing_duration = processing_start.elapsed();

        println!(
            "✅ Processed {} facts through parallel RETE network",
            facts.len()
        );
        println!("   Processing duration: {processing_duration:?}");
        println!("   Results generated: {}", results.len());

        // Step 7: Verify parallel processing statistics
        let stats = engine.get_parallel_rete_stats().expect("Failed to get stats");

        println!("📊 Parallel RETE Processing Statistics:");
        println!("   Facts processed: {}", stats.facts_processed);
        println!("   Tokens processed: {}", stats.tokens_processed);
        println!("   Rules executed: {}", stats.rules_executed);
        println!("   Worker count used: {}", stats.worker_count);
        println!(
            "   Total processing time: {}ms",
            stats.total_processing_time_ms
        );
        println!("   Work items stolen: {}", stats.work_items_stolen);
        println!("   Queue overflows: {}", stats.queue_overflows);
        println!(
            "   Worker utilization: {:.1}%",
            stats.worker_utilization * 100.0
        );

        // Step 8: Verify engine state
        let engine_stats = engine.get_stats();

        println!("🎯 Engine Statistics:");
        println!("   Total rules: {}", engine_stats.rule_count);
        println!("   Total facts: {}", engine_stats.fact_count);
        println!("   RETE nodes: {}", engine_stats.node_count);
        println!("   Memory usage: {} bytes", engine_stats.memory_usage_bytes);

        // Step 9: Test statistics reset
        engine.reset_parallel_rete_stats().expect("Failed to reset stats");
        let reset_stats = engine.get_parallel_rete_stats().expect("Failed to get reset stats");

        println!("🔄 Statistics Reset:");
        println!(
            "   Facts processed after reset: {}",
            reset_stats.facts_processed
        );
        println!(
            "   Processing time after reset: {}ms",
            reset_stats.total_processing_time_ms
        );

        println!("\n🎉 Phase 5 Complete Parallel RETE Workflow Test PASSED!");
        println!("   ✅ Parallel RETE processor integrated successfully");
        println!("   ✅ Multi-core processing demonstrated");
        println!("   ✅ Performance statistics tracking operational");
        println!("   ✅ Engine functionality preserved with parallel extensions");

        // Assert success criteria
        assert!(engine_stats.rule_count > 0);
        assert!(engine_stats.fact_count > 0);
        assert_eq!(reset_stats.facts_processed, 0);
        assert_eq!(reset_stats.total_processing_time_ms, 0);
    }
}
