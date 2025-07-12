//! Rule Optimization Module for RETE Network
//!
//! This module implements advanced RETE optimizations to improve rule processing performance
//! through intelligent rule reordering, condition optimization, and network structure analysis.
//!
//! ## Phase 5: Advanced RETE Optimizations
//!
//! ### Rule Reordering Strategies
//! - **Selectivity-based Ordering**: Place most selective conditions first
//! - **Cost-based Optimization**: Optimize for minimal evaluation cost
//! - **Join Selectivity**: Arrange joins to minimize intermediate results
//! - **Fact Frequency Analysis**: Use runtime statistics for optimization
//!
//! ### Condition Optimization Techniques
//! - **Condition Merging**: Combine compatible conditions
//! - **Predicate Pushdown**: Move simple conditions before complex ones
//! - **Index Optimization**: Leverage alpha memory indexing patterns
//! - **Cross-Rule Optimization**: Share conditions across multiple rules

use crate::types::{Condition, FactValue, Operator, Rule, RuleId};
use std::collections::HashMap;
use tracing::{debug, info, instrument};

/// Statistics about condition selectivity and performance
#[derive(Debug, Clone)]
pub struct ConditionStats {
    /// Number of facts that typically match this condition
    pub average_matches: f64,
    /// Standard deviation of match count
    pub match_variance: f64,
    /// Evaluation cost (microseconds)
    pub avg_evaluation_cost_us: f64,
    /// Frequency of this condition pattern in the rule set
    pub pattern_frequency: usize,
    /// Last update timestamp for statistics
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for ConditionStats {
    fn default() -> Self {
        Self {
            average_matches: 100.0, // Conservative default
            match_variance: 50.0,
            avg_evaluation_cost_us: 10.0, // 10 microseconds default
            pattern_frequency: 1,
            last_updated: chrono::Utc::now(),
        }
    }
}

/// Advanced rule optimization engine
#[derive(Debug)]
pub struct RuleOptimizer {
    /// Condition statistics for optimization decisions
    condition_stats: HashMap<String, ConditionStats>,
    /// Optimization metrics and performance tracking
    optimization_metrics: OptimizationMetrics,
    /// Configuration for optimization strategies
    config: OptimizerConfig,
}

/// Configuration for rule optimization strategies
#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// Enable selectivity-based condition reordering
    pub enable_selectivity_ordering: bool,
    /// Enable cost-based optimization
    pub enable_cost_based_optimization: bool,
    /// Enable cross-rule condition sharing
    pub enable_condition_sharing: bool,
    /// Minimum selectivity difference to trigger reordering
    pub min_selectivity_difference: f64,
    /// Maximum conditions to analyze per rule
    pub max_conditions_per_analysis: usize,
    /// Enable runtime statistics collection
    pub enable_runtime_statistics: bool,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            enable_selectivity_ordering: true,
            enable_cost_based_optimization: true,
            enable_condition_sharing: true,
            min_selectivity_difference: 0.2, // 20% difference threshold
            max_conditions_per_analysis: 10,
            enable_runtime_statistics: true,
        }
    }
}

/// Metrics tracking optimization effectiveness
#[derive(Debug, Clone, Default)]
pub struct OptimizationMetrics {
    /// Number of rules optimized
    pub rules_optimized: usize,
    /// Number of conditions reordered
    pub conditions_reordered: usize,
    /// Number of conditions merged
    pub conditions_merged: usize,
    /// Average performance improvement percentage
    pub avg_performance_improvement: f64,
    /// Total optimization time in milliseconds
    pub total_optimization_time_ms: u64,
    /// Number of shared condition patterns identified
    pub shared_patterns_found: usize,
}

/// Results of rule optimization analysis
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// Original rule
    pub original_rule: Rule,
    /// Optimized rule with reordered conditions
    pub optimized_rule: Rule,
    /// Estimated performance improvement percentage
    pub estimated_improvement: f64,
    /// Optimization strategies applied
    pub strategies_applied: Vec<OptimizationStrategy>,
    /// Detailed analysis of the optimization
    pub analysis: OptimizationAnalysis,
}

