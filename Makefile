# Makefile for Bingo RETE Engine
# 
# This Makefile provides convenient shortcuts for running different test categories
# and development tasks.

.PHONY: help test-unit test-integration test-api test-concurrency test-validation test-performance test-profiling test-debug test-benchmark test-quality test-fast test-ci test-all build build-release clean check fmt clippy security audit coverage dev-setup

# Default target
help:
	@echo "Bingo RETE Engine - Development Commands"
	@echo ""
	@echo "Test Categories:"
	@echo "  test-unit          Run unit tests (fast)"
	@echo "  test-integration   Run integration tests (medium)"
	@echo "  test-api          Run API tests (fast)"
	@echo "  test-concurrency  Run concurrency tests (slow)"
	@echo "  test-validation   Run validation tests (medium-slow)"
	@echo "  test-performance  Run performance tests (slow)"
	@echo "  test-profiling    Run profiling tests (very slow)"
	@echo "  test-debug        Run debug tests (extremely slow)"
	@echo "  test-benchmark    Run benchmark tests (very slow)"
	@echo "  test-quality      Run quality checks (formatting, clippy, compilation)"
	@echo ""
	@echo "Test Suites:"
	@echo "  test-fast         Run fast test suite (unit + integration + api)"
	@echo "  test-ci           Run CI test suite (fast + quality)"
	@echo "  test-all          Run all tests (very slow)"
	@echo ""
	@echo "Development:"
	@echo "  build             Build in debug mode"
	@echo "  build-release     Build in release mode"
	@echo "  clean             Clean build artifacts"
	@echo "  check             Check compilation without building"
	@echo "  fmt               Format code"
	@echo "  clippy            Run clippy lints"
	@echo "  security          Run security audit"
	@echo "  audit             Alias for security"
	@echo "  coverage          Generate code coverage report"
	@echo "  dev-setup         Setup development environment"
	@echo ""
	@echo "Examples:"
	@echo "  make test-fast           # Quick development testing"
	@echo "  make test-ci            # Full CI testing"
	@echo "  make test-performance   # Performance testing"
	@echo "  make build-release      # Release build"

# Test categories
test-unit:
	@./test-runner.sh unit

test-integration:
	@./test-runner.sh integration

test-api:
	@./test-runner.sh api

test-concurrency:
	@./test-runner.sh concurrency release

test-validation:
	@./test-runner.sh validation release

test-performance:
	@./test-runner.sh performance release

test-profiling:
	@./test-runner.sh profiling release

test-debug:
	@./test-runner.sh debug release

test-benchmark:
	@./test-runner.sh benchmark release

test-quality:
	@./test-runner.sh quality

# Test suites
test-fast:
	@./test-runner.sh fast

test-ci:
	@./test-runner.sh ci

test-all:
	@./test-runner.sh all release

# Development commands
build:
	@echo "Building in debug mode..."
	@cargo build --workspace

build-release:
	@echo "Building in release mode..."
	@cargo build --workspace --release

clean:
	@echo "Cleaning build artifacts..."
	@cargo clean

check:
	@echo "Checking compilation..."
	@cargo check --workspace --all-targets

fmt:
	@echo "Formatting code..."
	@cargo fmt --all

clippy:
	@echo "Running clippy..."
	@cargo clippy --workspace --all-targets -- -D warnings

security:
	@echo "Running security audit..."
	@cargo audit

audit: security

coverage:
	@echo "Generating code coverage..."
	@cargo llvm-cov --workspace --lcov --output-path lcov.info \
		--lib --bins \
		--ignore-filename-regex '(performance|benchmark|scaling|profiling|debug_|comprehensive_|million_fact|engine_bench)' \
		--ignore-filename-regex '(concurrency|threaded|parallel|rete_.*performance|beta_.*performance|alpha_.*performance)' \
		--ignore-filename-regex '(validation|monitoring|optimization|component_profiling|bottleneck_analysis)'
	@echo "Coverage report generated: lcov.info"

dev-setup:
	@echo "Setting up development environment..."
	@rustup component add rustfmt clippy llvm-tools-preview
	@cargo install cargo-audit cargo-llvm-cov
	@echo "Development environment setup complete!"

# Convenience aliases
t-unit: test-unit
t-int: test-integration
t-api: test-api
t-conc: test-concurrency
t-val: test-validation
t-perf: test-performance
t-prof: test-profiling
t-debug: test-debug
t-bench: test-benchmark
t-quality: test-quality
t-fast: test-fast
t-ci: test-ci
t-all: test-all

# Build aliases
b: build
br: build-release
c: clean
ch: check
f: fmt
cl: clippy
s: security
cov: coverage