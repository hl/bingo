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
        // 70% of rules only update existing facts (ratio 1:1)
        // 30% of rules create new facts (ratio 1:2)
        // Overall: 70% * 1 + 30% * 2 = 1.3 ratio
        let is_update_only = (i * 10 / count) < 7; // First 70% are update-only

        let rule = if is_update_only {
            // Update-only rules: modify existing facts with selective conditions
            match i % 4 {
                0 => Rule {
                    id: i as u64 + 1000,
                    name: format!("Update Hours Status {i}"),
                    conditions: vec![Condition::Simple {
                        field: "employee_mod_10".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::Integer((i % 10) as i64), // Each rule matches 10% of employees
                    }],
                    actions: vec![Action {
                        action_type: ActionType::SetField {
                            field: "hours_processed".to_string(),
                            value: FactValue::Boolean(true),
                        },
                    }],
                },
                1 => Rule {
                    id: i as u64 + 1000,
                    name: format!("Update Performance Score {i}"),
                    conditions: vec![Condition::Simple {
                        field: "employee_mod_10".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::Integer((i % 10) as i64), // Each rule matches 10% of employees
                    }],
                    actions: vec![Action {
                        action_type: ActionType::SetField {
                            field: "performance_updated".to_string(),
                            value: FactValue::Boolean(true),
                        },
                    }],
                },
                2 => Rule {
                    id: i as u64 + 1000,
                    name: format!("Update Compliance Status {i}"),
                    conditions: vec![Condition::Simple {
                        field: "employee_mod_10".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::Integer((i % 10) as i64), // Each rule matches 10% of employees
                    }],
                    actions: vec![Action {
                        action_type: ActionType::SetField {
                            field: "compliance_checked".to_string(),
                            value: FactValue::Boolean(true),
                        },
                    }],
                },
                _ => Rule {
                    id: i as u64 + 1000,
                    name: format!("Update Salary Status {i}"),
                    conditions: vec![Condition::Simple {
                        field: "employee_mod_10".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::Integer((i % 10) as i64), // Each rule matches 10% of employees
                    }],
                    actions: vec![Action {
                        action_type: ActionType::SetField {
                            field: "salary_validated".to_string(),
                            value: FactValue::Boolean(true),
                        },
                    }],
                },
            }
        } else {
            // New fact creation rules: create additional facts for specific conditions
            match i % 3 {
                0 => Rule {
                    id: i as u64 + 1000,
                    name: format!("Create Overtime Record {i}"),
                    conditions: vec![Condition::Simple {
                        field: "employee_mod_10".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::Integer((i % 10) as i64), // Each rule matches 10% of employees
                    }],
                    actions: vec![
                        Action {
                            action_type: ActionType::SetField {
                                field: "overtime_eligible".to_string(),
                                value: FactValue::Boolean(true),
                            },
                        },
                        Action {
                            action_type: ActionType::CreateFact {
                                data: FactData {
                                    fields: HashMap::from([
                                        (
                                            "fact_type".to_string(),
                                            FactValue::String("overtime_record".to_string()),
                                        ),
                                        (
                                            "employee_id".to_string(),
                                            FactValue::Integer((i % 10000) as i64),
                                        ),
                                        ("overtime_hours".to_string(), FactValue::Float(5.0)),
                                    ]),
                                },
                            },
                        },
                    ],
                },
                1 => Rule {
                    id: i as u64 + 1000,
                    name: format!("Create Holiday Pay {i}"),
                    conditions: vec![Condition::Simple {
                        field: "employee_mod_10".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::Integer((i % 10) as i64), // Each rule matches 10% of employees
                    }],
                    actions: vec![
                        Action {
                            action_type: ActionType::SetField {
                                field: "holiday_eligible".to_string(),
                                value: FactValue::Boolean(true),
                            },
                        },
                        Action {
                            action_type: ActionType::CreateFact {
                                data: FactData {
                                    fields: HashMap::from([
                                        (
                                            "fact_type".to_string(),
                                            FactValue::String("holiday_pay".to_string()),
                                        ),
                                        (
                                            "employee_id".to_string(),
                                            FactValue::Integer((i % 10000) as i64),
                                        ),
                                        ("holiday_amount".to_string(), FactValue::Float(100.0)),
                                    ]),
                                },
                            },
                        },
                    ],
                },
                _ => Rule {
                    id: i as u64 + 1000,
                    name: format!("Create Bonus Record {i}"),
                    conditions: vec![Condition::Simple {
                        field: "employee_mod_10".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::Integer((i % 10) as i64), // Each rule matches 10% of employees
                    }],
                    actions: vec![
                        Action {
                            action_type: ActionType::SetField {
                                field: "bonus_eligible".to_string(),
                                value: FactValue::Boolean(true),
                            },
                        },
                        Action {
                            action_type: ActionType::CreateFact {
                                data: FactData {
                                    fields: HashMap::from([
                                        (
                                            "fact_type".to_string(),
                                            FactValue::String("bonus_record".to_string()),
                                        ),
                                        (
                                            "employee_id".to_string(),
                                            FactValue::Integer((i % 10000) as i64),
                                        ),
                                        ("bonus_amount".to_string(), FactValue::Float(50.0)),
                                    ]),
                                },
                            },
                        },
                    ],
                },
            }
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
                "employee_mod_10".to_string(),
                FactValue::Integer((i % 10) as i64),
            );
            fields.insert(
                "first_name".to_string(),
                FactValue::String(format!("Employee{i}")),
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
    let engine = BingoEngine::with_capacity(100_000).unwrap();

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
    println!("Engine stats: {stats:?}");

    // Performance targets for complex rule processing
    assert!(elapsed.as_secs() < 25, "Should complete within 25 seconds");

    // Original facts + new facts created by 30% of rules
    // Each fact-creating rule matches ~10K employees (10% of 100K)
    let fact_creating_rules = 200 * 30 / 100; // 60 rules
    let employees_per_rule = 100_000 / 10; // 10K employees per rule (10 mod values)
    let expected_facts = 100_000 + (fact_creating_rules * employees_per_rule); // 100K + 600K
    assert_eq!(stats.fact_count, expected_facts);

    // Realistic payroll expectation: Each employee matches multiple rules
    // With 200 rules and 10% match rate: 100K facts × 20 rules/fact = 2M results
    let expected_max_results = 100_000 * 25; // 25x for multiple rule matches per employee
    assert!(
        results.len() <= expected_max_results,
        "Results should be realistic: {} results from 100K facts ({}x ratio) - expected max 60x ratio",
        results.len(),
        results.len() / 100_000
    );

    // Memory scales with fact count for rule evaluation, not just results
    // This is expected for RETE networks that store intermediate state
    let memory_per_fact = memory_delta as f64 / stats.fact_count as f64;
    assert!(
        memory_per_fact < 100_000.0, // Less than 100KB per fact processed
        "Memory per fact too high: {memory_per_fact:.2} bytes/fact"
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
    let engine = BingoEngine::with_capacity(200_000).unwrap();

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
    println!("Engine stats: {stats:?}");

    // Performance targets for complex rule processing
    assert!(elapsed.as_secs() < 45, "Should complete within 45 seconds");

    // Original facts + new facts created by 30% of rules
    // Each fact-creating rule matches ~20K employees (10% of 200K)
    let fact_creating_rules = 200 * 30 / 100; // 60 rules
    let employees_per_rule = 200_000 / 10; // 20K employees per rule
    let expected_facts = 200_000 + (fact_creating_rules * employees_per_rule); // 200K + 1.2M
    assert_eq!(stats.fact_count, expected_facts);

    // Realistic payroll expectation: Each employee matches multiple rules
    // With 200 rules and 10% match rate: 200K facts × 20 rules/fact = 4M results
    let expected_max_results = 200_000 * 25; // 25x for multiple rule matches per employee
    assert!(
        results.len() <= expected_max_results,
        "Results should be realistic: {} results from 200K facts ({}x ratio) - expected max 25x ratio",
        results.len(),
        results.len() / 200_000
    );

    // Memory scales with fact count for rule evaluation, not just results
    // This is expected for RETE networks that store intermediate state
    let memory_per_fact = memory_delta as f64 / stats.fact_count as f64;
    assert!(
        memory_per_fact < 100_000.0, // Less than 100KB per fact processed
        "Memory per fact too high: {memory_per_fact:.2} bytes/fact"
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
    let engine = BingoEngine::with_capacity(100_000).unwrap();

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
    println!("Engine stats: {stats:?}");

    // Performance targets for high rule complexity
    assert!(elapsed.as_secs() < 55, "Should complete within 55 seconds");

    // Original facts + new facts created by 30% of rules
    // Each fact-creating rule matches ~10K employees (10% of 100K)
    let fact_creating_rules = 500 * 30 / 100; // 150 rules
    let employees_per_rule = 100_000 / 10; // 10K employees per rule
    let expected_facts = 100_000 + (fact_creating_rules * employees_per_rule); // 100K + 1.5M
    assert_eq!(stats.fact_count, expected_facts);

    // Realistic payroll expectation: Each employee matches multiple rules
    // With 500 rules and 10% match rate: 100K facts × 50 rules/fact = 5M results
    let expected_max_results = 100_000 * 60; // 60x for multiple rule matches per employee
    assert!(
        results.len() <= expected_max_results,
        "Results should be realistic: {} results from 100K facts ({}x ratio) - expected max 60x ratio",
        results.len(),
        results.len() / 100_000
    );

    // Memory scales with fact count for rule evaluation, not just results
    // This is expected for RETE networks that store intermediate state
    let memory_per_fact = memory_delta as f64 / stats.fact_count as f64;
    assert!(
        memory_per_fact < 100_000.0, // Less than 100KB per fact processed
        "Memory per fact too high: {memory_per_fact:.2} bytes/fact"
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
    let engine = BingoEngine::with_capacity(200_000).unwrap();

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
    println!("Engine stats: {stats:?}");

    // Performance targets for maximum complexity scenario
    assert!(
        elapsed.as_secs() < 120,
        "Should complete within 120 seconds"
    );

    // Original facts + new facts created by 30% of rules
    // Each fact-creating rule matches ~20K employees (10% of 200K)
    let fact_creating_rules = 500 * 30 / 100; // 150 rules
    let employees_per_rule = 200_000 / 10; // 20K employees per rule
    let expected_facts = 200_000 + (fact_creating_rules * employees_per_rule); // 200K + 3M
    assert_eq!(stats.fact_count, expected_facts);

    // Realistic payroll expectation: Each employee matches multiple rules
    // With 500 rules and 10% match rate: 200K facts × 50 rules/fact = 10M results
    let expected_max_results = 200_000 * 60; // 60x for multiple rule matches per employee
    assert!(
        results.len() <= expected_max_results,
        "Results should be realistic: {} results from 200K facts ({}x ratio) - expected max 25x ratio",
        results.len(),
        results.len() / 200_000
    );

    // Memory scales with fact count for rule evaluation, not just results
    // This is expected for RETE networks that store intermediate state
    let memory_per_fact = memory_delta as f64 / stats.fact_count as f64;
    assert!(
        memory_per_fact < 100_000.0, // Less than 100KB per fact processed
        "Memory per fact too high: {memory_per_fact:.2} bytes/fact"
    );

    println!(
        "📊 Performance: {:.0} facts/sec | {:.1} MB memory",
        200_000.0 / elapsed.as_secs_f64(),
        memory_delta as f64 / (1024.0 * 1024.0)
    );
}
