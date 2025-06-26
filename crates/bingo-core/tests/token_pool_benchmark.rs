use bingo_core::*;
use std::time::Instant;

#[test]
fn benchmark_token_pool_performance() {
    println!("=== Token Pool Performance Benchmark ===");

    // Test different pool sizes to find optimal configuration
    let pool_sizes = vec![100, 500, 1000, 2500, 5000, 10000];
    let workload_sizes = vec![1000, 5000, 10000, 25000];

    for &pool_size in &pool_sizes {
        for &workload_size in &workload_sizes {
            benchmark_token_pool_with_config(pool_size, workload_size);
        }
    }
}

fn benchmark_token_pool_with_config(pool_size: usize, workload_size: usize) {
    let mut token_pool = TokenPool::new(pool_size);

    println!(
        "\n--- Pool Size: {}, Workload: {} tokens ---",
        pool_size, workload_size
    );

    // Phase 1: Allocation benchmark
    let start = Instant::now();
    let mut tokens = Vec::with_capacity(workload_size);

    for fact_id in 0..workload_size {
        let token = token_pool.get_single_token(fact_id as u64);
        tokens.push(token);
    }

    let allocation_time = start.elapsed();

    // Phase 2: Return benchmark
    let start = Instant::now();
    for token in tokens {
        token_pool.return_token(token);
    }
    let return_time = start.elapsed();

    // Phase 3: Reuse benchmark
    let start = Instant::now();
    let mut reused_tokens = Vec::with_capacity(workload_size);

    for fact_id in 0..workload_size {
        let token = token_pool.get_single_token(fact_id as u64);
        reused_tokens.push(token);
    }

    let reuse_time = start.elapsed();

    // Cleanup
    for token in reused_tokens {
        token_pool.return_token(token);
    }

    // Calculate performance metrics
    let stats = TokenPoolStats {
        pool_hits: token_pool.pool_hits,
        pool_misses: token_pool.pool_misses,
        utilization: token_pool.utilization(),
        allocated_count: token_pool.allocated_count,
        returned_count: token_pool.returned_count,
        memory_usage_bytes: token_pool.memory_usage_bytes(),
    };

    println!("  Allocation time: {:?}", allocation_time);
    println!("  Return time: {:?}", return_time);
    println!("  Reuse time: {:?}", reuse_time);
    println!("  Hit rate: {:.1}%", stats.hit_rate());
    println!("  Pool utilization: {:.1}%", stats.utilization);
    println!("  Pool hits: {}", stats.pool_hits);
    println!("  Pool misses: {}", stats.pool_misses);

    // Performance analysis
    analyze_performance(
        pool_size,
        workload_size,
        &stats,
        allocation_time,
        return_time,
        reuse_time,
    );
}

fn analyze_performance(
    pool_size: usize,
    workload_size: usize,
    stats: &TokenPoolStats,
    allocation_time: std::time::Duration,
    _return_time: std::time::Duration,
    reuse_time: std::time::Duration,
) {
    let efficiency_ratio = pool_size as f64 / workload_size as f64;

    // Identify potential issues
    if stats.hit_rate() < 50.0 {
        println!(
            "  ⚠️  Low hit rate ({:.1}%) - consider increasing pool size",
            stats.hit_rate()
        );
    }

    if efficiency_ratio > 2.0 {
        println!(
            "  ⚠️  Pool may be oversized (ratio: {:.1}) - consider reducing",
            efficiency_ratio
        );
    } else if efficiency_ratio < 0.5 {
        println!(
            "  ⚠️  Pool may be undersized (ratio: {:.1}) - consider increasing",
            efficiency_ratio
        );
    }

    if reuse_time > allocation_time * 2 {
        println!("  ⚠️  Reuse slower than allocation - pool overhead may be too high");
    }

    // Calculate throughput
    let allocation_throughput = workload_size as f64 / allocation_time.as_secs_f64();
    let reuse_throughput = workload_size as f64 / reuse_time.as_secs_f64();

    println!(
        "  Allocation throughput: {:.0} tokens/sec",
        allocation_throughput
    );
    println!("  Reuse throughput: {:.0} tokens/sec", reuse_throughput);

    if reuse_throughput > allocation_throughput * 1.5 {
        println!(
            "  ✅ Good reuse performance (+{:.1}% vs allocation)",
            (reuse_throughput / allocation_throughput - 1.0) * 100.0
        );
    }
}

#[test]
fn benchmark_token_pool_memory_usage() {
    println!("\n=== Token Pool Memory Usage Analysis ===");

    let pool_sizes = vec![100, 1000, 5000, 10000];

    for &pool_size in &pool_sizes {
        analyze_memory_usage(pool_size);
    }
}

fn analyze_memory_usage(pool_size: usize) {
    let mut token_pool = TokenPool::new(pool_size);

    // Fill the pool to capacity
    let mut tokens = Vec::new();
    for i in 0..pool_size {
        tokens.push(token_pool.get_single_token(i as u64));
    }

    // Return all tokens to pool
    for token in tokens {
        token_pool.return_token(token);
    }

    // Estimate memory usage
    let token_size = std::mem::size_of::<Token>();
    let pool_overhead = std::mem::size_of::<TokenPool>();
    let estimated_memory = (pool_size * token_size) + pool_overhead;

    println!("\nPool size: {} tokens", pool_size);
    println!(
        "  Estimated memory: {} bytes ({:.1} KB)",
        estimated_memory,
        estimated_memory as f64 / 1024.0
    );
    println!("  Per-token memory: {} bytes", token_size);
    println!("  Pool overhead: {} bytes", pool_overhead);

    // Test memory efficiency
    let utilization = token_pool.utilization();
    if utilization > 80.0 {
        println!("  ✅ Good memory utilization: {:.1}%", utilization);
    } else if utilization < 50.0 {
        println!("  ⚠️  Low memory utilization: {:.1}%", utilization);
    }
}

