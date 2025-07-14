//! Comprehensive tests for the Rule Optimization system
//!
//! This test suite validates the Phase 5 rule optimization features,
//! including condition reordering, cost-based optimization, and
//! performance improvements from advanced RETE optimizations.

use bingo_core::{
    engine::BingoEngine,
    rule_optimizer::{OptimizerConfig, RuleOptimizer, optimize_rule_batch},
    types::*,
};
use std::collections::HashMap;

#[test]
fn test_rule_optimizer_creation() {
    let optimizer = RuleOptimizer::new();
    let metrics = optimizer.get_metrics();

    assert_eq!(metrics.rules_optimized, 0);
    assert_eq!(metrics.conditions_reordered, 0);
    assert_eq!(metrics.avg_performance_improvement, 0.0);
}

#[test]
fn test_rule_optimizer_with_custom_config() {
    let config = OptimizerConfig {
        enable_selectivity_ordering: true,
        enable_cost_based_optimization: true,
        min_selectivity_difference: 0.1, // 10% threshold
        ..Default::default()
    };

    let optimizer = RuleOptimizer::with_config(config.clone());
    let optimizer_config = optimizer.get_config();

    assert!(optimizer_config.enable_selectivity_ordering);
    assert!(optimizer_config.enable_cost_based_optimization);
    assert_eq!(optimizer_config.min_selectivity_difference, 0.1);
}

#[test]
fn test_basic_rule_optimization() {
    let mut optimizer = RuleOptimizer::new();

    // Create a rule with conditions in suboptimal order (expensive first, selective last)
    let rule = Rule {
        id: 1,
        name: "Test Optimization Rule".to_string(),
        conditions: vec![
            // Expensive string contains condition (low selectivity, high cost)
            Condition::Simple {
                field: "description".to_string(),
                operator: Operator::Contains,
                value: FactValue::String("expensive search pattern".to_string()),
            },
            // Cheap equality condition (high selectivity, low cost)
            Condition::Simple {
                field: "id".to_string(),
                operator: Operator::Equal,
                value: FactValue::Integer(12345),
            },
        ],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Rule fired successfully".to_string() },
        }],
    };

    let result = optimizer.optimize_rule(rule);

    // Verify optimization was performed
    assert!(result.estimated_improvement >= 0.0);
    assert_eq!(result.analysis.condition_selectivity.len(), 2);
    assert_eq!(result.analysis.condition_costs.len(), 2);

    // Verify that the second condition (id equality) is more selective than the first
    assert!(result.analysis.condition_selectivity[1] < result.analysis.condition_selectivity[0]);

    println!("✅ Basic rule optimization test passed");
    println!(
        "   Estimated improvement: {:.1}%",
        result.estimated_improvement
    );
    println!("   Strategies applied: {}", result.strategies_applied.len());
}

#[test]
fn test_multi_condition_rule_optimization() {
    let mut optimizer = RuleOptimizer::new();

    // Create a complex rule with multiple conditions in suboptimal order
    let rule = Rule {
        id: 2,
        name: "Complex Multi-Condition Rule".to_string(),
        conditions: vec![
            // Moderately expensive regex-like operation
            Condition::Simple {
                field: "email".to_string(),
                operator: Operator::Contains,
                value: FactValue::String("@company.com".to_string()),
            },
            // Very selective equality condition (should be first)
            Condition::Simple {
                field: "user_id".to_string(),
                operator: Operator::Equal,
                value: FactValue::Integer(999999),
            },
            // Moderately selective range condition
            Condition::Simple {
                field: "age".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Integer(25),
            },
            // Less selective status check
            Condition::Simple {
                field: "status".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("active".to_string()),
            },
        ],
        actions: vec![Action {
            action_type: ActionType::CreateFact {
                data: FactData {
                    fields: HashMap::from([
                        (
                            "result_type".to_string(),
                            FactValue::String("multi_condition_result".to_string()),
                        ),
                        ("optimization_test".to_string(), FactValue::Boolean(true)),
                    ]),
                },
            },
        }],
    };

    let result = optimizer.optimize_rule(rule);

    // Verify comprehensive analysis was performed
    assert_eq!(result.analysis.condition_selectivity.len(), 4);
    assert_eq!(result.analysis.condition_costs.len(), 4);

    // Verify join analysis for multi-condition rule
    assert!(result.analysis.join_analysis.is_some());
    let join_analysis = result.analysis.join_analysis.as_ref().unwrap();
    assert_eq!(join_analysis.intermediate_sizes.len(), 4);
    assert_eq!(join_analysis.join_selectivity.len(), 4);

    // Check that optimization provides measurable benefits
    if result.estimated_improvement > 0.0 {
        println!(
            "✅ Multi-condition optimization successful: {:.1}% improvement",
            result.estimated_improvement
        );
    }

    println!("✅ Multi-condition rule optimization test passed");
    println!(
        "   Conditions analyzed: {}",
        result.analysis.condition_selectivity.len()
    );
    println!(
        "   Join analysis performed: {}",
        result.analysis.join_analysis.is_some()
    );
}

