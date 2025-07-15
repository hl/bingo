use bingo_core::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

/// Demonstration of complete client isolation in the Bingo engine
///
/// This test demonstrates that multiple clients can operate simultaneously
/// without any data mixing, ensuring complete isolation between clients.

#[test]
fn demonstrate_complete_client_isolation() {
    println!("🎯 DEMONSTRATION: Complete Client Isolation");
    println!("============================================");

    // Simulate 3 different clients with completely different business domains

    // CLIENT 1: E-commerce Order Processing
    let ecommerce_engine = Arc::new(BingoEngine::new().unwrap());

    // CLIENT 2: HR Payroll System
    let payroll_engine = Arc::new(BingoEngine::new().unwrap());

    // CLIENT 3: Financial Risk Management
    let risk_engine = Arc::new(BingoEngine::new().unwrap());

    println!("\n🏪 Setting up CLIENT 1: E-commerce Order Processing");
    let ecommerce_rule = Rule {
        id: 1,
        name: "High Value Order Discount".to_string(),
        conditions: vec![Condition::Simple {
            field: "order_total".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(1000),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "discount_percent".to_string(),
                value: FactValue::Integer(10),
            },
        }],
    };
    ecommerce_engine.add_rule(ecommerce_rule).unwrap();

    println!("💼 Setting up CLIENT 2: HR Payroll System");
    let payroll_rule = Rule {
        id: 1, // Same ID as ecommerce, but different engine = no conflict!
        name: "Overtime Calculation".to_string(),
        conditions: vec![Condition::Simple {
            field: "hours_worked".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(40),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "overtime_pay".to_string(),
                value: FactValue::Integer(150), // 1.5x rate
            },
        }],
    };
    payroll_engine.add_rule(payroll_rule).unwrap();

    println!("⚠️  Setting up CLIENT 3: Financial Risk Management");
    let risk_rule = Rule {
        id: 1, // Same ID again - still no conflict due to isolation!
        name: "High Risk Transaction Alert".to_string(),
        conditions: vec![Condition::Simple {
            field: "transaction_amount".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(10000),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "risk_level".to_string(),
                value: FactValue::String("HIGH".to_string()),
            },
        }],
    };
    risk_engine.add_rule(risk_rule).unwrap();

    println!("\n🚀 Processing facts simultaneously across all clients...");

    let start_time = Instant::now();
    let mut handles = vec![];

    // CLIENT 1: Process e-commerce orders
    let ecommerce_clone = Arc::clone(&ecommerce_engine);
    let ecommerce_handle = thread::spawn(move || {
        let orders: Vec<Fact> = vec![
            // Order 1: High value order (should get discount)
            Fact {
                id: 1001,
                external_id: Some("ORDER-2024-001".to_string()),
                timestamp: chrono::Utc::now(),
                data: FactData {
                    fields: HashMap::from([
                        (
                            "order_id".to_string(),
                            FactValue::String("ORDER-2024-001".to_string()),
                        ),
                        (
                            "customer_id".to_string(),
                            FactValue::String("CUST-12345".to_string()),
                        ),
                        ("order_total".to_string(), FactValue::Integer(1250)), // Triggers discount
                        ("currency".to_string(), FactValue::String("USD".to_string())),
                    ]),
                },
            },
            // Order 2: Regular order (no discount)
            Fact {
                id: 1002,
                external_id: Some("ORDER-2024-002".to_string()),
                timestamp: chrono::Utc::now(),
                data: FactData {
                    fields: HashMap::from([
                        (
                            "order_id".to_string(),
                            FactValue::String("ORDER-2024-002".to_string()),
                        ),
                        (
                            "customer_id".to_string(),
                            FactValue::String("CUST-67890".to_string()),
                        ),
                        ("order_total".to_string(), FactValue::Integer(750)), // No discount
                        ("currency".to_string(), FactValue::String("USD".to_string())),
                    ]),
                },
            },
        ];

        let results = ecommerce_clone.process_facts(orders).unwrap();
        println!(
            "🏪 E-commerce: Processed {} orders, {} rules fired",
            2,
            results.len()
        );
        (ecommerce_clone.get_stats(), results.len())
    });

    // CLIENT 2: Process payroll data
    let payroll_clone = Arc::clone(&payroll_engine);
    let payroll_handle = thread::spawn(move || {
        let timesheets: Vec<Fact> = vec![
            // Employee 1: Overtime hours
            Fact {
                id: 2001,
                external_id: Some("EMP-001-WEEK-42".to_string()),
                timestamp: chrono::Utc::now(),
                data: FactData {
                    fields: HashMap::from([
                        (
                            "employee_id".to_string(),
                            FactValue::String("EMP-001".to_string()),
                        ),
                        ("week".to_string(), FactValue::Integer(42)),
                        ("hours_worked".to_string(), FactValue::Integer(45)), // Overtime
                        ("hourly_rate".to_string(), FactValue::Integer(25)),
                    ]),
                },
            },
            // Employee 2: Regular hours
            Fact {
                id: 2002,
                external_id: Some("EMP-002-WEEK-42".to_string()),
                timestamp: chrono::Utc::now(),
                data: FactData {
                    fields: HashMap::from([
                        (
                            "employee_id".to_string(),
                            FactValue::String("EMP-002".to_string()),
                        ),
                        ("week".to_string(), FactValue::Integer(42)),
                        ("hours_worked".to_string(), FactValue::Integer(38)), // Regular
                        ("hourly_rate".to_string(), FactValue::Integer(30)),
                    ]),
                },
            },
        ];

        let results = payroll_clone.process_facts(timesheets).unwrap();
        println!(
            "💼 Payroll: Processed {} timesheets, {} rules fired",
            2,
            results.len()
        );
        (payroll_clone.get_stats(), results.len())
    });

    // CLIENT 3: Process financial transactions
    let risk_clone = Arc::clone(&risk_engine);
    let risk_handle = thread::spawn(move || {
        let transactions: Vec<Fact> = vec![
            // Transaction 1: High risk transaction
            Fact {
                id: 3001,
                external_id: Some("TXN-2024-12345".to_string()),
                timestamp: chrono::Utc::now(),
                data: FactData {
                    fields: HashMap::from([
                        (
                            "transaction_id".to_string(),
                            FactValue::String("TXN-2024-12345".to_string()),
                        ),
                        (
                            "account_id".to_string(),
                            FactValue::String("ACC-987654".to_string()),
                        ),
                        ("transaction_amount".to_string(), FactValue::Integer(25000)), // High risk
                        (
                            "transaction_type".to_string(),
                            FactValue::String("WIRE_TRANSFER".to_string()),
                        ),
                        ("country".to_string(), FactValue::String("US".to_string())),
                    ]),
                },
            },
            // Transaction 2: Normal transaction
            Fact {
                id: 3002,
                external_id: Some("TXN-2024-12346".to_string()),
                timestamp: chrono::Utc::now(),
                data: FactData {
                    fields: HashMap::from([
                        (
                            "transaction_id".to_string(),
                            FactValue::String("TXN-2024-12346".to_string()),
                        ),
                        (
                            "account_id".to_string(),
                            FactValue::String("ACC-123456".to_string()),
                        ),
                        ("transaction_amount".to_string(), FactValue::Integer(500)), // Normal
                        (
                            "transaction_type".to_string(),
                            FactValue::String("ACH".to_string()),
                        ),
                        ("country".to_string(), FactValue::String("US".to_string())),
                    ]),
                },
            },
        ];

        let results = risk_clone.process_facts(transactions).unwrap();
        println!(
            "⚠️  Risk: Processed {} transactions, {} rules fired",
            2,
            results.len()
        );
        (risk_clone.get_stats(), results.len())
    });

    handles.push(ecommerce_handle);
    handles.push(payroll_handle);
    handles.push(risk_handle);

    // Collect results
    let mut all_results = vec![];
    for handle in handles {
        all_results.push(handle.join().unwrap());
    }

    let total_time = start_time.elapsed();

    println!("\n📊 ISOLATION VERIFICATION RESULTS:");
    println!("===================================");

    let (ecommerce_stats, ecommerce_results) = &all_results[0];
    let (payroll_stats, payroll_results) = &all_results[1];
    let (risk_stats, risk_results) = &all_results[2];

    println!("🏪 E-commerce Engine:");
    println!(
        "   Rules: {} | Facts: {} | Results: {}",
        ecommerce_stats.rule_count, ecommerce_stats.fact_count, ecommerce_results
    );
    println!(
        "   Memory: {:.2} MB",
        ecommerce_stats.memory_usage_bytes as f64 / 1024.0 / 1024.0
    );

    println!("💼 Payroll Engine:");
    println!(
        "   Rules: {} | Facts: {} | Results: {}",
        payroll_stats.rule_count, payroll_stats.fact_count, payroll_results
    );
    println!(
        "   Memory: {:.2} MB",
        payroll_stats.memory_usage_bytes as f64 / 1024.0 / 1024.0
    );

    println!("⚠️  Risk Engine:");
    println!(
        "   Rules: {} | Facts: {} | Results: {}",
        risk_stats.rule_count, risk_stats.fact_count, risk_results
    );
    println!(
        "   Memory: {:.2} MB",
        risk_stats.memory_usage_bytes as f64 / 1024.0 / 1024.0
    );

    println!("\n⏱️  Performance:");
    println!("   Total processing time: {total_time:?}");
    println!(
        "   Concurrent throughput: {:.0} facts/sec",
        6.0 / total_time.as_secs_f64()
    );

    // CRITICAL ISOLATION ASSERTIONS
    println!("\n🔒 ISOLATION VERIFICATION:");

    // 1. Each client has exactly their own rules (no mixing)
    assert_eq!(
        ecommerce_stats.rule_count, 1,
        "E-commerce should have exactly 1 rule"
    );
    assert_eq!(
        payroll_stats.rule_count, 1,
        "Payroll should have exactly 1 rule"
    );
    assert_eq!(risk_stats.rule_count, 1, "Risk should have exactly 1 rule");
    println!("   ✅ Rule isolation: Each client has only their own rules");

    // 2. Each client has exactly their own facts (no mixing)
    assert_eq!(
        ecommerce_stats.fact_count, 2,
        "E-commerce should have exactly 2 facts"
    );
    assert_eq!(
        payroll_stats.fact_count, 2,
        "Payroll should have exactly 2 facts"
    );
    assert_eq!(risk_stats.fact_count, 2, "Risk should have exactly 2 facts");
    println!("   ✅ Fact isolation: Each client has only their own facts");

    // 3. Each client's rules only fired on their own facts
    assert_eq!(
        *ecommerce_results, 1,
        "E-commerce should have 1 result (1 order > $1000)"
    );
    assert_eq!(
        *payroll_results, 1,
        "Payroll should have 1 result (1 employee > 40 hours)"
    );
    assert_eq!(
        *risk_results, 1,
        "Risk should have 1 result (1 transaction > $10000)"
    );
    println!("   ✅ Processing isolation: Rules only fired on own data");

    // 4. Memory usage is isolated per client
    assert!(
        ecommerce_stats.memory_usage_bytes > 0,
        "E-commerce should use memory"
    );
    assert!(
        payroll_stats.memory_usage_bytes > 0,
        "Payroll should use memory"
    );
    assert!(risk_stats.memory_usage_bytes > 0, "Risk should use memory");
    println!("   ✅ Memory isolation: Each client has separate memory usage");

    println!("\n🎉 COMPLETE CLIENT ISOLATION VERIFIED!");
    println!("======================================");
    println!("✅ Multiple clients can operate simultaneously");
    println!("✅ No data mixing between clients");
    println!("✅ Each client has isolated rules, facts, and results");
    println!("✅ Same rule IDs in different clients don't conflict");
    println!("✅ Concurrent processing maintains perfect isolation");
    println!("✅ Memory usage is tracked separately per client");

    // Performance verification
    assert!(
        total_time.as_millis() < 1000,
        "Should complete in under 1 second"
    );
    println!("✅ High performance maintained with isolation");
}