/// Types of optimization strategies that can be applied
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationStrategy {
    /// Reordered conditions based on selectivity
    SelectivityReordering { from_index: usize, to_index: usize },
    /// Combined multiple conditions into a single test
    ConditionMerging { merged_indices: Vec<usize> },
    /// Moved expensive conditions later in evaluation order
    CostBasedReordering { high_cost_indices: Vec<usize> },
    /// Applied predicate pushdown optimization
    PredicatePushdown { pushed_conditions: Vec<usize> },
    /// Identified shared condition patterns
    ConditionSharing { pattern_key: String, shared_rules: Vec<RuleId> },
}

/// Detailed analysis of optimization decisions
#[derive(Debug, Clone)]
pub struct OptimizationAnalysis {
    /// Selectivity score for each condition (0.0 = most selective, 1.0 = least selective)
    pub condition_selectivity: Vec<f64>,
    /// Estimated evaluation cost for each condition (microseconds)
    pub condition_costs: Vec<f64>,
    /// Join selectivity analysis for multi-condition rules
    pub join_analysis: Option<JoinAnalysis>,
    /// Shared patterns with other rules
    pub shared_patterns: Vec<SharedPattern>,
    /// Estimated total rule evaluation improvement
    pub total_improvement_estimate: f64,
}

/// Analysis of join operations in multi-condition rules
#[derive(Debug, Clone)]
pub struct JoinAnalysis {
    /// Estimated intermediate result sizes for each join
    pub intermediate_sizes: Vec<usize>,
    /// Join selectivity factors
    pub join_selectivity: Vec<f64>,
    /// Recommended join order
    pub optimal_join_order: Vec<usize>,
    /// Cross-condition correlation analysis
    pub condition_correlations: HashMap<(usize, usize), f64>,
}

/// Shared pattern information for condition optimization
#[derive(Debug, Clone)]
pub struct SharedPattern {
    /// Pattern key for identification
    pub pattern_key: String,
    /// Rules that share this pattern
    pub sharing_rules: Vec<RuleId>,
    /// Frequency of this pattern
    pub frequency: usize,
    /// Potential alpha memory savings
    pub memory_savings_estimate: usize,
}

impl RuleOptimizer {
    /// Create a new rule optimizer with default configuration
    pub fn new() -> Self {
        Self::with_config(OptimizerConfig::default())
    }

    /// Create a new rule optimizer with custom configuration
    pub fn with_config(config: OptimizerConfig) -> Self {
        Self {
            condition_stats: HashMap::new(),
            optimization_metrics: OptimizationMetrics::default(),
            config,
        }
    }

    /// Optimize a single rule using available statistics and configuration
    #[instrument(skip(self))]
    pub fn optimize_rule(&mut self, rule: Rule) -> OptimizationResult {
        info!(
            "Optimizing rule {} with {} conditions",
            rule.id,
            rule.conditions.len()
        );

        let start_time = std::time::Instant::now();
        let original_rule = rule.clone();
        let mut optimized_rule = rule.clone();
        let mut strategies_applied = Vec::new();
        let mut estimated_improvement = 0.0;

        // Analyze condition selectivity and costs
        let analysis = self.analyze_rule_conditions(&rule);

        // Apply selectivity-based reordering if enabled
        if self.config.enable_selectivity_ordering {
            if let Some(reordering) =
                self.apply_selectivity_reordering(&mut optimized_rule, &analysis)
            {
                strategies_applied.push(reordering.0);
                estimated_improvement += reordering.1;
                self.optimization_metrics.conditions_reordered += 1;
            }
        }

        // Apply cost-based optimization if enabled
        if self.config.enable_cost_based_optimization {
            if let Some(cost_optimization) =
                self.apply_cost_based_optimization(&mut optimized_rule, &analysis)
            {
                strategies_applied.push(cost_optimization.0);
                estimated_improvement += cost_optimization.1;
            }
        }

        // Identify condition sharing opportunities
        if self.config.enable_condition_sharing {
            let sharing_analysis = self.analyze_condition_sharing(&rule);
            for shared_pattern in &sharing_analysis {
                strategies_applied.push(OptimizationStrategy::ConditionSharing {
                    pattern_key: shared_pattern.pattern_key.clone(),
                    shared_rules: shared_pattern.sharing_rules.clone(),
                });
                self.optimization_metrics.shared_patterns_found += 1;
            }
        }

        let optimization_time = start_time.elapsed().as_millis() as u64;
        self.optimization_metrics.total_optimization_time_ms += optimization_time;
        self.optimization_metrics.rules_optimized += 1;

        if estimated_improvement > 0.0 {
            self.optimization_metrics.avg_performance_improvement =
                (self.optimization_metrics.avg_performance_improvement
                    * (self.optimization_metrics.rules_optimized - 1) as f64
                    + estimated_improvement)
                    / self.optimization_metrics.rules_optimized as f64;
        }

        debug!(
            "Rule {} optimization completed: {:.1}% improvement, {} strategies applied",
            rule.id,
            estimated_improvement,
            strategies_applied.len()
        );

        OptimizationResult {
            original_rule,
            optimized_rule,
            estimated_improvement,
            strategies_applied,
            analysis,
        }
    }

