//! Comprehensive demonstration of the RETE network optimization framework
//!
//! This module provides examples and demonstrations of how to use the complete
//! optimization system for enterprise-grade RETE network performance.

use crate::adaptive_backends::{
    AccessPattern, AdaptationConfig, AdaptiveFactStore, DataDistribution, DatasetCharacteristics,
};
use crate::advanced_indexing::{AdvancedFieldIndexer, IndexStrategyType};
use crate::bloom_filter::{FactBloomConfig, FactBloomFilter};
use crate::optimization_coordinator::{OptimizationConfig, OptimizationCoordinator};
use crate::performance_regression_testing::TemporalPattern;
use crate::performance_regression_testing::{BenchmarkConfig, PerformanceBenchmarkSuite};
use crate::rete_network::ReteNetwork;
use crate::types::{Fact, FactData, FactId, FactValue};
use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;

/// Comprehensive optimization framework demonstration
pub struct OptimizationDemo {
    /// The RETE network being optimized
    network: ReteNetwork,
    /// Main optimization coordinator
    coordinator: OptimizationCoordinator,
    /// Adaptive fact storage
    adaptive_store: AdaptiveFactStore,
    /// Advanced field indexer
    field_indexer: AdvancedFieldIndexer,
    /// Bloom filter for fast existence checks
    bloom_filter: FactBloomFilter,
    /// Performance benchmark suite
    benchmark_suite: PerformanceBenchmarkSuite,
}

impl OptimizationDemo {
    /// Create a new optimization demo with enterprise configuration
    pub fn new() -> Result<Self> {
        // Create optimized RETE network
        let network = ReteNetwork::new()?;

        // Configure optimization coordinator for enterprise workloads
        let opt_config = OptimizationConfig {
            auto_optimize: true,
            optimization_interval: Duration::from_secs(30),
            memory_pressure_threshold: crate::memory_profiler::MemoryPressureLevel::Moderate,
            enable_adaptive_backends: true,
            enable_advanced_indexing: true,
            enable_bloom_filters: true,
            max_memory_usage: 2 * 1024 * 1024 * 1024, // 2GB
            target_improvement: 0.15,                 // 15% improvement target
            monitoring_window: 100,
        };
        let coordinator = OptimizationCoordinator::new(opt_config)?;

        // Configure adaptive fact store for large datasets
        let dataset_characteristics = DatasetCharacteristics {
            fact_count: 100_000,
            avg_fact_size: 512,
            read_write_ratio: 0.8, // Read-heavy workload
            miss_rate: 0.05,       // 5% miss rate
            hot_fields: vec![
                "entity_id".to_string(),
                "timestamp".to_string(),
                "status".to_string(),
            ],
            fields_per_fact: 8.0,
            memory_budget: 512 * 1024 * 1024, // 512MB
            growth_rate: 100.0,               // 100 facts per second
            access_patterns: AccessPattern::Recency,
            distribution: DataDistribution {
                field_cardinality: HashMap::new(),
                size_distribution: (0.6, 0.3, 0.1), // Most facts are small
                temporal_skew: 0.4,
            },
        };

        let adaptation_config = AdaptationConfig {
            min_adaptation_interval: Duration::from_secs(60),
            performance_threshold: 0.85,
            memory_threshold: 0.80,
            auto_adapt: true,
            adaptation_aggressiveness: 0.6,
            measurement_window: 100,
        };

        let adaptive_store = AdaptiveFactStore::new(dataset_characteristics, adaptation_config);

        // Configure advanced field indexer with bloom filter
        let mut field_indexer = AdvancedFieldIndexer::with_bloom_filter(Some(100_000));

        // Add strategic fields for indexing
        field_indexer
            .add_field_with_strategy("entity_id".to_string(), IndexStrategyType::HighCardinality);
        field_indexer
            .add_field_with_strategy("status".to_string(), IndexStrategyType::LowCardinality);
        field_indexer.add_field_with_strategy("score".to_string(), IndexStrategyType::Numeric);
        field_indexer.add_field_with_strategy("metadata".to_string(), IndexStrategyType::Hybrid);

        // Configure bloom filter for optimal performance
        let bloom_config = FactBloomConfig {
            enable_field_filtering: true,
            target_false_positive_rate: 0.01, // 1%
            auto_resize: true,
            resize_threshold: 0.7,
        };
        let bloom_filter = FactBloomFilter::new(100_000, bloom_config);

        // Configure comprehensive benchmark suite
        let benchmark_config = BenchmarkConfig {
            warmup_iterations: 5,
            measurement_iterations: 20,
            max_degradation_percent: 3.0, // Strict 3% regression threshold
            min_improvement_percent: 1.0, // 1% improvement considered significant
            include_memory_benchmarks: true,
            include_scalability_tests: true,
            scenario_timeout: Duration::from_secs(300),
        };
        let benchmark_suite = PerformanceBenchmarkSuite::with_config(benchmark_config);

        Ok(Self {
            network,
            coordinator,
            adaptive_store,
            field_indexer,
            bloom_filter,
            benchmark_suite,
        })
    }

