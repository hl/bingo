//! Calculator expression caching for performance optimization
//!
//! This module provides caching capabilities for calculator expressions to avoid
//! re-parsing and re-evaluating the same expressions multiple times.

use crate::cache::{CacheStats, LruCache};
use crate::calculator::{Calculator, CalculatorResult, EvaluationContext};
use crate::types::FactValue;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A cache key for calculator expressions including context hash
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionCacheKey {
    /// The expression string being evaluated
    expression: String,
    /// Hash of the evaluation context (fact fields that might affect the result)
    context_hash: u64,
}

impl ExpressionCacheKey {
    /// Create a new cache key from expression and context
    pub fn new(expression: &str, context: &EvaluationContext) -> Self {
        let context_hash = Self::hash_context(context);
        Self { expression: expression.to_string(), context_hash }
    }

    /// Hash the evaluation context for cache key generation
    fn hash_context(context: &EvaluationContext) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash current fact fields
        for (key, value) in &context.current_fact.data.fields {
            key.hash(&mut hasher);
            Self::hash_fact_value(value, &mut hasher);
        }

        // Hash globals (if any)
        for (key, value) in &context.globals {
            key.hash(&mut hasher);
            Self::hash_fact_value(value, &mut hasher);
        }

        hasher.finish()
    }

    /// Hash a FactValue for consistent context hashing
    fn hash_fact_value(value: &FactValue, hasher: &mut DefaultHasher) {
        match value {
            FactValue::String(s) => {
                0u8.hash(hasher);
                s.hash(hasher);
            }
            FactValue::Integer(i) => {
                1u8.hash(hasher);
                i.hash(hasher);
            }
            FactValue::Float(f) => {
                2u8.hash(hasher);
                f.to_bits().hash(hasher); // Use bits for consistent float hashing
            }
            FactValue::Boolean(b) => {
                3u8.hash(hasher);
                b.hash(hasher);
            }
            FactValue::Array(arr) => {
                4u8.hash(hasher);
                for item in arr {
                    Self::hash_fact_value(item, hasher);
                }
            }
            FactValue::Object(obj) => {
                5u8.hash(hasher);
                // Sort keys for consistent hashing
                let mut sorted_pairs: Vec<_> = obj.iter().collect();
                sorted_pairs.sort_by_key(|(k, _)| *k);
                for (key, value) in sorted_pairs {
                    key.hash(hasher);
                    Self::hash_fact_value(value, hasher);
                }
            }
            FactValue::Date(dt) => {
                6u8.hash(hasher);
                dt.timestamp_nanos_opt().unwrap_or(0).hash(hasher);
            }
            FactValue::Null => {
                7u8.hash(hasher);
            }
        }
    }
}

/// Cached calculator with expression compilation and result caching
#[derive(Debug)]
pub struct CachedCalculator {
    /// The underlying calculator engine
    calculator: Calculator,
    /// Cache for compiled expressions (avoid re-parsing)
    expression_cache: LruCache<String, String>, // Expression -> Compiled form (simplified for now)
    /// Cache for evaluation results
    result_cache: LruCache<ExpressionCacheKey, CalculatorResult>,
    /// Statistics for monitoring cache performance
    pub compilation_hits: usize,
    pub compilation_misses: usize,
    pub evaluation_hits: usize,
    pub evaluation_misses: usize,
    pub total_evaluations: usize,
}

impl CachedCalculator {
    /// Create a new cached calculator with specified cache capacities
    pub fn new(expression_cache_size: usize, result_cache_size: usize) -> Self {
        Self {
            calculator: Calculator::new(),
            expression_cache: LruCache::new(expression_cache_size),
            result_cache: LruCache::new(result_cache_size),
            compilation_hits: 0,
            compilation_misses: 0,
            evaluation_hits: 0,
            evaluation_misses: 0,
            total_evaluations: 0,
        }
    }

    /// Create with default cache sizes optimized for typical usage
    pub fn with_default_caches() -> Self {
        Self::new(1000, 5000) // 1K expressions, 5K results
    }