    /// Analyze conditions in a rule for optimization opportunities
    fn analyze_rule_conditions(&self, rule: &Rule) -> OptimizationAnalysis {
        let mut condition_selectivity = Vec::new();
        let mut condition_costs = Vec::new();

        for (index, condition) in rule.conditions.iter().enumerate() {
            // Calculate selectivity (lower = more selective = better to evaluate first)
            let selectivity = self.calculate_condition_selectivity(condition);
            condition_selectivity.push(selectivity);

            // Calculate evaluation cost
            let cost = Self::calculate_condition_cost(condition);
            condition_costs.push(cost);

            debug!(
                "Condition {}: selectivity={:.3}, cost={:.1}μs",
                index, selectivity, cost
            );
        }

        // Analyze joins for multi-condition rules
        let join_analysis = if rule.conditions.len() > 1 {
            Some(self.analyze_join_patterns(&rule.conditions))
        } else {
            None
        };

        // Analyze shared patterns
        let shared_patterns = self.analyze_condition_sharing(rule);

        // Calculate total improvement estimate
        let total_improvement_estimate = self.estimate_total_improvement(
            &condition_selectivity,
            &condition_costs,
            &join_analysis,
        );

        OptimizationAnalysis {
            condition_selectivity,
            condition_costs,
            join_analysis,
            shared_patterns,
            total_improvement_estimate,
        }
    }

    /// Calculate selectivity for a condition (0.0 = most selective, 1.0 = least selective)
    pub fn calculate_condition_selectivity(&self, condition: &Condition) -> f64 {
        match condition {
            Condition::Simple { field, operator, value } => {
                let pattern_key = format!("{field}_{operator:?}_{value:?}");

                if let Some(stats) = self.condition_stats.get(&pattern_key) {
                    // Use statistics if available
                    (stats.average_matches / 1000.0).min(1.0)
                } else {
                    // Estimate based on operator and value type
                    Self::estimate_selectivity_heuristic(operator, value)
                }
            }
            Condition::And { conditions } => {
                // AND conditions are more selective (multiplication of probabilities)
                conditions
                    .iter()
                    .map(|c| self.calculate_condition_selectivity(c))
                    .fold(1.0, |acc, sel| acc * sel)
            }
            Condition::Or { conditions } => {
                // OR conditions are less selective (addition of probabilities, capped at 1.0)
                conditions
                    .iter()
                    .map(|c| self.calculate_condition_selectivity(c))
                    .fold(0.0, |acc, sel| (acc + sel).min(1.0))
            }
            _ => 0.5, // Default for complex conditions
        }
    }

    /// Estimate selectivity using heuristics when no statistics are available
    fn estimate_selectivity_heuristic(_operator: &Operator, value: &FactValue) -> f64 {
        match _operator {
            Operator::Equal => {
                match value {
                    FactValue::Boolean(_) => 0.5,                // 50% chance for boolean
                    FactValue::String(s) if s.len() > 10 => 0.1, // Long strings are selective
                    FactValue::String(_) => 0.3,                 // Short strings less selective
                    FactValue::Integer(_) => 0.2,                // Specific integers are selective
                    FactValue::Float(_) => 0.15, // Specific floats are very selective
                    _ => 0.25,
                }
            }
            Operator::NotEqual => {
                // NotEqual is inverse selectivity
                1.0 - Self::estimate_selectivity_heuristic(&Operator::Equal, value)
            }
            Operator::GreaterThan | Operator::LessThan => 0.4, // Range queries are moderately selective
            Operator::GreaterThanOrEqual | Operator::LessThanOrEqual => 0.5,
            Operator::Contains | Operator::StartsWith | Operator::EndsWith => 0.3, // String matching
        }
    }

