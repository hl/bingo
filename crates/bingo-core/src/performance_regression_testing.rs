//! Performance regression testing framework for RETE network optimization validation
//!
//! This module provides comprehensive benchmarking and regression testing capabilities
//! to ensure that optimizations actually improve performance and don't introduce regressions.

use crate::optimization_coordinator::OptimizationCoordinator;
use crate::rete_network::ReteNetwork;
use crate::types::{Fact, FactData, FactId, FactValue};
use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Comprehensive performance benchmark suite
#[derive(Debug)]
pub struct PerformanceBenchmarkSuite {
    /// Benchmark scenarios to run
    scenarios: Vec<BenchmarkScenario>,
    /// Historical performance baselines
    baselines: HashMap<String, PerformanceBaseline>,
    /// Configuration for benchmark execution
    config: BenchmarkConfig,
    /// Results from benchmark runs
    results: Vec<BenchmarkResult>,
}

/// Configuration for benchmark execution
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of warmup iterations before measurement
    pub warmup_iterations: usize,
    /// Number of measurement iterations to average
    pub measurement_iterations: usize,
    /// Maximum acceptable performance degradation percentage
    pub max_degradation_percent: f64,
    /// Minimum performance improvement percentage to be considered significant
    pub min_improvement_percent: f64,
    /// Whether to run memory usage benchmarks
    pub include_memory_benchmarks: bool,
    /// Whether to run scalability tests
    pub include_scalability_tests: bool,
    /// Timeout for individual benchmark scenarios
    pub scenario_timeout: Duration,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            warmup_iterations: 3,
            measurement_iterations: 10,
            max_degradation_percent: 5.0, // 5% degradation is considered a regression
            min_improvement_percent: 2.0, // 2% improvement is considered significant
            include_memory_benchmarks: true,
            include_scalability_tests: true,
            scenario_timeout: Duration::from_secs(300), // 5 minutes per scenario
        }
    }
}

/// Individual benchmark scenario
#[derive(Debug, Clone)]
pub struct BenchmarkScenario {
    /// Unique identifier for the scenario
    pub name: String,
    /// Description of what this scenario tests
    pub description: String,
    /// Test data generator for this scenario
    pub data_generator: DataGenerator,
    /// Operations to benchmark
    pub operations: Vec<BenchmarkOperation>,
    /// Expected performance characteristics
    pub expected_complexity: ComplexityClass,
    /// Critical performance threshold (fail if exceeded)
    pub critical_threshold_ms: Option<f64>,
}

/// Data generator for creating test datasets
#[derive(Debug, Clone)]
pub struct DataGenerator {
    /// Number of facts to generate
    pub fact_count: usize,
    /// Number of rules to generate
    pub rule_count: usize,
    /// Distribution of fact characteristics
    pub fact_distribution: FactDistribution,
    /// Complexity of rules to generate
    pub rule_complexity: RuleComplexity,
    /// Random seed for reproducible tests
    pub seed: u64,
}

/// Distribution characteristics for generated facts
#[derive(Debug, Clone)]
pub struct FactDistribution {
    /// Number of unique field names
    pub field_count: usize,
    /// Cardinality distribution for field values
    pub field_cardinality: HashMap<String, usize>,
    /// Size distribution of facts (small, medium, large)
    pub size_distribution: (f64, f64, f64),
    /// Temporal patterns in fact creation
    pub temporal_pattern: TemporalPattern,
}

#[derive(Debug, Clone)]
pub enum TemporalPattern {
    Uniform,
    Recent,
    Clustered,
    Seasonal,
}

/// Complexity characteristics for generated rules
#[derive(Debug, Clone)]
pub struct RuleComplexity {
    /// Average number of conditions per rule
    pub avg_conditions: usize,
    /// Maximum join depth
    pub max_join_depth: usize,
    /// Use of complex pattern matching
    pub complex_patterns: bool,
    /// Frequency of rule conflicts/overlaps
    pub conflict_rate: f64,
}

/// Types of operations to benchmark
#[derive(Debug, Clone)]
pub enum BenchmarkOperation {
    /// Fact insertion performance
    FactInsertion { batch_size: usize },
    /// Fact lookup performance
    FactLookup { lookup_count: usize, miss_rate: f64 },
    /// Rule compilation performance
    RuleCompilation { rule_count: usize },
    /// Rule firing performance
    RuleFiring { fire_count: usize },
    /// Network optimization performance
    NetworkOptimization,
    /// Memory cleanup performance
    MemoryCleanup,
    /// Complex query performance
    ComplexQuery { query_complexity: QueryComplexity },
}