    /// Demonstrate the complete optimization workflow
    pub fn demonstrate_optimization_workflow(&mut self) -> Result<OptimizationReport> {
        println!("🚀 Starting Enterprise RETE Network Optimization Demo");
        println!("{}", "=".repeat(60));

        // Step 1: Generate realistic test data
        println!("\n📊 Step 1: Generating enterprise-scale test dataset...");
        let facts = self.generate_enterprise_test_data(50_000)?;
        println!(
            "✅ Generated {} facts with realistic enterprise patterns",
            facts.len()
        );

        // Step 2: Populate optimization components
        println!("\n🔍 Step 2: Populating optimization components...");
        self.populate_optimization_components(&facts)?;
        println!("✅ Populated adaptive storage, indexers, and bloom filters");

        // Step 3: Run initial performance baseline
        println!("\n📈 Step 3: Establishing performance baseline...");
        let baseline_metrics = self.establish_performance_baseline()?;
        println!(
            "✅ Baseline established - Average processing: {:.2}ms",
            baseline_metrics.avg_processing_time
        );

        // Step 4: Apply optimizations
        println!("\n⚙️ Step 4: Applying comprehensive optimizations...");
        let optimization_result = self.coordinator.run_optimization(&mut self.network)?;
        self.print_optimization_results(&optimization_result);

        // Step 5: Measure performance improvements
        println!("\n📊 Step 5: Measuring performance improvements...");
        let improved_metrics = self.measure_optimized_performance()?;
        let improvement = self.calculate_improvement(&baseline_metrics, &improved_metrics);
        self.print_performance_comparison(&baseline_metrics, &improved_metrics, &improvement);

        // Step 6: Run comprehensive benchmarks
        println!("\n🏃 Step 6: Running comprehensive benchmark suite...");
        // Create a new coordinator for benchmarking to avoid ownership issues
        let benchmark_coordinator =
            OptimizationCoordinator::new(crate::optimization_coordinator::OptimizationConfig {
                auto_optimize: true,
                optimization_interval: std::time::Duration::from_secs(30),
                memory_pressure_threshold: crate::memory_profiler::MemoryPressureLevel::Moderate,
                enable_adaptive_backends: true,
                enable_advanced_indexing: true,
                enable_bloom_filters: true,
                max_memory_usage: 2 * 1024 * 1024 * 1024,
                target_improvement: 0.15,
                monitoring_window: 100,
            })?;
        let benchmark_results = self.benchmark_suite.run_all_benchmarks(benchmark_coordinator)?;
        self.print_benchmark_results(&benchmark_results);

        // Step 7: Generate optimization report
        println!("\n📋 Step 7: Generating comprehensive optimization report...");
        let optimization_report = self.generate_comprehensive_report(
            &baseline_metrics,
            &improved_metrics,
            &improvement,
            &benchmark_results,
        );

        println!("\n🎉 Optimization Demo Complete!");
        println!("{}", "=".repeat(60));

        Ok(optimization_report)
    }

    /// Generate realistic enterprise test data
    fn generate_enterprise_test_data(&self, count: usize) -> Result<Vec<Fact>> {
        let mut facts = Vec::with_capacity(count);

        // Simulate realistic enterprise data patterns
        let statuses = ["active", "pending", "completed", "failed", "archived"];
        let departments = ["engineering", "sales", "marketing", "support", "finance"];
        let priorities = ["low", "medium", "high", "critical"];

        for i in 0..count {
            let mut fields = HashMap::new();

            // Entity identification
            fields.insert("entity_id".to_string(), FactValue::Integer(i as i64));
            fields.insert("user_id".to_string(), FactValue::Integer((i % 1000) as i64));

            // Status and workflow fields
            fields.insert(
                "status".to_string(),
                FactValue::String(statuses[i % statuses.len()].to_string()),
            );
            fields.insert(
                "department".to_string(),
                FactValue::String(departments[i % departments.len()].to_string()),
            );
            fields.insert(
                "priority".to_string(),
                FactValue::String(priorities[i % priorities.len()].to_string()),
            );

            // Numeric metrics
            fields.insert(
                "score".to_string(),
                FactValue::Float((i as f64 * 1.337) % 100.0),
            );
            fields.insert(
                "timestamp".to_string(),
                FactValue::Integer(1640995200 + (i * 3600) as i64),
            ); // Hourly progression

            // Complex metadata
            let metadata = format!(
                "{{\"batch_id\": {}, \"source\": \"enterprise_demo\", \"version\": \"1.0\"}}",
                i / 100
            );
            fields.insert("metadata".to_string(), FactValue::String(metadata));

            facts.push(Fact { id: i as FactId, data: FactData { fields } });
        }

        Ok(facts)
    }