    /// Calculate evaluation cost for a condition in microseconds
    pub fn calculate_condition_cost(condition: &Condition) -> f64 {
        match condition {
            Condition::Simple { operator, value, .. } => {
                let base_cost = match operator {
                    Operator::Equal | Operator::NotEqual => 1.0,
                    Operator::GreaterThan
                    | Operator::LessThan
                    | Operator::GreaterThanOrEqual
                    | Operator::LessThanOrEqual => 2.0,
                    Operator::Contains | Operator::StartsWith | Operator::EndsWith => 5.0,
                };

                let value_cost = match value {
                    FactValue::Boolean(_) | FactValue::Integer(_) => 1.0,
                    FactValue::Float(_) => 1.5,
                    FactValue::String(s) => 1.0 + (s.len() as f64 * 0.1),
                    FactValue::Array(_) => 3.0,
                    _ => 2.0,
                };

                base_cost * value_cost
            }
            Condition::And { conditions } => {
                conditions.iter().map(Self::calculate_condition_cost).sum()
            }
            Condition::Or { conditions } => {
                // OR requires evaluating conditions until one matches
                conditions.iter().map(Self::calculate_condition_cost).sum::<f64>() * 0.5
            }
            _ => 10.0, // Complex conditions are expensive
        }
    }

    /// Apply selectivity-based condition reordering
    fn apply_selectivity_reordering(
        &self,
        rule: &mut Rule,
        analysis: &OptimizationAnalysis,
    ) -> Option<(OptimizationStrategy, f64)> {
        if rule.conditions.len() <= 1 {
            return None;
        }

        // Find the most selective condition that's not already first
        let mut best_improvement = 0.0;
        let mut best_move: Option<(usize, usize)> = None;

        for (i, &_selectivity) in analysis.condition_selectivity.iter().enumerate() {
            if i == 0 {
                continue;
            } // Already first

            // Calculate improvement by moving this condition to the front
            let current_expected_cost = self.calculate_expected_evaluation_cost(
                &analysis.condition_selectivity,
                &analysis.condition_costs,
            );

            // Simulate moving condition i to position 0
            let mut reordered_selectivity = analysis.condition_selectivity.clone();
            let mut reordered_costs = analysis.condition_costs.clone();

            // Move condition to front
            let moved_sel = reordered_selectivity.remove(i);
            let moved_cost = reordered_costs.remove(i);
            reordered_selectivity.insert(0, moved_sel);
            reordered_costs.insert(0, moved_cost);

            let new_expected_cost =
                self.calculate_expected_evaluation_cost(&reordered_selectivity, &reordered_costs);

            let improvement =
                ((current_expected_cost - new_expected_cost) / current_expected_cost) * 100.0;

            if improvement > best_improvement
                && improvement > self.config.min_selectivity_difference * 100.0
            {
                best_improvement = improvement;
                best_move = Some((i, 0));
            }
        }

        if let Some((from_index, to_index)) = best_move {
            // Apply the reordering
            let moved_condition = rule.conditions.remove(from_index);
            rule.conditions.insert(to_index, moved_condition);

            debug!(
                "Applied selectivity reordering: moved condition {} to position {}, {:.1}% improvement",
                from_index, to_index, best_improvement
            );

            Some((
                OptimizationStrategy::SelectivityReordering { from_index, to_index },
                best_improvement,
            ))
        } else {
            None
        }
    }

