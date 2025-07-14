//! Alpha Memory Implementation for RETE Network
//!
//! This module implements the alpha memory component of the RETE algorithm, which provides
//! efficient indexing and retrieval of facts that match specific patterns. The alpha memory
//! is the foundation of RETE's O(Δfacts) performance characteristic.
//!
//! ## Alpha Memory Architecture
//!
//! ```text
//! Facts → Pattern Matching → Alpha Memory Index → Beta Network
//!   ↓           ↓                    ↓               ↓
//!  WM      Single-field         Hash Index      Multi-condition
//!        Conditions           O(1) lookup       Joins
//! ```
//!
//! ## Key Components
//!
//! - **FactPattern**: Represents a single-field condition pattern
//! - **AlphaMemory**: Indexed storage of facts matching specific patterns  
//! - **AlphaMemoryManager**: Manages multiple alpha memories with efficient indexing
//! - **PatternIndex**: Hash-based index for O(1) pattern lookups

use crate::types::{Condition, Fact, FactId, FactValue, NodeId, Operator, RuleId};
use std::collections::{HashMap, HashSet};
use tracing::{debug, instrument};

/// Represents a fact pattern for alpha memory indexing
///
/// A FactPattern captures the essential information needed to index facts
/// based on field values and operators. This enables O(1) lookups during
/// fact processing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactPattern {
    /// Field name being tested (e.g., "age", "status", "amount")
    pub field: String,
    /// Comparison operator (Equal, GreaterThan, etc.)
    pub operator: Operator,
    /// Expected value for comparison
    pub value: FactValue,
}

impl FactPattern {
    /// Create a new fact pattern from a condition
    pub fn from_condition(condition: &Condition) -> Option<Self> {
        match condition {
            Condition::Simple { field, operator, value } => Some(Self {
                field: field.clone(),
                operator: operator.clone(),
                value: value.clone(),
            }),
            // Complex conditions need special handling and don't map to single patterns
            Condition::Complex { .. }
            | Condition::And { .. }
            | Condition::Or { .. }
            | Condition::Aggregation(_)
            | Condition::Stream(_) => None,
        }
    }

    /// Generate a unique key for this pattern for hash indexing
    pub fn to_key(&self) -> String {
        format!("{}_{:?}_{:?}", self.field, self.operator, self.value)
    }

    /// Check if a fact matches this pattern
    pub fn matches_fact(&self, fact: &Fact) -> bool {
        if let Some(fact_value) = fact.data.fields.get(&self.field) {
            self.matches_value(fact_value)
        } else {
            false
        }
    }

    /// Check if a specific value matches this pattern
    pub fn matches_value(&self, fact_value: &FactValue) -> bool {
        match self.operator {
            Operator::Equal => fact_value == &self.value,
            Operator::NotEqual => fact_value != &self.value,
            Operator::GreaterThan => {
                if let (Some(fact_num), Some(pattern_num)) =
                    (fact_value.to_comparable(), self.value.to_comparable())
                {
                    fact_num > pattern_num
                } else {
                    false
                }
            }
            Operator::LessThan => {
                if let (Some(fact_num), Some(pattern_num)) =
                    (fact_value.to_comparable(), self.value.to_comparable())
                {
                    fact_num < pattern_num
                } else {
                    false
                }
            }
            Operator::GreaterThanOrEqual => {
                if let (Some(fact_num), Some(pattern_num)) =
                    (fact_value.to_comparable(), self.value.to_comparable())
                {
                    fact_num >= pattern_num
                } else {
                    false
                }
            }
            Operator::LessThanOrEqual => {
                if let (Some(fact_num), Some(pattern_num)) =
                    (fact_value.to_comparable(), self.value.to_comparable())
                {
                    fact_num <= pattern_num
                } else {
                    false
                }
            }
            Operator::Contains => match (&fact_value, &self.value) {
                (FactValue::String(fact_str), FactValue::String(pattern_str)) => {
                    fact_str.contains(pattern_str)
                }
                (FactValue::Array(fact_arr), search_value) => {
                    fact_arr.iter().any(|item| item == search_value)
                }
                _ => false,
            },
            Operator::StartsWith => match (&fact_value, &self.value) {
                (FactValue::String(fact_str), FactValue::String(pattern_str)) => {
                    fact_str.starts_with(pattern_str)
                }
                _ => false,
            },
            Operator::EndsWith => match (&fact_value, &self.value) {
                (FactValue::String(fact_str), FactValue::String(pattern_str)) => {
                    fact_str.ends_with(pattern_str)
                }
                _ => false,
            },
        }
    }
}