    /// Populate all optimization components with test data
    fn populate_optimization_components(&mut self, facts: &[Fact]) -> Result<()> {
        for fact in facts {
            // Add to adaptive store
            self.adaptive_store.insert(fact.clone());

            // Add to field indexer
            self.field_indexer.index_fact(fact);

            // Add to bloom filter
            self.bloom_filter.add_fact(fact);
        }

        // Optimize indexing strategies based on data patterns
        let fact_refs: Vec<&crate::types::Fact> = facts.iter().collect();
        self.field_indexer.optimize_indexes(&fact_refs);

        Ok(())
    }

    /// Establish performance baseline before optimizations
    fn establish_performance_baseline(&mut self) -> Result<PerformanceMetrics> {
        let start_time = std::time::Instant::now();

        // Simulate typical enterprise workload
        let test_facts = self.generate_enterprise_test_data(1000)?;
        let _ = self.network.process_facts(test_facts)?;

        let processing_time = start_time.elapsed().as_millis() as f64;

        Ok(PerformanceMetrics {
            avg_processing_time: processing_time,
            memory_usage: 0, // Would be measured in real implementation
            throughput: 1000.0 / (processing_time / 1000.0),
            cache_hit_rate: 0.0,
            index_efficiency: 0.0,
        })
    }

    /// Measure performance after optimizations
    fn measure_optimized_performance(&mut self) -> Result<PerformanceMetrics> {
        let start_time = std::time::Instant::now();

        // Same workload as baseline
        let test_facts = self.generate_enterprise_test_data(1000)?;
        let _ = self.network.process_facts(test_facts)?;

        let processing_time = start_time.elapsed().as_millis() as f64;

        // Get stats from optimization components
        let adaptive_stats = self.adaptive_store.stats();
        let indexer_stats = self.field_indexer.get_stats();
        let bloom_stats = self.bloom_filter.stats();

        Ok(PerformanceMetrics {
            avg_processing_time: processing_time,
            memory_usage: 0, // Would be measured in real implementation
            throughput: 1000.0 / (processing_time / 1000.0),
            cache_hit_rate: adaptive_stats.hit_rate,
            index_efficiency: indexer_stats.avg_lookup_time_micros,
        })
    }

    /// Calculate improvement metrics
    fn calculate_improvement(
        &self,
        baseline: &PerformanceMetrics,
        improved: &PerformanceMetrics,
    ) -> ImprovementMetrics {
        let processing_improvement = if baseline.avg_processing_time > 0.0 {
            ((baseline.avg_processing_time - improved.avg_processing_time)
                / baseline.avg_processing_time)
                * 100.0
        } else {
            0.0
        };

        let throughput_improvement =
            ((improved.throughput - baseline.throughput) / baseline.throughput.max(1.0)) * 100.0;

        ImprovementMetrics {
            processing_time_improvement: processing_improvement.max(0.0),
            throughput_improvement: throughput_improvement.max(0.0),
            memory_efficiency_improvement: 0.0, // Would be calculated in real implementation
            overall_improvement: (processing_improvement + throughput_improvement) / 2.0,
        }
    }