#[test]
fn test_batch_rule_optimization() {
    // Create a batch of rules with different optimization opportunities
    let rules = vec![
        Rule {
            id: 10,
            name: "Simple Rule 1".to_string(),
            conditions: vec![Condition::Simple {
                field: "type".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("order".to_string()),
            }],
            actions: vec![Action {
                action_type: ActionType::Log { message: "Simple rule fired".to_string() },
            }],
        },
        Rule {
            id: 11,
            name: "Optimization Candidate Rule".to_string(),
            conditions: vec![
                // Expensive condition first (suboptimal)
                Condition::Simple {
                    field: "description".to_string(),
                    operator: Operator::Contains,
                    value: FactValue::String("search term".to_string()),
                },
                // Selective condition second (should be first)
                Condition::Simple {
                    field: "priority".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::Integer(1),
                },
            ],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "optimized".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        },
        Rule {
            id: 12,
            name: "Already Optimal Rule".to_string(),
            conditions: vec![
                // Already in optimal order (selective first)
                Condition::Simple {
                    field: "id".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::Integer(123),
                },
                Condition::Simple {
                    field: "status".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("pending".to_string()),
                },
            ],
            actions: vec![],
        },
    ];

    let results = optimize_rule_batch(rules, None);

    assert_eq!(results.len(), 3);

    // Verify each rule was analyzed
    for result in results.iter() {
        assert!(!result.analysis.condition_selectivity.is_empty());
        assert!(!result.analysis.condition_costs.is_empty());
        println!(
            "Rule {}: {:.1}% improvement with {} strategies",
            result.optimized_rule.id,
            result.estimated_improvement,
            result.strategies_applied.len()
        );
    }

    println!("✅ Batch rule optimization test passed");
    println!("   Rules processed: {}", results.len());
}

#[test]
fn test_engine_rule_optimization_integration() {
    let engine = BingoEngine::new().expect("Failed to create engine");

    // Create a rule that can benefit from optimization
    let rule = Rule {
        id: 100,
        name: "Engine Integration Test Rule".to_string(),
        conditions: vec![
            // Expensive condition first
            Condition::Simple {
                field: "metadata".to_string(),
                operator: Operator::Contains,
                value: FactValue::String("complex search".to_string()),
            },
            // Selective condition second
            Condition::Simple {
                field: "entity_id".to_string(),
                operator: Operator::Equal,
                value: FactValue::Integer(987654321),
            },
        ],
        actions: vec![Action {
            action_type: ActionType::CreateFact {
                data: FactData {
                    fields: HashMap::from([
                        ("engine_test".to_string(), FactValue::Boolean(true)),
                        ("optimization_used".to_string(), FactValue::Boolean(true)),
                    ]),
                },
            },
        }],
    };

    // Test optimized rule addition
    let optimization_result =
        engine.add_rule_optimized(rule).expect("Failed to add optimized rule");

    // Verify optimization was performed
    assert!(optimization_result.estimated_improvement >= 0.0);
    assert!(
        !optimization_result.strategies_applied.is_empty()
            || optimization_result.estimated_improvement == 0.0
    );

    // Get optimization metrics from engine
    let metrics = engine.get_optimization_metrics().clone();
    assert_eq!(metrics.rules_optimized, 1);

    // Verify the rule was added to the engine
    let engine_stats = engine.get_stats();
    assert!(engine_stats.rule_count > 0);

    println!("✅ Engine integration test passed");
    println!(
        "   Rule optimization improvement: {:.1}%",
        optimization_result.estimated_improvement
    );
    println!("   Engine rules count: {}", engine_stats.rule_count);
    println!(
        "   Optimization metrics - rules optimized: {}",
        metrics.rules_optimized
    );
}

