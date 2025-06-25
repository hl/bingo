//! Pattern Compilation Cache
//!
//! This module implements caching for compiled RETE network patterns to avoid
//! redundant compilation work when similar rule structures are encountered.

use crate::rete_nodes::JoinCondition;
use crate::types::{Condition, Rule};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A signature representing a cacheable pattern for compilation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PatternSignature {
    /// Hash of the pattern structure
    pattern_hash: u64,
    /// Human-readable pattern description for debugging
    pattern_description: String,
}

impl PatternSignature {
    /// Create a signature from a rule's conditions
    pub fn from_rule_conditions(conditions: &[Condition]) -> Self {
        let mut hasher = DefaultHasher::new();

        // Create a deterministic hash of the condition structure
        for condition in conditions {
            Self::hash_condition(condition, &mut hasher);
        }

        let pattern_hash = hasher.finish();
        let pattern_description = Self::describe_conditions(conditions);

        Self { pattern_hash, pattern_description }
    }

    /// Create a signature from a single condition
    pub fn from_condition(condition: &Condition) -> Self {
        let mut hasher = DefaultHasher::new();
        Self::hash_condition(condition, &mut hasher);

        let pattern_hash = hasher.finish();
        let pattern_description = Self::describe_condition(condition);

        Self { pattern_hash, pattern_description }
    }

    /// Create a signature for join conditions
    pub fn from_join_conditions(conditions: &[JoinCondition]) -> Self {
        let mut hasher = DefaultHasher::new();

        // Sort conditions for deterministic hashing
        let mut sorted_conditions = conditions.to_vec();
        sorted_conditions.sort_by(|a, b| {
            a.left_field
                .cmp(&b.left_field)
                .then_with(|| a.right_field.cmp(&b.right_field))
                .then_with(|| format!("{:?}", a.operator).cmp(&format!("{:?}", b.operator)))
        });

        for condition in &sorted_conditions {
            condition.left_field.hash(&mut hasher);
            condition.right_field.hash(&mut hasher);
            format!("{:?}", condition.operator).hash(&mut hasher);
        }

        let pattern_hash = hasher.finish();
        let pattern_description = format!(
            "Join[{}]",
            sorted_conditions
                .iter()
                .map(|c| format!("{}={}", c.left_field, c.right_field))
                .collect::<Vec<_>>()
                .join(",")
        );

        Self { pattern_hash, pattern_description }
    }

    /// Hash a condition deterministically
    fn hash_condition(condition: &Condition, hasher: &mut DefaultHasher) {
        match condition {
            Condition::Simple { field, operator, value } => {
                "Simple".hash(hasher);
                field.hash(hasher);
                format!("{:?}", operator).hash(hasher);
                format!("{:?}", value).hash(hasher);
            }
            Condition::Complex { operator, conditions } => {
                "Complex".hash(hasher);
                format!("{:?}", operator).hash(hasher);
                for sub_condition in conditions {
                    Self::hash_condition(sub_condition, hasher);
                }
            }
            Condition::Aggregation(agg) => {
                "Aggregation".hash(hasher);
                format!("{:?}", agg).hash(hasher);
            }
            Condition::Stream(stream) => {
                "Stream".hash(hasher);
                format!("{:?}", stream).hash(hasher);
            }
        }
    }

    /// Create a human-readable description of conditions
    fn describe_conditions(conditions: &[Condition]) -> String {
        let descriptions: Vec<String> = conditions.iter().map(Self::describe_condition).collect();
        format!("Pattern[{}]", descriptions.join(","))
    }

    /// Create a human-readable description of a single condition
    fn describe_condition(condition: &Condition) -> String {
        match condition {
            Condition::Simple { field, operator, .. } => {
                format!("{}:{:?}", field, operator)
            }
            Condition::Complex { operator, conditions } => {
                format!("{:?}({})", operator, conditions.len())
            }
            Condition::Aggregation(_) => "Agg".to_string(),
            Condition::Stream(_) => "Stream".to_string(),
        }
    }
}

/// Cached compilation plan for a pattern
#[derive(Debug, Clone)]
pub struct CompilationPlan {
    /// Alpha nodes that need to be created or referenced
    pub alpha_nodes: Vec<AlphaNodePlan>,
    /// Beta nodes that need to be created
    pub beta_nodes: Vec<BetaNodePlan>,
    /// Join conditions for connecting nodes
    pub join_conditions: Vec<JoinCondition>,
    /// Estimated node count for this pattern
    pub estimated_node_count: usize,
}

