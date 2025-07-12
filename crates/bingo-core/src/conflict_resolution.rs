//! Conflict Resolution Strategies for Rule Execution Order
//!
//! This module implements various conflict resolution strategies to determine
//! the order of rule execution when multiple rules are triggered simultaneously
//! by the same fact or set of facts.
//!
//! ## Conflict Resolution Goals
//!
//! - **Deterministic execution**: Same input should always produce same output
//! - **Optimal performance**: Minimize execution time through intelligent ordering
//! - **Business logic support**: Allow domain-specific rule prioritization
//! - **Debugging support**: Provide clear execution order reasoning
//!
//! ## Supported Strategies
//!
//! 1. **Priority-Based**: Rules with higher priority execute first
//! 2. **Salience-Based**: Classic RETE salience values for rule ordering
//! 3. **Recency-Based**: Most recently added/modified rules execute first
//! 4. **Specificity-Based**: More specific rules (more conditions) execute first
//! 5. **Custom Strategy**: User-defined conflict resolution logic
//!
//! ## Architecture Overview
//!
//! ```text
//! Conflict Resolution Flow:
//!
//! Multiple Rules → Conflict Set → Resolution Strategy → Ordered Execution
//!     Triggered      Formation        Application           Queue
//!         ↓              ↓                ↓                  ↓
//!     Rule A,B,C    [A,B,C]         Strategy Logic    Execute: A→C→B
//! ```

use crate::error::BingoResult;
use crate::types::{Fact, FactId, Rule, RuleId};
use std::collections::HashMap;
use tracing::{debug, info, instrument};

/// Conflict resolution strategy types
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolutionStrategy {
    /// Execute rules in priority order (highest first)
    Priority,
    /// Execute rules by salience value (highest first)
    Salience,
    /// Execute most recently added rules first
    Recency,
    /// Execute more specific rules (more conditions) first
    Specificity,
    /// Execute rules in lexicographic order by name
    Lexicographic,
    /// Custom strategy with user-defined comparison function
    Custom(fn(&RuleExecution, &RuleExecution) -> std::cmp::Ordering),
}

/// Configuration for conflict resolution
#[derive(Debug, Clone)]
pub struct ConflictResolutionConfig {
    /// Primary resolution strategy
    pub primary_strategy: ConflictResolutionStrategy,
    /// Secondary strategy for tie-breaking
    pub tie_breaker: Option<ConflictResolutionStrategy>,
    /// Enable detailed logging of resolution decisions
    pub enable_logging: bool,
    /// Maximum rules to consider in conflict set
    pub max_conflict_set_size: usize,
}

impl Default for ConflictResolutionConfig {
    fn default() -> Self {
        Self {
            primary_strategy: ConflictResolutionStrategy::Priority,
            tie_breaker: Some(ConflictResolutionStrategy::Recency),
            enable_logging: false,
            max_conflict_set_size: 1000,
        }
    }
}

/// Represents a rule execution in the conflict set
#[derive(Debug, Clone)]
pub struct RuleExecution {
    /// The rule to execute
    pub rule: Rule,
    /// Facts that triggered this rule
    pub triggering_facts: Vec<Fact>,
    /// Primary fact ID that caused the trigger
    pub primary_fact_id: FactId,
    /// Timestamp when rule was triggered
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    /// Rule priority (higher values = higher priority)
    pub priority: i32,
    /// Rule salience value
    pub salience: i32,
    /// Specificity score (number of conditions)
    pub specificity: usize,
}

impl RuleExecution {
    /// Create a new rule execution
    pub fn new(
        rule: Rule,
        triggering_facts: Vec<Fact>,
        primary_fact_id: FactId,
        priority: i32,
        salience: i32,
    ) -> Self {
        let specificity = rule.conditions.len();
        Self {
            rule,
            triggering_facts,
            primary_fact_id,
            triggered_at: chrono::Utc::now(),
            priority,
            salience,
            specificity,
        }
    }

    /// Get rule ID
    pub fn rule_id(&self) -> RuleId {
        self.rule.id
    }

    /// Get rule name
    pub fn rule_name(&self) -> &str {
        &self.rule.name
    }
}

/// Statistics for conflict resolution
#[derive(Debug, Default, Clone)]
pub struct ConflictResolutionStats {
    /// Total conflict sets resolved
    pub conflict_sets_resolved: usize,
    /// Total rules ordered
    pub rules_ordered: usize,
    /// Average conflict set size
    pub average_conflict_set_size: f64,
    /// Maximum conflict set size encountered
    pub max_conflict_set_size: usize,
    /// Total resolution time in milliseconds
    pub total_resolution_time_ms: u64,
    /// Number of tie-breaking decisions
    pub tie_breaking_decisions: usize,
}

