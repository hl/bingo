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

### Complex Rule Performance Targets

Based on calculator-heavy business rule scenarios:

- **100K facts + 200 rules**: ~19s processing time, ~1.9GB memory
- **200K facts + 200 rules**: ~39s processing time, ~3.5GB memory
- **100K facts + 500 rules**: ~49s processing time, ~4.1GB memory
- **200K facts + 500 rules**: ~90s processing time, ~6GB memory

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

## Adding New Performance Tests

When adding new performance tests:

1. **Mark as ignored**: Add `#[ignore = "Performance test - run with --release: cargo test --release test_name"]`
2. **Require release mode**: Document release mode requirement
3. **Set reasonable timeouts**: Avoid blocking CI/quality checks
4. **Document expectations**: Include performance targets in test comments