    /// Print optimization results
    fn print_optimization_results(
        &self,
        result: &crate::optimization_coordinator::OptimizationResult,
    ) {
        match result {
            crate::optimization_coordinator::OptimizationResult::Success {
                improvements,
                actions_taken,
            } => {
                println!("✅ Optimization successful!");
                println!(
                    "   💾 Memory reduction: {} bytes",
                    improvements.memory_reduction
                );
                println!(
                    "   ⚡ Lookup improvement: {:.1}%",
                    improvements.lookup_improvement
                );
                println!(
                    "   📈 Overall improvement: {:.1}%",
                    improvements.overall_improvement
                );
                println!("   🔧 Actions taken: {}", actions_taken.len());
            }
            crate::optimization_coordinator::OptimizationResult::Skipped(reason) => {
                println!("⏭️ Optimization skipped: {}", reason);
            }
            crate::optimization_coordinator::OptimizationResult::Failed(error) => {
                println!("❌ Optimization failed: {}", error);
            }
        }
    }

    /// Print performance comparison
    fn print_performance_comparison(
        &self,
        baseline: &PerformanceMetrics,
        improved: &PerformanceMetrics,
        improvement: &ImprovementMetrics,
    ) {
        println!("📊 Performance Comparison:");
        println!(
            "   ⏱️  Processing Time: {:.2}ms → {:.2}ms ({:.1}% improvement)",
            baseline.avg_processing_time,
            improved.avg_processing_time,
            improvement.processing_time_improvement
        );
        println!(
            "   🚀 Throughput: {:.0} ops/s → {:.0} ops/s ({:.1}% improvement)",
            baseline.throughput, improved.throughput, improvement.throughput_improvement
        );
        println!(
            "   🎯 Cache Hit Rate: {:.1}% → {:.1}%",
            baseline.cache_hit_rate * 100.0,
            improved.cache_hit_rate * 100.0
        );
        println!(
            "   📈 Overall Improvement: {:.1}%",
            improvement.overall_improvement
        );
    }

    /// Print benchmark results
    fn print_benchmark_results(
        &self,
        results: &crate::performance_regression_testing::BenchmarkSummary,
    ) {
        println!("🏆 Benchmark Results:");
        println!("   📊 Total Scenarios: {}", results.total_scenarios);
        println!("   🟢 Improvements: {}", results.improvements);
        println!("   🟡 Stable: {}", results.stable);
        println!("   🔴 Regressions: {}", results.regressions);
        println!(
            "   📈 Average Change: {:.1}%",
            results.avg_performance_change
        );

        if results.passed() {
            println!("   ✅ All benchmarks PASSED!");
        } else {
            println!("   ⚠️  Some benchmarks failed - review needed");
        }
    }

    /// Generate comprehensive optimization report
    fn generate_comprehensive_report(
        &self,
        baseline: &PerformanceMetrics,
        improved: &PerformanceMetrics,
        improvement: &ImprovementMetrics,
        benchmarks: &crate::performance_regression_testing::BenchmarkSummary,
    ) -> OptimizationReport {
        OptimizationReport {
            baseline_metrics: baseline.clone(),
            optimized_metrics: improved.clone(),
            improvement_metrics: improvement.clone(),
            benchmark_summary: format!(
                "{} scenarios, {} improvements, {} regressions",
                benchmarks.total_scenarios, benchmarks.improvements, benchmarks.regressions
            ),
            adaptive_store_effectiveness: self.adaptive_store.stats().hit_rate,
            indexing_efficiency: self.field_indexer.get_stats().avg_lookup_time_micros,
            bloom_filter_effectiveness: self.bloom_filter.stats().effectiveness,
            optimization_recommendations: self.generate_recommendations(improvement),
        }
    }

    /// Generate optimization recommendations
    fn generate_recommendations(&self, improvement: &ImprovementMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();

        if improvement.overall_improvement > 20.0 {
            recommendations.push("🎉 Excellent optimization results! Consider applying these settings to production.".to_string());
        } else if improvement.overall_improvement > 10.0 {
            recommendations.push(
                "✅ Good optimization results. Monitor production performance after deployment."
                    .to_string(),
            );
        } else if improvement.overall_improvement > 5.0 {
            recommendations.push(
                "📈 Moderate improvements achieved. Consider additional optimization strategies."
                    .to_string(),
            );
        } else {
            recommendations.push("⚠️ Limited improvements. Investigate dataset characteristics and workload patterns.".to_string());
        }

        // Component-specific recommendations
        let adaptive_stats = self.adaptive_store.stats();
        if adaptive_stats.hit_rate < 0.8 {
            recommendations.push(
                "🔄 Consider tuning adaptive storage cache size or strategy selection.".to_string(),
            );
        }

        let bloom_stats = self.bloom_filter.stats();
        if bloom_stats.effectiveness < 70.0 {
            recommendations.push("🌸 Bloom filter effectiveness is low - consider resizing or tuning false positive rate.".to_string());
        }

        recommendations
    }
}