/// Main conflict resolution manager
pub struct ConflictResolutionManager {
    config: ConflictResolutionConfig,
    stats: ConflictResolutionStats,
    /// Rule priority mappings
    rule_priorities: HashMap<RuleId, i32>,
    /// Rule salience mappings  
    rule_salience: HashMap<RuleId, i32>,
    /// Rule addition timestamps for recency calculation
    rule_timestamps: HashMap<RuleId, chrono::DateTime<chrono::Utc>>,
}

impl ConflictResolutionManager {
    /// Create a new conflict resolution manager
    pub fn new(config: ConflictResolutionConfig) -> Self {
        Self {
            config,
            stats: ConflictResolutionStats::default(),
            rule_priorities: HashMap::new(),
            rule_salience: HashMap::new(),
            rule_timestamps: HashMap::new(),
        }
    }

    /// Register a rule with priority and salience values
    pub fn register_rule(
        &mut self,
        rule_id: RuleId,
        priority: i32,
        salience: i32,
    ) -> BingoResult<()> {
        self.rule_priorities.insert(rule_id, priority);
        self.rule_salience.insert(rule_id, salience);
        self.rule_timestamps.insert(rule_id, chrono::Utc::now());

        debug!(
            rule_id = rule_id,
            priority = priority,
            salience = salience,
            "Registered rule for conflict resolution"
        );

        Ok(())
    }

    /// Resolve conflicts in a set of triggered rules
    #[instrument(skip(self, conflict_set))]
    pub fn resolve_conflicts(
        &mut self,
        mut conflict_set: Vec<RuleExecution>,
    ) -> BingoResult<Vec<RuleExecution>> {
        let start_time = std::time::Instant::now();
        let original_size = conflict_set.len();

        if conflict_set.is_empty() {
            return Ok(conflict_set);
        }

        // Limit conflict set size if configured
        if conflict_set.len() > self.config.max_conflict_set_size {
            info!(
                original_size = conflict_set.len(),
                max_size = self.config.max_conflict_set_size,
                "Truncating large conflict set"
            );
            conflict_set.truncate(self.config.max_conflict_set_size);
        }

        if self.config.enable_logging {
            info!(
                conflict_set_size = conflict_set.len(),
                primary_strategy = ?self.config.primary_strategy,
                "Starting conflict resolution"
            );
        }

        // Apply primary resolution strategy
        conflict_set
            .sort_by(|a, b| self.compare_rule_executions(a, b, &self.config.primary_strategy));

        // Apply tie-breaker if configured
        if let Some(tie_breaker) = self.config.tie_breaker.clone() {
            conflict_set = self.apply_tie_breaker(conflict_set, &tie_breaker)?;
        }

        // Update statistics
        let resolution_time = start_time.elapsed();
        self.update_stats(original_size, resolution_time);

        if self.config.enable_logging {
            self.log_resolution_result(&conflict_set);
        }

        debug!(
            resolved_rules = conflict_set.len(),
            resolution_time_ms = resolution_time.as_millis(),
            "Conflict resolution completed"
        );

        Ok(conflict_set)
    }

    /// Compare two rule executions using the specified strategy
    fn compare_rule_executions(
        &self,
        a: &RuleExecution,
        b: &RuleExecution,
        strategy: &ConflictResolutionStrategy,
    ) -> std::cmp::Ordering {
        match strategy {
            ConflictResolutionStrategy::Priority => {
                // Higher priority first (reverse order)
                b.priority.cmp(&a.priority)
            }
            ConflictResolutionStrategy::Salience => {
                // Higher salience first (reverse order)
                b.salience.cmp(&a.salience)
            }
            ConflictResolutionStrategy::Recency => {
                // More recent first (reverse order)
                b.triggered_at.cmp(&a.triggered_at)
            }
            ConflictResolutionStrategy::Specificity => {
                // More specific (more conditions) first (reverse order)
                b.specificity.cmp(&a.specificity)
            }
            ConflictResolutionStrategy::Lexicographic => {
                // Alphabetical order by rule name
                a.rule_name().cmp(b.rule_name())
            }
            ConflictResolutionStrategy::Custom(comparator) => comparator(a, b),
        }
    }

