# Performance Testing Best Practices

This document outlines the best practices for writing and maintaining performance tests in the Bingo codebase.

## Guidelines for Adding New Performance Tests

When adding a new performance test, please follow these guidelines:

1.  **Create a new test file:** Create a new test file in the appropriate `tests` directory.
2.  **Use the `#[performance_test]` attribute:** Add the `#[performance_test]` attribute to your test function.
3.  **Use the `assert_time_performance!` and `assert_memory_performance!` macros:** Use these macros to set the performance expectations for your test.
4.  **Add a warm-up phase:** If your test is sensitive to cold starts, add a warm-up phase.
5.  **Add a descriptive test name:** The test name should clearly describe what the test is measuring.
6.  **Add comments:** Add comments to your test to explain what it is doing and what the expected outcome is.

- **Use the `#[performance_test]` attribute:** All performance tests should use the `#[performance_test]` attribute. This allows us to categorize and manage performance tests effectively.
- **Use adaptive thresholds:** Performance tests should use the `assert_time_performance!` and `assert_memory_performance!` macros to ensure that they can adapt to different environments.
- **Add warm-up phases:** Performance tests should include a warm-up phase to ensure that the system is "warm" before the actual measurement.

