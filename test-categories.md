# Test Categories for Bingo RETE Engine

## 1. Unit Tests (Fast - < 1 second each)
**Location**: `crates/*/src/` (with `#[cfg(test)]` modules)
**Execution**: `cargo test --lib`
**CI Stage**: Always run on every commit

## 2. Integration Tests (Medium - 1-10 seconds each)
**Location**: `crates/*/tests/`
**Examples**:
- `simple_rule_eval_test.rs`
- `fact_mutation_test.rs`
- `action_type_handlers_test.rs`
- `calculator_integration_test.rs`
- `api_payload_rule_test.rs`
- `conflict_resolution_test.rs`
- `cross_fact_matching_test.rs`
- `end_to_end_integration_test.rs`
- `formula_action_test.rs`
- `incremental_processing_test.rs`
- `rete_direct_test.rs`
- `rete_network_edge_cases_test.rs`
- `simplified_api_test.rs`
- `temporary_rule_eval.rs`

**Execution**: `cargo test --test <test_name>`
**CI Stage**: Run on every commit

## 3. Concurrency Tests (Slow - 10-30 seconds each)
**Location**: `crates/bingo-core/tests/`
**Examples**:
- `comprehensive_concurrency_test.rs`
- `engine_threaded_integration_test.rs`
- `simple_engine_threaded_test.rs`
- `parallel_rete_test.rs`
- `beta_memory_integration_test.rs`
- `lazy_aggregation_integration_test.rs`

**Execution**: `cargo test --test <test_name>`
**CI Stage**: Run on main branch pushes only

## 4. Performance Tests (Very Slow - 30+ seconds each)
**Location**: `crates/bingo-core/tests/`
**Examples**:
- `scaling_validation_test.rs`
- `alpha_memory_performance_test.rs`
- `complex_rule_performance_test.rs`
- `serialization_performance_test.rs`
- `simple_rete_performance_test.rs`
- `rete_network_performance_benchmarks.rs`

**Execution**: `cargo test --test <test_name> --release`
**CI Stage**: Nightly runs and manual triggers only

## 5. Profiling Tests (Very Slow - 60+ seconds each)
**Location**: `crates/bingo-core/tests/`
**Examples**:
- `comprehensive_rete_benchmark.rs`
- `comprehensive_performance_benchmarks.rs`
- `comprehensive_component_profiling.rs`
- `fact_store_profiling.rs`
- `performance_profiling.rs`
- `performance_bottleneck_analysis.rs`
- `quick_performance_analysis.rs`
- `simple_profiling_report.rs`

**Execution**: `cargo test --test <test_name> --release`
**CI Stage**: Manual triggers only

## 6. Debug/Scaling Tests (Extremely Slow - 5+ minutes each)
**Location**: `crates/bingo-core/tests/`
**Examples**:
- `debug_10k_scaling.rs`
- `debug_1k_scaling.rs`
- `debug_action_execution.rs`
- `debug_beta_test.rs`
- `debug_threaded_processing.rs`

**Execution**: `cargo test --test <test_name> --release`
**CI Stage**: Manual triggers only

## 7. Validation Tests (Medium-Slow - 10-30 seconds each)
**Location**: `crates/bingo-core/tests/`
**Examples**:
- `true_rete_architecture_validation.rs`
- `rete_algorithm_validation.rs`
- `optimization_validation_test.rs`
- `enhanced_monitoring_test.rs`
- `enhanced_test_coverage.rs`
- `comprehensive_optimization_summary.rs`
- `rule_optimization_test.rs`
- `beta_network_test.rs`

**Execution**: `cargo test --test <test_name>`
**CI Stage**: Run on main branch pushes only

## 8. Benchmark Tests (Very Slow - 30+ seconds each)
**Location**: `crates/bingo-core/benches/`, `benches/`
**Examples**:
- `million_fact_bench.rs`
- `engine_bench.rs`
- `performance_suite.rs`

**Execution**: `cargo bench`
**CI Stage**: Nightly runs only

## 9. API Tests (Fast-Medium - 1-5 seconds each)
**Location**: `crates/bingo-api/tests/`
**Examples**:
- `grpc_compliance_tests.rs`
- `grpc_payroll_tests.rs`
- `grpc_tronc_tests.rs`
- `grpc_wage_cost_tests.rs`

**Execution**: `cargo test --test <test_name>`
**CI Stage**: Run on every commit

## 10. Calculator Tests (Fast - < 1 second each)
**Location**: `crates/bingo-calculator/tests/`
**Examples**:
- `built_in_calculators_test.rs`

**Execution**: `cargo test --test <test_name>`
**CI Stage**: Run on every commit