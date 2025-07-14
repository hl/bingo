#!/bin/bash

# Quick Test Analysis for Identifying Slow Tests
# Focus on tests that are likely to be problematic

echo "🔍 Quick Test Analysis - Identifying Slow Tests"
echo "=============================================="

# List of test files to check (focusing on likely problematic ones)
test_files=(
    "debug_10k_scaling"
    "comprehensive_rete_benchmark"
    "performance_bottleneck_analysis"
    "alpha_memory_performance_test"
    "fact_store_profiling"
    "serialization_performance_test"
    "working_memory_test"
    "lazy_aggregation_integration_test"
    "thread_safety_test"
    "enhanced_test_coverage"
    "simple_profiling_report"
)

# Function to test with timeout
test_with_timeout() {
    local test_name="$1"
    local timeout_sec=60
    
    echo "📊 Testing: $test_name (${timeout_sec}s timeout)"
    
    start_time=$(date +%s.%3N)
    timeout ${timeout_sec}s cargo test --test "$test_name" --quiet 2>/dev/null
    exit_code=$?
    end_time=$(date +%s.%3N)
    
    duration=$(echo "$end_time - $start_time" | bc)
    
    if [ $exit_code -eq 124 ]; then
        echo "❌ $test_name: TIMEOUT (>${timeout_sec}s)"
        return 1
    elif [ $exit_code -ne 0 ]; then
        echo "❌ $test_name: FAILED (${duration}s)"
        return 1
    else
        echo "✅ $test_name: ${duration}s"
        
        # Check if it's slow (>10s)
        if (( $(echo "$duration > 10" | bc -l) )); then
            echo "⚠️  WARNING: $test_name is slow (${duration}s)"
        fi
        
        return 0
    fi
}

# Test each file
slow_tests=()
timeout_tests=()
failed_tests=()

for test_file in "${test_files[@]}"; do
    test_with_timeout "$test_file"
    exit_code=$?
    
    if [ $exit_code -eq 1 ]; then
        # Check if it was timeout or failure
        if timeout 1s cargo test --test "$test_file" --quiet 2>/dev/null; then
            failed_tests+=("$test_file")
        else
            timeout_tests+=("$test_file")
        fi
    else
        # Check if it was slow
        duration=$(echo "scale=2; $duration" | bc)
        if (( $(echo "$duration > 10" | bc -l) )); then
            slow_tests+=("$test_file")
        fi
    fi
done

# Summary
echo ""
echo "📋 SUMMARY"
echo "=========="
echo "Timeout tests (>60s): ${#timeout_tests[@]}"
for test in "${timeout_tests[@]}"; do
    echo "  - $test"
done

echo "Slow tests (>10s): ${#slow_tests[@]}"
for test in "${slow_tests[@]}"; do
    echo "  - $test"
done

echo "Failed tests: ${#failed_tests[@]}"
for test in "${failed_tests[@]}"; do
    echo "  - $test"
done

echo ""
echo "🎯 FOCUS AREAS:"
if [ ${#timeout_tests[@]} -gt 0 ]; then
    echo "1. Fix timeout tests - these are definitely over 60s"
fi
if [ ${#slow_tests[@]} -gt 0 ]; then
    echo "2. Optimize slow tests - these are 10-60s"
fi