    /// Apply cost-based optimization to reorder expensive conditions
    fn apply_cost_based_optimization(
        &self,
        rule: &mut Rule,
        analysis: &OptimizationAnalysis,
    ) -> Option<(OptimizationStrategy, f64)> {
        if rule.conditions.len() <= 1 {
            return None;
        }

        // Identify high-cost conditions
        let avg_cost =
            analysis.condition_costs.iter().sum::<f64>() / analysis.condition_costs.len() as f64;
        let high_cost_indices: Vec<usize> = analysis
            .condition_costs
            .iter()
            .enumerate()
            .filter(|&(_, &cost)| cost > avg_cost * 2.0) // Conditions that are 2x more expensive than average
            .map(|(i, _)| i)
            .collect();

        if high_cost_indices.is_empty() {
            return None;
        }

        // Move high-cost conditions towards the end, but maintain selectivity ordering
        let mut reordered_indices: Vec<usize> = (0..rule.conditions.len()).collect();

        // Sort by selectivity first, then by cost (ascending selectivity, descending cost for ties)
        reordered_indices.sort_by(|&a, &b| {
            let sel_cmp = analysis.condition_selectivity[a]
                .partial_cmp(&analysis.condition_selectivity[b])
                .unwrap();
            if sel_cmp == std::cmp::Ordering::Equal {
                analysis.condition_costs[b].partial_cmp(&analysis.condition_costs[a]).unwrap()
            } else {
                sel_cmp
            }
        });

        // Check if reordering would improve performance
        let current_cost = self.calculate_expected_evaluation_cost(
            &analysis.condition_selectivity,
            &analysis.condition_costs,
        );

        let mut reordered_selectivity = Vec::new();
        let mut reordered_costs = Vec::new();
        let mut reordered_conditions = Vec::new();

        for &i in &reordered_indices {
            reordered_selectivity.push(analysis.condition_selectivity[i]);
            reordered_costs.push(analysis.condition_costs[i]);
            reordered_conditions.push(rule.conditions[i].clone());
        }

        let new_cost =
            self.calculate_expected_evaluation_cost(&reordered_selectivity, &reordered_costs);
        let improvement = ((current_cost - new_cost) / current_cost) * 100.0;

        if improvement > self.config.min_selectivity_difference * 100.0 {
            rule.conditions = reordered_conditions;

            debug!(
                "Applied cost-based reordering: {:.1}% improvement, moved {} high-cost conditions",
                improvement,
                high_cost_indices.len()
            );

            Some((
                OptimizationStrategy::CostBasedReordering { high_cost_indices },
                improvement,
            ))
        } else {
            None
        }
    }

    /// Calculate expected evaluation cost considering selectivity and early termination
    fn calculate_expected_evaluation_cost(&self, selectivity: &[f64], costs: &[f64]) -> f64 {
        let mut total_cost = 0.0;
        let mut cumulative_selectivity = 1.0;

        for (&sel, &cost) in selectivity.iter().zip(costs.iter()) {
            // Cost of evaluating this condition
            total_cost += cost * cumulative_selectivity;

            // Update cumulative selectivity for early termination
            cumulative_selectivity *= sel;

            // If selectivity becomes very low, remaining conditions rarely execute
            if cumulative_selectivity < 0.01 {
                break;
            }
        }

        total_cost
    }

    /// Analyze join patterns for multi-condition rules
    fn analyze_join_patterns(&self, conditions: &[Condition]) -> JoinAnalysis {
        let mut intermediate_sizes = Vec::new();
        let mut join_selectivity = Vec::new();
        let mut condition_correlations = HashMap::new();

        // Estimate intermediate result sizes
        let mut current_size = 1000.0; // Assume 1000 facts on average
        for condition in conditions.iter() {
            let selectivity = self.calculate_condition_selectivity(condition);
            current_size *= selectivity;
            intermediate_sizes.push(current_size as usize);
            join_selectivity.push(selectivity);
        }

        // Analyze condition correlations (simplified)
        for i in 0..conditions.len() {
            for j in (i + 1)..conditions.len() {
                let correlation =
                    self.estimate_condition_correlation(&conditions[i], &conditions[j]);
                condition_correlations.insert((i, j), correlation);
            }
        }

        // Determine optimal join order (simplified - use selectivity ordering)
        let mut optimal_join_order: Vec<usize> = (0..conditions.len()).collect();
        optimal_join_order
            .sort_by(|&a, &b| join_selectivity[a].partial_cmp(&join_selectivity[b]).unwrap());

        JoinAnalysis {
            intermediate_sizes,
            join_selectivity,
            optimal_join_order,
            condition_correlations,
        }
    }

