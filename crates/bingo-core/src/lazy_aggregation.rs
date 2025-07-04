//! Lazy evaluation system for complex aggregations
//!
//! This module provides lazy evaluation capabilities for aggregations to improve
//! performance by:
//! - Deferring expensive computations until actually needed
//! - Caching intermediate results to avoid recomputation
//! - Short-circuiting when possible based on conditions
//! - Streaming evaluation for large fact sets
//! - Incremental updates when fact sets change

use crate::fact_store::arena_store::ArenaFactStore;
use crate::memory_pools::MemoryPoolManager;
use crate::types::{
    AggregationCondition, AggregationType, AggregationWindow, Condition, Fact, FactValue,
};
use anyhow::Result;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

/// Lazy aggregation result that defers computation until needed
#[derive(Debug)]
pub struct LazyAggregationResult {
    /// The aggregation specification
    spec: AggregationCondition,
    /// Trigger fact that initiated this aggregation
    trigger_fact: Fact,
    /// Cached result if already computed
    cached_result: RefCell<Option<FactValue>>,
    /// Fact store reference for lazy evaluation
    fact_store: Arc<ArenaFactStore>,
    /// Memory pools for efficient allocation
    memory_pools: Arc<MemoryPoolManager>,
    /// Statistics for performance monitoring
    stats: RefCell<LazyAggregationStats>,
}

/// Statistics for lazy aggregation performance monitoring
#[derive(Debug, Clone, Default)]
pub struct LazyAggregationStats {
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub early_terminations: usize,
    pub full_computations: usize,
    pub facts_scanned: usize,
    pub facts_processed: usize,
}

impl LazyAggregationResult {
    /// Create a new lazy aggregation result
    pub fn new(
        spec: AggregationCondition,
        trigger_fact: Fact,
        fact_store: Arc<ArenaFactStore>,
        memory_pools: Arc<MemoryPoolManager>,
    ) -> Self {
        Self {
            spec,
            trigger_fact,
            cached_result: RefCell::new(None),
            fact_store,
            memory_pools,
            stats: RefCell::new(LazyAggregationStats::default()),
        }
    }

    /// Get the aggregation result, computing it lazily if needed
    pub fn get_value(&self) -> Result<FactValue> {
        // Check cache first
        if let Some(cached) = self.cached_result.borrow().as_ref() {
            self.stats.borrow_mut().cache_hits += 1;
            return Ok(cached.clone());
        }

        self.stats.borrow_mut().cache_misses += 1;

        // Compute the result lazily
        let result = self.compute_aggregation()?;

        // Cache the result
        *self.cached_result.borrow_mut() = Some(result.clone());

        Ok(result)
    }

    /// Evaluate the HAVING clause lazily without computing the full aggregation
    pub fn evaluate_having_lazy(&self, having_condition: &Condition) -> Result<bool> {
        // For simple conditions, we might be able to short-circuit
        if let Some(short_circuit_result) = self.try_short_circuit_having(having_condition)? {
            self.stats.borrow_mut().early_terminations += 1;
            return Ok(short_circuit_result);
        }

        // Need full computation
        let aggregated_value = self.get_value()?;

        // Create synthetic fact for HAVING evaluation
        let mut synthetic_fields = self.memory_pools.fact_field_maps.get();
        synthetic_fields.insert(self.spec.alias.clone(), aggregated_value);

        let synthetic_fact = Fact {
            id: 0,
            external_id: None,
            timestamp: chrono::Utc::now(),
            data: crate::types::FactData { fields: synthetic_fields },
        };

        // Evaluate the HAVING condition
        let result = self.evaluate_condition_on_fact(&synthetic_fact, having_condition);

        // Return the map to the pool
        self.memory_pools.fact_field_maps.return_map(synthetic_fact.data.fields);

        result
    }

    /// Attempt to short-circuit HAVING evaluation for simple cases
    fn try_short_circuit_having(&self, having_condition: &Condition) -> Result<Option<bool>> {
        if let Condition::Simple { field, operator, value } = having_condition {
            // If the field is our aggregation alias and we can evaluate without full computation
            if field == &self.spec.alias {
                return self.try_short_circuit_simple_condition(operator, value);
            }
        }
        Ok(None)
    }