    /// Evaluate an expression with caching
    pub fn eval_cached(
        &mut self,
        expression: &str,
        context: &EvaluationContext,
    ) -> anyhow::Result<CalculatorResult> {
        self.total_evaluations += 1;

        // Check result cache first
        let cache_key = ExpressionCacheKey::new(expression, context);
        if let Some(cached_result) = self.result_cache.get(&cache_key) {
            self.evaluation_hits += 1;
            return Ok(cached_result.clone());
        }

        self.evaluation_misses += 1;

        // Check if expression is pre-compiled
        let compiled_expression =
            if let Some(cached_expr) = self.expression_cache.get(&expression.to_string()) {
                self.compilation_hits += 1;
                cached_expr.clone()
            } else {
                self.compilation_misses += 1;
                // For now, we'll use the expression as-is since the calculator doesn't expose compilation
                // In a real implementation, you'd cache the parsed AST or compiled bytecode
                let compiled = self.precompile_expression(expression)?;
                self.expression_cache.put(expression.to_string(), compiled.clone());
                compiled
            };

        // Evaluate the expression
        let result = self.calculator.eval(&compiled_expression, context)?;

        // Cache the result
        self.result_cache.put(cache_key, result.clone());

        Ok(result)
    }

    /// Pre-compile an expression (simplified - in reality would parse to AST)
    fn precompile_expression(&mut self, expression: &str) -> anyhow::Result<String> {
        // For now, just compile the expression without evaluation
        // This validates syntax without needing actual context values
        let _compiled = self.calculator.compile(expression)?;
        Ok(expression.to_string())
    }

    /// Clear all caches
    pub fn clear_caches(&mut self) {
        self.expression_cache.clear();
        self.result_cache.clear();
        self.compilation_hits = 0;
        self.compilation_misses = 0;
        self.evaluation_hits = 0;
        self.evaluation_misses = 0;
        self.total_evaluations = 0;
    }

    /// Get cache performance statistics
    pub fn cache_stats(&self) -> CalculatorCacheStats {
        CalculatorCacheStats {
            expression_cache_stats: self.expression_cache.stats(),
            result_cache_stats: self.result_cache.stats(),
            compilation_hit_rate: self.hit_rate(self.compilation_hits, self.compilation_misses),
            evaluation_hit_rate: self.hit_rate(self.evaluation_hits, self.evaluation_misses),
            total_evaluations: self.total_evaluations,
            compilation_hits: self.compilation_hits,
            compilation_misses: self.compilation_misses,
            evaluation_hits: self.evaluation_hits,
            evaluation_misses: self.evaluation_misses,
        }
    }

    /// Calculate hit rate percentage
    fn hit_rate(&self, hits: usize, misses: usize) -> f64 {
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            (hits as f64 / total as f64) * 100.0
        }
    }

    /// Get cache memory utilization
    pub fn cache_utilization(&self) -> CacheUtilization {
        CacheUtilization {
            expression_cache_utilization: self.expression_cache.stats().utilization(),
            result_cache_utilization: self.result_cache.stats().utilization(),
            total_expressions_cached: self.expression_cache.len(),
            total_results_cached: self.result_cache.len(),
        }
    }
}

/// Cache performance statistics
#[derive(Debug, Clone)]
pub struct CalculatorCacheStats {
    pub expression_cache_stats: CacheStats,
    pub result_cache_stats: CacheStats,
    pub compilation_hit_rate: f64,
    pub evaluation_hit_rate: f64,
    pub total_evaluations: usize,
    pub compilation_hits: usize,
    pub compilation_misses: usize,
    pub evaluation_hits: usize,
    pub evaluation_misses: usize,
}

impl CalculatorCacheStats {
    /// Get overall cache efficiency (weighted average of compilation and evaluation hit rates)
    pub fn overall_efficiency(&self) -> f64 {
        // Weight evaluation hits more heavily since they're more expensive to compute
        let compilation_weight = 0.3;
        let evaluation_weight = 0.7;

        (self.compilation_hit_rate * compilation_weight)
            + (self.evaluation_hit_rate * evaluation_weight)
    }