    /// Estimate correlation between two conditions
    fn estimate_condition_correlation(&self, cond1: &Condition, cond2: &Condition) -> f64 {
        match (cond1, cond2) {
            (Condition::Simple { field: f1, .. }, Condition::Simple { field: f2, .. }) => {
                if f1 == f2 {
                    0.8 // High correlation for same field
                } else if f1.contains("id") && f2.contains("id") {
                    0.3 // Moderate correlation for ID fields
                } else {
                    0.1 // Low correlation by default
                }
            }
            _ => 0.1, // Low correlation for complex conditions
        }
    }

    /// Analyze condition sharing opportunities across rules
    fn analyze_condition_sharing(&self, rule: &Rule) -> Vec<SharedPattern> {
        let mut shared_patterns = Vec::new();

        for condition in &rule.conditions {
            if let Condition::Simple { field, operator, value } = condition {
                let pattern_key = format!("{field}_{operator:?}_{value:?}");

                // Check if this pattern appears in other rules using accumulated stats
                let sharing_rules = vec![rule.id]; // Current rule always uses this pattern
                let frequency = if let Some(stats) = self.condition_stats.get(&pattern_key) {
                    // If we have stats for this pattern, it's been seen before
                    if stats.pattern_frequency > 1 {
                        // This pattern is shared across multiple rule optimizations
                        // Note: We don't have access to other rule IDs here, but frequency indicates sharing
                        stats.pattern_frequency
                    } else {
                        1 // Only used by current rule so far
                    }
                } else {
                    1 // First time seeing this pattern
                };
                let memory_savings_estimate = 64; // Bytes saved per shared pattern

                shared_patterns.push(SharedPattern {
                    pattern_key,
                    sharing_rules,
                    frequency,
                    memory_savings_estimate,
                });
            }
        }

        shared_patterns
    }

    /// Estimate total improvement from all optimizations
    fn estimate_total_improvement(
        &self,
        condition_selectivity: &[f64],
        condition_costs: &[f64],
        join_analysis: &Option<JoinAnalysis>,
    ) -> f64 {
        if condition_selectivity.is_empty() {
            return 0.0;
        }

        // Calculate baseline cost
        let baseline_cost =
            self.calculate_expected_evaluation_cost(condition_selectivity, condition_costs);

        // Calculate optimized cost (with perfect ordering)
        let mut optimized_selectivity = condition_selectivity.to_vec();
        let mut optimized_costs = condition_costs.to_vec();

        // Sort by selectivity for optimal ordering
        let mut indexed_conditions: Vec<(usize, f64, f64)> = optimized_selectivity
            .iter()
            .zip(optimized_costs.iter())
            .enumerate()
            .map(|(i, (&sel, &cost))| (i, sel, cost))
            .collect();

        indexed_conditions.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        optimized_selectivity = indexed_conditions.iter().map(|(_, sel, _)| *sel).collect();
        optimized_costs = indexed_conditions.iter().map(|(_, _, cost)| *cost).collect();

        let optimized_cost =
            self.calculate_expected_evaluation_cost(&optimized_selectivity, &optimized_costs);

        let improvement = if baseline_cost > 0.0 {
            ((baseline_cost - optimized_cost) / baseline_cost) * 100.0
        } else {
            0.0
        };

        // Add join optimization benefits if applicable
        let join_improvement = if let Some(join_analysis) = join_analysis {
            // Estimate 10-30% additional improvement from join optimization
            let join_complexity = join_analysis.intermediate_sizes.len() as f64;
            (join_complexity.log2() * 5.0).min(30.0)
        } else {
            0.0
        };

        improvement + join_improvement
    }

    /// Update condition statistics based on runtime observations
    pub fn update_condition_statistics(&mut self, pattern_key: String, stats: ConditionStats) {
        self.condition_stats.insert(pattern_key, stats);
    }

    /// Get current optimization metrics
    pub fn get_metrics(&self) -> &OptimizationMetrics {
        &self.optimization_metrics
    }