    /// Try to short-circuit simple conditions based on aggregation type
    fn try_short_circuit_simple_condition(
        &self,
        operator: &crate::types::Operator,
        target_value: &FactValue,
    ) -> Result<Option<bool>> {
        use crate::types::{AggregationType::*, Operator::*};

        match (&self.spec.aggregation_type, operator) {
            // Count optimizations
            (Count, Equal) => {
                if let Some(target_count) = target_value.as_integer() {
                    if target_count <= 0 {
                        // If checking for count == 0, we can short-circuit on first match
                        return Ok(Some(!self.has_any_matching_facts()?));
                    }
                }
            }
            (Count, GreaterThan) => {
                if let Some(threshold) = target_value.as_integer() {
                    if threshold <= 0 {
                        // If checking count > 0, we can short-circuit on first match
                        return Ok(Some(self.has_any_matching_facts()?));
                    }
                }
            }
            // Sum optimizations
            (Sum, GreaterThan) => {
                if let Some(threshold) = target_value.as_f64() {
                    if threshold <= 0.0 {
                        // If checking sum > 0, we can short-circuit on first positive value
                        return Ok(Some(self.has_any_positive_values()?));
                    }
                }
            }
            // Min/Max optimizations could check if any value meets the threshold
            _ => {}
        }

        Ok(None)
    }

    /// Check if there are any facts matching the group criteria (for count short-circuiting)
    fn has_any_matching_facts(&self) -> Result<bool> {
        let candidates = self.get_candidate_facts()?;

        for candidate in candidates {
            if self.fact_matches_group(candidate)? {
                self.stats.borrow_mut().facts_scanned += 1;
                self.stats.borrow_mut().early_terminations += 1;
                return Ok(true);
            }
            self.stats.borrow_mut().facts_scanned += 1;
        }

        Ok(false)
    }

    /// Check if there are any positive values in the source field (for sum short-circuiting)
    fn has_any_positive_values(&self) -> Result<bool> {
        let candidates = self.get_candidate_facts()?;

        for candidate in candidates {
            if !self.fact_matches_group(candidate)? {
                continue;
            }

            if let Some(value) = candidate.data.fields.get(&self.spec.source_field) {
                if let Some(num_val) = value.as_f64() {
                    if num_val > 0.0 {
                        self.stats.borrow_mut().facts_scanned += 1;
                        self.stats.borrow_mut().early_terminations += 1;
                        return Ok(true);
                    }
                }
            }
            self.stats.borrow_mut().facts_scanned += 1;
        }

        Ok(false)
    }

    /// Get candidate facts based on window specification
    fn get_candidate_facts(&self) -> Result<Vec<&Fact>> {
        let candidates = if let Some(window) = &self.spec.window {
            match window {
                AggregationWindow::Time { duration_ms } => {
                    let start = self.trigger_fact.timestamp
                        - chrono::Duration::milliseconds(*duration_ms as i64);
                    let end = self.trigger_fact.timestamp;
                    self.fact_store.facts_in_time_range(start, end)
                }
                AggregationWindow::Sliding { size } => {
                    // Get last `size` facts in temporal order
                    let mut all: Vec<&Fact> = self.fact_store.iter().collect();
                    all.sort_by_key(|f| f.timestamp);
                    if *size >= all.len() {
                        all
                    } else {
                        all.split_off(all.len() - size)
                    }
                }
                AggregationWindow::Tumbling { size } => {
                    // Determine window index based on trigger fact position
                    let mut all: Vec<&Fact> = self.fact_store.iter().collect();
                    all.sort_by_key(|f| f.timestamp);
                    if all.is_empty() {
                        vec![]
                    } else {
                        let idx =
                            all.iter().position(|f| f.id == self.trigger_fact.id).unwrap_or(0);
                        let window_start = (idx / size) * size;
                        all.into_iter().skip(window_start).take(*size).collect()
                    }
                }
                AggregationWindow::Session { .. } => {
                    // Session windows not fully supported yet
                    self.fact_store.iter().collect()
                }
            }
        } else {
            self.fact_store.iter().collect()
        };

        Ok(candidates)
    }