    /// Apply tie-breaker strategy to rules with equal primary ordering
    fn apply_tie_breaker(
        &mut self,
        conflict_set: Vec<RuleExecution>,
        tie_breaker: &ConflictResolutionStrategy,
    ) -> BingoResult<Vec<RuleExecution>> {
        // Group consecutive rules with the same primary ordering
        let mut result = Vec::new();
        let mut current_group = Vec::new();
        let mut last_primary_key: Option<String> = None;

        for rule_exec in conflict_set {
            let primary_key = self.get_primary_key(&rule_exec, &self.config.primary_strategy);

            if last_primary_key.as_ref() == Some(&primary_key) {
                current_group.push(rule_exec);
            } else {
                // Process previous group with tie-breaker
                if !current_group.is_empty() {
                    self.apply_tie_breaker_to_group(&mut current_group, tie_breaker);
                    result.extend(current_group);
                    current_group = Vec::new();
                    self.stats.tie_breaking_decisions += 1;
                }
                current_group.push(rule_exec);
                last_primary_key = Some(primary_key);
            }
        }

        // Process final group
        if !current_group.is_empty() {
            let group_size = current_group.len();
            self.apply_tie_breaker_to_group(&mut current_group, tie_breaker);
            result.extend(current_group);
            if group_size > 1 {
                self.stats.tie_breaking_decisions += 1;
            }
        }

        Ok(result)
    }

    /// Apply tie-breaker to a group of rules with equal primary ordering
    fn apply_tie_breaker_to_group(
        &self,
        group: &mut [RuleExecution],
        tie_breaker: &ConflictResolutionStrategy,
    ) {
        if group.len() <= 1 {
            return;
        }

        group.sort_by(|a, b| self.compare_rule_executions(a, b, tie_breaker));
    }

    /// Get a string key representing the primary ordering value
    fn get_primary_key(
        &self,
        rule_exec: &RuleExecution,
        strategy: &ConflictResolutionStrategy,
    ) -> String {
        match strategy {
            ConflictResolutionStrategy::Priority => rule_exec.priority.to_string(),
            ConflictResolutionStrategy::Salience => rule_exec.salience.to_string(),
            ConflictResolutionStrategy::Recency => rule_exec.triggered_at.to_rfc3339(),
            ConflictResolutionStrategy::Specificity => rule_exec.specificity.to_string(),
            ConflictResolutionStrategy::Lexicographic => rule_exec.rule_name().to_string(),
            ConflictResolutionStrategy::Custom(_) => {
                // For custom strategies, use rule ID as fallback
                rule_exec.rule_id().to_string()
            }
        }
    }

    /// Update conflict resolution statistics
    fn update_stats(&mut self, conflict_set_size: usize, resolution_time: std::time::Duration) {
        self.stats.conflict_sets_resolved += 1;
        self.stats.rules_ordered += conflict_set_size;
        self.stats.total_resolution_time_ms += resolution_time.as_millis() as u64;
        self.stats.max_conflict_set_size = self.stats.max_conflict_set_size.max(conflict_set_size);

        // Update running average
        let total_rules = self.stats.rules_ordered as f64;
        let total_sets = self.stats.conflict_sets_resolved as f64;
        self.stats.average_conflict_set_size = total_rules / total_sets;
    }

    /// Log the resolution result for debugging
    fn log_resolution_result(&self, ordered_rules: &[RuleExecution]) {
        info!("Conflict resolution result:");
        for (index, rule_exec) in ordered_rules.iter().enumerate() {
            info!(
                execution_order = index + 1,
                rule_id = rule_exec.rule_id(),
                rule_name = rule_exec.rule_name(),
                priority = rule_exec.priority,
                salience = rule_exec.salience,
                specificity = rule_exec.specificity,
                "Rule execution order"
            );
        }
    }

    /// Get current conflict resolution statistics
    pub fn get_stats(&self) -> &ConflictResolutionStats {
        &self.stats
    }

    /// Reset conflict resolution statistics
    pub fn reset_stats(&mut self) {
        self.stats = ConflictResolutionStats::default();
    }

