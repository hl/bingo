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

## Tips

1. **Use `make test-fast` during development** - provides quick feedback
2. **Run `make test-ci` before pushing** - catches issues early
3. **Use release mode for performance tests** - more realistic results
4. **Profile before optimizing** - use profiling tests to identify bottlenecks
5. **Monitor CI times** - keep fast tests under 2 minutes total
6. **Use manual triggers for heavy tests** - don't bog down CI pipeline