#[derive(Debug, Clone)]
pub enum QueryComplexity {
    Simple,      // Single field lookup
    Moderate,    // 2-3 field intersection
    Complex,     // 4+ fields with joins
    VeryComplex, // Nested conditions with temporal logic
}

/// Expected algorithmic complexity class
#[derive(Debug, Clone, PartialEq)]
pub enum ComplexityClass {
    Constant,     // O(1)
    Logarithmic,  // O(log n)
    Linear,       // O(n)
    Linearithmic, // O(n log n)
    Quadratic,    // O(n²)
    Cubic,        // O(n³)
}

/// Performance baseline for comparison
#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    /// Scenario name this baseline applies to
    pub scenario_name: String,
    /// Baseline timing measurements
    pub timing_baseline: TimingBaseline,
    /// Baseline memory measurements
    pub memory_baseline: MemoryBaseline,
    /// Baseline scalability characteristics
    pub scalability_baseline: ScalabilityBaseline,
    /// When this baseline was established
    pub established_at: Instant,
    /// Git commit or version this baseline represents
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct TimingBaseline {
    /// Average execution time
    pub avg_duration_ms: f64,
    /// Standard deviation of measurements
    pub std_dev_ms: f64,
    /// Minimum observed time
    pub min_duration_ms: f64,
    /// Maximum observed time
    pub max_duration_ms: f64,
    /// 95th percentile timing
    pub p95_duration_ms: f64,
}

#[derive(Debug, Clone)]
pub struct MemoryBaseline {
    /// Peak memory usage
    pub peak_memory_bytes: usize,
    /// Average memory usage
    pub avg_memory_bytes: usize,
    /// Memory efficiency (operations per MB)
    pub memory_efficiency: f64,
    /// Memory allocation count
    pub allocation_count: usize,
}

#[derive(Debug, Clone)]
pub struct ScalabilityBaseline {
    /// Performance at different scales
    pub scale_performance: Vec<(usize, f64)>, // (scale, duration_ms)
    /// Observed complexity class
    pub observed_complexity: ComplexityClass,
    /// Scaling coefficient
    pub scaling_coefficient: f64,
}

/// Result of a benchmark run
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Scenario that was benchmarked
    pub scenario_name: String,
    /// Timing measurements
    pub timing_result: TimingResult,
    /// Memory measurements
    pub memory_result: Option<MemoryResult>,
    /// Comparison with baseline
    pub baseline_comparison: Option<BaselineComparison>,
    /// Whether this result indicates a regression
    pub is_regression: bool,
    /// Whether this result shows significant improvement
    pub is_improvement: bool,
    /// Additional notes or observations
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TimingResult {
    /// Individual measurement times
    pub measurements_ms: Vec<f64>,
    /// Statistical summary
    pub timing_stats: TimingBaseline,
    /// Operations per second
    pub ops_per_second: f64,
}

#[derive(Debug, Clone)]
pub struct MemoryResult {
    /// Memory usage throughout the test
    pub memory_usage_samples: Vec<usize>,
    /// Memory statistics
    pub memory_stats: MemoryBaseline,
    /// Memory leaks detected
    pub potential_leaks: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BaselineComparison {
    /// Performance change percentage (positive = improvement)
    pub performance_change_percent: f64,
    /// Memory usage change percentage (negative = improvement)
    pub memory_change_percent: f64,
    /// Statistical significance of the change
    pub is_statistically_significant: bool,
    /// Confidence level of the measurement
    pub confidence_level: f64,
}

impl Default for PerformanceBenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceBenchmarkSuite {
    /// Create a new benchmark suite with default scenarios
    pub fn new() -> Self {
        let scenarios = Self::create_default_scenarios();

        Self {
            scenarios,
            baselines: HashMap::new(),
            config: BenchmarkConfig::default(),
            results: Vec::new(),
        }
    }