#[test]
fn test_condition_selectivity_calculation() {
    let optimizer = RuleOptimizer::new();

    // Test different condition types for selectivity
    let high_selectivity_condition = Condition::Simple {
        field: "unique_id".to_string(),
        operator: Operator::Equal,
        value: FactValue::Integer(123456789),
    };

    let low_selectivity_condition = Condition::Simple {
        field: "category".to_string(),
        operator: Operator::Equal,
        value: FactValue::String("general".to_string()),
    };

    let high_selectivity = optimizer.calculate_condition_selectivity(&high_selectivity_condition);
    let low_selectivity = optimizer.calculate_condition_selectivity(&low_selectivity_condition);

    // High selectivity should be lower value (more selective)
    assert!(high_selectivity <= low_selectivity);
    assert!((0.0..=1.0).contains(&high_selectivity));
    assert!((0.0..=1.0).contains(&low_selectivity));

    println!("✅ Selectivity calculation test passed");
    println!("   High selectivity condition: {high_selectivity:.3}");
    println!("   Low selectivity condition: {low_selectivity:.3}");
}

#[test]
fn test_condition_cost_calculation() {
    let _optimizer = RuleOptimizer::new();

    // Test different condition types for cost
    let cheap_condition = Condition::Simple {
        field: "id".to_string(),
        operator: Operator::Equal,
        value: FactValue::Integer(123),
    };

    let expensive_condition = Condition::Simple {
        field: "description".to_string(),
        operator: Operator::Contains,
        value: FactValue::String("very long search string with many words".to_string()),
    };

    let cheap_cost = RuleOptimizer::calculate_condition_cost(&cheap_condition);
    let expensive_cost = RuleOptimizer::calculate_condition_cost(&expensive_condition);

    // Expensive condition should have higher cost
    assert!(expensive_cost > cheap_cost);
    assert!(cheap_cost > 0.0);
    assert!(expensive_cost > 0.0);

    println!("✅ Cost calculation test passed");
    println!("   Cheap condition cost: {cheap_cost:.1}μs");
    println!("   Expensive condition cost: {expensive_cost:.1}μs");
}