/// Alpha memory storage for facts matching a specific pattern
///
/// Each alpha memory maintains:
/// - A set of fact IDs that match the pattern
/// - Reference count for memory management
/// - Statistics for performance monitoring
#[derive(Debug, Clone)]
pub struct AlphaMemory {
    /// Unique identifier for this alpha memory
    pub id: NodeId,
    /// Pattern that facts in this memory must match
    pub pattern: FactPattern,
    /// Set of fact IDs that match the pattern
    pub matching_facts: HashSet<FactId>,
    /// Rules that depend on this alpha memory
    pub dependent_rules: HashSet<RuleId>,
    /// Number of times this memory has been accessed
    pub access_count: u64,
    /// Number of facts added to this memory
    pub facts_added: u64,
    /// Number of facts removed from this memory
    pub facts_removed: u64,
}

impl AlphaMemory {
    /// Create a new alpha memory for the given pattern
    pub fn new(id: NodeId, pattern: FactPattern) -> Self {
        Self {
            id,
            pattern,
            matching_facts: HashSet::new(),
            dependent_rules: HashSet::new(),
            access_count: 0,
            facts_added: 0,
            facts_removed: 0,
        }
    }

    /// Add a fact to this alpha memory
    pub fn add_fact(&mut self, fact_id: FactId) -> bool {
        let was_new = self.matching_facts.insert(fact_id);
        if was_new {
            self.facts_added += 1;
        }
        was_new
    }

    /// Remove a fact from this alpha memory
    pub fn remove_fact(&mut self, fact_id: FactId) -> bool {
        let was_present = self.matching_facts.remove(&fact_id);
        if was_present {
            self.facts_removed += 1;
        }
        was_present
    }

    /// Get all matching facts (increments access counter)
    pub fn get_matching_facts(&mut self) -> &HashSet<FactId> {
        self.access_count += 1;
        &self.matching_facts
    }

    /// Get matching facts count without incrementing access counter
    pub fn count(&self) -> usize {
        self.matching_facts.len()
    }

    /// Add a rule dependency
    pub fn add_dependent_rule(&mut self, rule_id: RuleId) {
        self.dependent_rules.insert(rule_id);
    }

    /// Check if this alpha memory is still needed (has dependent rules)
    pub fn is_needed(&self) -> bool {
        !self.dependent_rules.is_empty()
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> AlphaMemoryStats {
        AlphaMemoryStats {
            pattern: self.pattern.clone(),
            matching_facts_count: self.matching_facts.len(),
            dependent_rules_count: self.dependent_rules.len(),
            access_count: self.access_count,
            facts_added: self.facts_added,
            facts_removed: self.facts_removed,
        }
    }
}

/// Performance statistics for alpha memory
#[derive(Debug, Clone)]
pub struct AlphaMemoryStats {
    pub pattern: FactPattern,
    pub matching_facts_count: usize,
    pub dependent_rules_count: usize,
    pub access_count: u64,
    pub facts_added: u64,
    pub facts_removed: u64,
}

/// Manages all alpha memories with efficient indexing
///
/// The AlphaMemoryManager provides:
/// - O(1) lookup of alpha memories by pattern
/// - Automatic creation of alpha memories as needed
/// - Efficient fact addition/removal propagation
/// - Memory cleanup when alpha memories are no longer needed
/// - Optimized indexing for frequently accessed field patterns
#[derive(Debug)]
pub struct AlphaMemoryManager {
    /// Alpha memories indexed by pattern key
    alpha_memories: HashMap<String, AlphaMemory>,
    /// Pattern index for efficient lookups
    pattern_index: HashMap<String, Vec<String>>, // field -> [pattern_keys]
    /// Fast lookup index for common equality patterns (field=value)
    equality_index: HashMap<String, HashMap<FactValue, Vec<String>>>, // field -> value -> [pattern_keys]
    /// Range index for numeric comparisons (field -> sorted list of thresholds)
    range_index: HashMap<String, Vec<(f64, Vec<String>)>>, // field -> [(threshold, pattern_keys)]
    /// Pattern access frequency tracking for optimization
    pattern_frequency: HashMap<String, u64>,
    /// Next alpha memory ID
    next_id: NodeId,
    /// Total facts processed
    total_facts_processed: u64,
    /// Total pattern matches found
    total_matches_found: u64,
}

impl AlphaMemoryManager {
    /// Create a new alpha memory manager
    pub fn new() -> Self {
        Self {
            alpha_memories: HashMap::new(),
            pattern_index: HashMap::new(),
            equality_index: HashMap::new(),
            range_index: HashMap::new(),
            pattern_frequency: HashMap::new(),
            next_id: 1,
            total_facts_processed: 0,
            total_matches_found: 0,
        }
    }