/// Performance metrics for comparison
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub avg_processing_time: f64,
    pub memory_usage: usize,
    pub throughput: f64,
    pub cache_hit_rate: f64,
    pub index_efficiency: f64,
}

/// Improvement metrics between baseline and optimized performance
#[derive(Debug, Clone)]
pub struct ImprovementMetrics {
    pub processing_time_improvement: f64,
    pub throughput_improvement: f64,
    pub memory_efficiency_improvement: f64,
    pub overall_improvement: f64,
}

/// Comprehensive optimization report
#[derive(Debug, Clone)]
pub struct OptimizationReport {
    pub baseline_metrics: PerformanceMetrics,
    pub optimized_metrics: PerformanceMetrics,
    pub improvement_metrics: ImprovementMetrics,
    pub benchmark_summary: String,
    pub adaptive_store_effectiveness: f64,
    pub indexing_efficiency: f64,
    pub bloom_filter_effectiveness: f64,
    pub optimization_recommendations: Vec<String>,
}

impl OptimizationReport {
    /// Generate a detailed report string
    pub fn generate_detailed_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# Enterprise RETE Network Optimization Report\n\n");

        report.push_str("## Executive Summary\n\n");
        report.push_str(&format!(
            "Overall Performance Improvement: **{:.1}%**\n\n",
            self.improvement_metrics.overall_improvement
        ));

        report.push_str("## Performance Metrics\n\n");
        report.push_str("| Metric | Baseline | Optimized | Improvement |\n");
        report.push_str("|--------|----------|-----------|-------------|\n");
        report.push_str(&format!(
            "| Processing Time | {:.2}ms | {:.2}ms | {:.1}% |\n",
            self.baseline_metrics.avg_processing_time,
            self.optimized_metrics.avg_processing_time,
            self.improvement_metrics.processing_time_improvement
        ));
        report.push_str(&format!(
            "| Throughput | {:.0} ops/s | {:.0} ops/s | {:.1}% |\n",
            self.baseline_metrics.throughput,
            self.optimized_metrics.throughput,
            self.improvement_metrics.throughput_improvement
        ));
        report.push_str(&format!(
            "| Cache Hit Rate | {:.1}% | {:.1}% | - |\n\n",
            self.baseline_metrics.cache_hit_rate * 100.0,
            self.optimized_metrics.cache_hit_rate * 100.0
        ));

        report.push_str("## Component Effectiveness\n\n");
        report.push_str(&format!(
            "- **Adaptive Storage**: {:.1}% hit rate\n",
            self.adaptive_store_effectiveness * 100.0
        ));
        report.push_str(&format!(
            "- **Bloom Filter**: {:.1}% effectiveness\n",
            self.bloom_filter_effectiveness
        ));
        report.push_str(&format!(
            "- **Advanced Indexing**: {:.2}μs average lookup\n\n",
            self.indexing_efficiency
        ));

        report.push_str("## Benchmark Results\n\n");
        report.push_str(&format!("{}\n\n", self.benchmark_summary));

        report.push_str("## Recommendations\n\n");
        for (i, recommendation) in self.optimization_recommendations.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, recommendation));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_demo_creation() {
        let demo = OptimizationDemo::new();
        assert!(demo.is_ok(), "Should be able to create optimization demo");
    }

    #[test]
    fn test_enterprise_data_generation() {
        let demo = OptimizationDemo::new().unwrap();
        let facts = demo.generate_enterprise_test_data(1000).unwrap();

        assert_eq!(facts.len(), 1000);
        assert!(
            facts.iter().all(|f| f.data.fields.len() >= 7),
            "All facts should have required fields"
        );
    }

    #[test]
    fn test_performance_metrics_calculation() {
        let baseline = PerformanceMetrics {
            avg_processing_time: 100.0,
            memory_usage: 1000,
            throughput: 10.0,
            cache_hit_rate: 0.5,
            index_efficiency: 50.0,
        };

        let improved = PerformanceMetrics {
            avg_processing_time: 80.0,
            memory_usage: 900,
            throughput: 12.0,
            cache_hit_rate: 0.7,
            index_efficiency: 40.0,
        };

        let demo = OptimizationDemo::new().unwrap();
        let improvement = demo.calculate_improvement(&baseline, &improved);

        assert!(improvement.processing_time_improvement > 0.0);
        assert!(improvement.throughput_improvement > 0.0);
        assert!(improvement.overall_improvement > 0.0);
    }
}