/// Plan for creating or reusing an alpha node
#[derive(Debug, Clone)]
pub struct AlphaNodePlan {
    /// The condition this alpha node tests
    pub condition: Condition,
    /// Pattern signature for this alpha node
    pub signature: PatternSignature,
    /// Whether this node is likely to be shareable
    pub shareable: bool,
}

/// Plan for creating a beta node
#[derive(Debug, Clone)]
pub struct BetaNodePlan {
    /// Join conditions for this beta node
    pub join_conditions: Vec<JoinCondition>,
    /// Pattern signature for this beta node
    pub signature: PatternSignature,
    /// Left input node index
    pub left_input: usize,
    /// Right input node index
    pub right_input: usize,
}

/// Pattern compilation cache
#[derive(Debug)]
pub struct PatternCache {
    /// Cache of compiled patterns
    pattern_cache: HashMap<PatternSignature, CompilationPlan>,
    /// Cache of alpha node patterns
    alpha_cache: HashMap<PatternSignature, AlphaNodePlan>,
    /// Cache of join condition patterns
    join_cache: HashMap<PatternSignature, Vec<JoinCondition>>,
    /// Statistics for cache performance
    pub stats: PatternCacheStats,
}

/// Statistics for pattern cache performance
#[derive(Debug, Clone, Default)]
pub struct PatternCacheStats {
    pub pattern_cache_hits: usize,
    pub pattern_cache_misses: usize,
    pub alpha_cache_hits: usize,
    pub alpha_cache_misses: usize,
    pub join_cache_hits: usize,
    pub join_cache_misses: usize,
    pub patterns_cached: usize,
    pub cache_memory_usage: usize,
}

impl PatternCacheStats {
    /// Get overall cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total_hits = self.pattern_cache_hits + self.alpha_cache_hits + self.join_cache_hits;
        let total_misses =
            self.pattern_cache_misses + self.alpha_cache_misses + self.join_cache_misses;
        let total_requests = total_hits + total_misses;

        if total_requests == 0 {
            0.0
        } else {
            (total_hits as f64 / total_requests as f64) * 100.0
        }
    }

    /// Get pattern cache hit rate
    pub fn pattern_hit_rate(&self) -> f64 {
        let total = self.pattern_cache_hits + self.pattern_cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.pattern_cache_hits as f64 / total as f64) * 100.0
        }
    }
}

impl PatternCache {
    /// Create a new pattern cache
    pub fn new() -> Self {
        Self {
            pattern_cache: HashMap::new(),
            alpha_cache: HashMap::new(),
            join_cache: HashMap::new(),
            stats: PatternCacheStats::default(),
        }
    }

