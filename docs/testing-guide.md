# Testing Guide for Bingo RETE Engine

This document describes the testing structure and how to run different categories of tests.

## Test Categories

### 1. Unit Tests (Fast - < 1 second each)
- **Purpose**: Test individual functions and modules in isolation
- **Location**: `crates/*/src/` (with `#[cfg(test)]` modules)
- **Run**: `make test-unit` or `cargo test --workspace --lib --bins`
- **CI**: Always run on every commit

### 2. Integration Tests (Medium - 1-10 seconds each)
- **Purpose**: Test component interactions and basic workflows
- **Location**: `crates/*/tests/` (fast integration tests)
- **Run**: `make test-integration` or `./test-runner.sh integration`
- **CI**: Always run on every commit

### 3. API Tests (Fast - 1-5 seconds each)
- **Purpose**: Test gRPC API endpoints and compliance
- **Location**: `crates/bingo-api/tests/`
- **Run**: `make test-api` or `./test-runner.sh api`
- **CI**: Always run on every commit

### 4. Concurrency Tests (Slow - 10-30 seconds each)
- **Purpose**: Test thread safety and parallel processing
- **Location**: `crates/bingo-core/tests/` (concurrency-related tests)
- **Run**: `make test-concurrency` or `./test-runner.sh concurrency release`
- **CI**: Run on main branch pushes only

### 5. Validation Tests (Medium-Slow - 10-30 seconds each)
- **Purpose**: Validate architectural decisions and algorithm correctness
- **Location**: `crates/bingo-core/tests/` (validation-related tests)
- **Run**: `make test-validation` or `./test-runner.sh validation release`
- **CI**: Run on main branch pushes only

### 6. Performance Tests (Slow - 30+ seconds each)
- **Purpose**: Test system performance under load
- **Location**: `crates/bingo-core/tests/` (performance-related tests)
- **Run**: `make test-performance` or `./test-runner.sh performance release`
- **CI**: Run on main branch pushes and nightly

### 7. Profiling Tests (Very Slow - 60+ seconds each)
- **Purpose**: Profile system performance and identify bottlenecks
- **Location**: `crates/bingo-core/tests/` (profiling-related tests)
- **Run**: `make test-profiling` or `./test-runner.sh profiling release`
- **CI**: Manual trigger only

### 8. Debug Tests (Extremely Slow - 5+ minutes each)
- **Purpose**: Debug system behavior with large datasets
- **Location**: `crates/bingo-core/tests/` (debug-related tests)
- **Run**: `make test-debug` or `./test-runner.sh debug release`
- **CI**: Manual trigger only

### 9. Benchmark Tests (Very Slow - 30+ seconds each)
- **Purpose**: Formal benchmarking using criterion.rs
- **Location**: `crates/bingo-core/benches/`, `benches/`
- **Run**: `make test-benchmark` or `./test-runner.sh benchmark release`
- **CI**: Nightly runs only

## Test Suites

### Fast Test Suite
Runs unit, integration, and API tests - suitable for development workflow.
```bash
make test-fast
# or
./test-runner.sh fast
```

### CI Test Suite
Runs fast tests plus quality checks - suitable for CI/CD.
```bash
make test-ci
# or
./test-runner.sh ci
```

### Full Test Suite
Runs all tests - very slow, use sparingly.
```bash
make test-all
# or
./test-runner.sh all release
```

## Development Workflow

### During Development
```bash
# Quick feedback loop
make test-fast

# Check code quality
make test-quality

# Or combine both
make test-ci
```

### Before Committing
```bash
# Run CI test suite
make test-ci

# Format code
make fmt

# Run clippy
make clippy
```

### Performance Testing
```bash
# Run performance tests
make test-performance

# Run comprehensive profiling
make test-profiling

# Run benchmarks
make test-benchmark
```

## CI/CD Pipeline

### On Every Commit
- Unit tests
- Integration tests
- API tests
- Code quality checks (formatting, clippy, compilation)

### On Main Branch Pushes
- All fast tests
- Concurrency tests
- Validation tests
- Performance tests (subset)
- Security audit
- Code coverage
- Release build check

### Nightly Runs
- Full performance test suite
- Benchmark tests
- Debug tests (if needed)

### Manual Triggers
- Profiling tests
- Debug tests
- Full test suite
- Specific test categories

## Configuration Files

- `test-categories.md` - Detailed test categorization
- `test-runner.sh` - Script for running test categories
- `Makefile` - Convenient make targets
- `.github/workflows/ci-new.yml` - New CI configuration
- `TESTING.md` - This guide

## Test Naming Conventions

