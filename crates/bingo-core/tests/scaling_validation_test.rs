//! Scaling Validation Tests with Realistic Production Benchmarks
//!
//! These tests validate performance against realistic production targets rather than
//! aggressive theoretical limits. Targets are based on:
//!
//! 1. Real-world rule engine performance in enterprise environments
//! 2. Acceptable latency for batch processing scenarios
//! 3. Memory constraints in typical server configurations
//! 4. Performance characteristics of similar systems (Drools, etc.)
//!
//! IMPORTANT: Performance tests MUST run in release mode for accurate results.
//! In debug mode, performance is 10x slower due to lack of optimizations.
//!
//! Performance Targets (Release Mode):
//! - 100K facts: 3 seconds (33K facts/sec) - good for CI
//! - 200K facts: 6 seconds (CI-appropriate test)  
//! - 500K facts: 10 seconds, 2.5GB memory (CI: ignored - too resource intensive)
//! - 1M facts: 30 seconds, 4GB memory (CI: ignored - too resource intensive)

use crate::memory::MemoryTracker;
use bingo_core::*;
use std::collections::HashMap;

#[test]
fn test_100k_fact_scaling() {
    let mut engine = BingoEngine::with_capacity(100_000).unwrap();

    // Add a simple rule
    let rule = Rule {
        id: 1,
        name: "Status Rule".to_string(),
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
    };

    engine.add_rule(rule).unwrap();

    // Generate 100K facts
    let facts: Vec<Fact> = (0..100_000)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("entity_id".to_string(), FactValue::Integer(i as i64));
            fields.insert(
                "status".to_string(),
                FactValue::String(if i % 3 == 0 { "active" } else { "inactive" }.to_string()),
            );
            fields.insert(
                "category".to_string(),
                FactValue::String(format!("cat_{}", i % 100)),
            );

            Fact {
                id: i as u64,
                external_id: None,
                timestamp: chrono::Utc::now(),
                data: FactData { fields },
            }
        })
        .collect();

    let start = std::time::Instant::now();
    let results = engine.process_facts(facts).unwrap();
    let elapsed = start.elapsed();

    println!(
        "✅ Processed 100K facts in {:?} (target: <3s), generated {} results",
        elapsed,
        results.len()
    );

    let stats = engine.get_stats();
    println!("Final engine stats: {:?}", stats);

    // Validate performance and results - Realistic production targets
    assert!(
        elapsed.as_millis() < 3000,
        "Should process 100K facts under 3 seconds (production target)"
    );
    assert_eq!(stats.fact_count, 100_000);
    assert!(
        results.len() > 30_000,
        "Should generate results for ~33% of facts"
    );
}

#[test]
#[ignore] // Skip in CI - too resource intensive for limited CI runners  
fn test_500k_fact_scaling() {
    let memory_tracker = MemoryTracker::start().unwrap();
    let mut engine = BingoEngine::with_capacity(500_000).unwrap();

    // Add a simple rule
    let rule = Rule {
        id: 1,
        name: "Category Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "category".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("cat_1".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "flagged".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Generate 500K facts
    let facts: Vec<Fact> = (0..500_000)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("entity_id".to_string(), FactValue::Integer(i as i64));
            fields.insert(
                "category".to_string(),
                FactValue::String(format!("cat_{}", i % 100)),
            );
            fields.insert("value".to_string(), FactValue::Float(i as f64 * 1.5));

            Fact {
                id: i as u64,
                external_id: None,
                timestamp: chrono::Utc::now(),
                data: FactData { fields },
            }
        })
        .collect();

    let start = std::time::Instant::now();
    let results = engine.process_facts(facts).unwrap();
    let elapsed = start.elapsed();

    let (start_stats, end_stats, memory_delta) = memory_tracker.finish().unwrap();

    println!(
        "✅ Processed 500K facts in {:?} (target: <10s), generated {} results",
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
    println!("Final engine stats: {:?}", stats);

    // Validate performance and memory usage - Realistic production targets
    assert!(
        elapsed.as_millis() < 10000,
        "Should process 500K facts under 10 seconds (production target)"
    );
    assert!(
        memory_delta < 2_500_000_000,
        "Memory usage should be under 2.5GB for 500K facts (production target)"
    );
    assert_eq!(stats.fact_count, 500_000);
    assert!(
        results.len() > 4_000,
        "Should generate results for cat_1 matches"
    );
}

