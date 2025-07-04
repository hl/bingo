//! JSON Test Framework Integration Tests
//!
//! This module provides the main integration between the JSON test framework
//! and the Rust test harness, allowing execution via `cargo test`.

mod json_test_runner;

use json_test_runner::JsonTestFramework;
use std::env;

/// Run all JSON-based integration tests
#[tokio::test]
async fn json_test_framework_all() {
    let framework = create_framework().await;
    let results = framework.run_all_tests().await;

    framework.print_summary(&results);

    let failed_tests: Vec<_> = results.iter().filter(|r| !r.passed).collect();
    if !failed_tests.is_empty() {
        panic!("❌ {} JSON tests failed", failed_tests.len());
    }

    println!("✅ All {} JSON tests passed!", results.len());
}

/// Run tests for basic functionality
#[tokio::test]
async fn json_test_framework_basic() {
    let framework = create_framework().await;
    let results = framework.run_category_tests("basic").await;

    assert_results(&results, "basic");
}

/// Run tests for streaming functionality
#[tokio::test]
async fn json_test_framework_streaming() {
    let framework = create_framework().await;
    let results = framework.run_category_tests("streaming").await;

    assert_results(&results, "streaming");
}

/// Run tests for caching functionality
#[tokio::test]
async fn json_test_framework_caching() {
    let framework = create_framework().await;
    let results = framework.run_category_tests("caching").await;

    assert_results(&results, "caching");
}

/// Run tests for calculator functionality
#[tokio::test]
async fn json_test_framework_calculator() {
    let framework = create_framework().await;
    let results = framework.run_category_tests("calculator").await;

    assert_results(&results, "calculator");
}

/// Run tests for multi-rule functionality
#[tokio::test]
async fn json_test_framework_multi_rule() {
    let framework = create_framework().await;
    let results = framework.run_category_tests("multi-rule").await;

    assert_results(&results, "multi-rule");
}

/// Run performance validation tests
#[tokio::test]
async fn json_test_framework_performance() {
    let framework = create_framework().await;
    let results = framework.run_category_tests("performance").await;

    assert_results(&results, "performance");
}

/// Run edge case and error handling tests
#[tokio::test]
async fn json_test_framework_edge_cases() {
    let framework = create_framework().await;
    let results = framework.run_category_tests("edge-cases").await;

    assert_results(&results, "edge-cases");
}

/// Run comprehensive error handling tests
#[tokio::test]
async fn json_test_framework_error_handling() {
    let framework = create_framework().await;
    let results = framework.run_category_tests("error-handling").await;

    assert_results(&results, "error-handling");
}

/// Run integration workflow tests
#[tokio::test]
async fn json_test_framework_integration() {
    let framework = create_framework().await;
    let results = framework.run_category_tests("integration").await;

    assert_results(&results, "integration");
}

/// Create a configured test framework instance
async fn create_framework() -> JsonTestFramework {
    let verbose = env::var("JSON_TEST_VERBOSE").unwrap_or_default() == "true";
    let skip_performance = env::var("JSON_TEST_SKIP_PERFORMANCE").unwrap_or_default() == "true";

    JsonTestFramework::new()
        .await
        .with_verbose(verbose)
        .skip_performance(skip_performance)
}

/// Assert test results and provide detailed failure information
fn assert_results(results: &[json_test_runner::TestResult], category: &str) {
    if results.is_empty() {
        println!("⚠️  No tests found in category: {}", category);
        return;
    }

    let failed_tests: Vec<_> = results.iter().filter(|r| !r.passed).collect();

    if !failed_tests.is_empty() {
        println!("❌ Failed tests in category '{}':", category);
        for test in &failed_tests {
            println!("   - {}: {:?}", test.test_name, test.error_message);
        }
        panic!(
            "❌ {} tests failed in category '{}'",
            failed_tests.len(),
            category
        );
    }

    println!(
        "✅ All {} tests passed in category '{}'!",
        results.len(),
        category
    );
}

/// Test framework initialization and configuration
#[tokio::test]
async fn test_framework_initialization() {
    let _framework = JsonTestFramework::new().await;

    // Test that framework can be created successfully
    // No assertion needed - successful creation is the test
}

/// Test specific test case execution
#[tokio::test]
async fn test_single_test_execution() {
    let framework = create_framework().await;

    // Try to run the overtime detection test
    if let Some(result) = framework.run_named_test("overtime_detection").await {
        if !result.passed {
            panic!("Test failed: {:?}", result.error_message);
        }
        println!("✅ Single test execution successful: {}", result.test_name);
    } else {
        println!(
            "⚠️  Test 'overtime_detection' not found - this is expected if tests aren't created yet"
        );
    }
}

/// Performance benchmark test
#[tokio::test]
async fn test_performance_benchmarking() {
    let framework = create_framework().await.skip_performance(false); // Enable performance validation

    let results = framework.run_category_tests("basic").await;

    // Check that at least some tests have performance notes
    let with_performance_notes = results.iter().filter(|r| !r.performance_notes.is_empty()).count();

    println!(
        "📊 {} tests included performance metrics",
        with_performance_notes
    );

    // Print performance summary
    if !results.is_empty() {
        let avg_duration: f64 = results.iter().map(|r| r.duration.as_millis() as f64).sum::<f64>()
            / results.len() as f64;

        println!("⏱️  Average test duration: {:.1}ms", avg_duration);

        // Assert that tests are reasonably fast (under 1 second each)
        for result in &results {
            assert!(
                result.duration.as_millis() < 1000,
                "Test '{}' took too long: {}ms",
                result.test_name,
                result.duration.as_millis()
            );
        }
    }
}

#[cfg(test)]
mod framework_tests {
    use super::*;

    #[tokio::test]
    async fn test_framework_configuration_options() {
        let _framework = JsonTestFramework::new().await.with_verbose(true).skip_performance(true);

        // Framework should be configurable
        // No assertion needed - successful configuration is the test
    }

    #[test]
    fn test_environment_variable_parsing() {
        // Test environment variable handling
        // SAFETY: see note in tracing_setup tests – single-threaded test.
        unsafe {
            env::set_var("JSON_TEST_VERBOSE", "true");
        }
        let verbose = env::var("JSON_TEST_VERBOSE").unwrap_or_default() == "true";
        assert!(verbose);

        unsafe {
            env::set_var("JSON_TEST_SKIP_PERFORMANCE", "false");
        }
        let skip_perf = env::var("JSON_TEST_SKIP_PERFORMANCE").unwrap_or_default() == "true";
        assert!(!skip_perf);
    }
}
