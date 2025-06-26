//! Performance benchmarks for token sharing optimization
//!
//! This module demonstrates the memory benefits of token sharing.

use bingo_core::{
    Action, ActionType, Condition, Fact, FactData, FactIdSet, FactValue, Operator, ReteNetwork,
    Rule, Token, TokenPool, memory::MemoryStats,
};
use std::collections::HashMap;

fn create_test_fact(id: u64, status: &str, value: i64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert("status".to_string(), FactValue::String(status.to_string()));
    fields.insert("entity_id".to_string(), FactValue::Integer(value));

    Fact { id, data: FactData { fields } }
}

#[test]
fn test_token_sharing_memory_efficiency() {
    // Test memory efficiency of token sharing vs naive approach

    let network = ReteNetwork::new().unwrap();

    // Create a rule that will generate many tokens
    let rule = Rule {
        id: 1,
        name: "Status Processing Rule".to_string(),
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

    network.add_rule(rule).unwrap();

    // Create many facts with the same status (will create many identical tokens)
    let facts: Vec<_> = (0..1000).map(|i| create_test_fact(i, "active", i as i64)).collect();

    let start_memory = MemoryStats::current().unwrap();

    // Process facts
    let results = network.process_facts(facts).unwrap();

    let end_memory = MemoryStats::current().unwrap();
    let memory_delta = end_memory.delta_from(&start_memory);

    // Verify we got results
    assert_eq!(results.len(), 1000, "Should process all 1000 facts");

    // Check token pool utilization
    let token_stats = network.get_token_pool_stats();

    println!("Token sharing performance results:");
    println!("  Facts processed: 1000");
    println!("  Results generated: {}", results.len());
    println!(
        "  Memory usage delta: {} MB",
        memory_delta as f64 / (1024.0 * 1024.0)
    );
    println!("  Token pool hits: {}", token_stats.pool_hits);
    println!("  Token pool misses: {}", token_stats.pool_misses);
    println!("  Pool utilization: {:.1}%", token_stats.utilization);

    // With token sharing, we should see some reuse
    assert!(
        token_stats.pool_misses > 0,
        "Should have created some tokens"
    );

    // Memory usage should be reasonable for 1000 facts
    assert!(
        memory_delta < 50 * 1024 * 1024,
        "Memory usage should be under 50MB for 1000 facts"
    );
}

#[test]
fn test_token_pool_stress_test() {
    let mut pool = TokenPool::new(100);

    // Stress test the token pool with many allocations and returns
    let mut tokens = Vec::new();

    // Create many tokens
    for i in 0..500 {
        let token = pool.get_single_token(i);
        tokens.push(token);
    }

    println!("After creating 500 tokens:");
    println!("  Pool hits: {}", pool.pool_hits);
    println!("  Pool misses: {}", pool.pool_misses);

    // Return half of them
    for _ in 0..250 {
        if let Some(token) = tokens.pop() {
            pool.return_token(token);
        }
    }

    println!("After returning 250 tokens:");
    println!("  Returned count: {}", pool.returned_count);

    // Create more tokens (should reuse some)
    for i in 500..750 {
        let token = pool.get_single_token(i);
        tokens.push(token);
    }

    println!("After creating 250 more tokens:");
    println!("  Pool hits: {}", pool.pool_hits);
    println!("  Pool misses: {}", pool.pool_misses);
    println!("  Total utilization: {:.1}%", pool.utilization());

    // Should have significant reuse
    assert!(pool.pool_hits > 0, "Should have reused some tokens");
    assert!(
        pool.utilization() > 10.0,
        "Should have reasonable utilization"
    );
}

#[test]
fn test_fact_id_set_arc_sharing() {
    // Test that Arc sharing works correctly for FactIdSet

    let ids = vec![1, 2, 3, 4, 5];
    let set1 = FactIdSet::new(ids.clone());
    let set2 = set1.clone(); // Should share the Arc

    // Both sets should have the same content
    assert_eq!(set1.as_slice(), set2.as_slice());
    assert_eq!(set1.len(), 5);
    assert_eq!(set2.len(), 5);

    // Create tokens using these sets
    let token1 = Token { fact_ids: set1 };
    let token2 = Token { fact_ids: set2 };

    // Verify content
    assert_eq!(token1.fact_ids.as_slice(), &[1, 2, 3, 4, 5]);
    assert_eq!(token2.fact_ids.as_slice(), &[1, 2, 3, 4, 5]);

    // Test joining doesn't affect originals
    let joined = token1.join(6);
    assert_eq!(joined.fact_ids.as_slice(), &[1, 2, 3, 4, 5, 6]);
    assert_eq!(token1.fact_ids.as_slice(), &[1, 2, 3, 4, 5]); // Original unchanged
    assert_eq!(token2.fact_ids.as_slice(), &[1, 2, 3, 4, 5]); // Other copy unchanged
}