    /// Get total cache operations
    pub fn total_operations(&self) -> usize {
        self.compilation_hits
            + self.compilation_misses
            + self.evaluation_hits
            + self.evaluation_misses
    }
}

/// Cache memory utilization information
#[derive(Debug, Clone)]
pub struct CacheUtilization {
    pub expression_cache_utilization: f64,
    pub result_cache_utilization: f64,
    pub total_expressions_cached: usize,
    pub total_results_cached: usize,
}

impl CacheUtilization {
    /// Get average cache utilization across both caches
    pub fn average_utilization(&self) -> f64 {
        (self.expression_cache_utilization + self.result_cache_utilization) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Fact, FactData, FactValue};
    use std::collections::HashMap;

    fn create_test_context() -> (Fact, EvaluationContext<'static>) {
        use std::collections::HashMap;
        use std::sync::LazyLock;

        static TEST_FACT: LazyLock<Fact> = LazyLock::new(|| {
            let mut fields = HashMap::new();
            fields.insert("amount".to_string(), FactValue::Float(100.0));
            fields.insert("tax_rate".to_string(), FactValue::Float(0.15));
            fields.insert("customer_id".to_string(), FactValue::Integer(12345));

            Fact { id: 1, data: FactData { fields } }
        });

        static EMPTY_FACTS: &[Fact] = &[];

        let context = EvaluationContext {
            current_fact: &TEST_FACT,
            facts: EMPTY_FACTS,
            globals: HashMap::new(),
        };

        (TEST_FACT.clone(), context)
    }

    #[test]
    fn test_cached_calculator_basic_operations() {
        let mut calc = CachedCalculator::new(100, 200);
        let (_fact, context) = create_test_context();

        // First evaluation should be a cache miss
        let result1 = calc.eval_cached("amount * tax_rate", &context).unwrap();
        let stats1 = calc.cache_stats();

        assert_eq!(stats1.evaluation_misses, 1);
        assert_eq!(stats1.evaluation_hits, 0);
        assert_eq!(stats1.compilation_misses, 1);

        // Second evaluation of same expression should be a cache hit
        let result2 = calc.eval_cached("amount * tax_rate", &context).unwrap();
        let stats2 = calc.cache_stats();

        assert_eq!(result1, result2);
        assert_eq!(stats2.evaluation_hits, 1);
        assert_eq!(stats2.compilation_hits, 0); // No compilation hit because result cache hit first

        // Test compilation cache by using same expression with different context
        let mut different_fields = std::collections::HashMap::new();
        different_fields.insert("amount".to_string(), FactValue::Float(200.0)); // Different value
        different_fields.insert("tax_rate".to_string(), FactValue::Float(0.15));

        use std::sync::LazyLock;
        static DIFFERENT_FACT: LazyLock<Fact> = LazyLock::new(|| Fact {
            id: 2,
            data: FactData {
                fields: {
                    let mut fields = std::collections::HashMap::new();
                    fields.insert("amount".to_string(), FactValue::Float(200.0));
                    fields.insert("tax_rate".to_string(), FactValue::Float(0.15));
                    fields
                },
            },
        });

        let different_context = EvaluationContext {
            current_fact: &DIFFERENT_FACT,
            facts: &[],
            globals: std::collections::HashMap::new(),
        };

        let _result3 = calc.eval_cached("amount * tax_rate", &different_context).unwrap();
        let stats3 = calc.cache_stats();

        assert_eq!(stats3.compilation_hits, 1); // Now we should see compilation cache hit
    }