    /// Get current configuration
    pub fn get_config(&self) -> &OptimizerConfig {
        &self.config
    }

    /// Update optimizer configuration
    pub fn update_config(&mut self, config: OptimizerConfig) {
        self.config = config;
    }

    /// Reset optimization metrics
    pub fn reset_metrics(&mut self) {
        self.optimization_metrics = OptimizationMetrics::default();
    }
}

impl Default for RuleOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimize a batch of rules for maximum performance
pub fn optimize_rule_batch(
    rules: Vec<Rule>,
    config: Option<OptimizerConfig>,
) -> Vec<OptimizationResult> {
    let mut optimizer = if let Some(config) = config {
        RuleOptimizer::with_config(config)
    } else {
        RuleOptimizer::new()
    };

    let mut results = Vec::new();

    info!("Starting batch optimization of {} rules", rules.len());
    let start_time = std::time::Instant::now();

    for rule in rules {
        let result = optimizer.optimize_rule(rule);
        results.push(result);
    }

    let total_time = start_time.elapsed();
    let metrics = optimizer.get_metrics();

    info!(
        "Batch optimization completed: {} rules optimized in {:?}, avg improvement: {:.1}%",
        metrics.rules_optimized, total_time, metrics.avg_performance_improvement
    );

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Action, ActionType};

    #[test]
    fn test_rule_optimizer_creation() {
        let optimizer = RuleOptimizer::new();
        assert!(optimizer.condition_stats.is_empty());
    }

    #[test]
    fn test_selectivity_calculation() {
        let optimizer = RuleOptimizer::new();

        // Test equal condition with string
        let condition = Condition::Simple {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("active".to_string()),
        };

        let selectivity = optimizer.calculate_condition_selectivity(&condition);
        assert!(selectivity > 0.0 && selectivity <= 1.0);
    }

    #[test]
    fn test_cost_calculation() {
        let _optimizer = RuleOptimizer::new();

        // Test simple equal condition
        let simple_condition = Condition::Simple {
            field: "id".to_string(),
            operator: Operator::Equal,
            value: FactValue::Integer(123),
        };

        let simple_cost = RuleOptimizer::calculate_condition_cost(&simple_condition);

        // Test expensive string contains condition
        let expensive_condition = Condition::Simple {
            field: "description".to_string(),
            operator: Operator::Contains,
            value: FactValue::String("very long search string".to_string()),
        };

        let expensive_cost = RuleOptimizer::calculate_condition_cost(&expensive_condition);

        assert!(expensive_cost > simple_cost);
    }

    #[test]
    fn test_rule_optimization() {
        let mut optimizer = RuleOptimizer::new();

        // Create a rule with multiple conditions (expensive first, selective last)
        let rule = Rule {
            id: 1,
            name: "Test Rule".to_string(),
            conditions: vec![
                // Expensive, less selective condition first
                Condition::Simple {
                    field: "description".to_string(),
                    operator: Operator::Contains,
                    value: FactValue::String("search".to_string()),
                },
                // Cheap, very selective condition second
                Condition::Simple {
                    field: "id".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::Integer(12345),
                },
            ],
            actions: vec![Action {
                action_type: ActionType::Log { message: "Rule fired".to_string() },
            }],
        };

        let result = optimizer.optimize_rule(rule);

        // Should have some improvement
        assert!(result.estimated_improvement >= 0.0);
        assert!(!result.strategies_applied.is_empty() || result.estimated_improvement == 0.0);

        // Check that analysis was performed
        assert_eq!(result.analysis.condition_selectivity.len(), 2);
        assert_eq!(result.analysis.condition_costs.len(), 2);
    }

    #[test]
    fn test_batch_optimization() {
        let rules = vec![
            Rule {
                id: 1,
                name: "Rule 1".to_string(),
                conditions: vec![Condition::Simple {
                    field: "status".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("active".to_string()),
                }],
                actions: vec![],
            },
            Rule {
                id: 2,
                name: "Rule 2".to_string(),
                conditions: vec![Condition::Simple {
                    field: "amount".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Float(100.0),
                }],
                actions: vec![],
            },
        ];

        let results = optimize_rule_batch(rules, None);
        assert_eq!(results.len(), 2);
    }
}
