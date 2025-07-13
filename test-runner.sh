#!/bin/bash

# Test Runner Script for Bingo RETE Engine
# Usage: ./test-runner.sh <category> [options]

set -e

CATEGORY="${1:-all}"
RELEASE_MODE="${2:-debug}"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')] $1${NC}"
}

success() {
    echo -e "${GREEN}[SUCCESS] $1${NC}"
}

warning() {
    echo -e "${YELLOW}[WARNING] $1${NC}"
}

error() {
    echo -e "${RED}[ERROR] $1${NC}"
}

run_unit_tests() {
    log "Running unit tests..."
    cargo test --workspace --lib --bins
    success "Unit tests completed"
}

run_integration_tests() {
    log "Running integration tests..."
    
    # Fast integration tests
    local fast_tests=(
        "simple_rule_eval_test"
        "fact_mutation_test"
        "action_type_handlers_test"
        "calculator_integration_test"
        "api_payload_rule_test"
        "conflict_resolution_test"
        "cross_fact_matching_test"
        "end_to_end_integration_test"
        "formula_action_test"
        "incremental_processing_test"
        "rete_direct_test"
        "rete_network_edge_cases_test"
        "simplified_api_test"
        "temporary_rule_eval"
        "fact_lookup_test"
        "rule_dependency_integration_test"
        "session_window_integration_test"
        "built_in_calculators_test"
    )
    
    for test in "${fast_tests[@]}"; do
        log "Running $test..."
        cargo test --test "$test" || error "Failed: $test"
    done
    
    success "Integration tests completed"
}

run_api_tests() {
    log "Running API tests..."
    
    local api_tests=(
        "grpc_compliance_tests"
        "grpc_payroll_tests"
        "grpc_tronc_tests"
        "grpc_wage_cost_tests"
    )
    
    for test in "${api_tests[@]}"; do
        log "Running $test..."
        cargo test --test "$test" || error "Failed: $test"
    done
    
    success "API tests completed"
}

run_concurrency_tests() {
    log "Running concurrency tests..."
    
    local concurrency_tests=(
        "comprehensive_concurrency_test"
        "engine_threaded_integration_test"
        "simple_engine_threaded_test"
        "parallel_rete_test"
        "beta_memory_integration_test"
        "lazy_aggregation_integration_test"
    )
    
    local flags=""
    if [[ "$RELEASE_MODE" == "release" ]]; then
        flags="--release"
    fi
    
    for test in "${concurrency_tests[@]}"; do
        log "Running $test..."
        cargo test --test "$test" $flags || error "Failed: $test"
    done
    
    success "Concurrency tests completed"
}

run_validation_tests() {
    log "Running validation tests..."
    
    local validation_tests=(
        "true_rete_architecture_validation"
        "rete_algorithm_validation"
        "optimization_validation_test"
        "enhanced_monitoring_test"
        "enhanced_test_coverage"
        "comprehensive_optimization_summary"
        "rule_optimization_test"
        "beta_network_test"
        "working_memory_test"
    )
    
    local flags=""
    if [[ "$RELEASE_MODE" == "release" ]]; then
        flags="--release"
    fi
    
    for test in "${validation_tests[@]}"; do
        log "Running $test..."
        cargo test --test "$test" $flags || error "Failed: $test"
    done
    
    success "Validation tests completed"
}

run_performance_tests() {
    log "Running performance tests..."
    
    local performance_tests=(
        "scaling_validation_test"
        "alpha_memory_performance_test"
        "complex_rule_performance_test"
        "serialization_performance_test"
        "simple_rete_performance_test"
        "rete_network_performance_benchmarks"
    )
    
    for test in "${performance_tests[@]}"; do
        log "Running $test..."
        if [[ "$test" == "scaling_validation_test" ]]; then
            cargo test --test "$test" --release -- --skip test_extreme_scaling || error "Failed: $test"
        else
            cargo test --test "$test" --release || error "Failed: $test"
        fi
    done
    
    success "Performance tests completed"
}

