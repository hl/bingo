use bingo_core::*;
use std::time::Instant;

#[test]
fn analyze_token_pool_performance() {
    println!("=== Token Pool Performance Analysis ===");

    // Test different pool sizes to find optimal configuration
    let configurations = vec![
        (100, "Minimal"),
        (500, "Small"),
        (1000, "Current Default"),
        (2500, "Large"),
        (5000, "Extra Large"),
    ];

    let test_workload = 5000;

    for &(pool_size, description) in &configurations {
        analyze_pool_configuration(pool_size, description, test_workload);
    }

    // Test workload patterns
    analyze_workload_patterns();
}

fn analyze_pool_configuration(pool_size: usize, description: &str, workload_size: usize) {
    let mut token_pool = TokenPool::new(pool_size);

    println!("\n--- {} (Pool Size: {}) ---", description, pool_size);

    // Phase 1: Initial allocation (cold pool)
    let start = Instant::now();
    let mut tokens = Vec::with_capacity(workload_size);

    for fact_id in 0..workload_size {
        let token = token_pool.get_single_token(fact_id as u64);
        tokens.push(token);
    }

    let cold_allocation_time = start.elapsed();

    // Phase 2: Return all tokens
    let start = Instant::now();
    for token in tokens {
        token_pool.return_token(token);
    }
    let return_time = start.elapsed();

    // Phase 3: Hot allocation (warm pool)
    let start = Instant::now();
    let mut reused_tokens = Vec::with_capacity(workload_size);

    for fact_id in 0..workload_size {
        let token = token_pool.get_single_token(fact_id as u64);
        reused_tokens.push(token);
    }

    let hot_allocation_time = start.elapsed();

    // Cleanup
    for token in reused_tokens {
        token_pool.return_token(token);
    }

    // Calculate metrics
    let hit_rate = token_pool.utilization();
    let pool_efficiency = if pool_size >= workload_size {
        workload_size as f64 / pool_size as f64
    } else {
        pool_size as f64 / workload_size as f64
    };

    let speedup = if hot_allocation_time.as_nanos() > 0 {
        cold_allocation_time.as_nanos() as f64 / hot_allocation_time.as_nanos() as f64
    } else {
        1.0
    };

    println!(
        "  Cold allocation: {:?} ({:.0} tokens/sec)",
        cold_allocation_time,
        workload_size as f64 / cold_allocation_time.as_secs_f64()
    );
    println!(
        "  Hot allocation:  {:?} ({:.0} tokens/sec)",
        hot_allocation_time,
        workload_size as f64 / hot_allocation_time.as_secs_f64()
    );
    println!("  Return time:     {:?}", return_time);
    println!("  Hit rate:        {:.1}%", hit_rate);
    println!("  Pool efficiency: {:.1}%", pool_efficiency * 100.0);
    println!("  Speedup:         {:.2}x", speedup);

    // Performance recommendations
    if hit_rate < 50.0 {
        println!("  ⚠️  Consider increasing pool size for better hit rate");
    } else if hit_rate > 95.0 {
        println!("  ✅ Excellent hit rate");
    }

    if pool_efficiency < 0.5 {
        println!("  ⚠️  Pool may be oversized, consider reducing");
    } else if pool_efficiency > 0.8 {
        println!("  ✅ Good memory efficiency");
    }

    if speedup > 1.5 {
        println!(
            "  ✅ Good pooling benefit (+{:.0}% performance)",
            (speedup - 1.0) * 100.0
        );
    } else {
        println!("  ⚠️  Limited pooling benefit");
    }
}

fn analyze_workload_patterns() {
    println!("\n=== Workload Pattern Analysis ===");

    // Pattern 1: Burst workload
    println!("\n--- Burst Workload ---");
    let mut token_pool = TokenPool::new(1000);

    for burst in 0..3 {
        let start = Instant::now();
        let burst_size = 2000; // Exceeds pool capacity
        let mut tokens = Vec::new();

        for i in 0..burst_size {
            tokens.push(token_pool.get_single_token(i as u64));
        }

        for token in tokens {
            token_pool.return_token(token);
        }

        let burst_time = start.elapsed();
        println!(
            "  Burst {}: {} tokens in {:?} (hit rate: {:.1}%)",
            burst + 1,
            burst_size,
            burst_time,
            token_pool.utilization()
        );
    }

    // Pattern 2: Steady state
    println!("\n--- Steady State Workload ---");
    let mut token_pool = TokenPool::new(1000);
    let start = Instant::now();
    let mut total_tokens = 0;

    // Simulate 1000 small batches
    for batch in 0..1000 {
        let mut tokens = Vec::new();

        // Small batch size that fits in pool
        for i in 0..10 {
            tokens.push(token_pool.get_single_token((batch * 10 + i) as u64));
        }

        total_tokens += 10;

        for token in tokens {
            token_pool.return_token(token);
        }
    }

    let steady_time = start.elapsed();
    println!(
        "  {} tokens in {:?} ({:.0} tokens/sec, hit rate: {:.1}%)",
        total_tokens,
        steady_time,
        total_tokens as f64 / steady_time.as_secs_f64(),
        token_pool.utilization()
    );

    // Pattern 3: Mixed single/multi-fact tokens
    println!("\n--- Mixed Token Types ---");
    let mut token_pool = TokenPool::new(1000);
    let start = Instant::now();

    for i in 0..1000 {
        if i % 4 == 0 {
            // Multi-fact token (not pooled efficiently)
            let token = Token::from_facts(vec![i as u64, (i + 1) as u64]);
            token_pool.return_token(token);
        } else {
            // Single-fact token (pools well)
            let token = token_pool.get_single_token(i as u64);
            token_pool.return_token(token);
        }
    }

    let mixed_time = start.elapsed();
    println!(
        "  Mixed workload: {:?} (hit rate: {:.1}%)",
        mixed_time,
        token_pool.utilization()
    );

    if token_pool.utilization() < 40.0 {
        println!("  ⚠️  Mixed workload reduces pooling efficiency");
        println!("     Consider separate pools for different token types");
    }
}

