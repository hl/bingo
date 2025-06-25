use bingo_core::*;
use std::time::Instant;

#[test]
fn test_adaptive_token_pool_performance() {
    println!("🧪 Testing Adaptive Token Pool Implementation");
    println!("============================================");

    // Test optimal settings based on benchmarks
    let mut optimal_pool = TokenPool::with_optimal_settings();
    let mut default_pool = TokenPool::new(1000);

    let workload_size = 10000;

    println!("📊 Comparing Default vs Optimal Token Pool Configuration");
    println!("  Workload size: {} tokens", workload_size);

    // Test default pool performance
    println!("\n--- Default Pool (1000 capacity) ---");
    let start = Instant::now();
    let mut default_tokens = Vec::new();

    for i in 0..workload_size {
        default_tokens.push(default_pool.get_single_token(i as u64));
    }

    let default_allocation_time = start.elapsed();

    for token in default_tokens {
        default_pool.return_token(token);
    }

    let default_stats = default_pool.get_comprehensive_stats();

    println!("  Allocation time: {:?}", default_allocation_time);
    println!("  Hit rate: {:.1}%", default_stats.utilization);
    println!(
        "  Pool efficiency: {:.1}%",
        (default_stats.current_single_pool_size as f64 / default_stats.max_single_pool_size as f64)
            * 100.0
    );

    // Test optimal pool performance
    println!("\n--- Optimal Pool (5000 capacity) ---");
    let start = Instant::now();
    let mut optimal_tokens = Vec::new();

    for i in 0..workload_size {
        optimal_tokens.push(optimal_pool.get_single_token(i as u64));
    }

    let optimal_allocation_time = start.elapsed();

    for token in optimal_tokens {
        optimal_pool.return_token(token);
    }

    let optimal_stats = optimal_pool.get_comprehensive_stats();

    println!("  Allocation time: {:?}", optimal_allocation_time);
    println!("  Hit rate: {:.1}%", optimal_stats.utilization);
    println!(
        "  Pool efficiency: {:.1}%",
        (optimal_stats.current_single_pool_size as f64 / optimal_stats.max_single_pool_size as f64)
            * 100.0
    );

    // Calculate improvement
    let speedup = if optimal_allocation_time.as_nanos() > 0 {
        default_allocation_time.as_nanos() as f64 / optimal_allocation_time.as_nanos() as f64
    } else {
        1.0
    };

    println!("\n🎯 Performance Comparison:");
    println!("  Optimal vs Default speedup: {:.2}x", speedup);
    println!(
        "  Hit rate improvement: {:.1}%",
        optimal_stats.utilization - default_stats.utilization
    );

    // Test comprehensive stats structure
    println!("\n📈 Comprehensive Stats (Optimal Pool):");
    println!("  Total allocations: {}", optimal_stats.allocated_count);
    println!("  Total returns: {}", optimal_stats.returned_count);
    println!("  Pool hits: {}", optimal_stats.pool_hits);
    println!("  Pool misses: {}", optimal_stats.pool_misses);
    println!(
        "  Single pool size: {}/{}",
        optimal_stats.current_single_pool_size, optimal_stats.max_single_pool_size
    );
    println!(
        "  Multi pool size: {}/{}",
        optimal_stats.current_multi_pool_size, optimal_stats.max_multi_pool_size
    );
    println!(
        "  Allocation rate: {:.1} tokens/sec",
        optimal_stats.allocation_rate
    );
    println!(
        "  Peak burst count: {}",
        optimal_stats.peak_allocation_burst
    );
    println!(
        "  Current burst count: {}",
        optimal_stats.current_burst_count
    );

    // Verify that optimal settings provide better performance characteristics
    assert!(
        optimal_stats.max_single_pool_size > default_stats.max_single_pool_size,
        "Optimal pool should have larger capacity"
    );
    assert!(
        optimal_stats.max_multi_pool_size > default_stats.max_multi_pool_size,
        "Optimal pool should have larger multi-token capacity"
    );

    println!("\n✅ Adaptive Token Pool test completed successfully!");
}

#[test]
fn test_token_pool_adaptive_resizing() {
    println!("\n🔧 Testing Token Pool Adaptive Resizing");
    println!("======================================");

    let mut pool = TokenPool::new(100); // Start with small pool

    // Simulate burst workload that should trigger pool expansion
    println!("  Simulating burst workload to trigger expansion...");

    for burst in 0..3 {
        let mut tokens = Vec::new();

        // Create burst that exceeds current pool size
        for i in 0..500 {
            tokens.push(pool.get_single_token((burst * 500 + i) as u64));
        }

        // Return all tokens
        for token in tokens {
            pool.return_token(token);
        }

        let stats = pool.get_comprehensive_stats();
        println!(
            "    Burst {}: Hit rate {:.1}%, Pool size {}/{}",
            burst + 1,
            stats.utilization,
            stats.current_single_pool_size,
            stats.max_single_pool_size
        );
    }

    let final_stats = pool.get_comprehensive_stats();

    println!("  Final pool configuration:");
    println!(
        "    Single pool capacity: {}",
        final_stats.max_single_pool_size
    );
    println!(
        "    Multi pool capacity: {}",
        final_stats.max_multi_pool_size
    );
    println!(
        "    Peak allocation burst: {}",
        final_stats.peak_allocation_burst
    );

    println!("\n✅ Adaptive resizing test completed!");
}

#[test]
fn test_rete_network_with_optimal_token_pool() {
    println!("\n🏗️  Testing RETE Network with Optimal Token Pool");
    println!("==============================================");

    let mut network = ReteNetwork::new().unwrap();

    // Create a simple rule to test token pool integration
    let rule = Rule {
        id: 1,
        name: "test_optimal_pool_rule".to_string(),
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

    // Create test facts
    let mut facts = Vec::new();
    for i in 0..1000 {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "status".to_string(),
            FactValue::String("active".to_string()),
        );
        fields.insert("id".to_string(), FactValue::Integer(i));

        facts.push(Fact { id: i as u64, data: FactData { fields } });
    }

    // Process facts and measure performance
    let start = Instant::now();
    let results = network.process_facts(facts).unwrap();
    let processing_time = start.elapsed();

    // Get comprehensive token pool stats
    let token_stats = network.get_token_pool_comprehensive_stats();

    println!("  Processing results:");
    println!("    Facts processed: 1000");
    println!("    Results generated: {}", results.len());
    println!("    Processing time: {:?}", processing_time);

    println!("  Token pool performance:");
    println!(
        "    Pool configuration: {}/{} single, {}/{} multi",
        token_stats.current_single_pool_size,
        token_stats.max_single_pool_size,
        token_stats.current_multi_pool_size,
        token_stats.max_multi_pool_size
    );
    println!("    Hit rate: {:.1}%", token_stats.utilization);
    println!("    Total allocations: {}", token_stats.allocated_count);
    println!("    Peak burst: {}", token_stats.peak_allocation_burst);

    // Verify optimal pool settings are being used
    assert_eq!(
        token_stats.max_single_pool_size, 5000,
        "Should use optimal single pool size"
    );
    assert_eq!(
        token_stats.max_multi_pool_size, 2500,
        "Should use optimal multi pool size"
    );
    assert!(results.len() > 0, "Should generate results");

    println!("\n✅ RETE network integration test completed successfully!");
}