run_profiling_tests() {
    log "Running profiling tests..."
    
    local profiling_tests=(
        "comprehensive_rete_benchmark"
        "comprehensive_performance_benchmarks"
        "comprehensive_component_profiling"
        "fact_store_profiling"
        "performance_profiling"
        "performance_bottleneck_analysis"
        "quick_performance_analysis"
        "simple_profiling_report"
    )
    
    for test in "${profiling_tests[@]}"; do
        log "Running $test..."
        cargo test --test "$test" --release || error "Failed: $test"
    done
    
    success "Profiling tests completed"
}

run_debug_tests() {
    log "Running debug tests..."
    
    local debug_tests=(
        "debug_10k_scaling"
        "debug_1k_scaling"
        "debug_action_execution"
        "debug_beta_test"
        "debug_threaded_processing"
    )
    
    for test in "${debug_tests[@]}"; do
        log "Running $test..."
        cargo test --test "$test" --release || error "Failed: $test"
    done
    
    success "Debug tests completed"
}

run_benchmark_tests() {
    log "Running benchmark tests..."
    
    local benchmarks=(
        "million_fact_bench"
        "engine_bench"
        "performance_suite"
    )
    
    for bench in "${benchmarks[@]}"; do
        log "Running $bench..."
        cargo bench --bench "$bench" || error "Failed: $bench"
    done
    
    success "Benchmark tests completed"
}

run_quality_checks() {
    log "Running quality checks..."
    
    log "Checking formatting..."
    cargo fmt --check || error "Formatting check failed"
    
    log "Running clippy..."
    cargo clippy --workspace --all-targets -- -D warnings || error "Clippy check failed"
    
    log "Checking compilation..."
    cargo check --workspace --all-targets || error "Compilation check failed"
    
    success "Quality checks completed"
}

show_help() {
    cat << EOF
Test Runner for Bingo RETE Engine

Usage: $0 <category> [release|debug]

Categories:
  unit         - Unit tests (fast)
  integration  - Integration tests (medium)
  api          - API tests (fast)
  concurrency  - Concurrency tests (slow)
  validation   - Validation tests (medium-slow)
  performance  - Performance tests (slow)
  profiling    - Profiling tests (very slow)
  debug        - Debug tests (extremely slow)
  benchmark    - Benchmark tests (very slow)
  quality      - Quality checks (formatting, clippy, compilation)
  fast         - Unit + Integration + API tests
  ci           - Fast tests + quality checks
  all          - All tests (very slow)

Options:
  release      - Run tests in release mode (default for performance tests)
  debug        - Run tests in debug mode (default for most tests)

Examples:
  $0 unit                    # Run unit tests
  $0 performance release     # Run performance tests in release mode
  $0 fast                    # Run fast test suite
  $0 ci                      # Run CI test suite
  $0 all release             # Run all tests in release mode

EOF
}

# Main execution
case "$CATEGORY" in
    "unit")
        run_unit_tests
        ;;
    "integration")
        run_integration_tests
        ;;
    "api")
        run_api_tests
        ;;
    "concurrency")
        run_concurrency_tests
        ;;
    "validation")
        run_validation_tests
        ;;
    "performance")
        run_performance_tests
        ;;
    "profiling")
        run_profiling_tests
        ;;
    "debug")
        run_debug_tests
        ;;
    "benchmark")
        run_benchmark_tests
        ;;
    "quality")
        run_quality_checks
        ;;
    "fast")
        run_quality_checks
        run_unit_tests
        run_integration_tests
        run_api_tests
        ;;
    "ci")
        run_quality_checks
        run_unit_tests
        run_integration_tests
        run_api_tests
        ;;
    "all")
        run_quality_checks
        run_unit_tests
        run_integration_tests
        run_api_tests
        run_concurrency_tests
        run_validation_tests
        run_performance_tests
        run_profiling_tests
        run_debug_tests
        run_benchmark_tests
        ;;
    "help"|"-h"|"--help")
        show_help
        ;;
    *)
        error "Unknown category: $CATEGORY"
        echo ""
        show_help
        exit 1
        ;;
esac

success "Test run completed successfully!"