    /// Create benchmark suite with custom configuration
    pub fn with_config(config: BenchmarkConfig) -> Self {
        let scenarios = Self::create_default_scenarios();

        Self { scenarios, baselines: HashMap::new(), config, results: Vec::new() }
    }

    /// Add a baseline for comparison
    pub fn add_baseline(&mut self, baseline: PerformanceBaseline) {
        self.baselines.insert(baseline.scenario_name.clone(), baseline);
    }

    /// Load baselines from historical data
    pub fn load_baselines(&mut self, baselines: Vec<PerformanceBaseline>) {
        for baseline in baselines {
            self.add_baseline(baseline);
        }
    }

    /// Run all benchmark scenarios
    pub fn run_all_benchmarks(
        &mut self,
        mut coordinator: OptimizationCoordinator,
    ) -> Result<BenchmarkSummary> {
        self.results.clear();
        let start_time = Instant::now();

        for scenario in &self.scenarios.clone() {
            let result = self.run_scenario(scenario, &mut coordinator)?;
            self.results.push(result);
        }

        let total_duration = start_time.elapsed();
        Ok(self.generate_summary(total_duration))
    }

    /// Run a specific benchmark scenario
    pub fn run_scenario(
        &mut self,
        scenario: &BenchmarkScenario,
        coordinator: &mut OptimizationCoordinator,
    ) -> Result<BenchmarkResult> {
        // Generate test data
        let (facts, _rules) = self.generate_test_data(&scenario.data_generator)?;

        // Create test network
        let mut network = ReteNetwork::new()?;

        // Run warmup iterations
        for _ in 0..self.config.warmup_iterations {
            self.execute_scenario_operations(scenario, &mut network, &facts, coordinator)?;
        }

        // Run measurement iterations
        let mut measurements = Vec::new();
        let mut memory_samples = Vec::new();

        for _ in 0..self.config.measurement_iterations {
            let start_memory = self.get_memory_usage(&network);
            let start_time = Instant::now();

            self.execute_scenario_operations(scenario, &mut network, &facts, coordinator)?;

            let duration = start_time.elapsed();
            let end_memory = self.get_memory_usage(&network);

            measurements.push(duration.as_secs_f64() * 1000.0); // Convert to milliseconds
            memory_samples.push(end_memory.saturating_sub(start_memory));
        }

        // Calculate statistics
        let timing_result = self.calculate_timing_stats(measurements);
        let memory_result = if self.config.include_memory_benchmarks {
            Some(self.calculate_memory_stats(memory_samples))
        } else {
            None
        };

        // Compare with baseline if available
        let baseline_comparison = self.baselines.get(&scenario.name).map(|baseline| {
            self.compare_with_baseline(&timing_result, memory_result.as_ref(), baseline)
        });

        // Determine if this is a regression or improvement
        let (is_regression, is_improvement) =
            self.analyze_performance_change(baseline_comparison.as_ref());

        Ok(BenchmarkResult {
            scenario_name: scenario.name.clone(),
            timing_result,
            memory_result,
            baseline_comparison,
            is_regression,
            is_improvement,
            notes: Vec::new(),
        })
    }

    /// Generate comprehensive benchmark summary
    pub fn generate_summary(&self, total_duration: Duration) -> BenchmarkSummary {
        let total_scenarios = self.results.len();
        let regressions = self.results.iter().filter(|r| r.is_regression).count();
        let improvements = self.results.iter().filter(|r| r.is_improvement).count();
        let stable = total_scenarios - regressions - improvements;

        let avg_performance_change = if !self.results.is_empty() {
            self.results
                .iter()
                .filter_map(|r| r.baseline_comparison.as_ref())
                .map(|c| c.performance_change_percent)
                .sum::<f64>()
                / self.results.len() as f64
        } else {
            0.0
        };

        BenchmarkSummary {
            total_scenarios,
            regressions,
            improvements,
            stable,
            avg_performance_change,
            total_duration,
            results: self.results.clone(),
            has_critical_regressions: self.has_critical_regressions(),
        }
    }

    // Private helper methods

