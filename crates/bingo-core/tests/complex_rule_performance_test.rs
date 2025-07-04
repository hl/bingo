//! Complex Rule Performance Tests with Calculation-Heavy Workloads
//!
//! These tests validate performance under high rule complexity scenarios where multiple
//! calculation-heavy rules are applied to large fact sets. Each rule performs mathematical
//! operations, string manipulations, and conditional logic to simulate real-world business
//! rule complexity.
//!
//! Test Scenarios:
//! - 100K facts + 200 calculation rules
//! - 200K facts + 200 calculation rules  
//! - 100K facts + 500 calculation rules
//! - 200K facts + 500 calculation rules
//!
//! Each rule includes Calculator DSL expressions for:
//! - Arithmetic calculations (salary, bonus, commission)
//! - String operations (name formatting, status derivation)
//! - Conditional logic (tier assignment, eligibility checks)
//! - Date/time calculations (tenure, performance periods)
//!
//! IMPORTANT: Performance tests MUST run in release mode for accurate results.

use crate::memory::MemoryTracker;
use bingo_core::*;
use std::collections::HashMap;

/// Generate calculation-heavy rules that simulate real business logic using calculators
fn create_calculation_rules(count: usize) -> Vec<Rule> {
    let mut rules = Vec::with_capacity(count);

    for i in 0..count {
        let rule = match i % 8 {
            0 => Rule {
                id: i as u64 + 1000,
                name: format!("Overtime Compliance Check {}", i),
                conditions: vec![Condition::Simple {
                    field: "hours_worked".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Float(35.0),
                }],
                actions: vec![Action {
                    action_type: ActionType::CallCalculator {
                        calculator_name: "threshold_checker".to_string(),
                        input_mapping: HashMap::from([
                            ("value".to_string(), "hours_worked".to_string()),
                            ("threshold".to_string(), "overtime_threshold".to_string()),
                            ("operator".to_string(), "GreaterThan".to_string()),
                        ]),
                        output_field: "overtime_compliance".to_string(),
                    },
                }],
            },
            1 => Rule {
                id: i as u64 + 1000,
                name: format!("Performance Threshold Check {}", i),
                conditions: vec![Condition::Simple {
                    field: "performance_score".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Float(60.0),
                }],
                actions: vec![Action {
                    action_type: ActionType::CallCalculator {
                        calculator_name: "threshold_checker".to_string(),
                        input_mapping: HashMap::from([
                            ("value".to_string(), "performance_score".to_string()),
                            ("threshold".to_string(), "target_performance".to_string()),
                            ("operator".to_string(), "GreaterThanOrEqual".to_string()),
                        ]),
                        output_field: "performance_compliance".to_string(),
                    },
                }],
            },
            2 => Rule {
                id: i as u64 + 1000,
                name: format!("Sales Limit Validation {}", i),
                conditions: vec![Condition::Simple {
                    field: "sales_role".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::Boolean(true),
                }],
                actions: vec![Action {
                    action_type: ActionType::CallCalculator {
                        calculator_name: "limit_validator".to_string(),
                        input_mapping: HashMap::from([
                            ("value".to_string(), "sales_amount".to_string()),
                            ("warning_threshold".to_string(), "sales_warning".to_string()),
                            (
                                "critical_threshold".to_string(),
                                "sales_critical".to_string(),
                            ),
                            ("max_threshold".to_string(), "sales_maximum".to_string()),
                        ]),
                        output_field: "sales_validation".to_string(),
                    },
                }],
            },
            3 => Rule {
                id: i as u64 + 1000,
                name: format!("Experience Threshold {}", i),
                conditions: vec![Condition::Simple {
                    field: "years_experience".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Integer(0),
                }],
                actions: vec![Action {
                    action_type: ActionType::CallCalculator {
                        calculator_name: "threshold_checker".to_string(),
                        input_mapping: HashMap::from([
                            ("value".to_string(), "years_experience".to_string()),
                            ("threshold".to_string(), "senior_threshold".to_string()),
                            ("operator".to_string(), "GreaterThanOrEqual".to_string()),
                        ]),
                        output_field: "seniority_check".to_string(),
                    },
                }],
            },
            4 => Rule {
                id: i as u64 + 1000,
                name: format!("Shift Hours Calculation {}", i),
                conditions: vec![Condition::Simple {
                    field: "shift_start".to_string(),
                    operator: Operator::NotEqual,
                    value: FactValue::String("".to_string()),
                }],
                actions: vec![Action {
                    action_type: ActionType::SetField {
                        field: "hours_calculated".to_string(),
                        value: FactValue::Boolean(true),
                    },
                }],
            },
            5 => Rule {
                id: i as u64 + 1000,
                name: format!("Time Difference Analysis {}", i),
                conditions: vec![Condition::Simple {
                    field: "schedule_start".to_string(),
                    operator: Operator::NotEqual,
                    value: FactValue::String("".to_string()),
                }],
                actions: vec![Action {
                    action_type: ActionType::SetField {
                        field: "schedule_analyzed".to_string(),
                        value: FactValue::Boolean(true),
                    },
                }],
            },
            6 => Rule {
                id: i as u64 + 1000,
                name: format!("Weekly Hours Limit {}", i),
                conditions: vec![Condition::Simple {
                    field: "employment_status".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("active".to_string()),
                }],
                actions: vec![Action {
                    action_type: ActionType::CallCalculator {
                        calculator_name: "limit_validator".to_string(),
                        input_mapping: HashMap::from([
                            ("value".to_string(), "weekly_hours".to_string()),
                            ("warning_threshold".to_string(), "warning_limit".to_string()),
                            (
                                "critical_threshold".to_string(),
                                "critical_limit".to_string(),
                            ),
                            ("max_threshold".to_string(), "legal_limit".to_string()),
                        ]),
                        output_field: "hours_compliance".to_string(),
                    },
                }],
            },
            7 => Rule {
                id: i as u64 + 1000,
                name: format!("Salary Range Check {}", i),
                conditions: vec![Condition::Simple {
                    field: "base_salary".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Float(0.0),
                }],
                actions: vec![Action {
                    action_type: ActionType::CallCalculator {
                        calculator_name: "threshold_checker".to_string(),
                        input_mapping: HashMap::from([
                            ("value".to_string(), "base_salary".to_string()),
                            ("threshold".to_string(), "minimum_wage".to_string()),
                            ("operator".to_string(), "GreaterThanOrEqual".to_string()),
                        ]),
                        output_field: "salary_compliance".to_string(),
                    },
                }],
            },
            _ => Rule {
                id: i as u64 + 1000,
                name: format!("Default Compliance Check {}", i),
                conditions: vec![Condition::Simple {
                    field: "active".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::Boolean(true),
                }],
                actions: vec![Action {
                    action_type: ActionType::CallCalculator {
                        calculator_name: "threshold_checker".to_string(),
                        input_mapping: HashMap::from([
                            ("value".to_string(), "metric_value".to_string()),
                            ("threshold".to_string(), "compliance_threshold".to_string()),
                            ("operator".to_string(), "LessThanOrEqual".to_string()),
                        ]),
                        output_field: "default_compliance".to_string(),
                    },
                }],
            },
        };
        rules.push(rule);
    }

    rules
}

