# Performance Test Suite

This document describes the performance and stress tests that are separated from the core quality validation suite.

## Overview

Performance tests are marked with `#[ignore]` to prevent them from blocking quality checks. These tests require explicit execution with the `--release` flag for accurate performance measurements.

## Current Performance Benchmarks (Release Mode)

**Validated Performance Results:**
- **100K facts + 1 rule**: 64.5ms (1.55M facts/sec) - 46x faster than 3s target
- **200K facts + 1 rule**: 116.6ms (1.72M facts/sec) - 51x faster than 6s target
- **1K facts + 4 rules**: 5.4ms (186K facts/sec) - Complex multi-condition rules
- **Memory Usage**: 250.5MB for 200K facts (1.25MB per 1K facts)

## Performance Test Categories

### Core RETE Network Stress Tests

These tests validate network robustness under error conditions and complex processing scenarios:

```bash
# Run individual RETE stress tests
cargo test --release test_missing_node_error_handling_in_incremental_processing -- --ignored
cargo test --release test_error_propagation_in_fact_processing -- --ignored
cargo test --release test_successful_fact_processing_with_error_handling -- --ignored
cargo test --release test_corrupted_network_state_recovery -- --ignored
cargo test --release test_empty_network_processing -- --ignored
cargo test --release test_processing_mode_switching_with_error_conditions -- --ignored
cargo test --release test_rete_network_with_optimal_token_pool -- --ignored
cargo test --release test_calculator_cache_performance_improvement -- --ignored
cargo test --release test_expression_compilation_caching -- --ignored
cargo test --release test_context_sensitive_result_caching -- --ignored
cargo test --release test_calculator_formula_action_integration -- --ignored
cargo test --release test_calculator_complex_formula_with_functions -- --ignored
cargo test --release test_calculator_conditional_formula -- --ignored
cargo test --release test_calculator_error_handling -- --ignored
cargo test --release test_join_conditions_with_shared_entity_id -- --ignored
```

### API Concurrent Performance Tests

These tests validate API performance under concurrent load:

```bash
# Run concurrent API performance tests
cargo test --release test_mixed_operations_performance -- --ignored
cargo test --release test_api_correctness_after_concurrency_changes -- --ignored
cargo test --release test_fact_processing_with_formula_actions -- --ignored
```

## Running All Performance Tests

### Run all ignored performance tests
```bash
cargo test --release -- --ignored
```

### Run specific performance test categories
```bash
# Core engine stress tests only
cargo test --release --package bingo-core -- --ignored

# API performance tests only  
cargo test --release --package bingo-api -- --ignored
```

## Scaling Validation Tests

The project includes dedicated scaling tests that validate performance at enterprise scale:

```bash
# CI-appropriate tests (100K, 200K facts)
cargo test --package bingo-core --test scaling_validation_test --release

# Heavy performance tests (500K, 1M facts) - manual execution only
cargo test --package bingo-core --test scaling_validation_test --ignored --release
```

## Complex Rule Performance Tests

High-rule-complexity tests that validate performance with calculation-heavy business rules using calculators:

```bash
# Complex rule scenarios with calculator-based business logic
cargo test --package bingo-core --test complex_rule_performance_test --release -- --ignored

# Individual complex rule performance tests
cargo test --release test_100k_facts_200_rules_performance -- --ignored  # 100K facts + 200 calculator rules
cargo test --release test_200k_facts_200_rules_performance -- --ignored  # 200K facts + 200 calculator rules  
cargo test --release test_100k_facts_500_rules_performance -- --ignored  # 100K facts + 500 calculator rules
cargo test --release test_200k_facts_500_rules_performance -- --ignored  # 200K facts + 500 calculator rules
```

### Complex Rule Performance Results

Current performance characteristics for realistic payroll business rule scenarios:

| Facts | Rules | Results | Output Ratio | Memory (GB) | Facts/sec | Total Time |
|-------|-------|---------|--------------|-------------|-----------|------------|
| 100K  | 200   | 2M      | 20x          | 15.1        | 47,885    | 2.09s      |
| 200K  | 200   | 4M      | 20x          | 19.7        | 45,651    | 4.38s      |
| 100K  | 500   | 5M      | 50x          | 17.9        | 18,437    | 5.42s      |
| 200K  | 500   | 10M     | 50x          | 14.8        | 19,542    | 10.23s     |