    #[test]
    fn test_cache_key_context_sensitivity() {
        let mut calc = CachedCalculator::new(100, 200);

        // Create two different contexts
        let mut fields1 = HashMap::new();
        fields1.insert("amount".to_string(), FactValue::Float(100.0));
        let fact1 = Fact { id: 1, data: FactData { fields: fields1 } };
        let context1 =
            EvaluationContext { current_fact: &fact1, facts: &[], globals: HashMap::new() };

        let mut fields2 = HashMap::new();
        fields2.insert("amount".to_string(), FactValue::Float(200.0));
        let fact2 = Fact { id: 2, data: FactData { fields: fields2 } };
        let context2 =
            EvaluationContext { current_fact: &fact2, facts: &[], globals: HashMap::new() };

        // Same expression with different contexts should produce different cache keys
        let key1 = ExpressionCacheKey::new("amount * 2", &context1);
        let key2 = ExpressionCacheKey::new("amount * 2", &context2);

        assert_ne!(
            key1, key2,
            "Cache keys should differ for different contexts"
        );

        // Evaluate with both contexts
        let result1 = calc.eval_cached("amount * 2", &context1).unwrap();
        let result2 = calc.eval_cached("amount * 2", &context2).unwrap();

        // Results should be different and both should be cache misses initially
        assert_ne!(result1, result2);

        let stats = calc.cache_stats();
        assert_eq!(
            stats.evaluation_misses, 2,
            "Both evaluations should be cache misses"
        );
    }

    #[test]
    fn test_cache_statistics_accuracy() {
        let mut calc = CachedCalculator::new(50, 100);
        let (_fact, context) = create_test_context();

        // Evaluate multiple expressions
        let expressions = [
            "amount + 10",
            "tax_rate * 100",
            "amount * tax_rate",
            "amount + 10", // Repeat first expression
            "customer_id / 1000",
        ];

        for expr in &expressions {
            calc.eval_cached(expr, &context).unwrap();
        }

        let stats = calc.cache_stats();

        assert_eq!(stats.total_evaluations, 5);
        assert_eq!(stats.evaluation_hits, 1); // One repeat
        assert_eq!(stats.evaluation_misses, 4); // Four unique
        assert_eq!(stats.compilation_hits, 0); // No compilation hits because result cache hit first
        assert_eq!(stats.compilation_misses, 4); // Four unique expressions

        // Test hit rates
        assert_eq!(stats.evaluation_hit_rate, 20.0); // 1/5 = 20%
        assert_eq!(stats.compilation_hit_rate, 0.0); // 0/4 = 0% (no compilation cache hits)

        // Test overall efficiency
        let efficiency = stats.overall_efficiency();
        assert!((0.0..=100.0).contains(&efficiency));

        // Test utilization
        let utilization = calc.cache_utilization();
        assert!(utilization.average_utilization() >= 0.0);
        assert_eq!(utilization.total_expressions_cached, 4);
        assert_eq!(utilization.total_results_cached, 4);
    }

    #[test]
    fn test_cache_memory_management() {
        // Create a small cache to test eviction
        let mut calc = CachedCalculator::new(2, 3);
        let (_fact, context) = create_test_context();

        // Fill beyond cache capacity
        let expressions = [
            "amount * 1",
            "amount * 2",
            "amount * 3",
            "amount * 4", // Should evict oldest
        ];

        for expr in &expressions {
            calc.eval_cached(expr, &context).unwrap();
        }

        let utilization = calc.cache_utilization();

        // Should not exceed cache capacity
        assert!(utilization.total_expressions_cached <= 2);
        assert!(utilization.total_results_cached <= 3);

        // Utilization should be high since we're at capacity
        assert!(utilization.expression_cache_utilization >= 90.0);
    }

    #[test]
    fn test_cache_clearing() {
        let mut calc = CachedCalculator::new(100, 200);
        let (_fact, context) = create_test_context();

        // Populate caches
        calc.eval_cached("amount * tax_rate", &context).unwrap();
        calc.eval_cached("customer_id + 1", &context).unwrap();

        let stats_before = calc.cache_stats();
        assert!(stats_before.total_evaluations > 0);

        // Clear caches
        calc.clear_caches();

        let stats_after = calc.cache_stats();
        assert_eq!(stats_after.total_evaluations, 0);
        assert_eq!(stats_after.evaluation_hits, 0);
        assert_eq!(stats_after.evaluation_misses, 0);
        assert_eq!(stats_after.compilation_hits, 0);
        assert_eq!(stats_after.compilation_misses, 0);

        let utilization = calc.cache_utilization();
        assert_eq!(utilization.total_expressions_cached, 0);
        assert_eq!(utilization.total_results_cached, 0);
    }
}