/// Generate diverse employee facts for testing with calculator-required fields
fn create_employee_facts(count: usize) -> Vec<Fact> {
    (0..count)
        .map(|i| {
            let mut fields = HashMap::new();

            // Basic employee info
            fields.insert("employee_id".to_string(), FactValue::Integer(i as i64));
            fields.insert(
                "first_name".to_string(),
                FactValue::String(format!("Employee{}", i)),
            );
            fields.insert(
                "last_name".to_string(),
                FactValue::String(format!("Surname{}", i % 1000)),
            );

            // Employment details
            fields.insert(
                "employee_type".to_string(),
                FactValue::String(if i % 4 == 0 { "permanent" } else { "contract" }.to_string()),
            );
            fields.insert(
                "employment_status".to_string(),
                FactValue::String(if i % 10 == 0 { "inactive" } else { "active" }.to_string()),
            );
            fields.insert("sales_role".to_string(), FactValue::Boolean(i % 5 == 0));
            fields.insert("active".to_string(), FactValue::Boolean(i % 15 != 0));

            // Numeric performance data
            fields.insert(
                "years_experience".to_string(),
                FactValue::Integer((i % 25) as i64),
            );
            fields.insert(
                "performance_score".to_string(),
                FactValue::Float(50.0 + (i % 50) as f64),
            );
            fields.insert(
                "base_salary".to_string(),
                FactValue::Float(40000.0 + (i % 80000) as f64),
            );
            fields.insert(
                "hourly_rate".to_string(),
                FactValue::Float(20.0 + (i % 60) as f64),
            );
            fields.insert(
                "hours_worked".to_string(),
                FactValue::Float(25.0 + (i % 30) as f64),
            );
            fields.insert(
                "sales_amount".to_string(),
                FactValue::Float((i * 100) as f64),
            );
            fields.insert(
                "commission_rate".to_string(),
                FactValue::Float(1.0 + (i % 10) as f64),
            );

            // Calculator-specific threshold and limit fields
            fields.insert("overtime_threshold".to_string(), FactValue::Float(40.0));
            fields.insert("target_performance".to_string(), FactValue::Float(75.0));
            fields.insert("senior_threshold".to_string(), FactValue::Float(10.0));
            fields.insert("minimum_wage".to_string(), FactValue::Float(15.0));
            fields.insert(
                "metric_value".to_string(),
                FactValue::Float((i % 100) as f64),
            );
            fields.insert("compliance_threshold".to_string(), FactValue::Float(80.0));

            // Multi-tier validation thresholds
            fields.insert("sales_warning".to_string(), FactValue::Float(50000.0));
            fields.insert("sales_critical".to_string(), FactValue::Float(75000.0));
            fields.insert("sales_maximum".to_string(), FactValue::Float(100000.0));
            fields.insert("warning_limit".to_string(), FactValue::Float(35.0));
            fields.insert("critical_limit".to_string(), FactValue::Float(45.0));
            fields.insert("legal_limit".to_string(), FactValue::Float(50.0));
            fields.insert(
                "weekly_hours".to_string(),
                FactValue::Float(30.0 + (i % 25) as f64),
            );

            // DateTime fields for time calculations
            let base_datetime = "2024-01-01T08:00:00Z";
            let end_datetime = "2024-01-01T17:00:00Z";
            fields.insert(
                "shift_start".to_string(),
                FactValue::String(base_datetime.to_string()),
            );
            fields.insert(
                "shift_end".to_string(),
                FactValue::String(end_datetime.to_string()),
            );
            fields.insert(
                "schedule_start".to_string(),
                FactValue::String(base_datetime.to_string()),
            );
            fields.insert(
                "schedule_end".to_string(),
                FactValue::String(end_datetime.to_string()),
            );

            Fact {
                id: i as u64,
                external_id: None,
                timestamp: chrono::Utc::now(),
                data: FactData { fields },
            }
        })
        .collect()
}