    fn create_default_scenarios() -> Vec<BenchmarkScenario> {
        vec![
            BenchmarkScenario {
                name: "small_dataset_insertion".to_string(),
                description: "Fact insertion performance with small dataset (1K facts)".to_string(),
                data_generator: DataGenerator {
                    fact_count: 1000,
                    rule_count: 10,
                    fact_distribution: FactDistribution {
                        field_count: 5,
                        field_cardinality: HashMap::new(),
                        size_distribution: (0.7, 0.2, 0.1),
                        temporal_pattern: TemporalPattern::Uniform,
                    },
                    rule_complexity: RuleComplexity {
                        avg_conditions: 2,
                        max_join_depth: 2,
                        complex_patterns: false,
                        conflict_rate: 0.1,
                    },
                    seed: 12345,
                },
                operations: vec![BenchmarkOperation::FactInsertion { batch_size: 100 }],
                expected_complexity: ComplexityClass::Linear,
                critical_threshold_ms: Some(1000.0),
            },
            BenchmarkScenario {
                name: "large_dataset_lookup".to_string(),
                description: "Fact lookup performance with large dataset (100K facts)".to_string(),
                data_generator: DataGenerator {
                    fact_count: 100_000,
                    rule_count: 50,
                    fact_distribution: FactDistribution {
                        field_count: 10,
                        field_cardinality: HashMap::new(),
                        size_distribution: (0.6, 0.3, 0.1),
                        temporal_pattern: TemporalPattern::Recent,
                    },
                    rule_complexity: RuleComplexity {
                        avg_conditions: 3,
                        max_join_depth: 3,
                        complex_patterns: true,
                        conflict_rate: 0.2,
                    },
                    seed: 67890,
                },
                operations: vec![BenchmarkOperation::FactLookup {
                    lookup_count: 1000,
                    miss_rate: 0.1,
                }],
                expected_complexity: ComplexityClass::Logarithmic,
                critical_threshold_ms: Some(500.0),
            },
            BenchmarkScenario {
                name: "complex_rule_compilation".to_string(),
                description: "Rule compilation performance with complex patterns".to_string(),
                data_generator: DataGenerator {
                    fact_count: 10_000,
                    rule_count: 100,
                    fact_distribution: FactDistribution {
                        field_count: 15,
                        field_cardinality: HashMap::new(),
                        size_distribution: (0.5, 0.3, 0.2),
                        temporal_pattern: TemporalPattern::Clustered,
                    },
                    rule_complexity: RuleComplexity {
                        avg_conditions: 5,
                        max_join_depth: 4,
                        complex_patterns: true,
                        conflict_rate: 0.3,
                    },
                    seed: 54321,
                },
                operations: vec![BenchmarkOperation::RuleCompilation { rule_count: 50 }],
                expected_complexity: ComplexityClass::Linearithmic,
                critical_threshold_ms: Some(2000.0),
            },
            BenchmarkScenario {
                name: "optimization_performance".to_string(),
                description: "Network optimization performance under various loads".to_string(),
                data_generator: DataGenerator {
                    fact_count: 50_000,
                    rule_count: 25,
                    fact_distribution: FactDistribution {
                        field_count: 8,
                        field_cardinality: HashMap::new(),
                        size_distribution: (0.6, 0.3, 0.1),
                        temporal_pattern: TemporalPattern::Uniform,
                    },
                    rule_complexity: RuleComplexity {
                        avg_conditions: 3,
                        max_join_depth: 3,
                        complex_patterns: false,
                        conflict_rate: 0.15,
                    },
                    seed: 98765,
                },
                operations: vec![BenchmarkOperation::NetworkOptimization],
                expected_complexity: ComplexityClass::Linear,
                critical_threshold_ms: Some(5000.0),
            },
        ]
    }

    fn generate_test_data(&self, generator: &DataGenerator) -> Result<(Vec<Fact>, Vec<String>)> {
        let mut facts = Vec::with_capacity(generator.fact_count);
        let mut rules = Vec::with_capacity(generator.rule_count);

        // Use a simple PRNG for reproducible test data
        let mut rng_state = generator.seed;

        // Generate facts
        for i in 0..generator.fact_count {
            let mut fields = HashMap::new();

            for field_idx in 0..generator.fact_distribution.field_count {
                let field_name = format!("field_{}", field_idx);
                let field_value = self.generate_field_value(&mut rng_state, field_idx);
                fields.insert(field_name, field_value);
            }

            facts.push(Fact { id: i as FactId, data: FactData { fields } });
        }

        // Generate rule names (simplified for benchmarking)
        for i in 0..generator.rule_count {
            rules.push(format!("benchmark_rule_{}", i));
        }

        Ok((facts, rules))
    }