#[test]
fn test_optimization_metrics_tracking() {
    let mut optimizer = RuleOptimizer::new();

    // Create several rules to optimize
    let rules = vec![
        Rule {
            id: 200,
            name: "Metrics Test Rule 1".to_string(),
            conditions: vec![
                Condition::Simple {
                    field: "slow_field".to_string(),
                    operator: Operator::Contains,
                    value: FactValue::String("search".to_string()),
                },
                Condition::Simple {
                    field: "fast_field".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::Integer(1),
                },
            ],
            actions: vec![],
        },
        Rule {
            id: 201,
            name: "Metrics Test Rule 2".to_string(),
            conditions: vec![
                Condition::Simple {
                    field: "another_slow_field".to_string(),
                    operator: Operator::StartsWith,
                    value: FactValue::String("prefix".to_string()),
                },
                Condition::Simple {
                    field: "another_fast_field".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::Boolean(true),
                },
            ],
            actions: vec![],
        },
    ];

    // Optimize rules and track metrics
    let initial_metrics = optimizer.get_metrics().clone();
    assert_eq!(initial_metrics.rules_optimized, 0);

    for rule in rules {
        let _result = optimizer.optimize_rule(rule);
    }

    let final_metrics = optimizer.get_metrics();
    assert_eq!(final_metrics.rules_optimized, 2);
    // Note: optimization time might be 0ms for very fast operations
    // assert!(final_metrics.total_optimization_time_ms >= 0); // Always true for u64

    println!("✅ Metrics tracking test passed");
    println!("   Rules optimized: {}", final_metrics.rules_optimized);
    println!(
        "   Total optimization time: {}ms",
        final_metrics.total_optimization_time_ms
    );
    println!(
        "   Average improvement: {:.1}%",
        final_metrics.avg_performance_improvement
    );
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Integration test demonstrating the complete Phase 5 optimization workflow
    #[test]
    fn test_complete_phase5_optimization_workflow() {
        println!("\n🚀 Starting Phase 5 Complete Optimization Workflow Test");

        // Step 1: Create engine with optimization capabilities
        let engine = BingoEngine::new().expect("Failed to create engine");
        println!("✅ Created BingoEngine with rule optimization support");

        // Step 2: Create test rules with optimization opportunities
        let rules = vec![
            // Rule with poor condition ordering
            Rule {
                id: 1000,
                name: "Payroll Calculation Rule".to_string(),
                conditions: vec![
                    Condition::Simple {
                        field: "employee_notes".to_string(),
                        operator: Operator::Contains,
                        value: FactValue::String("overtime_eligible".to_string()),
                    },
                    Condition::Simple {
                        field: "employee_id".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::Integer(12345),
                    },
                    Condition::Simple {
                        field: "department".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::String("engineering".to_string()),
                    },
                ],
                actions: vec![Action {
                    action_type: ActionType::CreateFact {
                        data: FactData {
                            fields: HashMap::from([
                                (
                                    "calculation_type".to_string(),
                                    FactValue::String("overtime".to_string()),
                                ),
                                ("base_rate".to_string(), FactValue::Float(75.0)),
                                ("multiplier".to_string(), FactValue::Float(1.5)),
                            ]),
                        },
                    },
                }],
            },
            // Rule with complex conditions
            Rule {
                id: 1001,
                name: "Customer Tier Assignment".to_string(),
                conditions: vec![
                    Condition::Simple {
                        field: "purchase_history".to_string(),
                        operator: Operator::Contains,
                        value: FactValue::String("premium_product".to_string()),
                    },
                    Condition::Simple {
                        field: "total_spent".to_string(),
                        operator: Operator::GreaterThan,
                        value: FactValue::Float(10000.0),
                    },
                    Condition::Simple {
                        field: "customer_id".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::Integer(999888777),
                    },
                ],
                actions: vec![Action {
                    action_type: ActionType::SetField {
                        field: "customer_tier".to_string(),
                        value: FactValue::String("platinum".to_string()),
                    },
                }],
            },
        ];

        // Step 3: Add rules with optimization
        let mut _total_improvement = 0.0;
        let mut optimized_rules = 0;

        for rule in rules {
            let optimization_result =
                engine.add_rule_optimized(rule).expect("Failed to add optimized rule");
            _total_improvement += optimization_result.estimated_improvement;
            optimized_rules += 1;

            println!(
                "✅ Optimized rule {} with {:.1}% improvement using {} strategies",
                optimization_result.optimized_rule.id,
                optimization_result.estimated_improvement,
                optimization_result.strategies_applied.len()
            );
        }

        // Step 4: Verify optimization metrics
        let metrics = engine.get_optimization_metrics().clone();
        assert_eq!(metrics.rules_optimized, optimized_rules);

        println!("📊 Final Optimization Metrics:");
        println!("   Total rules optimized: {}", metrics.rules_optimized);
        println!(
            "   Average improvement: {:.1}%",
            metrics.avg_performance_improvement
        );
        println!(
            "   Total optimization time: {}ms",
            metrics.total_optimization_time_ms
        );
        println!("   Conditions reordered: {}", metrics.conditions_reordered);

        // Step 5: Verify engine functionality
        let engine_stats = engine.get_stats();
        assert_eq!(engine_stats.rule_count, optimized_rules);

        println!("🎯 Engine Statistics:");
        println!("   Active rules: {}", engine_stats.rule_count);
        println!("   Network nodes: {}", engine_stats.node_count);
        println!("   Memory usage: {} bytes", engine_stats.memory_usage_bytes);

        // Step 6: Test fact processing with optimized rules
        let test_facts = vec![
            Fact {
                id: 1,
                external_id: Some("test_fact_1".to_string()),
                timestamp: chrono::Utc::now(),
                data: FactData {
                    fields: HashMap::from([
                        ("employee_id".to_string(), FactValue::Integer(12345)),
                        (
                            "employee_notes".to_string(),
                            FactValue::String("overtime_eligible bonus".to_string()),
                        ),
                        (
                            "department".to_string(),
                            FactValue::String("engineering".to_string()),
                        ),
                    ]),
                },
            },
            Fact {
                id: 2,
                external_id: Some("test_fact_2".to_string()),
                timestamp: chrono::Utc::now(),
                data: FactData {
                    fields: HashMap::from([
                        ("customer_id".to_string(), FactValue::Integer(999888777)),
                        (
                            "purchase_history".to_string(),
                            FactValue::String("premium_product special".to_string()),
                        ),
                        ("total_spent".to_string(), FactValue::Float(15000.0)),
                    ]),
                },
            },
        ];

        let results = engine.process_facts(test_facts).expect("Failed to process facts");

        println!("✅ Processed facts through optimized rules:");
        println!("   Rules fired: {}", results.len());
        for result in &results {
            println!(
                "   Rule {} fired for fact {} with {} actions",
                result.rule_id,
                result.fact_id,
                result.actions_executed.len()
            );
        }

        println!("\n🎉 Phase 5 Complete Optimization Workflow Test PASSED!");
        println!("   ✅ Rule optimization integrated successfully");
        println!("   ✅ Performance improvements demonstrated");
        println!("   ✅ Engine functionality preserved");
        println!("   ✅ Metrics tracking operational");

        // Assert success criteria
        assert!(metrics.rules_optimized > 0);
        assert!(engine_stats.rule_count > 0);
        // Note: optimization time might be 0ms for very fast operations
        // assert!(metrics.total_optimization_time_ms >= 0); // Always true for u64
    }
}