#[test]
#[ignore] // Performance test - run with --release
fn test_100k_facts_200_rules_performance() {
    let memory_tracker = MemoryTracker::start().unwrap();
    let mut engine = BingoEngine::with_capacity(100_000).unwrap();

    println!("🧪 Testing 100K facts with 200 calculation-heavy rules...");

    // Add 200 calculation rules
    let rules = create_calculation_rules(200);
    for rule in rules {
        engine.add_rule(rule).unwrap();
    }

    // Generate 100K employee facts
    let facts = create_employee_facts(100_000);

    let start = std::time::Instant::now();
    let results = engine.process_facts(facts).unwrap();
    let elapsed = start.elapsed();

    let (start_stats, end_stats, memory_delta) = memory_tracker.finish().unwrap();

    println!(
        "✅ Processed 100K facts with 200 rules in {:?}, generated {} results",
        elapsed,
        results.len()
    );
    println!(
        "Memory usage: {} -> {}, Delta: {} bytes ({:.2} MB)",
        start_stats.format_rss(),
        end_stats.format_rss(),
        memory_delta,
        memory_delta as f64 / (1024.0 * 1024.0)
    );

    let stats = engine.get_stats();
    println!("Engine stats: {:?}", stats);

    // Performance targets for complex rule processing
    assert!(elapsed.as_secs() < 25, "Should complete within 25 seconds");
    assert!(
        memory_delta < 12_000_000_000,
        "Memory should stay under 12GB"
    );
    assert_eq!(stats.fact_count, 100_000);
    assert!(
        results.len() > 50_000,
        "Should generate substantial results"
    );

    println!(
        "📊 Performance: {:.0} facts/sec | {:.1} MB memory",
        100_000.0 / elapsed.as_secs_f64(),
        memory_delta as f64 / (1024.0 * 1024.0)
    );
}