    /// Get or create an alpha memory for the given pattern
    #[instrument(skip(self))]
    pub fn get_or_create_alpha_memory(&mut self, pattern: FactPattern) -> &mut AlphaMemory {
        let pattern_key = pattern.to_key();

        if !self.alpha_memories.contains_key(&pattern_key) {
            debug!("Creating new alpha memory for pattern: {}", pattern_key);

            let alpha_memory = AlphaMemory::new(self.next_id, pattern.clone());
            self.next_id += 1;

            // Add to pattern index for efficient field-based lookups
            self.pattern_index
                .entry(pattern.field.clone())
                .or_default()
                .push(pattern_key.clone());

            // Add to optimized indexes based on operator type
            self.add_to_optimized_indexes(&pattern, &pattern_key);

            self.alpha_memories.insert(pattern_key.clone(), alpha_memory);
        }

        self.alpha_memories.get_mut(&pattern_key).unwrap()
    }

    /// Process a new fact through all alpha memories using optimized indexing
    #[instrument(skip(self, fact))]
    pub fn process_fact_addition(&mut self, fact_id: FactId, fact: &Fact) -> Vec<String> {
        self.total_facts_processed += 1;
        let mut matching_patterns = HashSet::new(); // Use HashSet to avoid duplicates

        // Use optimized indexing for faster pattern matching
        for (field_name, field_value) in &fact.data.fields {
            // Check equality patterns using equality index
            if let Some(value_map) = self.equality_index.get(field_name) {
                if let Some(pattern_keys) = value_map.get(field_value) {
                    for pattern_key in pattern_keys {
                        // Track pattern access frequency
                        *self.pattern_frequency.entry(pattern_key.clone()).or_insert(0) += 1;

                        if let Some(alpha_memory) = self.alpha_memories.get_mut(pattern_key) {
                            if alpha_memory.add_fact(fact_id) {
                                matching_patterns.insert(pattern_key.clone());
                                self.total_matches_found += 1;
                                debug!("Fact {} matches equality pattern {}", fact_id, pattern_key);
                            }
                        }
                    }
                }
            }

            // Check range patterns using range index for numeric values
            if let Some(threshold_list) = self.range_index.get(field_name) {
                if let Some(_numeric_value) = field_value.to_comparable() {
                    for (_threshold, pattern_keys) in threshold_list {
                        for pattern_key in pattern_keys {
                            // Track pattern access frequency
                            *self.pattern_frequency.entry(pattern_key.clone()).or_insert(0) += 1;

                            if let Some(alpha_memory) = self.alpha_memories.get_mut(pattern_key) {
                                if alpha_memory.pattern.matches_fact(fact)
                                    && alpha_memory.add_fact(fact_id)
                                {
                                    matching_patterns.insert(pattern_key.clone());
                                    self.total_matches_found += 1;
                                    debug!(
                                        "Fact {} matches range pattern {}",
                                        fact_id, pattern_key
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback: check any remaining patterns not covered by optimized indexes
        for (pattern_key, alpha_memory) in &mut self.alpha_memories {
            if !matching_patterns.contains(pattern_key) {
                // Track pattern access frequency
                *self.pattern_frequency.entry(pattern_key.clone()).or_insert(0) += 1;

                if alpha_memory.pattern.matches_fact(fact) && alpha_memory.add_fact(fact_id) {
                    matching_patterns.insert(pattern_key.clone());
                    self.total_matches_found += 1;
                    debug!("Fact {} matches fallback pattern {}", fact_id, pattern_key);
                }
            }
        }

        matching_patterns.into_iter().collect()
    }

    /// Process fact removal through all alpha memories
    #[instrument(skip(self))]
    pub fn process_fact_removal(&mut self, fact_id: FactId) -> Vec<String> {
        let mut affected_patterns = Vec::new();

        for (pattern_key, alpha_memory) in &mut self.alpha_memories {
            if alpha_memory.remove_fact(fact_id) {
                affected_patterns.push(pattern_key.clone());
                debug!("Removed fact {} from pattern {}", fact_id, pattern_key);
            }
        }

        affected_patterns
    }

    /// Get alpha memory by pattern
    pub fn get_alpha_memory(&mut self, pattern: &FactPattern) -> Option<&mut AlphaMemory> {
        let pattern_key = pattern.to_key();
        self.alpha_memories.get_mut(&pattern_key)
    }

    /// Get alpha memory by pattern key
    pub fn get_alpha_memory_by_key(&mut self, pattern_key: &str) -> Option<&mut AlphaMemory> {
        self.alpha_memories.get_mut(pattern_key)
    }

    /// Get all alpha memories that might be affected by a field change
    pub fn get_alpha_memories_for_field(&self, field: &str) -> Vec<&AlphaMemory> {
        if let Some(pattern_keys) = self.pattern_index.get(field) {
            pattern_keys.iter().filter_map(|key| self.alpha_memories.get(key)).collect()
        } else {
            Vec::new()
        }
    }

    /// Register a rule dependency on an alpha memory
    pub fn register_rule_dependency(&mut self, pattern: &FactPattern, rule_id: RuleId) {
        if let Some(alpha_memory) = self.get_alpha_memory(pattern) {
            alpha_memory.add_dependent_rule(rule_id);
        }
    }

    /// Clean up unused alpha memories
    pub fn cleanup_unused_memories(&mut self) -> usize {
        let mut removed_count = 0;
        let mut keys_to_remove = Vec::new();

        for (pattern_key, alpha_memory) in &self.alpha_memories {
            if !alpha_memory.is_needed() {
                keys_to_remove.push(pattern_key.clone());
            }
        }

        for key in &keys_to_remove {
            if let Some(alpha_memory) = self.alpha_memories.remove(key) {
                // Remove from pattern index
                if let Some(pattern_keys) = self.pattern_index.get_mut(&alpha_memory.pattern.field)
                {
                    pattern_keys.retain(|k| k != key);
                    if pattern_keys.is_empty() {
                        self.pattern_index.remove(&alpha_memory.pattern.field);
                    }
                }
                removed_count += 1;
            }
        }

        debug!("Cleaned up {} unused alpha memories", removed_count);
        removed_count
    }

    /// Get comprehensive statistics
    pub fn get_statistics(&self) -> AlphaMemoryManagerStats {
        let memory_stats: Vec<AlphaMemoryStats> =
            self.alpha_memories.values().map(|am| am.get_stats()).collect();

        AlphaMemoryManagerStats {
            total_alpha_memories: self.alpha_memories.len(),
            total_patterns_indexed: self.pattern_index.len(),
            total_facts_processed: self.total_facts_processed,
            total_matches_found: self.total_matches_found,
            memory_stats,
        }
    }

    /// Get memory usage estimate in bytes
    pub fn estimate_memory_usage(&self) -> usize {
        let mut total_size = 0;

        // Base structure size
        total_size += std::mem::size_of::<Self>();

        // Alpha memories
        for (key, alpha_memory) in &self.alpha_memories {
            total_size += key.len();
            total_size += std::mem::size_of::<AlphaMemory>();
            total_size += alpha_memory.matching_facts.len() * std::mem::size_of::<FactId>();
            total_size += alpha_memory.dependent_rules.len() * std::mem::size_of::<RuleId>();
        }

        // Pattern index
        for (field, patterns) in &self.pattern_index {
            total_size += field.len();
            total_size += patterns.iter().map(|p| p.len()).sum::<usize>();
        }

        total_size
    }

    /// Add pattern to optimized indexes based on operator type
    fn add_to_optimized_indexes(&mut self, pattern: &FactPattern, pattern_key: &str) {
        match pattern.operator {
            Operator::Equal => {
                // Add to equality index for fast O(1) equality lookups
                self.equality_index
                    .entry(pattern.field.clone())
                    .or_default()
                    .entry(pattern.value.clone())
                    .or_default()
                    .push(pattern_key.to_string());
            }
            Operator::GreaterThan
            | Operator::LessThan
            | Operator::GreaterThanOrEqual
            | Operator::LessThanOrEqual => {
                // Add to range index for numeric comparisons
                if let Some(threshold) = pattern.value.to_comparable() {
                    let range_list = self.range_index.entry(pattern.field.clone()).or_default();

                    // Find the right position to insert (keep sorted by threshold)
                    let insert_pos = range_list
                        .binary_search_by(|(t, _)| {
                            t.partial_cmp(&threshold).unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .unwrap_or_else(|e| e);

                    if insert_pos < range_list.len()
                        && (range_list[insert_pos].0 - threshold).abs() < f64::EPSILON
                    {
                        // Same threshold exists, add to existing entry
                        range_list[insert_pos].1.push(pattern_key.to_string());
                    } else {
                        // Insert new threshold entry
                        range_list.insert(insert_pos, (threshold, vec![pattern_key.to_string()]));
                    }
                }
            }
            _ => {
                // Other operators don't have specialized indexes yet
                // They will be handled by the fallback linear search
            }
        }
    }

    /// Get candidate rules for a fact using optimized indexing
    /// This is the core RETE optimization: O(1) hash lookups instead of O(n) iteration
    pub fn get_candidate_rules_for_fact(&self, fact: &Fact) -> Vec<RuleId> {
        let mut candidate_rules = HashSet::new();

        for (field_name, field_value) in &fact.data.fields {
            // Check equality patterns using equality index (O(1) lookup)
            if let Some(value_map) = self.equality_index.get(field_name) {
                if let Some(pattern_keys) = value_map.get(field_value) {
                    for pattern_key in pattern_keys {
                        if let Some(alpha_memory) = self.alpha_memories.get(pattern_key) {
                            candidate_rules.extend(alpha_memory.dependent_rules.iter().copied());
                        }
                    }
                }
            }

            // Check range patterns using range index for numeric values
            if let Some(threshold_list) = self.range_index.get(field_name) {
                if let Some(_numeric_value) = field_value.to_comparable() {
                    for (_threshold, pattern_keys) in threshold_list {
                        for pattern_key in pattern_keys {
                            if let Some(alpha_memory) = self.alpha_memories.get(pattern_key) {
                                // Only check pattern match for range patterns (more expensive but necessary)
                                if alpha_memory.pattern.matches_fact(fact) {
                                    candidate_rules
                                        .extend(alpha_memory.dependent_rules.iter().copied());
                                }
                            }
                        }
                    }
                }
            }
        }

        candidate_rules.into_iter().collect()
    }

    /// Get pattern access frequency statistics (most frequently accessed patterns)
    pub fn get_pattern_frequency_stats(&self) -> Vec<(String, u64)> {
        let mut frequencies: Vec<_> = self
            .pattern_frequency
            .iter()
            .map(|(pattern, count)| (pattern.clone(), *count))
            .collect();
        frequencies.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by frequency descending
        frequencies
    }

    /// Get optimization statistics showing index effectiveness
    pub fn get_optimization_stats(&self) -> OptimizationStats {
        let equality_patterns = self
            .equality_index
            .values()
            .map(|value_map| value_map.values().map(|patterns| patterns.len()).sum::<usize>())
            .sum::<usize>();

        let range_patterns = self
            .range_index
            .values()
            .map(|threshold_list| {
                threshold_list.iter().map(|(_, patterns)| patterns.len()).sum::<usize>()
            })
            .sum::<usize>();

        let total_patterns = self.alpha_memories.len();
        let unoptimized_patterns =
            total_patterns.saturating_sub(equality_patterns + range_patterns);

        OptimizationStats {
            total_patterns,
            equality_patterns,
            range_patterns,
            unoptimized_patterns,
            optimization_coverage_percentage: if total_patterns > 0 {
                ((equality_patterns + range_patterns) as f64 / total_patterns as f64) * 100.0
            } else {
                0.0
            },
            most_accessed_patterns: self
                .get_pattern_frequency_stats()
                .into_iter()
                .take(10)
                .collect(),
        }
    }
}

impl Default for AlphaMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive statistics for alpha memory manager
#[derive(Debug, Clone)]
pub struct AlphaMemoryManagerStats {
    pub total_alpha_memories: usize,
    pub total_patterns_indexed: usize,
    pub total_facts_processed: u64,
    pub total_matches_found: u64,
    pub memory_stats: Vec<AlphaMemoryStats>,
}

/// Statistics about alpha memory optimization effectiveness
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    pub total_patterns: usize,
    pub equality_patterns: usize,
    pub range_patterns: usize,
    pub unoptimized_patterns: usize,
    pub optimization_coverage_percentage: f64,
    pub most_accessed_patterns: Vec<(String, u64)>,
}

impl std::fmt::Display for AlphaMemoryManagerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Alpha Memory Statistics ===")?;
        writeln!(f, "Total Alpha Memories: {}", self.total_alpha_memories)?;
        writeln!(f, "Total Patterns Indexed: {}", self.total_patterns_indexed)?;
        writeln!(f, "Total Facts Processed: {}", self.total_facts_processed)?;
        writeln!(f, "Total Matches Found: {}", self.total_matches_found)?;

        if self.total_facts_processed > 0 {
            let match_rate =
                (self.total_matches_found as f64 / self.total_facts_processed as f64) * 100.0;
            writeln!(f, "Match Rate: {match_rate:.2}%")?;
        }

        writeln!(f, "\nTop Alpha Memory Usage:")?;
        let mut sorted_memories = self.memory_stats.clone();
        sorted_memories.sort_by(|a, b| b.matching_facts_count.cmp(&a.matching_facts_count));

        for (i, stats) in sorted_memories.iter().take(5).enumerate() {
            writeln!(
                f,
                "  {}. {} -> {} facts ({} accesses)",
                i + 1,
                stats.pattern.to_key(),
                stats.matching_facts_count,
                stats.access_count
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_fact(id: FactId, age: i64, status: &str) -> Fact {
        let mut fields = HashMap::new();
        fields.insert("age".to_string(), FactValue::Integer(age));
        fields.insert("status".to_string(), FactValue::String(status.to_string()));

        Fact {
            id,
            external_id: Some(format!("fact_{id}")),
            timestamp: chrono::Utc::now(),
            data: crate::types::FactData { fields },
        }
    }

    #[test]
    fn test_fact_pattern_creation() {
        let condition = Condition::Simple {
            field: "age".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(21),
        };

        let pattern = FactPattern::from_condition(&condition).unwrap();
        assert_eq!(pattern.field, "age");
        assert_eq!(pattern.operator, Operator::GreaterThan);
        assert_eq!(pattern.value, FactValue::Integer(21));
    }

    #[test]
    fn test_pattern_matching() {
        let pattern = FactPattern {
            field: "age".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(21),
        };

        let fact1 = create_test_fact(1, 25, "active");
        let fact2 = create_test_fact(2, 18, "active");

        assert!(pattern.matches_fact(&fact1));
        assert!(!pattern.matches_fact(&fact2));
    }

    #[test]
    fn test_alpha_memory_basic_operations() {
        let pattern = FactPattern {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("active".to_string()),
        };

        let mut alpha_memory = AlphaMemory::new(1, pattern);

        // Test fact addition
        assert!(alpha_memory.add_fact(1));
        assert!(!alpha_memory.add_fact(1)); // duplicate
        assert_eq!(alpha_memory.count(), 1);

        // Test fact removal
        assert!(alpha_memory.remove_fact(1));
        assert!(!alpha_memory.remove_fact(1)); // already removed
        assert_eq!(alpha_memory.count(), 0);
    }

    #[test]
    fn test_alpha_memory_manager() {
        let mut manager = AlphaMemoryManager::new();

        let pattern1 = FactPattern {
            field: "age".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(21),
        };

        let pattern2 = FactPattern {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("active".to_string()),
        };

        // Create alpha memories
        let _alpha1 = manager.get_or_create_alpha_memory(pattern1.clone());
        let _alpha2 = manager.get_or_create_alpha_memory(pattern2.clone());

        // Test fact processing
        let fact1 = create_test_fact(1, 25, "active");
        let fact2 = create_test_fact(2, 18, "inactive");

        let matches1 = manager.process_fact_addition(1, &fact1);
        let matches2 = manager.process_fact_addition(2, &fact2);

        assert_eq!(matches1.len(), 2); // matches both patterns
        assert_eq!(matches2.len(), 0); // matches neither pattern
    }

    #[test]
    fn test_pattern_key_generation() {
        let pattern = FactPattern {
            field: "age".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(21),
        };

        let key = pattern.to_key();
        assert!(key.contains("age"));
        assert!(key.contains("GreaterThan"));
        assert!(key.contains("21"));
    }

    #[test]
    fn test_alpha_memory_cleanup() {
        let mut manager = AlphaMemoryManager::new();

        let pattern = FactPattern {
            field: "age".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(21),
        };

        // Create alpha memory and add dependency
        {
            let alpha_memory = manager.get_or_create_alpha_memory(pattern.clone());
            alpha_memory.add_dependent_rule(1);
        }

        // Should not be cleaned up while it has dependencies
        assert_eq!(manager.cleanup_unused_memories(), 0);

        // Remove dependency and cleanup
        {
            let alpha_memory = manager.get_alpha_memory(&pattern).unwrap();
            alpha_memory.dependent_rules.clear();
        }
        assert_eq!(manager.cleanup_unused_memories(), 1);
    }
}