    /// Update configuration
    pub fn update_config(&mut self, new_config: ConflictResolutionConfig) {
        info!("Updating conflict resolution configuration");
        self.config = new_config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ConflictResolutionConfig {
        &self.config
    }

    /// Get rule priority
    pub fn get_rule_priority(&self, rule_id: RuleId) -> Option<i32> {
        self.rule_priorities.get(&rule_id).copied()
    }

    /// Set rule priority
    pub fn set_rule_priority(&mut self, rule_id: RuleId, priority: i32) -> BingoResult<()> {
        self.rule_priorities.insert(rule_id, priority);
        debug!(
            rule_id = rule_id,
            priority = priority,
            "Updated rule priority"
        );
        Ok(())
    }

    /// Get rule salience
    pub fn get_rule_salience(&self, rule_id: RuleId) -> Option<i32> {
        self.rule_salience.get(&rule_id).copied()
    }

    /// Set rule salience
    pub fn set_rule_salience(&mut self, rule_id: RuleId, salience: i32) -> BingoResult<()> {
        self.rule_salience.insert(rule_id, salience);
        debug!(
            rule_id = rule_id,
            salience = salience,
            "Updated rule salience"
        );
        Ok(())
    }
}

impl Default for ConflictResolutionManager {
    fn default() -> Self {
        Self::new(ConflictResolutionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Action, ActionType, Condition, FactData, FactValue, Operator};
    use std::collections::HashMap;

    fn create_test_rule(id: RuleId, name: &str, conditions_count: usize) -> Rule {
        let conditions = (0..conditions_count)
            .map(|i| Condition::Simple {
                field: format!("field_{i}"),
                operator: Operator::Equal,
                value: FactValue::Integer(i as i64),
            })
            .collect();

        Rule {
            id,
            name: name.to_string(),
            conditions,
            actions: vec![Action {
                action_type: ActionType::Log { message: format!("Rule {name} fired") },
            }],
        }
    }

    fn create_test_fact(id: FactId) -> Fact {
        let mut fields = HashMap::new();
        fields.insert("test_field".to_string(), FactValue::Integer(42));

        Fact {
            id,
            external_id: Some(format!("fact_{id}")),
            timestamp: chrono::Utc::now(),
            data: FactData { fields },
        }
    }

    fn create_test_rule_execution(
        rule_id: RuleId,
        name: &str,
        priority: i32,
        salience: i32,
        conditions_count: usize,
    ) -> RuleExecution {
        let rule = create_test_rule(rule_id, name, conditions_count);
        let fact = create_test_fact(1);
        RuleExecution::new(rule, vec![fact], 1, priority, salience)
    }

    #[test]
    fn test_conflict_resolution_manager_creation() {
        let config = ConflictResolutionConfig::default();
        let manager = ConflictResolutionManager::new(config);

        assert_eq!(manager.get_stats().conflict_sets_resolved, 0);
        assert_eq!(manager.get_stats().rules_ordered, 0);
    }

    #[test]
    fn test_rule_registration() {
        let mut manager = ConflictResolutionManager::default();

        let result = manager.register_rule(1, 10, 5);
        assert!(result.is_ok());

        assert_eq!(manager.get_rule_priority(1), Some(10));
        assert_eq!(manager.get_rule_salience(1), Some(5));
    }

    #[test]
    fn test_priority_based_resolution() {
        let mut manager = ConflictResolutionManager::default();

        let conflict_set = vec![
            create_test_rule_execution(1, "Low Priority", 1, 0, 1),
            create_test_rule_execution(2, "High Priority", 10, 0, 1),
            create_test_rule_execution(3, "Medium Priority", 5, 0, 1),
        ];

        let result = manager.resolve_conflicts(conflict_set).unwrap();

        // Should be ordered by priority: High (10), Medium (5), Low (1)
        assert_eq!(result[0].rule_id(), 2);
        assert_eq!(result[1].rule_id(), 3);
        assert_eq!(result[2].rule_id(), 1);
    }

    #[test]
    fn test_salience_based_resolution() {
        let config = ConflictResolutionConfig {
            primary_strategy: ConflictResolutionStrategy::Salience,
            tie_breaker: None,
            enable_logging: false,
            max_conflict_set_size: 1000,
        };
        let mut manager = ConflictResolutionManager::new(config);

        let conflict_set = vec![
            create_test_rule_execution(1, "Low Salience", 0, 1, 1),
            create_test_rule_execution(2, "High Salience", 0, 10, 1),
            create_test_rule_execution(3, "Medium Salience", 0, 5, 1),
        ];

        let result = manager.resolve_conflicts(conflict_set).unwrap();

        // Should be ordered by salience: High (10), Medium (5), Low (1)
        assert_eq!(result[0].rule_id(), 2);
        assert_eq!(result[1].rule_id(), 3);
        assert_eq!(result[2].rule_id(), 1);
    }

    #[test]
    fn test_specificity_based_resolution() {
        let config = ConflictResolutionConfig {
            primary_strategy: ConflictResolutionStrategy::Specificity,
            tie_breaker: None,
            enable_logging: false,
            max_conflict_set_size: 1000,
        };
        let mut manager = ConflictResolutionManager::new(config);

        let conflict_set = vec![
            create_test_rule_execution(1, "Simple Rule", 0, 0, 1), // 1 condition
            create_test_rule_execution(2, "Complex Rule", 0, 0, 3), // 3 conditions
            create_test_rule_execution(3, "Medium Rule", 0, 0, 2), // 2 conditions
        ];

        let result = manager.resolve_conflicts(conflict_set).unwrap();

        // Should be ordered by specificity: Complex (3), Medium (2), Simple (1)
        assert_eq!(result[0].rule_id(), 2);
        assert_eq!(result[1].rule_id(), 3);
        assert_eq!(result[2].rule_id(), 1);
    }

    #[test]
    fn test_lexicographic_resolution() {
        let config = ConflictResolutionConfig {
            primary_strategy: ConflictResolutionStrategy::Lexicographic,
            tie_breaker: None,
            enable_logging: false,
            max_conflict_set_size: 1000,
        };
        let mut manager = ConflictResolutionManager::new(config);

        let conflict_set = vec![
            create_test_rule_execution(1, "Zebra Rule", 0, 0, 1),
            create_test_rule_execution(2, "Alpha Rule", 0, 0, 1),
            create_test_rule_execution(3, "Beta Rule", 0, 0, 1),
        ];

        let result = manager.resolve_conflicts(conflict_set).unwrap();

        // Should be ordered alphabetically: Alpha, Beta, Zebra
        assert_eq!(result[0].rule_name(), "Alpha Rule");
        assert_eq!(result[1].rule_name(), "Beta Rule");
        assert_eq!(result[2].rule_name(), "Zebra Rule");
    }

    #[test]
    fn test_tie_breaker_resolution() {
        let config = ConflictResolutionConfig {
            primary_strategy: ConflictResolutionStrategy::Priority,
            tie_breaker: Some(ConflictResolutionStrategy::Lexicographic),
            enable_logging: false,
            max_conflict_set_size: 1000,
        };
        let mut manager = ConflictResolutionManager::new(config);

        let conflict_set = vec![
            create_test_rule_execution(1, "Zebra Rule", 10, 0, 1), // Same priority
            create_test_rule_execution(2, "Alpha Rule", 10, 0, 1), // Same priority
            create_test_rule_execution(3, "Low Priority", 1, 0, 1), // Different priority
        ];

        let result = manager.resolve_conflicts(conflict_set).unwrap();

        // Should be ordered by priority first, then alphabetically for ties
        // High priority rules first: Alpha (alphabetically before Zebra), then Low Priority
        assert_eq!(result[0].rule_name(), "Alpha Rule");
        assert_eq!(result[1].rule_name(), "Zebra Rule");
        assert_eq!(result[2].rule_name(), "Low Priority");
    }

    #[test]
    fn test_empty_conflict_set() {
        let mut manager = ConflictResolutionManager::default();
        let result = manager.resolve_conflicts(vec![]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_conflict_set_size_limit() {
        let config = ConflictResolutionConfig { max_conflict_set_size: 2, ..Default::default() };
        let mut manager = ConflictResolutionManager::new(config);

        let conflict_set = vec![
            create_test_rule_execution(1, "Rule 1", 1, 0, 1),
            create_test_rule_execution(2, "Rule 2", 2, 0, 1),
            create_test_rule_execution(3, "Rule 3", 3, 0, 1),
        ];

        let result = manager.resolve_conflicts(conflict_set).unwrap();
        assert_eq!(result.len(), 2); // Truncated to max size
    }

    #[test]
    fn test_statistics_tracking() {
        let mut manager = ConflictResolutionManager::default();

        let conflict_set = vec![
            create_test_rule_execution(1, "Rule 1", 1, 0, 1),
            create_test_rule_execution(2, "Rule 2", 2, 0, 1),
        ];

        manager.resolve_conflicts(conflict_set).unwrap();

        let stats = manager.get_stats();
        assert_eq!(stats.conflict_sets_resolved, 1);
        assert_eq!(stats.rules_ordered, 2);
        assert_eq!(stats.max_conflict_set_size, 2);
        assert_eq!(stats.average_conflict_set_size, 2.0);
    }

    #[test]
    fn test_rule_priority_updates() {
        let mut manager = ConflictResolutionManager::default();

        manager.set_rule_priority(1, 10).unwrap();
        assert_eq!(manager.get_rule_priority(1), Some(10));

        manager.set_rule_priority(1, 20).unwrap();
        assert_eq!(manager.get_rule_priority(1), Some(20));
    }

    #[test]
    fn test_rule_salience_updates() {
        let mut manager = ConflictResolutionManager::default();

        manager.set_rule_salience(1, 5).unwrap();
        assert_eq!(manager.get_rule_salience(1), Some(5));

        manager.set_rule_salience(1, 15).unwrap();
        assert_eq!(manager.get_rule_salience(1), Some(15));
    }
}