#[test]
#[ignore] // Skip in CI - too resource intensive for limited CI runners
fn test_1m_fact_scaling() {
    let memory_tracker = MemoryTracker::start().unwrap();
    let mut engine = BingoEngine::with_capacity(1_000_000).unwrap();

    // Add a simple rule
    let rule = Rule {
        id: 1,
        name: "Million Fact Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("active".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "million_processed".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Generate 1M facts
    let facts: Vec<Fact> = (0..1_000_000)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("entity_id".to_string(), FactValue::Integer(i as i64));
            fields.insert(
                "status".to_string(),
                FactValue::String(if i % 4 == 0 { "active" } else { "inactive" }.to_string()),
            );
            fields.insert(
                "region".to_string(),
                FactValue::String(format!("region_{}", i % 50)),
            );

            Fact {
                id: i as u64,
                external_id: None,
                timestamp: chrono::Utc::now(),
                data: FactData { fields },
            }
        })
        .collect();

    let start = std::time::Instant::now();
    let results = engine.process_facts(facts).unwrap();
    let elapsed = start.elapsed();

    let (start_stats, end_stats, memory_delta) = memory_tracker.finish().unwrap();

    println!(
        "✅ Processed 1M facts in {:?} (target: <30s), generated {} results",
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
    println!("Final engine stats: {:?}", stats);

    // Validate against realistic enterprise targets for production deployment
    assert!(
        elapsed.as_secs() < 30,
        "Should process 1M facts under 30 seconds (enterprise production target)"
    );
    assert!(
        memory_delta < 4_000_000_000,
        "Memory usage should be under 4GB for 1M facts (enterprise production target)"
    );
    assert_eq!(stats.fact_count, 1_000_000);
    assert!(
        results.len() > 200_000,
        "Should generate results for ~25% of facts"
    );

    println!("🚀 1M fact scaling test passed with realistic production targets!");
    println!(
        "📊 Performance: {:.0} facts/second | Memory: {:.1} GB",
        1_000_000.0 / elapsed.as_secs_f64(),
        memory_delta as f64 / (1024.0 * 1024.0 * 1024.0)
    );
    println!("🎯 Ready for enterprise production deployment!");
}

#[test]
fn test_200k_fact_scaling_ci_appropriate() {
    // CI-appropriate test: smaller scale but validates same performance characteristics
    let memory_tracker = MemoryTracker::start().unwrap();
    let mut engine = BingoEngine::with_capacity(200_000).unwrap();

    // Add a simple rule
    let rule = Rule {
        id: 1,
        name: "CI Scale Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("active".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "ci_processed".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Generate 200K facts (CI-appropriate scale)
    let facts: Vec<Fact> = (0..200_000)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("entity_id".to_string(), FactValue::Integer(i as i64));
            fields.insert(
                "status".to_string(),
                FactValue::String(if i % 4 == 0 { "active" } else { "inactive" }.to_string()),
            );
            fields.insert(
                "region".to_string(),
                FactValue::String(format!("region_{}", i % 20)),
            );

            Fact {
                id: i as u64,
                external_id: None,
                timestamp: chrono::Utc::now(),
                data: FactData { fields },
            }
        })
        .collect();

    let start = std::time::Instant::now();
    let results = engine.process_facts(facts).unwrap();
    let elapsed = start.elapsed();

    let (start_stats, end_stats, memory_delta) = memory_tracker.finish().unwrap();

    println!(
        "✅ Processed 200K facts in {:?} (CI target: <6s), generated {} results",
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
    println!("Final engine stats: {:?}", stats);

    // CI-appropriate performance targets (scaled down for limited resources)
    assert!(
        elapsed.as_secs() < 6,
        "Should process 200K facts under 6 seconds (CI target)"
    );
    assert!(
        memory_delta < 1_000_000_000,
        "Memory usage should be under 1GB for 200K facts (CI target)"
    );
    assert_eq!(stats.fact_count, 200_000);
    assert!(
        results.len() > 40_000,
        "Should generate results for ~25% of facts"
    );

    println!("🚀 200K fact CI scaling test passed!");
    println!(
        "📊 Performance: {:.0} facts/second | Memory: {:.1} MB",
        200_000.0 / elapsed.as_secs_f64(),
        memory_delta as f64 / (1024.0 * 1024.0)
    );
}