    fn generate_field_value(&self, rng_state: &mut u64, field_idx: usize) -> FactValue {
        // Simple LCG for reproducible pseudo-random values
        *rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let rand_val = (*rng_state >> 16) & 0x7fff;

        match field_idx % 4 {
            0 => FactValue::Integer(rand_val as i64),
            1 => FactValue::String(format!("value_{}", rand_val)),
            2 => FactValue::Boolean(rand_val % 2 == 0),
            3 => FactValue::Float((rand_val as f64) / 1000.0),
            _ => FactValue::Null,
        }
    }

    fn execute_scenario_operations(
        &self,
        scenario: &BenchmarkScenario,
        network: &mut ReteNetwork,
        facts: &[Fact],
        coordinator: &mut OptimizationCoordinator,
    ) -> Result<()> {
        for operation in &scenario.operations {
            match operation {
                BenchmarkOperation::FactInsertion { batch_size } => {
                    for chunk in facts.chunks(*batch_size) {
                        network.process_facts(chunk.to_vec())?;
                    }
                }
                BenchmarkOperation::FactLookup { lookup_count, miss_rate: _ } => {
                    // Perform fact processing to simulate lookups
                    for i in 0..*lookup_count {
                        let fact_index = i % facts.len();
                        if let Some(fact) = facts.get(fact_index) {
                            let _ = network.process_facts(vec![fact.clone()]);
                        }
                    }
                }
                BenchmarkOperation::RuleCompilation { rule_count: _ } => {
                    // Rule compilation happens during add_rule calls
                }
                BenchmarkOperation::RuleFiring { fire_count: _ } => {
                    // Process facts to trigger rule firing
                    network.process_facts(facts.to_vec())?;
                }
                BenchmarkOperation::NetworkOptimization => {
                    coordinator.run_optimization(network)?;
                }
                BenchmarkOperation::MemoryCleanup => {
                    network.perform_adaptive_memory_sizing()?;
                }
                BenchmarkOperation::ComplexQuery { query_complexity: _ } => {
                    // Perform complex queries based on complexity level
                    // Note: Complex query operations would be implemented here
                    // For now, we'll perform fact processing as a placeholder
                    let sample_facts: Vec<Fact> = facts.iter().take(100).cloned().collect();
                    let _ = network.process_facts(sample_facts)?;
                }
            }
        }
        Ok(())
    }

    fn get_memory_usage(&self, _network: &ReteNetwork) -> usize {
        // Simplified memory usage calculation
        // In a real implementation, this would use system memory APIs
        1024 * 1024 // Placeholder: 1MB
    }

    fn calculate_timing_stats(&self, measurements: Vec<f64>) -> TimingResult {
        let mut sorted_measurements = measurements.clone();
        sorted_measurements.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let count = measurements.len() as f64;
        let sum: f64 = measurements.iter().sum();
        let avg = sum / count;

        let variance = measurements.iter().map(|x| (x - avg).powi(2)).sum::<f64>() / count;
        let std_dev = variance.sqrt();

        let min = sorted_measurements.first().copied().unwrap_or(0.0);
        let max = sorted_measurements.last().copied().unwrap_or(0.0);
        let p95_index = ((count * 0.95) as usize).min(sorted_measurements.len() - 1);
        let p95 = sorted_measurements[p95_index];

        let ops_per_second = if avg > 0.0 { 1000.0 / avg } else { 0.0 };

        TimingResult {
            measurements_ms: measurements,
            timing_stats: TimingBaseline {
                avg_duration_ms: avg,
                std_dev_ms: std_dev,
                min_duration_ms: min,
                max_duration_ms: max,
                p95_duration_ms: p95,
            },
            ops_per_second,
        }
    }

    fn calculate_memory_stats(&self, memory_samples: Vec<usize>) -> MemoryResult {
        let peak_memory = memory_samples.iter().max().copied().unwrap_or(0);
        let avg_memory = if !memory_samples.is_empty() {
            memory_samples.iter().sum::<usize>() / memory_samples.len()
        } else {
            0
        };

        MemoryResult {
            memory_usage_samples: memory_samples,
            memory_stats: MemoryBaseline {
                peak_memory_bytes: peak_memory,
                avg_memory_bytes: avg_memory,
                memory_efficiency: if peak_memory > 0 {
                    1000.0 / (peak_memory as f64 / 1024.0 / 1024.0)
                } else {
                    0.0
                },
                allocation_count: 0, // Would need instrumentation
            },
            potential_leaks: Vec::new(),
        }
    }