#[test]
fn benchmark_token_allocation_overhead() {
    println!("\n=== Token Allocation Overhead Analysis ===");

    let iterations = 10000;

    // Benchmark 1: Direct allocation (no pool)
    let start = Instant::now();
    let mut direct_tokens = Vec::new();
    for i in 0..iterations {
        direct_tokens.push(Token::new(i as u64));
    }
    let direct_time = start.elapsed();

    // Benchmark 2: Pool allocation (cold)
    let mut pool = TokenPool::new(1000);
    let start = Instant::now();
    let mut pool_tokens = Vec::new();
    for i in 0..iterations {
        pool_tokens.push(pool.get_single_token(i as u64));
    }
    let pool_cold_time = start.elapsed();

    // Return tokens to warm up pool
    for token in pool_tokens {
        pool.return_token(token);
    }

    // Benchmark 3: Pool allocation (hot)
    let start = Instant::now();
    let mut hot_tokens = Vec::new();
    for i in 0..iterations {
        hot_tokens.push(pool.get_single_token(i as u64));
    }
    let pool_hot_time = start.elapsed();

    // Calculate overhead
    let cold_overhead = if direct_time.as_nanos() > 0 {
        (pool_cold_time.as_nanos() as f64 / direct_time.as_nanos() as f64 - 1.0) * 100.0
    } else {
        0.0
    };

    let hot_benefit = if pool_hot_time.as_nanos() > 0 {
        (1.0 - direct_time.as_nanos() as f64 / pool_hot_time.as_nanos() as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "  Direct allocation: {:?} ({:.0} tokens/sec)",
        direct_time,
        iterations as f64 / direct_time.as_secs_f64()
    );
    println!(
        "  Pool cold:         {:?} ({:.0} tokens/sec, {:.1}% overhead)",
        pool_cold_time,
        iterations as f64 / pool_cold_time.as_secs_f64(),
        cold_overhead
    );
    println!(
        "  Pool hot:          {:?} ({:.0} tokens/sec, {:.1}% benefit)",
        pool_hot_time,
        iterations as f64 / pool_hot_time.as_secs_f64(),
        hot_benefit
    );

    println!("  Pool hit rate: {:.1}%", pool.utilization());

    if hot_benefit > 20.0 {
        println!("  ✅ Pool provides significant performance benefit");
    } else if cold_overhead > 50.0 {
        println!("  ⚠️  High cold allocation overhead");
    }

    // Cleanup
    for token in hot_tokens {
        pool.return_token(token);
    }
}

#[test]
fn analyze_memory_characteristics() {
    println!("\n=== Memory Characteristics Analysis ===");

    let pool_sizes = vec![100, 500, 1000, 2500, 5000];

    for &pool_size in &pool_sizes {
        let mut pool = TokenPool::new(pool_size);

        // Fill pool to capacity
        let mut tokens = Vec::new();
        for i in 0..pool_size {
            tokens.push(pool.get_single_token(i as u64));
        }

        // Return all tokens
        for token in tokens {
            pool.return_token(token);
        }

        // Estimate memory usage
        let token_size = std::mem::size_of::<Token>();
        let pool_overhead = std::mem::size_of::<TokenPool>();
        let vector_overhead = pool_size * std::mem::size_of::<*const Token>(); // Vec capacity overhead
        let total_memory = (pool_size * token_size) + pool_overhead + vector_overhead;

        println!(
            "  Pool size {}: ~{} bytes ({:.1} KB)",
            pool_size,
            total_memory,
            total_memory as f64 / 1024.0
        );
        println!("    Per-token: {} bytes", token_size);
        println!("    Pool overhead: {} bytes", pool_overhead);
        println!("    Vector overhead: {} bytes", vector_overhead);

        // Memory efficiency assessment
        let efficiency = (pool_size * token_size) as f64 / total_memory as f64 * 100.0;
        println!("    Memory efficiency: {:.1}%", efficiency);

        if efficiency > 80.0 {
            println!("    ✅ Good memory efficiency");
        } else if efficiency < 60.0 {
            println!("    ⚠️  High overhead, consider larger pool sizes");
        }
    }
}
