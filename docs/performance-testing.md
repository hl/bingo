# Performance Testing Best Practices

This document outlines the best practices for writing and maintaining performance tests in the Bingo codebase.

## Performance Testing Philosophy

Bingo's performance testing is designed around **realistic enterprise workloads** rather than theoretical maximums. Our approach focuses on:

- **Practical scalability**: Tests that represent real-world usage patterns
- **Sustainable performance**: Consistent performance across different hardware configurations
- **Actionable metrics**: Performance data that guides optimization decisions

## Current Performance Baseline

Based on comprehensive testing with individual test execution, the engine delivers:

### Verified Performance Results (Release Mode)

| Test Scenario | Facts | Rules | Time | Memory | Throughput |
|--------------|-------|-------|------|---------|-----------|
| Simple 100K | 100,000 | 1 | 66ms | 20MB | 1.5M facts/sec |
| Simple 200K | 200,000 | 1 | 128ms | 247MB | 1.6M facts/sec |
| Simple 500K | 500,000 | 1 | 280ms | 550MB | 1.8M facts/sec |
| Simple 1M | 1,000,000 | 1 | 687ms | 1.3GB | 1.5M facts/sec |
| **Simple 2M** | **2,000,000** | **1** | **1.79s** | **3.36GB** | **1.1M facts/sec** |
| Payroll 100K | 100,000 | 4 | 115ms | 218MB | 871K facts/sec |
| Payroll 500K | 500,000 | 4 | 552ms | 1.0GB | 905K facts/sec |
| Payroll 1M | 1,000,000 | 4 | 1.11s | 2.2GB | 902K facts/sec |
| Enterprise 250K | 250,000 | 200 | 396ms | 658MB | 631K facts/sec |
| Enterprise 500K | 500,000 | 300 | 855ms | 1.3GB | 585K facts/sec |
| Enterprise 1M | 1,000,000 | 400 | 1.88s | 2.7GB | 531K facts/sec |
| **Enterprise 2M** | **2,000,000** | **500** | **4.47s** | **5.2GB** | **448K facts/sec** |

### Core Operations (Release Mode)
- **Basic fact processing**: 1.1M-1.8M facts/sec (simple scenarios)
- **Payroll processing**: 871K-905K facts/sec (multi-rule scenarios)
- **Enterprise processing**: 448K-631K facts/sec (complex rule sets)
- **Memory efficiency**: ~1.7KB per fact average overhead
- **Rule complexity**: Up to 500 business rules per dataset
- **Multi-core scaling**: 3-12x throughput improvement on multi-core systems
- **Parallel processing**: Automatically utilizes all available CPU cores

### Production Targets (Validated)
- **100K facts**: <3 seconds processing time ✅ (66ms achieved)
- **200K facts**: <6 seconds processing time ✅ (128ms achieved)  
- **500K facts**: <10 seconds processing time ✅ (280ms achieved)
- **1M facts**: <30 seconds processing time ✅ (687ms achieved)
- **2M facts**: <60 seconds processing time ✅ (1.79s achieved)
- **Memory efficiency**: <3GB for 1M facts ✅ (1.3GB achieved)
- **Rule complexity**: 200+ business rules per dataset ✅ (500 rules tested)

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

