# Performance Testing Best Practices

This document outlines the best practices for writing and maintaining performance tests in the Bingo codebase.

## Performance Testing Philosophy

Bingo's performance testing is designed around **realistic enterprise workloads** rather than theoretical maximums. Our approach focuses on:

- **Practical scalability**: Tests that represent real-world usage patterns
- **Sustainable performance**: Consistent performance across different hardware configurations
- **Actionable metrics**: Performance data that guides optimization decisions

## Current Performance Baseline

Based on comprehensive testing, the engine delivers:

### Core Operations (Release Mode)
- **Basic fact processing**: 560K facts/sec
- **Rule compilation**: 886K rules/sec  
- **Fact lookup**: 13M lookups/sec
- **Small scale (10K facts)**: 280K facts/sec
- **Medium scale (25K facts, 5 rules)**: 50K facts/sec

### Production Targets
- **100K facts**: <3 seconds processing time
- **200K facts**: <6 seconds processing time
- **Memory efficiency**: <3GB for 1M facts
- **Rule complexity**: 200+ business rules per dataset

## Guidelines for Adding New Performance Tests

When adding a new performance test, please follow these guidelines:

1.  **Create a new test file:** Create a new test file in the appropriate `tests` directory.
2.  **Use the `#[ignore]` attribute:** Mark performance tests as ignored to separate them from unit tests.
3.  **Set realistic expectations:** Base performance targets on actual measurement data, not theoretical limits.
4.  **Add a warm-up phase:** If your test is sensitive to cold starts, add a warm-up phase.
5.  **Add a descriptive test name:** The test name should clearly describe what the test is measuring.
6.  **Add comments:** Add comments to your test to explain what it is doing and what the expected outcome is.

## Performance Test Categories

### Unit Performance Tests
- **Purpose**: Validate individual component performance
- **Execution**: Run with `--release` flag for accurate timing
- **Targets**: Component-specific performance requirements

### Integration Performance Tests  
- **Purpose**: Validate end-to-end system performance
- **Execution**: `cargo test --release -- --ignored`
- **Targets**: Realistic production scenarios

### Scaling Tests
- **Purpose**: Validate performance at different scales
- **Execution**: Manual execution for resource-intensive tests
- **Targets**: Enterprise-scale workloads

## Best Practices

- **Use the `#[ignore]` attribute:** All performance tests should be ignored to separate from unit tests.
- **Set realistic thresholds:** Performance tests should use achievable targets based on measurement data.
- **Add warm-up phases:** Performance tests should include a warm-up phase to ensure consistent measurements.
- **Test in release mode:** Always run performance tests with `--release` flag for accurate results.
- **Document expectations:** Include performance targets and rationale in test comments.

