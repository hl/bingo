//! Tests for token sharing optimization
//!
//! This module tests the token sharing implementation and its impact on memory usage.

use bingo_core::*;
use std::collections::HashMap;

fn create_test_fact(id: u64, name: &str, value: i64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert("status".to_string(), FactValue::String(name.to_string()));
    fields.insert("entity_id".to_string(), FactValue::Integer(value));

    Fact { id, data: FactData { fields } }
}

#[test]
fn test_token_pool_basic_functionality() {
    let mut pool = TokenPool::new(5);

    // Test initial state
    assert_eq!(pool.pool_hits, 0);
    assert_eq!(pool.pool_misses, 0);
    assert_eq!(pool.utilization(), 0.0);

    // Get a token - should be a miss since pool is empty
    let token1 = pool.get_single_token(1);
    assert_eq!(pool.pool_misses, 1);
    assert_eq!(pool.pool_hits, 0);
    assert_eq!(token1.fact_ids.as_slice(), &[1]);

    // Return the token to the pool
    pool.return_token(token1);
    assert_eq!(pool.returned_count, 1);

    // Get another token - should be a hit since we have one in pool
    let token2 = pool.get_single_token(2);
    assert_eq!(pool.pool_hits, 1);
    assert_eq!(pool.pool_misses, 1);
    assert_eq!(token2.fact_ids.as_slice(), &[2]);

    // Check utilization calculation
    assert_eq!(pool.utilization(), 50.0); // 1 hit out of 2 total requests
}

#[test]
fn test_token_pool_capacity_limit() {
    let mut pool = TokenPool::new(2);

    // Add tokens up to capacity
    let token1 = pool.get_single_token(1);
    let token2 = pool.get_single_token(2);
    let token3 = pool.get_single_token(3);

    pool.return_token(token1);
    pool.return_token(token2);
    pool.return_token(token3); // This should be ignored due to capacity limit

    assert_eq!(pool.returned_count, 2); // Only 2 should be returned due to capacity

    // Verify we can get back the pooled tokens
    let reused1 = pool.get_single_token(10);
    let reused2 = pool.get_single_token(20);

    assert_eq!(pool.pool_hits, 2); // Both should be hits
    assert_eq!(reused1.fact_ids.as_slice(), &[10]);
    assert_eq!(reused2.fact_ids.as_slice(), &[20]);
}

#[test]
fn test_fact_id_set_memory_sharing() {
    let fact_ids1 = vec![1, 2, 3];
    let fact_ids2 = vec![1, 2, 3]; // Same content

    let set1 = FactIdSet::new(fact_ids1);
    let set2 = FactIdSet::new(fact_ids2);

    // Create tokens with these sets
    let token1 = Token { fact_ids: set1.clone() };
    let token2 = Token { fact_ids: set2 };

    // Verify content is correct
    assert_eq!(token1.fact_ids.as_slice(), &[1, 2, 3]);
    assert_eq!(token2.fact_ids.as_slice(), &[1, 2, 3]);

    // Test joining
    let joined = token1.join(4);
    assert_eq!(joined.fact_ids.as_slice(), &[1, 2, 3, 4]);

    // Original token should be unchanged
    assert_eq!(token1.fact_ids.as_slice(), &[1, 2, 3]);
}

#[test]
fn test_token_join_operations() {
    let token1 = Token::new(1);
    let token2 = Token::new(2);

    // Test single fact join
    let joined_single = token1.join(3);
    assert_eq!(joined_single.fact_ids.as_slice(), &[1, 3]);

    // Test token-to-token join
    let joined_tokens = token1.join_tokens(&token2);
    assert_eq!(joined_tokens.fact_ids.as_slice(), &[1, 2]);

    // Test multi-fact join
    let joined_multi = token1.join_many(&[4, 5, 6]);
    assert_eq!(joined_multi.fact_ids.as_slice(), &[1, 4, 5, 6]);
}

#[test]
fn test_alpha_node_with_token_pool() {
    let condition = Condition::Simple {
        field: "status".to_string(),
        operator: Operator::Equal,
        value: FactValue::String("active".to_string()),
    };

    let mut alpha_node = AlphaNode::new(1, condition);
    let mut token_pool = TokenPool::new(10);

    // Create facts
    let fact1 = create_test_fact(1, "active", 100);
    let fact2 = create_test_fact(2, "inactive", 200);
    let fact3 = create_test_fact(3, "active", 300);

    // Process facts through alpha node
    let tokens1 = alpha_node.process_fact(&fact1, &mut token_pool);
    let tokens2 = alpha_node.process_fact(&fact2, &mut token_pool);
    let tokens3 = alpha_node.process_fact(&fact3, &mut token_pool);

    // Verify results
    assert_eq!(tokens1.len(), 1); // Should match
    assert_eq!(tokens2.len(), 0); // Should not match
    assert_eq!(tokens3.len(), 1); // Should match

    assert_eq!(tokens1[0].fact_ids.as_slice(), &[1]);
    assert_eq!(tokens3[0].fact_ids.as_slice(), &[3]);

    // Verify token pool was used
    assert_eq!(token_pool.pool_misses, 2); // Two tokens created
    assert_eq!(token_pool.pool_hits, 0); // No reuse yet

    // Alpha node memory should track matching facts
    assert_eq!(alpha_node.memory.len(), 2); // fact1 and fact3
    assert_eq!(alpha_node.memory, vec![1, 3]);
}

#[test]
fn test_token_pool_statistics() {
    let mut pool = TokenPool::new(3);

    // Initial stats
    let initial_stats = TokenPoolStats {
        pool_hits: 0,
        pool_misses: 0,
        utilization: 0.0,
        allocated_count: 0,
        returned_count: 0,
    };

    assert_eq!(pool.utilization(), initial_stats.utilization);

    // Use the pool
    let token1 = pool.get_single_token(1);
    let token2 = pool.get_single_token(2);

    assert_eq!(pool.pool_misses, 2);
    assert_eq!(pool.pool_hits, 0);

    // Return tokens
    pool.return_token(token1);
    pool.return_token(token2);

    assert_eq!(pool.returned_count, 2);

    // Reuse tokens
    let _token3 = pool.get_single_token(3);
    let _token4 = pool.get_single_token(4);

    assert_eq!(pool.pool_hits, 2);
    assert_eq!(pool.pool_misses, 2);
    assert_eq!(pool.utilization(), 50.0); // 2 hits out of 4 total requests

    // Clear pool
    pool.clear();
    assert_eq!(pool.pool_hits, 0);
    assert_eq!(pool.pool_misses, 0);
    assert_eq!(pool.utilization(), 0.0);
}

#[test]
fn test_rete_network_token_pool_integration() {
    let mut network = ReteNetwork::new().unwrap();

    // Create a simple rule
    let rule = Rule {
        id: 1,
        name: "Simple Status Rule".to_string(),
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

    // Process facts
    let facts = vec![
        create_test_fact(1, "active", 100),
        create_test_fact(2, "inactive", 200),
        create_test_fact(3, "active", 300),
    ];

    let results = network.process_facts(facts).unwrap();

    // Verify results
    assert!(!results.is_empty(), "Should have results for active facts");

    // Check token pool statistics
    let token_stats = network.get_token_pool_stats();
    assert!(
        token_stats.pool_misses > 0,
        "Should have some token creation"
    );

    println!(
        "Token pool stats: hits={}, misses={}, utilization={:.1}%",
        token_stats.pool_hits, token_stats.pool_misses, token_stats.utilization
    );
}