    /// Create a pattern cache with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pattern_cache: HashMap::with_capacity(capacity),
            alpha_cache: HashMap::with_capacity(capacity),
            join_cache: HashMap::with_capacity(capacity),
            stats: PatternCacheStats::default(),
        }
    }

    /// Get a cached compilation plan for a rule
    pub fn get_rule_pattern(&mut self, rule: &Rule) -> Option<&CompilationPlan> {
        let signature = PatternSignature::from_rule_conditions(&rule.conditions);

        if let Some(plan) = self.pattern_cache.get(&signature) {
            self.stats.pattern_cache_hits += 1;
            Some(plan)
        } else {
            self.stats.pattern_cache_misses += 1;
            None
        }
    }

    /// Cache a compilation plan for a rule
    pub fn cache_rule_pattern(&mut self, rule: &Rule, plan: CompilationPlan) {
        let signature = PatternSignature::from_rule_conditions(&rule.conditions);
        self.pattern_cache.insert(signature, plan);
        self.stats.patterns_cached += 1;
        self.update_memory_usage();
    }

    /// Get a cached alpha node plan
    pub fn get_alpha_pattern(&mut self, condition: &Condition) -> Option<&AlphaNodePlan> {
        let signature = PatternSignature::from_condition(condition);

        if let Some(plan) = self.alpha_cache.get(&signature) {
            self.stats.alpha_cache_hits += 1;
            Some(plan)
        } else {
            self.stats.alpha_cache_misses += 1;
            None
        }
    }

    /// Cache an alpha node plan
    pub fn cache_alpha_pattern(&mut self, condition: &Condition, plan: AlphaNodePlan) {
        let signature = PatternSignature::from_condition(condition);
        self.alpha_cache.insert(signature, plan);
        self.update_memory_usage();
    }

    /// Get cached join conditions
    pub fn get_join_pattern(&mut self, field_names: &[String]) -> Option<&Vec<JoinCondition>> {
        // Create a simple signature based on field names
        let mut hasher = DefaultHasher::new();
        let mut sorted_fields = field_names.to_vec();
        sorted_fields.sort();
        for field in &sorted_fields {
            field.hash(&mut hasher);
        }

        let signature = PatternSignature {
            pattern_hash: hasher.finish(),
            pattern_description: format!("Fields[{}]", sorted_fields.join(",")),
        };

        if let Some(conditions) = self.join_cache.get(&signature) {
            self.stats.join_cache_hits += 1;
            Some(conditions)
        } else {
            self.stats.join_cache_misses += 1;
            None
        }
    }

    /// Cache join conditions
    pub fn cache_join_pattern(&mut self, field_names: &[String], conditions: Vec<JoinCondition>) {
        // Create a simple signature based on field names
        let mut hasher = DefaultHasher::new();
        let mut sorted_fields = field_names.to_vec();
        sorted_fields.sort();
        for field in &sorted_fields {
            field.hash(&mut hasher);
        }

        let signature = PatternSignature {
            pattern_hash: hasher.finish(),
            pattern_description: format!("Fields[{}]", sorted_fields.join(",")),
        };

        self.join_cache.insert(signature, conditions);
        self.update_memory_usage();
    }

    /// Create a compilation plan for a rule
    pub fn create_compilation_plan(&self, rule: &Rule) -> CompilationPlan {
        let mut alpha_nodes = Vec::new();
        let mut beta_nodes = Vec::new();

        // Analyze conditions to create alpha node plans
        for condition in &rule.conditions {
            match condition {
                Condition::Simple { .. } => {
                    let signature = PatternSignature::from_condition(condition);
                    let plan = AlphaNodePlan {
                        condition: condition.clone(),
                        signature,
                        shareable: true, // Simple conditions are always shareable
                    };
                    alpha_nodes.push(plan);
                }
                Condition::Complex { conditions: sub_conditions, .. } => {
                    // Expand complex conditions into alpha nodes
                    for sub_condition in sub_conditions {
                        if let Condition::Simple { .. } = sub_condition {
                            let signature = PatternSignature::from_condition(sub_condition);
                            let plan = AlphaNodePlan {
                                condition: sub_condition.clone(),
                                signature,
                                shareable: true,
                            };
                            alpha_nodes.push(plan);
                        }
                    }
                }
                _ => {
                    // For non-simple conditions, create a placeholder
                    let signature = PatternSignature::from_condition(condition);
                    let plan =
                        AlphaNodePlan { condition: condition.clone(), signature, shareable: false };
                    alpha_nodes.push(plan);
                }
            }
        }

        // Generate join conditions
        let join_conditions = self.generate_join_conditions_for_pattern(&rule.conditions);

        // Create beta nodes if we have multiple alpha nodes
        if alpha_nodes.len() > 1 {
            let signature = PatternSignature::from_join_conditions(&join_conditions);
            let beta_plan = BetaNodePlan {
                join_conditions: join_conditions.clone(),
                signature,
                left_input: 0,
                right_input: 1,
            };
            beta_nodes.push(beta_plan);
        }

        let estimated_node_count = alpha_nodes.len() + beta_nodes.len() + 1; // +1 for terminal

        CompilationPlan { alpha_nodes, beta_nodes, join_conditions, estimated_node_count }
    }

    /// Generate join conditions for a pattern (similar to RETE network's method)
    fn generate_join_conditions_for_pattern(&self, conditions: &[Condition]) -> Vec<JoinCondition> {
        let mut join_conditions = Vec::new();

        // Look for common join fields
        let join_fields = ["entity_id", "id", "user_id", "customer_id"];

        for &field in &join_fields {
            let field_conditions: Vec<_> = conditions
                .iter()
                .filter(|cond| {
                    if let Condition::Simple { field: cond_field, .. } = cond {
                        cond_field == field
                    } else {
                        false
                    }
                })
                .collect();

            if field_conditions.len() >= 2 {
                join_conditions.push(JoinCondition {
                    left_field: field.to_string(),
                    right_field: field.to_string(),
                    operator: crate::types::Operator::Equal,
                });
                break; // Only need one join condition for simple cases
            }
        }

        join_conditions
    }

    /// Update memory usage statistics
    fn update_memory_usage(&mut self) {
        let pattern_memory = self.pattern_cache.len() * std::mem::size_of::<CompilationPlan>();
        let alpha_memory = self.alpha_cache.len() * std::mem::size_of::<AlphaNodePlan>();
        let join_memory = self.join_cache.len() * std::mem::size_of::<Vec<JoinCondition>>();

        self.stats.cache_memory_usage = pattern_memory + alpha_memory + join_memory;
    }

    /// Clear all caches
    pub fn clear(&mut self) {
        self.pattern_cache.clear();
        self.alpha_cache.clear();
        self.join_cache.clear();
        self.stats = PatternCacheStats::default();
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.pattern_cache.len() + self.alpha_cache.len() + self.join_cache.len()
    }

    /// Emergency cleanup for critical memory pressure
    pub fn emergency_cleanup(&mut self) {
        // Keep only the most recent 10% of entries
        let keep_count = (self.pattern_cache.len() / 10).max(5);

        // Clear most pattern cache entries
        if self.pattern_cache.len() > keep_count {
            let excess = self.pattern_cache.len() - keep_count;
            // Remove entries (HashMap doesn't preserve order, so this removes arbitrary entries)
            let keys_to_remove: Vec<_> = self.pattern_cache.keys().take(excess).cloned().collect();
            for key in keys_to_remove {
                self.pattern_cache.remove(&key);
            }
        }

        // Clear alpha cache more aggressively
        let alpha_keep = (self.alpha_cache.len() / 20).max(3);
        if self.alpha_cache.len() > alpha_keep {
            let excess = self.alpha_cache.len() - alpha_keep;
            let keys_to_remove: Vec<_> = self.alpha_cache.keys().take(excess).cloned().collect();
            for key in keys_to_remove {
                self.alpha_cache.remove(&key);
            }
        }

        // Clear join cache completely - it's typically small
        self.join_cache.clear();

        // Update statistics
        self.update_memory_usage();
    }

    /// Reduce cache capacity by the given factor (0.0 to 1.0)
    pub fn reduce_capacity(&mut self, reduction_factor: f64) {
        let factor = reduction_factor.clamp(0.0, 1.0);

        // Calculate target sizes
        let pattern_target = ((self.pattern_cache.len() as f64) * (1.0 - factor)) as usize;
        let alpha_target = ((self.alpha_cache.len() as f64) * (1.0 - factor)) as usize;
        let join_target = ((self.join_cache.len() as f64) * (1.0 - factor)) as usize;

        // Remove excess pattern cache entries
        if self.pattern_cache.len() > pattern_target {
            let excess = self.pattern_cache.len() - pattern_target;
            let keys_to_remove: Vec<_> = self.pattern_cache.keys().take(excess).cloned().collect();
            for key in keys_to_remove {
                self.pattern_cache.remove(&key);
            }
        }

        // Remove excess alpha cache entries
        if self.alpha_cache.len() > alpha_target {
            let excess = self.alpha_cache.len() - alpha_target;
            let keys_to_remove: Vec<_> = self.alpha_cache.keys().take(excess).cloned().collect();
            for key in keys_to_remove {
                self.alpha_cache.remove(&key);
            }
        }

        // Remove excess join cache entries
        if self.join_cache.len() > join_target {
            let excess = self.join_cache.len() - join_target;
            let keys_to_remove: Vec<_> = self.join_cache.keys().take(excess).cloned().collect();
            for key in keys_to_remove {
                self.join_cache.remove(&key);
            }
        }

        self.update_memory_usage();
    }

    /// Clean up old cache entries based on age
    pub fn cleanup_old_entries(&mut self, max_age: std::time::Duration) {
        // For simplicity, we'll clean up a portion based on the age threshold
        // In a real implementation, we'd track timestamps for each entry
        let cleanup_ratio = if max_age.as_secs() < 300 { 0.5 } else { 0.2 }; // More aggressive for shorter durations

        let pattern_remove = (self.pattern_cache.len() as f64 * cleanup_ratio) as usize;
        let alpha_remove = (self.alpha_cache.len() as f64 * cleanup_ratio) as usize;
        let join_remove = (self.join_cache.len() as f64 * cleanup_ratio) as usize;

        if pattern_remove > 0 {
            let keys_to_remove: Vec<_> =
                self.pattern_cache.keys().take(pattern_remove).cloned().collect();
            for key in keys_to_remove {
                self.pattern_cache.remove(&key);
            }
        }

        if alpha_remove > 0 {
            let keys_to_remove: Vec<_> =
                self.alpha_cache.keys().take(alpha_remove).cloned().collect();
            for key in keys_to_remove {
                self.alpha_cache.remove(&key);
            }
        }

        if join_remove > 0 {
            let keys_to_remove: Vec<_> =
                self.join_cache.keys().take(join_remove).cloned().collect();
            for key in keys_to_remove {
                self.join_cache.remove(&key);
            }
        }

        self.update_memory_usage();
    }
}