**Performance Analysis**:
- **Realistic Ratios**: Each employee now triggers multiple rules (20-50x) as expected in payroll
- **Memory Scaling**: 15-20GB for large scenarios, scales with rule complexity
- **Throughput**: 18K-48K facts/sec depending on rule complexity
- **Rule Efficiency**: 70% update rules + 30% fact-creating rules simulate real payroll patterns

**Key Improvements**: 
- Fixed unrealistic 160x result explosion to realistic 20-50x ratios
- Rules now properly match employee populations via modulo matching
- Performance targets are achievable and reflect real payroll scenarios

These tests simulate real-world business rule complexity with:
- Threshold checking for compliance validation
- Multi-tier limit validation with warning/critical/max levels
- Time-based calculations for payroll and scheduling
- Performance scoring and ranking calculations

## Performance Targets

The performance tests validate these enterprise targets:

- **100K facts**: <3s processing time
- **200K facts**: <6s processing time (CI target)
- **500K facts**: <10s processing time
- **1M facts**: <30s processing time
- **Memory usage**: <3GB for 1M facts

## Quality vs Performance Separation

### Quality Tests (Core Validation)
- **Purpose**: Code quality, correctness, basic functionality
- **Execution**: `cargo test --workspace` (excludes performance tests)
- **Criteria**: Must pass for quality validation
- **Timing**: Fast execution (<60s total)

### Performance Tests (Stress/Load Testing)  
- **Purpose**: Performance validation, stress testing, scalability
- **Execution**: `cargo test --release -- --ignored`
- **Criteria**: Validates performance targets
- **Timing**: May take several minutes per test

## Integration with CI/CD

### Quality Gate (Required)
```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace --all-targets
cargo test --workspace  # Excludes performance tests
```

### Performance Gate (Optional/Nightly)
```bash
cargo test --release -- --ignored  # All performance tests
cargo test --package bingo-core --test scaling_validation_test --release  # Scaling tests
```

## Troubleshooting Performance Tests

If performance tests fail:

1. **Ensure release mode**: Performance tests require `--release` flag
2. **Check system resources**: Tests may need adequate CPU/memory
3. **Verify targets**: Review performance targets in test code
4. **Environment factors**: System load may affect timing-sensitive tests

## Memory Efficiency Analysis

### Performance Scaling Patterns

**Observations from Complex Rule Tests**:
- Memory usage ranges from 14.8GB to 19.7GB for realistic payroll scenarios
- Result set sizes range from 2M to 10M results (realistic 20-50x ratios)
- Memory scaling driven by rule complexity and fact processing patterns
- Employee-based rule matching creates predictable memory usage patterns

### Optimization Opportunities

#### High Priority Areas

1. **Calculator Result Caching Analysis**
   - Profile memory usage patterns in calculator result caching
   - Analyze cache hit rates vs memory overhead trade-offs
   - Consider cache size limits or eviction strategies

2. **Result Set Memory Management**
   - 2M-10M results represent significant memory allocation
   - Evaluate result streaming vs accumulation strategies for payroll batches
   - Consider result pagination for large employee populations

3. **Memory Scaling Investigation**
   - Profile memory usage scaling with rule count (200 vs 500 rules)
   - Analyze memory usage scaling with fact count (100K vs 200K facts)
   - Identify primary memory growth factors

#### Medium Priority Areas

4. **Calculator Hashmap Pooling Effectiveness**
   - Review hashmap pooling performance in high-rule scenarios
   - Assess pool size limits and reuse efficiency
   - Optimize for calculator instance lifecycle management

5. **Action Result Pooling Optimization**
   - Analyze action result pool configuration for large result sets
   - Consider memory pressure handling strategies
   - Evaluate pool size limits vs allocation patterns

6. **Memory Profiling Infrastructure**
   - Implement detailed memory usage tracking for calculator scenarios
   - Add allocation pattern analysis during rule execution
   - Create memory usage benchmarking tools

#### Low Priority Areas

7. **Result Processing Strategies**
   - Evaluate streaming vs batch processing for large result sets
   - Consider API response pagination options
   - Assess client-side result processing patterns

## Adding New Performance Tests

When adding new performance tests:

1. **Mark as ignored**: Add `#[ignore = "Performance test - run with --release: cargo test --release test_name"]`
2. **Require release mode**: Document release mode requirement
3. **Set reasonable timeouts**: Avoid blocking CI/quality checks
4. **Document expectations**: Include performance targets in test comments