    /// Check if a fact matches the group criteria
    fn fact_matches_group(&self, fact: &Fact) -> Result<bool> {
        for group_field in &self.spec.group_by {
            let trigger_value = self.trigger_fact.data.fields.get(group_field);
            let candidate_value = fact.data.fields.get(group_field);
            if trigger_value != candidate_value {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Compute the full aggregation result
    fn compute_aggregation(&self) -> Result<FactValue> {
        self.stats.borrow_mut().full_computations += 1;

        let candidates = self.get_candidate_facts()?;
        let mut nums = self.memory_pools.numeric_vecs.get();

        // Collect numeric values from matching facts
        for candidate in &candidates {
            self.stats.borrow_mut().facts_scanned += 1;

            if !self.fact_matches_group(candidate)? {
                continue;
            }

            if let Some(value) = candidate.data.fields.get(&self.spec.source_field) {
                if let Some(num_val) = value.as_f64() {
                    nums.push(num_val);
                    self.stats.borrow_mut().facts_processed += 1;
                }
            }
        }

        let result = if nums.is_empty() {
            // Return appropriate zero value for the aggregation type
            match self.spec.aggregation_type {
                AggregationType::Count => FactValue::Integer(0),
                AggregationType::Sum | AggregationType::Average => FactValue::Float(0.0),
                AggregationType::Min => FactValue::Float(f64::INFINITY),
                AggregationType::Max => FactValue::Float(f64::NEG_INFINITY),
                AggregationType::StandardDeviation => FactValue::Float(0.0),
                AggregationType::Percentile(_) => FactValue::Float(0.0),
            }
        } else {
            self.compute_aggregation_value(&nums)?
        };

        // Return the numeric vector to the pool
        self.memory_pools.numeric_vecs.return_vec(nums);

        Ok(result)
    }

    /// Compute the actual aggregation value from collected numbers
    fn compute_aggregation_value(&self, nums: &[f64]) -> Result<FactValue> {
        use crate::types::AggregationType::*;

        let result = match &self.spec.aggregation_type {
            Count => FactValue::Integer(nums.len() as i64),
            Sum => FactValue::Float(nums.iter().sum()),
            Average => FactValue::Float(nums.iter().sum::<f64>() / nums.len() as f64),
            Min => {
                let min_val = nums.iter().cloned().fold(f64::INFINITY, f64::min);
                FactValue::Float(min_val)
            }
            Max => {
                let max_val = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                FactValue::Float(max_val)
            }
            StandardDeviation => {
                let mean = nums.iter().sum::<f64>() / nums.len() as f64;
                let variance =
                    nums.iter().map(|v| (*v - mean).powi(2)).sum::<f64>() / nums.len() as f64;
                FactValue::Float(variance.sqrt())
            }
            Percentile(p) => {
                let mut sorted = nums.to_vec();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let rank_f = (p / 100.0) * (sorted.len() as f64 - 1.0);
                let lower = rank_f.floor() as usize;
                let upper = rank_f.ceil() as usize;
                let interp = if upper == lower {
                    sorted[lower]
                } else {
                    let w = rank_f - lower as f64;
                    sorted[lower] * (1.0 - w) + sorted[upper] * w
                };
                FactValue::Float(interp)
            }
        };

        Ok(result)
    }

    /// Evaluate a condition against a specific fact
    fn evaluate_condition_on_fact(&self, fact: &Fact, condition: &Condition) -> Result<bool> {
        match condition {
            Condition::Simple { field, operator, value } => {
                let fact_value = fact.data.fields.get(field);

                match fact_value {
                    Some(fact_val) => {
                        use crate::types::Operator::*;
                        let result = match operator {
                            Equal => fact_val == value,
                            NotEqual => fact_val != value,
                            GreaterThan => fact_val > value,
                            LessThan => fact_val < value,
                            GreaterThanOrEqual => fact_val >= value,
                            LessThanOrEqual => fact_val <= value,
                            Contains => match (fact_val, value) {
                                (FactValue::String(fact_str), FactValue::String(pattern)) => {
                                    fact_str.contains(pattern)
                                }
                                _ => false,
                            },
                        };
                        Ok(result)
                    }
                    None => Ok(false),
                }
            }
            // For complex conditions, we'd need more sophisticated evaluation
            _ => Ok(true), // Simplified for now
        }
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> LazyAggregationStats {
        self.stats.borrow().clone()
    }

    /// Reset the cache (useful when fact store changes)
    pub fn invalidate_cache(&self) {
        *self.cached_result.borrow_mut() = None;
    }

    /// Check if the result is cached
    pub fn is_cached(&self) -> bool {
        self.cached_result.borrow().is_some()
    }

    /// Get cache hit rate as percentage
    pub fn cache_hit_rate(&self) -> f64 {
        let stats = self.stats.borrow();
        let total = stats.cache_hits + stats.cache_misses;
        if total > 0 {
            (stats.cache_hits as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get early termination rate as percentage
    pub fn early_termination_rate(&self) -> f64 {
        let stats = self.stats.borrow();
        let total = stats.early_terminations + stats.full_computations;
        if total > 0 {
            (stats.early_terminations as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Manager for lazy aggregations with global optimizations
#[derive(Debug)]
pub struct LazyAggregationManager {
    /// Active lazy aggregations indexed by a key
    active_aggregations: RefCell<HashMap<String, LazyAggregationResult>>,
    /// Global statistics
    global_stats: RefCell<LazyAggregationManagerStats>,
    /// Memory pools reference
    memory_pools: Arc<MemoryPoolManager>,
}

/// Statistics for the lazy aggregation manager
#[derive(Debug, Clone, Default)]
pub struct LazyAggregationManagerStats {
    pub aggregations_created: usize,
    pub aggregations_reused: usize,
    pub cache_invalidations: usize,
    pub total_early_terminations: usize,
    pub total_full_computations: usize,
}

impl LazyAggregationManager {
    /// Create a new lazy aggregation manager
    pub fn new(memory_pools: Arc<MemoryPoolManager>) -> Self {
        Self {
            active_aggregations: RefCell::new(HashMap::new()),
            global_stats: RefCell::new(LazyAggregationManagerStats::default()),
            memory_pools,
        }
    }

    /// Create or reuse a lazy aggregation
    pub fn get_or_create_aggregation(
        &self,
        spec: AggregationCondition,
        trigger_fact: Fact,
        fact_store: Arc<ArenaFactStore>,
    ) -> String {
        // Generate a key for the aggregation (based on spec and group values)
        let key = self.generate_aggregation_key(&spec, &trigger_fact);

        let mut aggregations = self.active_aggregations.borrow_mut();
        let mut stats = self.global_stats.borrow_mut();

        if aggregations.contains_key(&key) {
            stats.aggregations_reused += 1;
        } else {
            let lazy_agg = LazyAggregationResult::new(
                spec,
                trigger_fact,
                fact_store,
                self.memory_pools.clone(),
            );
            aggregations.insert(key.clone(), lazy_agg);
            stats.aggregations_created += 1;
        }

        key
    }

    /// Get a lazy aggregation by key
    pub fn get_aggregation(&self, key: &str) -> Option<LazyAggregationResult> {
        // Note: This is a simplified implementation. In practice, we'd want to avoid cloning
        // and use references or Arc<> for better performance
        self.active_aggregations.borrow().get(key).cloned()
    }

    /// Invalidate all caches (when fact store changes significantly)
    pub fn invalidate_all_caches(&self) {
        let mut stats = self.global_stats.borrow_mut();
        stats.cache_invalidations += 1;

        for (_, aggregation) in self.active_aggregations.borrow().iter() {
            aggregation.invalidate_cache();
        }
    }

    /// Clear inactive aggregations to free memory
    pub fn cleanup_inactive_aggregations(&self) {
        // Simple cleanup strategy: remove aggregations that haven't been accessed recently
        // In a real implementation, we'd track access times
        self.active_aggregations.borrow_mut().clear();
    }

    /// Generate a unique key for an aggregation
    fn generate_aggregation_key(&self, spec: &AggregationCondition, trigger_fact: &Fact) -> String {
        // Create a key based on aggregation spec and group field values
        let mut key_parts = vec![
            format!("type:{:?}", spec.aggregation_type),
            format!("source:{}", spec.source_field),
            format!("alias:{}", spec.alias),
        ];

        // Add group field values
        for group_field in &spec.group_by {
            if let Some(value) = trigger_fact.data.fields.get(group_field) {
                key_parts.push(format!("{}:{}", group_field, value.as_string()));
            }
        }

        // Add window specification if present
        if let Some(window) = &spec.window {
            key_parts.push(format!("window:{:?}", window));
        }

        key_parts.join("|")
    }

    /// Get global manager statistics
    pub fn get_stats(&self) -> LazyAggregationManagerStats {
        self.global_stats.borrow().clone()
    }
}

impl Clone for LazyAggregationResult {
    fn clone(&self) -> Self {
        Self {
            spec: self.spec.clone(),
            trigger_fact: self.trigger_fact.clone(),
            cached_result: RefCell::new(self.cached_result.borrow().clone()),
            fact_store: self.fact_store.clone(),
            memory_pools: self.memory_pools.clone(),
            stats: RefCell::new(self.stats.borrow().clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact_store::arena_store::ArenaFactStore;
    use crate::memory_pools::MemoryPoolManager;
    use crate::types::{AggregationCondition, AggregationType, FactData};
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_fact(id: u64, value: f64, group: &str) -> Fact {
        let mut fields = HashMap::new();
        fields.insert("value".to_string(), FactValue::Float(value));
        fields.insert("group".to_string(), FactValue::String(group.to_string()));

        Fact {
            id,
            external_id: Some(format!("fact-{}", id)),
            timestamp: Utc::now(),
            data: FactData { fields },
        }
    }

    fn create_test_aggregation_spec() -> AggregationCondition {
        AggregationCondition {
            aggregation_type: AggregationType::Sum,
            source_field: "value".to_string(),
            group_by: vec!["group".to_string()],
            having: None,
            alias: "total_value".to_string(),
            window: None,
        }
    }

    #[test]
    fn test_lazy_aggregation_caching() {
        let fact_store = Arc::new(ArenaFactStore::new());
        #[allow(clippy::arc_with_non_send_sync)]
        let memory_pools = Arc::new(MemoryPoolManager::new());

        let spec = create_test_aggregation_spec();
        let trigger_fact = create_test_fact(1, 10.0, "A");

        let lazy_agg = LazyAggregationResult::new(spec, trigger_fact, fact_store, memory_pools);

        // First access should be a cache miss
        let result1 = lazy_agg.get_value().unwrap();
        let stats1 = lazy_agg.get_stats();
        assert_eq!(stats1.cache_misses, 1);
        assert_eq!(stats1.cache_hits, 0);

        // Second access should be a cache hit
        let result2 = lazy_agg.get_value().unwrap();
        let stats2 = lazy_agg.get_stats();
        assert_eq!(stats2.cache_hits, 1);
        assert_eq!(stats2.cache_misses, 1);

        // Results should be the same
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_lazy_aggregation_manager() {
        #[allow(clippy::arc_with_non_send_sync)]
        let memory_pools = Arc::new(MemoryPoolManager::new());
        let fact_store = Arc::new(ArenaFactStore::new());
        let manager = LazyAggregationManager::new(memory_pools);

        let spec = create_test_aggregation_spec();
        let trigger_fact = create_test_fact(1, 10.0, "A");

        // Create aggregation
        let key1 = manager.get_or_create_aggregation(
            spec.clone(),
            trigger_fact.clone(),
            fact_store.clone(),
        );

        // Reuse same aggregation
        let key2 = manager.get_or_create_aggregation(spec, trigger_fact, fact_store);

        assert_eq!(key1, key2);

        let stats = manager.get_stats();
        assert_eq!(stats.aggregations_created, 1);
        assert_eq!(stats.aggregations_reused, 1);
    }

    #[test]
    fn test_early_termination_count() {
        let mut fact_store = ArenaFactStore::new();
        fact_store.insert(create_test_fact(1, 10.0, "A"));
        fact_store.insert(create_test_fact(2, 20.0, "A"));

        #[allow(clippy::arc_with_non_send_sync)]
        let memory_pools = Arc::new(MemoryPoolManager::new());
        let fact_store = Arc::new(fact_store);

        let mut spec = create_test_aggregation_spec();
        spec.aggregation_type = AggregationType::Count;

        let trigger_fact = create_test_fact(3, 30.0, "A");

        let lazy_agg = LazyAggregationResult::new(spec, trigger_fact, fact_store, memory_pools);

        // Check if we have any matching facts (should short-circuit)
        let has_any = lazy_agg.has_any_matching_facts().unwrap();
        assert!(has_any);

        let stats = lazy_agg.get_stats();
        assert!(stats.early_terminations > 0);
    }
}