    fn compare_with_baseline(
        &self,
        timing_result: &TimingResult,
        memory_result: Option<&MemoryResult>,
        baseline: &PerformanceBaseline,
    ) -> BaselineComparison {
        let performance_change = if baseline.timing_baseline.avg_duration_ms > 0.0 {
            ((baseline.timing_baseline.avg_duration_ms
                - timing_result.timing_stats.avg_duration_ms)
                / baseline.timing_baseline.avg_duration_ms)
                * 100.0
        } else {
            0.0
        };

        let memory_change = if let Some(memory) = memory_result {
            if baseline.memory_baseline.avg_memory_bytes > 0 {
                ((memory.memory_stats.avg_memory_bytes as f64
                    - baseline.memory_baseline.avg_memory_bytes as f64)
                    / baseline.memory_baseline.avg_memory_bytes as f64)
                    * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Simple statistical significance test (would use t-test in practice)
        let is_significant = performance_change.abs() > 1.0; // 1% change threshold

        BaselineComparison {
            performance_change_percent: performance_change,
            memory_change_percent: memory_change,
            is_statistically_significant: is_significant,
            confidence_level: 0.95, // Placeholder
        }
    }

    fn analyze_performance_change(&self, comparison: Option<&BaselineComparison>) -> (bool, bool) {
        if let Some(comp) = comparison {
            let is_regression =
                comp.performance_change_percent < -self.config.max_degradation_percent;
            let is_improvement =
                comp.performance_change_percent > self.config.min_improvement_percent;
            (is_regression, is_improvement)
        } else {
            (false, false)
        }
    }

    fn has_critical_regressions(&self) -> bool {
        self.results.iter().any(|r| {
            r.is_regression
                && r.baseline_comparison
                    .as_ref()
                    .map(|c| {
                        c.performance_change_percent.abs()
                            > self.config.max_degradation_percent * 2.0
                    })
                    .unwrap_or(false)
        })
    }
}

/// Summary of benchmark run results
#[derive(Debug, Clone)]
pub struct BenchmarkSummary {
    pub total_scenarios: usize,
    pub regressions: usize,
    pub improvements: usize,
    pub stable: usize,
    pub avg_performance_change: f64,
    pub total_duration: Duration,
    pub results: Vec<BenchmarkResult>,
    pub has_critical_regressions: bool,
}

impl BenchmarkSummary {
    /// Generate a human-readable report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# Performance Benchmark Report\n\n");
        report.push_str(&format!(
            "**Total Duration:** {:.2}s\n",
            self.total_duration.as_secs_f64()
        ));
        report.push_str(&format!("**Scenarios Run:** {}\n", self.total_scenarios));
        report.push_str(&format!(
            "**Average Performance Change:** {:.2}%\n\n",
            self.avg_performance_change
        ));

        report.push_str("## Summary\n\n");
        report.push_str(&format!(
            "- **Improvements:** {} scenarios\n",
            self.improvements
        ));
        report.push_str(&format!("- **Stable:** {} scenarios\n", self.stable));
        report.push_str(&format!(
            "- **Regressions:** {} scenarios\n",
            self.regressions
        ));

        if self.has_critical_regressions {
            report.push_str("\n⚠️  **CRITICAL REGRESSIONS DETECTED** ⚠️\n");
        }

        report.push_str("\n## Detailed Results\n\n");

        for result in &self.results {
            report.push_str(&format!("### {}\n\n", result.scenario_name));
            report.push_str(&format!(
                "- **Average Time:** {:.2}ms\n",
                result.timing_result.timing_stats.avg_duration_ms
            ));
            report.push_str(&format!(
                "- **Operations/Second:** {:.0}\n",
                result.timing_result.ops_per_second
            ));

            if let Some(comparison) = &result.baseline_comparison {
                let status = if result.is_regression {
                    "🔴 REGRESSION"
                } else if result.is_improvement {
                    "🟢 IMPROVEMENT"
                } else {
                    "🟡 STABLE"
                };

                report.push_str(&format!(
                    "- **Change:** {:.2}% ({})\n",
                    comparison.performance_change_percent, status
                ));
            }

            report.push('\n');
        }

        report
    }

    /// Check if benchmarks passed (no critical regressions)
    pub fn passed(&self) -> bool {
        !self.has_critical_regressions && self.regressions == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_suite_creation() {
        let suite = PerformanceBenchmarkSuite::new();
        assert!(!suite.scenarios.is_empty());
        assert_eq!(suite.results.len(), 0);
    }

    #[test]
    fn test_data_generator() {
        let generator = DataGenerator {
            fact_count: 100,
            rule_count: 5,
            fact_distribution: FactDistribution {
                field_count: 3,
                field_cardinality: HashMap::new(),
                size_distribution: (0.7, 0.2, 0.1),
                temporal_pattern: TemporalPattern::Uniform,
            },
            rule_complexity: RuleComplexity {
                avg_conditions: 2,
                max_join_depth: 2,
                complex_patterns: false,
                conflict_rate: 0.1,
            },
            seed: 12345,
        };

        let suite = PerformanceBenchmarkSuite::new();
        let result = suite.generate_test_data(&generator);
        assert!(result.is_ok());

        let (facts, rules) = result.unwrap();
        assert_eq!(facts.len(), 100);
        assert_eq!(rules.len(), 5);
    }

    #[test]
    fn test_timing_statistics() {
        let suite = PerformanceBenchmarkSuite::new();
        let measurements = vec![10.0, 15.0, 12.0, 18.0, 11.0];
        let result = suite.calculate_timing_stats(measurements);

        assert!(result.timing_stats.avg_duration_ms > 0.0);
        assert!(result.timing_stats.std_dev_ms >= 0.0);
        assert!(result.ops_per_second > 0.0);
    }

    #[test]
    fn test_baseline_comparison() {
        let suite = PerformanceBenchmarkSuite::new();

        let timing_result = TimingResult {
            measurements_ms: vec![10.0],
            timing_stats: TimingBaseline {
                avg_duration_ms: 10.0,
                std_dev_ms: 0.0,
                min_duration_ms: 10.0,
                max_duration_ms: 10.0,
                p95_duration_ms: 10.0,
            },
            ops_per_second: 100.0,
        };

        let baseline = PerformanceBaseline {
            scenario_name: "test".to_string(),
            timing_baseline: TimingBaseline {
                avg_duration_ms: 20.0, // Baseline was slower
                std_dev_ms: 0.0,
                min_duration_ms: 20.0,
                max_duration_ms: 20.0,
                p95_duration_ms: 20.0,
            },
            memory_baseline: MemoryBaseline {
                peak_memory_bytes: 1024,
                avg_memory_bytes: 1024,
                memory_efficiency: 1.0,
                allocation_count: 0,
            },
            scalability_baseline: ScalabilityBaseline {
                scale_performance: vec![],
                observed_complexity: ComplexityClass::Linear,
                scaling_coefficient: 1.0,
            },
            established_at: Instant::now(),
            version: "1.0.0".to_string(),
        };

        let comparison = suite.compare_with_baseline(&timing_result, None, &baseline);
        assert!(comparison.performance_change_percent > 0.0); // Should show improvement
    }

    #[test]
    fn test_benchmark_summary() {
        let results = vec![BenchmarkResult {
            scenario_name: "test1".to_string(),
            timing_result: TimingResult {
                measurements_ms: vec![10.0],
                timing_stats: TimingBaseline {
                    avg_duration_ms: 10.0,
                    std_dev_ms: 0.0,
                    min_duration_ms: 10.0,
                    max_duration_ms: 10.0,
                    p95_duration_ms: 10.0,
                },
                ops_per_second: 100.0,
            },
            memory_result: None,
            baseline_comparison: None,
            is_regression: false,
            is_improvement: false,
            notes: vec![],
        }];

        let mut suite = PerformanceBenchmarkSuite::new();
        suite.results = results;

        let summary = suite.generate_summary(Duration::from_secs(5));
        assert_eq!(summary.total_scenarios, 1);
        assert_eq!(summary.stable, 1);
        assert!(!summary.has_critical_regressions);
        assert!(summary.passed());
    }
}
