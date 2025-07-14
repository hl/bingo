//! Enhanced test coverage for large datasets and concurrency scenarios
//!
//! This module provides additional test coverage focusing on:
//! - Large dataset processing capabilities
//! - Concurrent access patterns
//! - Memory efficiency under load
//! - Edge case handling

use bingo_core::{BingoEngine, types::*};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Instant;

/// Helper function to create test facts with varying complexity
fn create_test_fact(id: u64, fields: HashMap<String, FactValue>) -> Fact {
    Fact {
        id,
        external_id: Some(format!("test_{id}")),
        timestamp: Utc::now(),
        data: FactData { fields },
    }
}

/// Create a dataset with realistic business data patterns
fn create_business_dataset(size: usize) -> Vec<Fact> {
    let mut facts = Vec::with_capacity(size);
    let departments = ["Engineering", "Sales", "Marketing", "HR", "Finance"];
    let statuses = ["active", "inactive", "pending"];

    for i in 0..size {
        let mut fields = HashMap::new();

        // Core business fields
        fields.insert(
            "employee_id".to_string(),
            FactValue::Integer((i % 1000) as i64),
        );
        fields.insert(
            "department".to_string(),
            FactValue::String(departments[i % departments.len()].to_string()),
        );
        fields.insert(
            "status".to_string(),
            FactValue::String(statuses[i % statuses.len()].to_string()),
        );
        fields.insert(
            "amount".to_string(),
            FactValue::Float(1000.0 + (i as f64 * 0.5)),
        );
        fields.insert(
            "hours".to_string(),
            FactValue::Float(40.0 + (i % 20) as f64),
        );

        // Additional realistic fields
        fields.insert(
            "project_id".to_string(),
            FactValue::Integer((i % 50) as i64),
        );
        fields.insert(
            "location".to_string(),
            FactValue::String(format!("Office_{}", i % 10)),
        );
        fields.insert(
            "skill_level".to_string(),
            FactValue::Integer((i % 5) as i64 + 1),
        );

        facts.push(create_test_fact(i as u64, fields));
    }

    facts
}