#[test]
#[ignore] // Performance test - run with --release
fn test_200k_facts_200_rules_performance() {
    let memory_tracker = MemoryTracker::start().unwrap();
    let mut engine = BingoEngine::with_capacity(200_000).unwrap();

    println!("🧪 Testing 200K facts with 200 calculation-heavy rules...");

    // Add 200 calculation rules
    let rules = create_calculation_rules(200);
    for rule in rules {
        engine.add_rule(rule).unwrap();
    }

    // Generate 200K employee facts
    let facts = create_employee_facts(200_000);

    let start = std::time::Instant::now();
    let results = engine.process_facts(facts).unwrap();
    let elapsed = start.elapsed();

    let (start_stats, end_stats, memory_delta) = memory_tracker.finish().unwrap();

    println!(
        "✅ Processed 200K facts with 200 rules in {:?}, generated {} results",
        elapsed,
        results.len()
    );
    println!(
        "Memory usage: {} -> {}, Delta: {} bytes ({:.2} MB)",
        start_stats.format_rss(),
        end_stats.format_rss(),
        memory_delta,
        memory_delta as f64 / (1024.0 * 1024.0)
    );

    let stats = engine.get_stats();
    println!("Engine stats: {:?}", stats);

    // Performance targets for complex rule processing
    assert!(elapsed.as_secs() < 45, "Should complete within 45 seconds");
    assert!(memory_delta < 6_000_000_000, "Memory should stay under 6GB");
    assert_eq!(stats.fact_count, 200_000);
    assert!(
        results.len() > 100_000,
        "Should generate substantial results"
    );

    println!(
        "📊 Performance: {:.0} facts/sec | {:.1} MB memory",
        200_000.0 / elapsed.as_secs_f64(),
        memory_delta as f64 / (1024.0 * 1024.0)
    );
}

#[test]
#[ignore] // Performance test - run with --release
fn test_100k_facts_500_rules_performance() {
    let memory_tracker = MemoryTracker::start().unwrap();
    let mut engine = BingoEngine::with_capacity(100_000).unwrap();

    println!("🧪 Testing 100K facts with 500 calculation-heavy rules...");

    // Add 500 calculation rules
    let rules = create_calculation_rules(500);
    for rule in rules {
        engine.add_rule(rule).unwrap();
    }

    // Generate 100K employee facts
    let facts = create_employee_facts(100_000);

    let start = std::time::Instant::now();
    let results = engine.process_facts(facts).unwrap();
    let elapsed = start.elapsed();

    let (start_stats, end_stats, memory_delta) = memory_tracker.finish().unwrap();

    println!(
        "✅ Processed 100K facts with 500 rules in {:?}, generated {} results",
        elapsed,
        results.len()
    );
    println!(
        "Memory usage: {} -> {}, Delta: {} bytes ({:.2} MB)",
        start_stats.format_rss(),
        end_stats.format_rss(),
        memory_delta,
        memory_delta as f64 / (1024.0 * 1024.0)
    );

    let stats = engine.get_stats();
    println!("Engine stats: {:?}", stats);

    // Performance targets for high rule complexity
    assert!(elapsed.as_secs() < 55, "Should complete within 55 seconds");
    assert!(memory_delta < 5_000_000_000, "Memory should stay under 5GB");
    assert_eq!(stats.fact_count, 100_000);
    assert!(
        results.len() > 100_000,
        "Should generate substantial results"
    );

    println!(
        "📊 Performance: {:.0} facts/sec | {:.1} MB memory",
        100_000.0 / elapsed.as_secs_f64(),
        memory_delta as f64 / (1024.0 * 1024.0)
    );
}

#[test]
#[ignore] // Performance test - run with --release
fn test_200k_facts_500_rules_performance() {
    let memory_tracker = MemoryTracker::start().unwrap();
    let mut engine = BingoEngine::with_capacity(200_000).unwrap();

    println!("🧪 Testing 200K facts with 500 calculation-heavy rules...");

    // Add 500 calculation rules
    let rules = create_calculation_rules(500);
    for rule in rules {
        engine.add_rule(rule).unwrap();
    }

    // Generate 200K employee facts
    let facts = create_employee_facts(200_000);

    let start = std::time::Instant::now();
    let results = engine.process_facts(facts).unwrap();
    let elapsed = start.elapsed();

    let (start_stats, end_stats, memory_delta) = memory_tracker.finish().unwrap();

    println!(
        "✅ Processed 200K facts with 500 rules in {:?}, generated {} results",
        elapsed,
        results.len()
    );
    println!(
        "Memory usage: {} -> {}, Delta: {} bytes ({:.2} MB)",
        start_stats.format_rss(),
        end_stats.format_rss(),
        memory_delta,
        memory_delta as f64 / (1024.0 * 1024.0)
    );

    let stats = engine.get_stats();
    println!("Engine stats: {:?}", stats);

    // Performance targets for maximum complexity scenario
    assert!(
        elapsed.as_secs() < 120,
        "Should complete within 120 seconds"
    );
    assert!(memory_delta < 8_000_000_000, "Memory should stay under 8GB");
    assert_eq!(stats.fact_count, 200_000);
    assert!(
        results.len() > 200_000,
        "Should generate substantial results"
    );

    println!(
        "📊 Performance: {:.0} facts/sec | {:.1} MB memory",
        200_000.0 / elapsed.as_secs_f64(),
        memory_delta as f64 / (1024.0 * 1024.0)
    );
}