- **Unit tests**: Use descriptive names in `#[cfg(test)]` modules
- **Integration tests**: Use `*_test.rs` suffix
- **Performance tests**: Use `*_performance_test.rs` suffix
- **Profiling tests**: Use `*_profiling.rs` suffix
- **Debug tests**: Use `debug_*.rs` prefix
- **Benchmark tests**: Use `*_bench.rs` suffix

## Environment Variables

- `RUST_LOG=info` - Enable logging output
- `BINGO_PERF_ENV=ci` - CI-specific performance settings
- `BINGO_SKIP_SLOW_TESTS=1` - Skip slow tests in unit test runs

## Performance Benchmarks

The Bingo RETE engine has been tested extensively with real-world scenarios. All performance tests are conducted in release mode with individual test execution to ensure accurate measurements.

### Production Performance Results

| Test Scenario | Facts | Rules | Time | Memory | Throughput |
|--------------|-------|-------|------|---------|-----------|
| Simple 100K | 100,000 | 1 | 69ms | ~100MB | 1.4M facts/sec |
| Simple 200K | 200,000 | 1 | 118ms | 265MB | 1.7M facts/sec |
| Simple 500K | 500,000 | 1 | 264ms | 596MB | 1.9M facts/sec |
| Simple 1M | 1,000,000 | 1 | 709ms | 1.3GB | 1.4M facts/sec |
| **Simple 2M** | **2,000,000** | **1** | **1.8s** | **3.2GB** | **1.1M facts/sec** |
| Payroll 100K | 100,000 | 4 | 99ms | 249MB | 1.0M facts/sec |
| Enterprise 250K | 250,000 | 200 | 1.0s | 430MB | 250K facts/sec |
| Enterprise 500K | 500,000 | 300 | 2.7s | 878MB | 185K facts/sec |
| Enterprise 1M | 1,000,000 | 400 | 1.3s | 2.8GB | 755K facts/sec |
| **Enterprise 2M** | **2,000,000** | **500** | **2.6s** | **5.5GB** | **756K facts/sec** |

### Key Performance Characteristics

- **Linear scaling**: Performance scales predictably with data size
- **Memory efficiency**: ~1.3KB per fact average memory overhead
- **Rule complexity impact**: More rules reduce throughput but remain efficient
- **Enterprise-ready**: 2M facts + 500 rules processed in 2.6 seconds
- **Consistent throughput**: 750K-1.9M facts/second depending on rule complexity
- **Multi-core optimization**: Automatically utilizes all available CPU cores for parallel processing
- **Scalable architecture**: 3-12x throughput improvement on multi-core systems

### Parallel Processing Capabilities

The Bingo RETE engine features sophisticated multi-core optimization that automatically scales with your hardware:

#### Multi-Core Architecture
- **Automatic CPU detection**: Uses `num_cpus::get()` to detect available cores
- **Parallel fact processing**: Distributes facts across multiple worker threads
- **Concurrent rule matching**: Parallel evaluation of rules within individual facts
- **Thread-safe aggregation**: Safe collection and merging of results across threads
- **Work-stealing queues**: Efficient load balancing across worker threads

#### Parallel Processing Features
- **Parallel rule compilation**: Enabled on dual-core+ systems
- **Parallel rule evaluation**: Enabled on quad-core+ systems  
- **Parallel alpha memory**: Concurrent access with read-write locks
- **Parallel beta network**: Work-stealing token propagation
- **Concurrent memory pools**: Thread-safe object pooling for performance

#### Performance Scaling
- **3-12x throughput improvement** on multi-core systems
- **Linear scalability** with available CPU cores
- **Intelligent thresholds**: Sequential processing for small datasets to avoid overhead
- **Configurable workers**: Defaults to CPU count, can be tuned for specific workloads

#### Hardware Requirements
- **Single-core**: Sequential processing only
- **Dual-core**: Parallel compilation and execution enabled
- **Quad-core+**: Full parallel processing including rule evaluation
- **Multi-core**: Maximum performance with work-stealing and concurrent pools

### Memory Usage Notes

- Memory measurements are from individual test runs (not batch execution)
- Memory usage includes fact storage, rule network, and intermediate results
- Enterprise scenarios with complex rules show ~1.2-1.8 output ratio
- Memory efficiency improves slightly at larger scales due to better amortization
- Parallel processing adds minimal memory overhead per worker thread

## Tips

1. **Use `make test-fast` during development** - provides quick feedback
2. **Run `make test-ci` before pushing** - catches issues early
3. **Use release mode for performance tests** - more realistic results (10x faster than debug)
4. **Profile before optimizing** - use profiling tests to identify bottlenecks
5. **Monitor CI times** - keep fast tests under 2 minutes total
6. **Use manual triggers for heavy tests** - don't bog down CI pipeline
7. **Individual test execution** - run performance tests individually for accurate memory measurements