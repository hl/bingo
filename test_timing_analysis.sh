#!/bin/bash

# Test Timing Analysis Script
# This script runs all tests individually and measures their execution time

echo "🔍 Starting Comprehensive Test Timing Analysis"
echo "=============================================="

# Create results file
RESULTS_FILE="test_timing_results.txt"
echo "Test Timing Analysis Results - $(date)" > $RESULTS_FILE
echo "=======================================" >> $RESULTS_FILE
echo "" >> $RESULTS_FILE

# Function to run and time a test
run_test() {
    local test_name="$1"
    local test_path="$2"
    echo "📊 Testing: $test_name"
    
    start_time=$(date +%s.%3N)
    timeout 120s cargo test --test "$test_name" --quiet 2>/dev/null
    exit_code=$?
    end_time=$(date +%s.%3N)
    
    duration=$(echo "$end_time - $start_time" | bc)
    
    if [ $exit_code -eq 124 ]; then
        echo "⏰ $test_name: TIMEOUT (>120s)" | tee -a $RESULTS_FILE
        return 1
    elif [ $exit_code -ne 0 ]; then
        echo "❌ $test_name: FAILED (${duration}s)" | tee -a $RESULTS_FILE
        return 1
    else
        echo "✅ $test_name: ${duration}s" | tee -a $RESULTS_FILE
        return 0
    fi
}

# Function to categorize timing
categorize_time() {
    local duration="$1"
    if (( $(echo "$duration > 60" | bc -l) )); then
        echo "SLOW (>60s)"
    elif (( $(echo "$duration > 10" | bc -l) )); then
        echo "MODERATE (10-60s)"
    elif (( $(echo "$duration > 1" | bc -l) )); then
        echo "FAST (1-10s)"
    else
        echo "VERY_FAST (<1s)"
    fi
}

# Get list of test files
echo "📋 Discovering test files..."
test_files=($(find crates -name "*.rs" -path "*/tests/*" -exec basename {} .rs \;))

echo "Found ${#test_files[@]} test files to analyze"
echo "" >> $RESULTS_FILE

# Initialize counters
total_tests=0
fast_tests=0
moderate_tests=0
slow_tests=0
timeout_tests=0
failed_tests=0

# Run each test
for test_file in "${test_files[@]}"; do
    total_tests=$((total_tests + 1))
    run_test "$test_file"
    
    # Read the last result to categorize
    last_line=$(tail -1 $RESULTS_FILE)
    if [[ $last_line == *"TIMEOUT"* ]]; then
        timeout_tests=$((timeout_tests + 1))
    elif [[ $last_line == *"FAILED"* ]]; then
        failed_tests=$((failed_tests + 1))
    else
        duration=$(echo $last_line | grep -o '[0-9]*\.[0-9]*s' | sed 's/s//')
        category=$(categorize_time "$duration")
        case $category in
            "VERY_FAST"*|"FAST"*) fast_tests=$((fast_tests + 1)) ;;
            "MODERATE"*) moderate_tests=$((moderate_tests + 1)) ;;
            "SLOW"*) slow_tests=$((slow_tests + 1)) ;;
        esac
    fi
done

# Generate summary
echo "" >> $RESULTS_FILE
echo "SUMMARY" >> $RESULTS_FILE
echo "=======" >> $RESULTS_FILE
echo "Total tests analyzed: $total_tests" >> $RESULTS_FILE
echo "Fast tests (<10s): $fast_tests" >> $RESULTS_FILE
echo "Moderate tests (10-60s): $moderate_tests" >> $RESULTS_FILE
echo "Slow tests (>60s): $slow_tests" >> $RESULTS_FILE
echo "Timeout tests (>120s): $timeout_tests" >> $RESULTS_FILE
echo "Failed tests: $failed_tests" >> $RESULTS_FILE

echo ""
echo "📊 Test Timing Analysis Complete!"
echo "Results saved to: $RESULTS_FILE"
cat $RESULTS_FILE