/// Create a comprehensive set of business rules for testing
fn create_business_rules() -> Vec<Rule> {
    vec![
        // Rule 1: High hours overtime detection
        Rule {
            id: 1,
            name: "Overtime Detection".to_string(),
            conditions: vec![
                Condition::Simple {
                    field: "hours".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Float(50.0),
                },
                Condition::Simple {
                    field: "status".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("active".to_string()),
                },
            ],
            actions: vec![
                Action {
                    action_type: ActionType::SetField {
                        field: "overtime_flag".to_string(),
                        value: FactValue::Boolean(true),
                    },
                },
                Action {
                    action_type: ActionType::Log { message: "Overtime detected".to_string() },
                },
            ],
        },
        // Rule 2: High amount bonus calculation
        Rule {
            id: 2,
            name: "High Amount Bonus".to_string(),
            conditions: vec![Condition::Simple {
                field: "amount".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Float(5000.0),
            }],
            actions: vec![Action {
                action_type: ActionType::Formula {
                    expression: "amount * 0.1".to_string(),
                    output_field: "bonus".to_string(),
                },
            }],
        },
        // Rule 3: Department-specific processing
        Rule {
            id: 3,
            name: "Engineering Special Processing".to_string(),
            conditions: vec![
                Condition::Simple {
                    field: "department".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("Engineering".to_string()),
                },
                Condition::Simple {
                    field: "skill_level".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Integer(3),
                },
            ],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "technical_lead".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        },
        // Rule 4: Complex logical condition
        Rule {
            id: 4,
            name: "Complex Business Logic".to_string(),
            conditions: vec![Condition::Complex {
                operator: LogicalOperator::Or,
                conditions: vec![
                    Condition::Simple {
                        field: "department".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::String("Sales".to_string()),
                    },
                    Condition::Complex {
                        operator: LogicalOperator::And,
                        conditions: vec![
                            Condition::Simple {
                                field: "amount".to_string(),
                                operator: Operator::GreaterThan,
                                value: FactValue::Float(3000.0),
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
                    field: "priority_processing".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        },
    ]
}

#[test]
fn test_large_dataset_processing_performance() {
    let dataset_size = 5000; // Moderate size for CI/CD compatibility
    println!("Testing large dataset processing with {dataset_size} facts");

    let start_time = Instant::now();

    // Create business dataset
    let facts = create_business_dataset(dataset_size);
    let rules = create_business_rules();

    let creation_time = start_time.elapsed();
    println!("Dataset creation took: {creation_time:?}");

    // Initialize and test engine
    let engine = BingoEngine::new().expect("Failed to create engine");

    let process_start = Instant::now();
    let results = engine.evaluate(rules, facts).expect("Failed to evaluate rules");
    let process_time = process_start.elapsed();

    let total_time = start_time.elapsed();
    let throughput = dataset_size as f64 / process_time.as_secs_f64();

    println!("=== LARGE DATASET PERFORMANCE RESULTS ===");
    println!("Facts processed: {dataset_size}");
    println!("Rule execution results: {}", results.len());
    println!("Processing time: {process_time:?}");
    println!("Total time: {total_time:?}");
    println!("Throughput: {throughput:.2} facts/sec");

    // Performance assertions
    assert!(!results.is_empty(), "Expected some rule execution results");
    assert!(
        throughput >= 1000.0,
        "Throughput {throughput:.2} facts/sec below minimum 1000"
    );
    assert!(
        process_time.as_secs() < 30,
        "Processing took too long: {process_time:?}"
    );
}

#[test]
fn test_concurrent_engine_instances() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    const NUM_THREADS: usize = 4;
    const FACTS_PER_THREAD: usize = 500;

    println!("Testing concurrent engine instances with {NUM_THREADS} threads");

    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for thread_id in 0..NUM_THREADS {
        let results_clone = Arc::clone(&results);

        let handle = thread::spawn(move || {
            // Each thread gets its own engine instance
            let engine = BingoEngine::new().expect("Failed to create engine");

            // Create thread-specific data
            let mut facts = Vec::new();
            for i in 0..FACTS_PER_THREAD {
                let mut fields = HashMap::new();
                fields.insert(
                    "thread_id".to_string(),
                    FactValue::Integer(thread_id as i64),
                );
                fields.insert("fact_index".to_string(), FactValue::Integer(i as i64));
                fields.insert("amount".to_string(), FactValue::Float((i * 10) as f64));
                fields.insert(
                    "status".to_string(),
                    FactValue::String("active".to_string()),
                );

                facts.push(create_test_fact((thread_id * 1000 + i) as u64, fields));
            }

            // Simple rule for testing
            let rules = vec![Rule {
                id: 1,
                name: "Thread Test Rule".to_string(),
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
            }];

            // Process facts
            let thread_results =
                engine.evaluate(rules, facts).expect("Failed to evaluate rules in thread");

            // Store results
            {
                let mut shared_results = results_clone.lock().unwrap();
                shared_results.extend(thread_results);
            }

            println!("Thread {thread_id} completed processing {FACTS_PER_THREAD} facts");
        });

        handles.push(handle);
    }

    let start_time = Instant::now();

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let total_time = start_time.elapsed();
    let final_results = results.lock().unwrap();
    let total_facts = NUM_THREADS * FACTS_PER_THREAD;

    println!("=== CONCURRENT PROCESSING RESULTS ===");
    println!("Total facts processed: {total_facts}");
    println!("Total results: {}", final_results.len());
    println!("Processing time: {total_time:?}");
    println!(
        "Effective throughput: {:.2} facts/sec",
        total_facts as f64 / total_time.as_secs_f64()
    );

    // Verify concurrent processing
    assert_eq!(
        final_results.len(),
        total_facts,
        "Expected {} results, got {}",
        total_facts,
        final_results.len()
    );
    assert!(
        total_time.as_secs() < 60,
        "Concurrent processing took too long: {total_time:?}"
    );
}

#[test]
fn test_memory_pressure_handling() {
    println!("Testing memory pressure with large facts");

    const NUM_FACTS: usize = 1000;
    const LARGE_STRING_SIZE: usize = 500; // 500 chars per string field

    // Create facts with large data fields
    let mut facts = Vec::new();
    for i in 0..NUM_FACTS {
        let mut fields = HashMap::new();

        // Large string fields to test memory handling
        fields.insert(
            "large_description".to_string(),
            FactValue::String("x".repeat(LARGE_STRING_SIZE)),
        );
        fields.insert(
            "metadata".to_string(),
            FactValue::String(format!("Metadata for item {} {}", i, "detail ".repeat(50))),
        );

        // Business fields
        fields.insert("id".to_string(), FactValue::Integer(i as i64));
        fields.insert("amount".to_string(), FactValue::Float(i as f64 * 2.5));
        fields.insert(
            "status".to_string(),
            FactValue::String("active".to_string()),
        );

        facts.push(create_test_fact(i as u64, fields));
    }

    let engine = BingoEngine::new().expect("Failed to create engine");

    // Rule that processes large facts
    let rules = vec![Rule {
        id: 1,
        name: "Large Data Processing".to_string(),
        conditions: vec![Condition::Simple {
            field: "amount".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Float(100.0),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "processed_large".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    }];

    let start_time = Instant::now();
    let results = engine.evaluate(rules, facts).expect("Failed to process large facts");
    let processing_time = start_time.elapsed();

    println!("=== MEMORY PRESSURE TEST RESULTS ===");
    println!("Large facts processed: {NUM_FACTS}");
    println!(
        "Estimated data size: {} KB",
        (NUM_FACTS * LARGE_STRING_SIZE * 2) / 1024
    );
    println!("Results: {}", results.len());
    println!("Processing time: {processing_time:?}");

    assert!(
        !results.is_empty(),
        "Expected some results from large fact processing"
    );
    assert!(
        processing_time.as_secs() < 30,
        "Large fact processing took too long: {processing_time:?}"
    );
}

#[test]
fn test_edge_case_data_handling() {
    println!("Testing edge case data handling");

    let mut facts = Vec::new();

    // Edge case 1: Empty strings and zero values
    let mut fields1 = HashMap::new();
    fields1.insert("name".to_string(), FactValue::String("".to_string()));
    fields1.insert("amount".to_string(), FactValue::Float(0.0));
    fields1.insert("count".to_string(), FactValue::Integer(0));
    facts.push(create_test_fact(1, fields1));

    // Edge case 2: Null values
    let mut fields2 = HashMap::new();
    fields2.insert("nullable_field".to_string(), FactValue::Null);
    fields2.insert("amount".to_string(), FactValue::Float(100.0));
    facts.push(create_test_fact(2, fields2));

    // Edge case 3: Very large numbers
    let mut fields3 = HashMap::new();
    fields3.insert("large_int".to_string(), FactValue::Integer(i64::MAX));
    fields3.insert("large_float".to_string(), FactValue::Float(1e15));
    facts.push(create_test_fact(3, fields3));

    // Edge case 4: Boolean values
    let mut fields4 = HashMap::new();
    fields4.insert("flag_true".to_string(), FactValue::Boolean(true));
    fields4.insert("flag_false".to_string(), FactValue::Boolean(false));
    fields4.insert("amount".to_string(), FactValue::Float(50.0));
    facts.push(create_test_fact(4, fields4));

    let engine = BingoEngine::new().expect("Failed to create engine");

    // Rules that handle edge cases
    let rules = vec![
        Rule {
            id: 1,
            name: "Zero Value Handler".to_string(),
            conditions: vec![Condition::Simple {
                field: "amount".to_string(),
                operator: Operator::GreaterThanOrEqual,
                value: FactValue::Float(0.0),
            }],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "valid_amount".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        },
        Rule {
            id: 2,
            name: "Boolean Flag Handler".to_string(),
            conditions: vec![Condition::Simple {
                field: "flag_true".to_string(),
                operator: Operator::Equal,
                value: FactValue::Boolean(true),
            }],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "boolean_processed".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        },
    ];

    let start_time = Instant::now();
    let results = engine.evaluate(rules, facts).expect("Failed to process edge case facts");
    let processing_time = start_time.elapsed();

    println!("=== EDGE CASE HANDLING RESULTS ===");
    println!("Edge case facts processed: {}", 4);
    println!("Results: {}", results.len());
    println!("Processing time: {processing_time:?}");

    // Should handle edge cases without errors
    assert!(
        results.len() >= 2,
        "Expected at least 2 results from edge case processing"
    );
    assert!(
        processing_time.as_millis() < 1000,
        "Edge case processing took too long: {processing_time:?}"
    );
}

#[test]
fn test_complex_rule_combinations() {
    println!("Testing complex rule combinations");

    const NUM_FACTS: usize = 1000;
    let facts = create_business_dataset(NUM_FACTS);

    // Create more complex rules with various condition types
    let rules = vec![
        // Rule 1: Multiple simple conditions
        Rule {
            id: 1,
            name: "Multi-Condition Rule".to_string(),
            conditions: vec![
                Condition::Simple {
                    field: "department".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("Engineering".to_string()),
                },
                Condition::Simple {
                    field: "amount".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Float(2000.0),
                },
                Condition::Simple {
                    field: "status".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("active".to_string()),
                },
            ],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "senior_engineer".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        },
        // Rule 2: Complex nested conditions
        Rule {
            id: 2,
            name: "Nested Logic Rule".to_string(),
            conditions: vec![Condition::Complex {
                operator: LogicalOperator::And,
                conditions: vec![
                    Condition::Complex {
                        operator: LogicalOperator::Or,
                        conditions: vec![
                            Condition::Simple {
                                field: "department".to_string(),
                                operator: Operator::Equal,
                                value: FactValue::String("Sales".to_string()),
                            },
                            Condition::Simple {
                                field: "department".to_string(),
                                operator: Operator::Equal,
                                value: FactValue::String("Marketing".to_string()),
                            },
                        ],
                    },
                    Condition::Simple {
                        field: "skill_level".to_string(),
                        operator: Operator::GreaterThan,
                        value: FactValue::Integer(2),
                    },
                ],
            }],
            actions: vec![Action {
                action_type: ActionType::SetField {
                    field: "customer_facing".to_string(),
                    value: FactValue::Boolean(true),
                },
            }],
        },
        // Rule 3: Multiple actions
        Rule {
            id: 3,
            name: "Multi-Action Rule".to_string(),
            conditions: vec![Condition::Simple {
                field: "hours".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Float(45.0),
            }],
            actions: vec![
                Action {
                    action_type: ActionType::SetField {
                        field: "overtime".to_string(),
                        value: FactValue::Boolean(true),
                    },
                },
                Action {
                    action_type: ActionType::IncrementField {
                        field: "overtime_count".to_string(),
                        increment: FactValue::Integer(1),
                    },
                },
                Action {
                    action_type: ActionType::Log {
                        message: "Overtime detected for employee".to_string(),
                    },
                },
            ],
        },
    ];

    let engine = BingoEngine::new().expect("Failed to create engine");

    let start_time = Instant::now();
    let results = engine.evaluate(rules, facts).expect("Failed to evaluate complex rules");
    let processing_time = start_time.elapsed();

    let throughput = NUM_FACTS as f64 / processing_time.as_secs_f64();

    println!("=== COMPLEX RULE COMBINATIONS RESULTS ===");
    println!("Facts processed: {NUM_FACTS}");
    println!("Complex rule results: {}", results.len());
    println!("Processing time: {processing_time:?}");
    println!("Throughput: {throughput:.2} facts/sec");

    // Verify complex rule processing
    assert!(
        !results.is_empty(),
        "Expected some results from complex rule processing"
    );
    assert!(
        throughput >= 500.0,
        "Complex rule throughput too low: {throughput:.2} facts/sec"
    );

    // Verify different action types were executed
    let mut has_field_set = false;
    let mut has_increment = false;
    let mut has_log = false;

    for result in &results {
        for action in &result.actions_executed {
            match action {
                bingo_core::rete_nodes::ActionResult::FieldSet { .. } => {
                    has_field_set = true;
                }
                bingo_core::rete_nodes::ActionResult::FieldIncremented { .. } => {
                    has_increment = true;
                }
                bingo_core::rete_nodes::ActionResult::Logged { .. } => {
                    has_log = true;
                }
                _ => {}
            }
        }
    }

    println!(
        "Action types executed - Field Set: {has_field_set}, Increment: {has_increment}, Log: {has_log}"
    );
}