#[test]
fn benchmark_token_pool_under_realistic_workloads() {
    println!("\n=== Realistic Workload Simulation ===");

    // Simulate real RETE network usage patterns
    simulate_burst_workload();
    simulate_steady_state_workload();
    simulate_mixed_workload();
}

fn simulate_burst_workload() {
    println!("\n--- Burst Workload Simulation ---");
    let mut token_pool = TokenPool::new(1000);

    // Simulate periods of high activity followed by low activity
    for burst in 0..5 {
        let start = Instant::now();
        let burst_size = 2000; // More tokens than pool size
        let mut tokens = Vec::new();

        // Burst allocation
        for i in 0..burst_size {
            tokens.push(token_pool.get_single_token(i as u64));
        }

        // Immediate return (simulating short-lived tokens)
        for token in tokens {
            token_pool.return_token(token);
        }

        let burst_time = start.elapsed();
        println!(
            "  Burst {}: {} tokens in {:?} ({:.0} tokens/sec)",
            burst + 1,
            burst_size,
            burst_time,
            burst_size as f64 / burst_time.as_secs_f64()
        );
    }

    println!("  Final hit rate: {:.1}%", token_pool.utilization());
}

fn simulate_steady_state_workload() {
    println!("\n--- Steady State Workload Simulation ---");
    let mut token_pool = TokenPool::new(1000);

    // Simulate continuous moderate load
    let start = Instant::now();
    let duration = std::time::Duration::from_millis(100);
    let mut total_tokens = 0;

    while start.elapsed() < duration {
        let mut tokens = Vec::new();

        // Allocate small batch
        for i in 0..50 {
            tokens.push(token_pool.get_single_token((total_tokens + i) as u64));
        }

        total_tokens += 50;

        // Return tokens after processing
        for token in tokens {
            token_pool.return_token(token);
        }
    }

    let throughput = total_tokens as f64 / start.elapsed().as_secs_f64();
    println!(
        "  Processed {} tokens in {:?} ({:.0} tokens/sec)",
        total_tokens,
        start.elapsed(),
        throughput
    );
    println!("  Hit rate: {:.1}%", token_pool.utilization());
}

fn simulate_mixed_workload() {
    println!("\n--- Mixed Workload Simulation ---");
    let mut token_pool = TokenPool::new(1000);

    let start = Instant::now();
    let mut single_token_count = 0;
    let mut multi_token_count = 0;

    // Mix of single and multi-fact tokens
    for i in 0..1000 {
        if i % 3 == 0 {
            // Single-fact token (should use pool)
            let token = token_pool.get_single_token(i as u64);
            token_pool.return_token(token);
            single_token_count += 1;
        } else {
            // Multi-fact token (creates new, may not pool efficiently)
            let token = Token::from_facts(vec![i as u64, (i + 1) as u64]);
            token_pool.return_token(token);
            multi_token_count += 1;
        }
    }

    println!("  Single-fact tokens: {}", single_token_count);
    println!("  Multi-fact tokens: {}", multi_token_count);
    println!("  Mixed workload time: {:?}", start.elapsed());
    println!("  Hit rate: {:.1}%", token_pool.utilization());

    // Analyze pooling effectiveness for mixed workload
    if token_pool.utilization() < 30.0 {
        println!("  ⚠️  Low pooling effectiveness with mixed workload");
        println!("     Consider separate pools for single vs multi-fact tokens");
    }
}

#[test]
fn test_optimal_pool_sizing() {
    println!("\n=== Optimal Pool Sizing Analysis ===");

    // Test various pool configurations to find optimal settings
    let configurations = vec![
        (100, "Minimal"),
        (500, "Small"),
        (1000, "Medium"),
        (2500, "Large"),
        (5000, "Extra Large"),
    ];

    let test_workload = 10000;
    let mut best_config = (0, "None", 0.0);

    for &(pool_size, description) in &configurations {
        let score = evaluate_pool_configuration(pool_size, test_workload);
        println!(
            "  {}: Pool size {} - Score: {:.2}",
            description, pool_size, score
        );

        if score > best_config.2 {
            best_config = (pool_size, description, score);
        }
    }

    println!(
        "\n  ✅ Recommended configuration: {} (size: {}, score: {:.2})",
        best_config.1, best_config.0, best_config.2
    );
}

fn evaluate_pool_configuration(pool_size: usize, workload_size: usize) -> f64 {
    let mut token_pool = TokenPool::new(pool_size);

    // Run standardized test
    let start = Instant::now();

    // Allocation phase
    let mut tokens = Vec::new();
    for i in 0..workload_size {
        tokens.push(token_pool.get_single_token(i as u64));
    }

    // Return phase
    for token in tokens {
        token_pool.return_token(token);
    }

    // Reuse phase
    let mut reused_tokens = Vec::new();
    for i in 0..workload_size {
        reused_tokens.push(token_pool.get_single_token(i as u64));
    }

    // Cleanup
    for token in reused_tokens {
        token_pool.return_token(token);
    }

    let total_time = start.elapsed();

    // Calculate composite score based on multiple factors
    let hit_rate = token_pool.utilization();
    let throughput = (workload_size * 2) as f64 / total_time.as_secs_f64(); // 2x for allocation + reuse
    let memory_efficiency = if pool_size > workload_size {
        workload_size as f64 / pool_size as f64
    } else {
        1.0
    };

    // Weighted score: 40% hit rate, 40% throughput, 20% memory efficiency
    (hit_rate / 100.0) * 0.4 + (throughput / 100000.0).min(1.0) * 0.4 + memory_efficiency * 0.2
}