impl Default for PatternCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Action, FactValue, Operator, Rule};

    fn create_test_rule(id: u64, field: &str, value: i64) -> Rule {
        Rule {
            id,
            name: format!("test_rule_{}", id),
            conditions: vec![Condition::Simple {
                field: field.to_string(),
                operator: Operator::Equal,
                value: FactValue::Integer(value),
            }],
            actions: vec![Action {
                action_type: crate::types::ActionType::Log { message: "test".to_string() },
            }],
        }
    }

    #[test]
    fn test_pattern_signature_consistency() {
        let rule1 = create_test_rule(1, "age", 25);
        let rule2 = create_test_rule(2, "age", 25); // Same pattern, different ID
        let rule3 = create_test_rule(3, "score", 100); // Different pattern

        let sig1 = PatternSignature::from_rule_conditions(&rule1.conditions);
        let sig2 = PatternSignature::from_rule_conditions(&rule2.conditions);
        let sig3 = PatternSignature::from_rule_conditions(&rule3.conditions);

        assert_eq!(sig1, sig2); // Same pattern should have same signature
        assert_ne!(sig1, sig3); // Different patterns should have different signatures
    }

    #[test]
    fn test_pattern_cache_basic_operations() {
        let mut cache = PatternCache::new();
        let rule = create_test_rule(1, "status", 1);

        // First lookup should miss
        assert!(cache.get_rule_pattern(&rule).is_none());
        assert_eq!(cache.stats.pattern_cache_misses, 1);

        // Create and cache a plan
        let plan = cache.create_compilation_plan(&rule);
        cache.cache_rule_pattern(&rule, plan);

        // Second lookup should hit
        assert!(cache.get_rule_pattern(&rule).is_some());
        assert_eq!(cache.stats.pattern_cache_hits, 1);
        assert_eq!(cache.stats.patterns_cached, 1);
    }

    #[test]
    fn test_alpha_pattern_caching() {
        let mut cache = PatternCache::new();
        let condition = Condition::Simple {
            field: "user_id".to_string(),
            operator: Operator::Equal,
            value: FactValue::Integer(123),
        };

        // First lookup should miss
        assert!(cache.get_alpha_pattern(&condition).is_none());
        assert_eq!(cache.stats.alpha_cache_misses, 1);

        // Cache an alpha pattern
        let plan = AlphaNodePlan {
            condition: condition.clone(),
            signature: PatternSignature::from_condition(&condition),
            shareable: true,
        };
        cache.cache_alpha_pattern(&condition, plan);

        // Second lookup should hit
        assert!(cache.get_alpha_pattern(&condition).is_some());
        assert_eq!(cache.stats.alpha_cache_hits, 1);
    }

    #[test]
    fn test_compilation_plan_creation() {
        let cache = PatternCache::new();
        let rule = Rule {
            id: 1,
            name: "multi_condition_rule".to_string(),
            conditions: vec![
                Condition::Simple {
                    field: "user_id".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::Integer(123),
                },
                Condition::Simple {
                    field: "user_id".to_string(),
                    operator: Operator::GreaterThan,
                    value: FactValue::Integer(0),
                },
            ],
            actions: vec![],
        };

        let plan = cache.create_compilation_plan(&rule);
        assert_eq!(plan.alpha_nodes.len(), 2);
        assert_eq!(plan.beta_nodes.len(), 1); // Should create one beta node for joining
        assert_eq!(plan.estimated_node_count, 4); // 2 alpha + 1 beta + 1 terminal
    }

    #[test]
    fn test_cache_statistics() {
        let mut cache = PatternCache::new();
        let rule1 = create_test_rule(1, "age", 25);
        let rule2 = create_test_rule(2, "age", 25); // Same pattern

        // Generate some cache activity
        let _miss1 = cache.get_rule_pattern(&rule1);
        let _miss2 = cache.get_rule_pattern(&rule2);

        let plan = cache.create_compilation_plan(&rule1);
        cache.cache_rule_pattern(&rule1, plan);

        let _hit1 = cache.get_rule_pattern(&rule1);
        let _hit2 = cache.get_rule_pattern(&rule2); // Should hit with same pattern

        assert_eq!(cache.stats.pattern_cache_hits, 2);
        assert_eq!(cache.stats.pattern_cache_misses, 2);
        assert_eq!(cache.stats.pattern_hit_rate(), 50.0);
    }
}
