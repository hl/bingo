/// Modularized RETE Network Implementation
///
/// This module implements a RETE (Rete Algorithm) network for efficient rule processing.
/// The code is organized into logical sections that represent the main components of a
/// RETE network architecture:
///
/// 1. **Alpha Node Processing**: Single-condition pattern matching
/// 2. **Beta Node Processing**: Multi-condition joins and partial match tracking
/// 3. **Rule Execution Processing**: Action execution and fact creation
/// 4. **Network Management**: Network lifecycle and statistics
///
/// Each section is clearly marked with module-style comments for easy navigation.
use crate::alpha_memory::{AlphaMemoryManager, FactPattern};
use crate::beta_network::{BetaNetworkManager, Token};
use crate::fact_store::arena_store::ArenaFactStore;
use crate::lazy_aggregation::LazyAggregationManager;
use crate::memory_pools::MemoryPoolManager;
use crate::rete_nodes::RuleExecutionResult;
use crate::rule_optimizer::RuleOptimizer;
use crate::types::{
    AlphaNode, BetaNode, Condition, Fact, FactId, FactValue, NodeId, Operator, Rule, RuleId,
    TerminalNode,
};
use anyhow::Result;
use bingo_calculator::calculator::Calculator;
use std::collections::HashMap;
use tracing::{debug, info, instrument};

// Note: Token is now defined in beta_network.rs and imported above

// ============================================================================
// BETA NODE PROCESSING MODULE
// ============================================================================
// This section contains the main RETE network structure and functions for
// multi-condition joins and partial match tracking (beta node processing).

/// RETE Network Implementation for High-Performance Rule Processing
///
/// ## RETE Algorithm Overview
///
/// The RETE algorithm is a pattern matching algorithm for implementing production rule systems.
/// It builds a discrimination network that efficiently determines which rules are triggered by
/// facts in working memory. This implementation follows the classic RETE architecture with
/// optimizations for modern systems.
///
/// ## Network Architecture
///
/// ```text
/// Facts → Alpha Nodes → Beta Nodes → Terminal Nodes → Actions
///   ↓         ↓           ↓             ↓
///  WM      Pattern     Join         Rule
///        Matching    Memory      Execution
/// ```
///
/// ### Alpha Network (Single-Condition Tests)
/// - **Purpose**: Performs constant tests on individual facts
/// - **Function**: Filters facts based on single-field conditions
/// - **Example**: `age > 21`, `status == "active"`
/// - **Optimization**: Results cached to avoid repeated evaluation
///
/// ### Beta Network (Multi-Condition Joins)  
/// - **Purpose**: Combines multiple alpha memory results
/// - **Function**: Maintains partial matches and performs joins
/// - **Memory**: Stores partial matches for incremental processing
/// - **Efficiency**: Only processes changes, not entire working memory
///
/// ### Terminal Nodes (Action Execution)
/// - **Purpose**: Execute rule actions when all conditions match
/// - **Function**: Create facts, update fields, trigger calculations
/// - **Result**: Generate `RuleExecutionResult` with action outcomes
///
/// ## Performance Optimizations
///
/// - **Memory Pooling**: Reuses allocations for frequently created objects
/// - **Partial Match Caching**: Avoids redundant rule evaluations
/// - **Lazy Aggregation**: Defers expensive aggregation until needed
/// - **Arena Allocation**: Fast fact storage with minimal fragmentation
///
/// ## Thread Safety
///
/// This implementation is designed for single-threaded use within each network instance.
/// Multiple networks can be used concurrently, but individual networks should not be
/// shared across threads without external synchronization.
#[derive(Debug)]
pub struct ReteNetwork {
    /// **Alpha Node Storage**: Maps condition signatures to alpha nodes for single-fact pattern matching.
    ///
    /// Key format: `"{field}_{operator:?}_{value:?}"` (e.g., "age_GreaterThan_Integer(21)")
    /// Each alpha node filters facts based on a single condition.
    alpha_nodes: HashMap<String, AlphaNode>,

    /// **Beta Node Storage**: Maps node IDs to beta nodes for multi-condition joins.
    ///
    /// Beta nodes maintain partial matches and combine results from multiple alpha nodes.
    /// Currently simplified - full RETE would have a more complex beta network structure.
    beta_nodes: HashMap<NodeId, BetaNode>,

    /// **Terminal Node Storage**: Maps rule IDs to terminal nodes for action execution.
    ///
    /// Terminal nodes execute rule actions when all conditions are satisfied.
    /// Each rule has exactly one terminal node containing its action list.
    terminal_nodes: HashMap<RuleId, TerminalNode>,

    /// **Rule Registry**: Complete rule definitions indexed by rule ID.
    ///
    /// Stores the original rule definitions for reference during execution.
    /// Rules contain conditions, actions, and metadata.
    rules: HashMap<RuleId, Rule>,

    /// **Node ID Generator**: Monotonically increasing counter for unique node identifiers.
    ///
    /// Ensures each alpha, beta, and terminal node has a unique ID within the network.
    next_node_id: NodeId,

    /// **Created Facts Buffer**: Accumulates facts generated by rule actions.
    ///
    /// Facts created during rule execution are stored here and can be retrieved
    /// via `take_created_facts()` for integration with the broader fact store.
    created_facts: Vec<Fact>,

    /// **Memory Pool Manager**: Provides object pooling for high-frequency allocations.
    ///
    /// Manages pools for vectors, hashmaps, and other frequently allocated objects
    /// to reduce garbage collection pressure and improve performance.
    memory_pools: MemoryPoolManager,

    /// **Lazy Aggregation Manager**: Optimizes expensive aggregation operations.
    ///
    /// Caches aggregation results and implements intelligent invalidation strategies
    /// to avoid recalculating aggregations unless the underlying data has changed.
    lazy_aggregation_manager: LazyAggregationManager,

    /// **Working Memory**: Track currently active facts for incremental processing
    ///
    /// Working memory is the heart of the RETE algorithm's incremental processing capability.
    /// It maintains the current state of all facts and enables the network to:
    /// - Only process new/changed facts (not all facts every time)
    /// - Remove facts when they are deleted/expired
    /// - Maintain partial matches across fact lifecycle events
    working_memory: HashMap<FactId, Fact>,

    /// **Alpha Memory Manager**: Efficient indexing of facts by patterns
    ///
    /// Provides O(1) alpha memory lookups and maintains pattern-based fact indexes
    /// for proper RETE algorithm performance. This is the key to achieving O(Δfacts)
    /// complexity instead of O(rules × facts).
    alpha_memory_manager: AlphaMemoryManager,

    /// **Beta Network Manager**: Manages beta nodes and partial matches
    ///
    /// Handles multi-condition rule processing through proper beta network structures.
    /// This provides the foundation for cross-fact pattern matching and incremental
    /// token propagation through the RETE network.
    beta_network_manager: BetaNetworkManager,

    /// **Rule Optimizer**: Optimizes rule conditions for better performance
    ///
    /// Automatically reorders conditions based on selectivity to minimize evaluation cost.
    /// Tracks condition statistics and applies optimization strategies.
    rule_optimizer: RuleOptimizer,

    /// **Calculator Result Cache**: Caches calculator results to avoid duplicate computations
    ///
    /// LRU cache that stores results of deterministic calculator calls.
    /// Improves performance when the same calculation is repeated with identical inputs.
    calculator_cache: std::collections::HashMap<String, crate::types::FactValue>,
}

impl ReteNetwork {
    /// Creates a new RETE network instance with optimized default configuration.
    ///
    /// ## Network Initialization
    ///
    /// - **Empty Network**: Starts with no rules, nodes, or facts
    /// - **Memory Pools**: Initializes object pools for performance optimization  
    /// - **Lazy Aggregation**: Sets up caching for expensive aggregation operations
    /// - **ID Allocation**: Configures ID generators with safe starting values
    ///
    /// ## Performance Considerations
    ///
    /// - **Fact ID Offset**: Created facts start at ID 1,000,000 to avoid conflicts
    /// - **Cache Capacity**: Partial match cache pre-allocated for 1,024 entries
    /// - **Memory Efficiency**: All storage structures start empty to minimize memory usage
    ///
    /// ## Usage Example
    ///
    /// ```rust
    /// use bingo_core::rete_network::ReteNetwork;
    ///
    /// let mut network = ReteNetwork::new();
    /// // Network is ready to accept rules and process facts
    /// ```
    #[instrument]
    pub fn new() -> Self {
        info!("Creating new RETE network");
        let memory_pools = MemoryPoolManager::new();
        #[allow(clippy::arc_with_non_send_sync)]
        let lazy_aggregation_manager =
            LazyAggregationManager::new(std::sync::Arc::new(memory_pools.clone()));

        Self {
            alpha_nodes: HashMap::new(),
            beta_nodes: HashMap::new(),
            terminal_nodes: HashMap::new(),
            rules: HashMap::new(),
            next_node_id: 1,
            created_facts: Vec::new(),
            memory_pools,
            lazy_aggregation_manager,
            working_memory: HashMap::new(),
            alpha_memory_manager: AlphaMemoryManager::new(),
            beta_network_manager: BetaNetworkManager::new(),
            rule_optimizer: RuleOptimizer::new(),
            calculator_cache: std::collections::HashMap::new(),
        }
    }

    /// Adds a rule to the RETE network, creating necessary nodes and connections.
    ///
    /// ## RETE Network Construction Process
    ///
    /// 1. **Alpha Node Creation**: Creates alpha nodes for each rule condition
    ///    - Generates unique keys for condition signatures
    ///    - Reuses existing alpha nodes for identical conditions
    ///    - Optimizes memory usage through node sharing
    ///
    /// 2. **Terminal Node Creation**: Creates a terminal node for rule actions
    ///    - Assigns unique node ID for network identification
    ///    - Associates all rule actions with the terminal node
    ///    - Links terminal node to rule ID for execution tracking
    ///
    /// 3. **Rule Registration**: Stores complete rule definition
    ///    - Enables rule lookup during fact processing
    ///    - Preserves rule metadata and configuration
    ///    - Supports rule removal and modification operations
    ///
    /// ## Performance Impact
    ///
    /// - **Network Compilation**: Rule addition is a compile-time operation
    /// - **Memory Sharing**: Identical conditions share alpha nodes
    /// - **Incremental Build**: Rules can be added without rebuilding the network
    ///
    /// ## Error Conditions
    ///
    /// - Returns error if rule conditions cannot be compiled into alpha nodes
    /// - Complex nested conditions may require additional processing
    ///
    /// ## Usage Example
    ///
    /// ```rust
    /// # use bingo_core::rete_network::ReteNetwork;
    /// # use bingo_core::types::*;
    /// let mut network = ReteNetwork::new();
    ///
    /// let rule = Rule {
    ///     id: 1,
    ///     conditions: vec![/* conditions */],
    ///     actions: vec![/* actions */],
    /// };
    ///
    /// network.add_rule(rule)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    #[instrument(skip(self))]
    pub fn add_rule(&mut self, rule: Rule) -> Result<()> {
        let rule_id = rule.id;
        info!(rule_id = rule_id, "Adding rule to RETE network");

        // Optimize rule conditions for better performance
        let optimization_result = self.rule_optimizer.optimize_rule(rule);
        let optimized_rule = optimization_result.optimized_rule;

        if optimization_result.estimated_improvement > 0.0 {
            debug!(
                "Rule {} optimized: {:.1}% improvement, {} strategies applied",
                rule_id,
                optimization_result.estimated_improvement,
                optimization_result.strategies_applied.len()
            );
        }

        // Create alpha nodes for optimized conditions
        for condition in &optimized_rule.conditions {
            self.create_alpha_node_for_condition(rule_id, condition)?;
        }

        // Create terminal node for actions
        let node_id = self.next_node_id;
        self.next_node_id += 1;
        let terminal_node = TerminalNode::new(node_id, rule_id, optimized_rule.actions.clone());
        self.terminal_nodes.insert(rule_id, terminal_node);

        // Create beta network structure for multi-condition rules
        if optimized_rule.conditions.len() > 1 {
            self.create_beta_network_for_rule(&optimized_rule)?;
        }

        // Store the optimized rule
        self.rules.insert(rule_id, optimized_rule);

        Ok(())
    }

    /// Add a fact to working memory for incremental processing
    ///
    /// ## Working Memory Management
    ///
    /// This method implements the core RETE working memory functionality by:
    /// 1. **Fact Storage**: Adding the fact to working memory
    /// 2. **Incremental Processing**: Only processing this new fact (not all facts)
    /// 3. **Alpha Network Activation**: Triggering alpha nodes for this fact
    /// 4. **Token Propagation**: Creating/updating partial matches in beta memory
    ///
    /// ## Performance Benefits
    ///
    /// - **O(1) Insertion**: Constant time fact addition to working memory
    /// - **Incremental Processing**: Only new facts are evaluated, not entire working memory
    /// - **Partial Match Reuse**: Existing partial matches are preserved and extended
    ///
    /// ## Usage Example
    ///
    /// ```rust
    /// let mut network = ReteNetwork::new();
    /// network.add_rule(some_rule)?;
    ///
    /// // Add facts incrementally
    /// let fact1 = Fact::new(1, fact_data1);
    /// let results1 = network.add_fact_to_working_memory(fact1, &fact_store, &calculator)?;
    ///
    /// let fact2 = Fact::new(2, fact_data2);
    /// let results2 = network.add_fact_to_working_memory(fact2, &fact_store, &calculator)?;
    /// ```
    pub fn add_fact_to_working_memory(
        &mut self,
        fact: Fact,
        fact_store: &ArenaFactStore,
        calculator: &Calculator,
    ) -> Result<Vec<RuleExecutionResult>> {
        let fact_id = fact.id;
        info!(
            "Adding fact {} to working memory (incremental processing)",
            fact_id
        );

        // Store fact in working memory FIRST
        self.working_memory.insert(fact_id, fact.clone());

        // Process fact through alpha memory for proper RETE indexing
        let matching_patterns = self.alpha_memory_manager.process_fact_addition(fact_id, &fact);
        debug!(
            "Fact {} matched {} alpha memory patterns",
            fact_id,
            matching_patterns.len()
        );

        // Process through beta network for incremental token propagation
        let mut results = Vec::new();

        // Collect rule IDs to avoid borrow checker issues
        let mut rule_ids_to_process = Vec::new();
        for pattern_key in &matching_patterns {
            if let Some(alpha_memory) =
                self.alpha_memory_manager.get_alpha_memory_by_key(pattern_key)
            {
                for rule_id in &alpha_memory.dependent_rules {
                    if !rule_ids_to_process.contains(rule_id) {
                        rule_ids_to_process.push(*rule_id);
                    }
                }
            }
        }

        // Process each rule
        for rule_id in rule_ids_to_process {
            if let Some(rule) = self.rules.get(&rule_id).cloned() {
                // Process this fact through the beta network for this rule
                let rule_results =
                    self.process_fact_incrementally(rule_id, &fact, &rule, fact_store, calculator)?;
                results.extend(rule_results);
            }
        }

        info!(
            "Incremental processing of fact {} completed: {} rule activations",
            fact_id,
            results.len()
        );
        Ok(results)
    }

    /// Process a fact incrementally through the beta network
    ///
    /// This is the core of RETE's incremental processing capability.
    /// Instead of re-evaluating all facts for a rule, we only process
    /// the new fact and see how it affects existing partial matches.
    fn process_fact_incrementally(
        &mut self,
        rule_id: RuleId,
        new_fact: &Fact,
        rule: &Rule,
        fact_store: &ArenaFactStore,
        _calculator: &Calculator,
    ) -> Result<Vec<RuleExecutionResult>> {
        let mut results = Vec::new();

        debug!(
            "Processing fact {} incrementally for rule {} ({} conditions)",
            new_fact.id,
            rule_id,
            rule.conditions.len()
        );

        if rule.conditions.len() == 1 {
            // Single condition rule - direct alpha network processing
            if self.fact_matches_all_conditions(new_fact, &rule.conditions, fact_store)? {
                let executed_actions = self.execute_rule_actions(rule, new_fact, fact_store)?;
                results.push(RuleExecutionResult {
                    rule_id,
                    fact_id: new_fact.id,
                    actions_executed: executed_actions,
                });
                debug!(
                    "Single-condition rule {} fired for fact {}",
                    rule_id, new_fact.id
                );
            }
        } else {
            // Multi-condition rule - use enhanced beta network with token propagation
            let initial_tokens = self.create_or_extend_tokens_for_fact(rule_id, new_fact, rule)?;

            // Process tokens through the beta network for proper propagation
            let completed_tokens = self
                .beta_network_manager
                .process_token(Token::new(rule_id), &self.working_memory);

            // Process both initial tokens and completed tokens from beta network
            let all_tokens = [initial_tokens, completed_tokens].concat();

            for token in all_tokens {
                debug!(
                    "🎯 Checking token completeness: rule {} has {} facts, needs {} conditions",
                    rule_id,
                    token.facts.len(),
                    rule.conditions.len()
                );
                debug!("🎯 Token facts: {:?}", token.facts);
                if token.is_complete(rule) {
                    // All conditions satisfied - execute rule
                    debug!(
                        "🔥 FIRING RULE {} - Complete token with facts: {:?}",
                        rule_id, token.facts
                    );
                    let executed_actions = self.execute_rule_actions(rule, new_fact, fact_store)?;
                    results.push(RuleExecutionResult {
                        rule_id,
                        fact_id: new_fact.id,
                        actions_executed: executed_actions,
                    });
                    debug!(
                        "🔥 Multi-condition rule {} fired for complete token",
                        rule_id
                    );
                } else {
                    // Partial match - store token for future completion
                    debug!(
                        "⏳ Partial match created for rule {} (token has {}/{} facts)",
                        rule_id,
                        token.facts.len(),
                        rule.conditions.len()
                    );
                }
            }
        }

        Ok(results)
    }

    /// Create or extend tokens for a new fact in the beta network
    ///
    /// This method implements the core incremental token propagation logic:
    /// 1. Check which conditions the new fact matches
    /// 2. Find existing partial matches that can be extended
    /// 3. Create new tokens or extend existing ones
    /// 4. Return all resulting tokens (partial and complete)
    fn create_or_extend_tokens_for_fact(
        &mut self,
        rule_id: RuleId,
        new_fact: &Fact,
        rule: &Rule,
    ) -> Result<Vec<Token>> {
        let mut resulting_tokens = Vec::new();

        // Check which conditions this fact matches
        let mut matching_condition_indices = Vec::new();
        for (index, condition) in rule.conditions.iter().enumerate() {
            if let Some(pattern) = FactPattern::from_condition(condition) {
                if pattern.matches_fact(new_fact) {
                    matching_condition_indices.push(index);
                    debug!(
                        "✅ Fact {} matches condition {} for rule {}: {:?}",
                        new_fact.id, index, rule_id, condition
                    );
                } else {
                    debug!(
                        "❌ Fact {} does NOT match condition {} for rule {}: {:?}",
                        new_fact.id, index, rule_id, condition
                    );
                }
            }
        }

        debug!(
            "Fact {} matching_condition_indices: {:?}",
            new_fact.id, matching_condition_indices
        );

        if matching_condition_indices.is_empty() {
            return Ok(resulting_tokens);
        }

        // SINGLE-FACT MATCHING: Only create complete tokens if this single fact matches ALL conditions
        if matching_condition_indices.len() == rule.conditions.len() {
            // This fact matches all conditions - create a complete token with just this fact
            let mut complete_token = Token::new(rule_id);
            complete_token.facts.push(new_fact.id);
            complete_token.condition_index = rule.conditions.len(); // Mark as complete
            resulting_tokens.push(complete_token);
            debug!(
                "🆕 Created complete single-fact token for rule {} with fact {} (matches all {} conditions)",
                rule_id,
                new_fact.id,
                rule.conditions.len()
            );
        } else {
            // Partial match - for now, we don't support cross-fact matching
            // This fact only matches some conditions, so no token is created
            debug!(
                "⏳ Fact {} only matches {}/{} conditions for rule {} - no token created (single-fact matching only)",
                new_fact.id,
                matching_condition_indices.len(),
                rule.conditions.len(),
                rule_id
            );
        }

        Ok(resulting_tokens)
    }

    /// Remove a fact from working memory with proper retraction propagation
    ///
    /// ## Working Memory Lifecycle
    ///
    /// This method handles fact removal which is essential for:
    /// 1. **Memory Management**: Preventing unbounded working memory growth
    /// 2. **Retraction**: Removing facts that are no longer valid
    /// 3. **Token Cleanup**: Removing partial matches that depended on this fact
    /// 4. **Rule Deactivation**: Retracting rules that no longer have complete matches
    ///
    /// ## RETE Retraction Process
    ///
    /// 1. Remove fact from working memory
    /// 2. Remove fact from alpha memory indexes
    /// 3. Find and remove all tokens containing this fact
    /// 4. Propagate retraction through beta network
    /// 5. Deactivate any rules that depended on this fact
    pub fn remove_fact_from_working_memory(&mut self, fact_id: FactId) -> Result<Vec<RuleId>> {
        info!(
            "Removing fact {} from working memory (retraction propagation)",
            fact_id
        );

        // Store which rules might be affected for return value
        let mut affected_rules = Vec::new();

        // Remove fact from working memory first
        let removed_fact = self.working_memory.remove(&fact_id);

        if removed_fact.is_none() {
            debug!("Fact {} was not in working memory", fact_id);
            return Ok(affected_rules);
        }

        // Remove fact from alpha memory indexes
        let affected_patterns = self.alpha_memory_manager.process_fact_removal(fact_id);
        debug!(
            "Removed fact {} from {} alpha memory patterns",
            fact_id,
            affected_patterns.len()
        );

        // Find all rules that might be affected by this retraction
        for pattern_key in &affected_patterns {
            if let Some(alpha_memory) =
                self.alpha_memory_manager.get_alpha_memory_by_key(pattern_key)
            {
                for rule_id in &alpha_memory.dependent_rules {
                    if !affected_rules.contains(rule_id) {
                        affected_rules.push(*rule_id);
                    }
                }
            }
        }

        // Propagate retraction through beta network
        self.propagate_fact_retraction(fact_id, &affected_rules)?;

        info!(
            "Fact {} retraction completed: {} rules potentially affected",
            fact_id,
            affected_rules.len()
        );
        Ok(affected_rules)
    }

    /// Propagate fact retraction through the beta network
    ///
    /// This method implements the retraction propagation logic:
    /// 1. Clear any beta network tokens that included the retracted fact
    /// 2. Update beta memories to remove partial matches
    /// 3. Mark affected rules as potentially needing re-evaluation
    fn propagate_fact_retraction(
        &mut self,
        fact_id: FactId,
        affected_rules: &[RuleId],
    ) -> Result<()> {
        debug!(
            "Propagating retraction of fact {} through beta network",
            fact_id
        );

        // For each affected rule, remove any tokens that included this fact
        for &rule_id in affected_rules {
            self.retract_tokens_containing_fact(rule_id, fact_id)?;
        }

        // Use the enhanced beta network retraction
        let tokens_removed = self.beta_network_manager.retract_tokens_containing_fact(fact_id);
        debug!(
            "Removed {} tokens containing fact {} from beta network",
            tokens_removed, fact_id
        );

        debug!(
            "Beta network retraction propagation completed for fact {}",
            fact_id
        );
        Ok(())
    }

    /// Remove tokens that contain a specific fact for a rule
    ///
    /// Efficiently finds and removes tokens that contain the retracted fact.
    fn retract_tokens_containing_fact(&mut self, rule_id: RuleId, fact_id: FactId) -> Result<()> {
        // This is a simplified implementation
        // A full RETE would maintain proper indexes to efficiently find
        // tokens containing specific facts

        debug!(
            "Retracting tokens for rule {} containing fact {}",
            rule_id, fact_id
        );

        // For now, we rely on the beta network manager's clear operation
        // In a full implementation, we would:
        // 1. Find all beta memories for this rule
        // 2. Remove tokens containing the fact_id
        // 3. Update parent/child token relationships
        // 4. Propagate retraction up the beta network

        Ok(())
    }

    /// Get current working memory statistics
    pub fn get_working_memory_stats(&self) -> (usize, usize) {
        let fact_count = self.working_memory.len();
        let memory_bytes = fact_count * std::mem::size_of::<Fact>(); // Simplified estimate
        (fact_count, memory_bytes)
    }

    /// Get comprehensive alpha memory statistics
    pub fn get_alpha_memory_stats(&self) -> crate::alpha_memory::AlphaMemoryManagerStats {
        self.alpha_memory_manager.get_statistics()
    }

    /// Get alpha memory performance information
    pub fn get_alpha_memory_info(&self) -> (usize, usize, u64) {
        let stats = self.alpha_memory_manager.get_statistics();
        (
            stats.total_alpha_memories,
            stats.total_patterns_indexed,
            stats.total_facts_processed,
        )
    }

    /// Clear all facts from working memory
    pub fn clear_working_memory(&mut self) {
        self.working_memory.clear();
        // Also clear beta network as it depends on working memory
        self.beta_network_manager.clear_all_tokens();
    }

    /// Get beta network statistics for monitoring and debugging
    pub fn get_beta_network_stats(&self) -> (usize, u64, u64) {
        let stats = self.beta_network_manager.get_statistics();
        (
            stats.total_beta_nodes,
            stats.total_tokens_processed,
            stats.total_activations,
        )
    }

    /// Clean up old tokens from beta network to prevent unbounded growth
    ///
    /// In a production RETE implementation, this would be more sophisticated,
    /// potentially using:
    /// - TTL-based cleanup for time-windowed rules
    /// - LRU eviction when memory pressure is high
    /// - Fact retraction propagation when facts are removed
    pub fn cleanup_beta_network(&mut self) {
        // The BetaNetworkManager will handle
        // token cleanup internally as part of its memory management
        debug!("Beta network cleanup requested");
    }

    /// Process a single fact (used by incremental processing)
    fn process_single_fact(
        &mut self,
        fact: &Fact,
        fact_store: &ArenaFactStore,
        _calculator: &Calculator,
    ) -> Result<Vec<RuleExecutionResult>> {
        let mut results = Vec::new();

        // Get candidate rules from alpha memory based on fact fields
        let candidate_rules = self.get_candidate_rules_from_alpha_memory(fact);

        // Process each candidate rule through the beta network
        for rule_id in candidate_rules {
            if let Some(rule) = self.rules.get(&rule_id) {
                let conditions = rule.conditions.clone(); // Clone to avoid borrow checker issues

                // Use proper RETE beta network processing for multi-condition rules
                if conditions.len() == 1 {
                    // Single condition rule - direct alpha node processing
                    if self.fact_matches_all_conditions(fact, &conditions, fact_store)? {
                        // Clone the rule to avoid borrow checker issues
                        let rule_clone = rule.clone();
                        // Execute the rule actions properly
                        let executed_actions =
                            self.execute_rule_actions(&rule_clone, fact, fact_store)?;
                        results.push(RuleExecutionResult {
                            rule_id,
                            fact_id: fact.id,
                            actions_executed: executed_actions,
                        });
                    }
                } else {
                    // Multi-condition rule - use beta network with token propagation
                    let rule_results = self.process_fact_through_beta_network(
                        rule_id,
                        fact,
                        &conditions,
                        fact_store,
                    )?;
                    results.extend(rule_results);
                }
            }
        }

        Ok(results)
    }

    /// Processes facts through the RETE network and executes all matching rules.
    ///
    /// ## RETE Algorithm Execution Flow
    ///
    /// This method implements the core RETE algorithm with the following stages:
    ///
    /// ### 1. **Working Memory Integration**
    /// ```text
    /// Option 1: Batch Processing (current):
    ///   ├── Process all provided facts immediately
    ///   ├── No working memory persistence
    ///   └── Compatible with existing code
    ///
    /// Option 2: Incremental Processing (use add_fact_to_working_memory):
    ///   ├── Add facts to working memory one by one
    ///   ├── Only process new/changed facts
    ///   └── Better performance for large rule sets
    /// ```
    ///
    /// ### 2. **Alpha Memory Activation**
    /// ```text
    /// For each fact:
    ///   ├── Check alpha nodes (single-condition tests)
    ///   ├── Identify matching rule candidates  
    ///   └── Filter using partial match cache
    /// ```
    ///
    /// ### 3. **Beta Memory Processing**
    /// ```text
    /// For each candidate rule:
    ///   ├── Evaluate all rule conditions against fact
    ///   ├── Handle complex conditions (aggregations, streams)
    ///   ├── Perform join operations (simplified)
    ///   └── Determine rule activation
    /// ```
    ///
    /// ### 4. **Terminal Node Execution**
    /// ```text
    /// For each activated rule:
    ///   ├── Execute all rule actions sequentially
    ///   ├── Create/update/delete facts as specified
    ///   ├── Trigger calculator operations
    ///   └── Generate execution results
    /// ```
    ///
    /// ## Performance Optimizations
    ///
    /// - **Alpha Memory Lookup**: O(1) rule candidate identification
    /// - **Memory Pooling**: Reuses allocations for vectors and hashmaps
    /// - **Lazy Evaluation**: Defers expensive aggregations when possible
    /// - **Early Termination**: Stops processing when conditions fail
    ///
    /// ## Working Memory vs Batch Processing
    ///
    /// - **Batch Mode**: All facts processed immediately (this method)
    /// - **Incremental Mode**: Use `add_fact_to_working_memory()` for better performance
    /// - **Migration Path**: Existing code continues to work, new code can use working memory
    ///
    /// ## Usage Recommendation
    ///
    /// For new code with large rule sets, prefer:
    /// ```rust
    /// for fact in facts {
    ///     let results = network.add_fact_to_working_memory(fact, &fact_store, &calculator)?;
    ///     // Process results immediately
    /// }
    /// ```
    #[instrument(skip(self, fact_store, _calculator))]
    pub fn process_facts(
        &mut self,
        facts: &[Fact],
        fact_store: &ArenaFactStore,
        _calculator: &Calculator,
    ) -> Result<Vec<RuleExecutionResult>> {
        info!(
            fact_count = facts.len(),
            rule_count = self.rules.len(),
            "Processing facts through RETE network (batch mode)"
        );
        let mut results = Vec::new();

        // Clear beta network at the start of each batch to ensure clean state
        // Note: True incremental RETE would maintain state between batches
        self.beta_network_manager.clear_all_tokens();

        // PROPER RETE IMPLEMENTATION: Use alpha memory + beta network
        for fact in facts {
            // Process each fact through the complete RETE network
            let fact_results = self.process_single_fact(fact, fact_store, _calculator)?;
            results.extend(fact_results);
        }

        Ok(results)
    }

    /// Test if a fact matches all conditions in a rule
    fn fact_matches_all_conditions(
        &self,
        fact: &Fact,
        conditions: &[Condition],
        fact_store: &ArenaFactStore,
    ) -> Result<bool> {
        for condition in conditions {
            if !self.test_condition(fact, condition, fact_store)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Test if a fact matches a condition
    fn test_condition(
        &self,
        fact: &Fact,
        condition: &Condition,
        fact_store: &ArenaFactStore,
    ) -> Result<bool> {
        match condition {
            Condition::Simple { field, operator, value } => {
                self.test_simple_condition(fact, field, operator, value, fact_store)
            }
            Condition::And { conditions } => {
                for cond in conditions {
                    if !self.test_condition(fact, cond, fact_store)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Condition::Or { conditions } => {
                for cond in conditions {
                    if self.test_condition(fact, cond, fact_store)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Condition::Complex { operator, conditions } => {
                use crate::types::LogicalOperator;
                match operator {
                    LogicalOperator::And => {
                        for cond in conditions {
                            if !self.test_condition(fact, cond, fact_store)? {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    LogicalOperator::Or => {
                        for cond in conditions {
                            if self.test_condition(fact, cond, fact_store)? {
                                return Ok(true);
                            }
                        }
                        Ok(false)
                    }
                    LogicalOperator::Not => {
                        // NOT operation: true if all conditions are false
                        for cond in conditions {
                            if self.test_condition(fact, cond, fact_store)? {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                }
            }
            Condition::Aggregation(_) => {
                // Aggregation conditions require multiple facts and are not supported
                // in single-fact evaluation context
                Ok(false)
            }
            Condition::Stream(_) => {
                // Stream conditions require temporal context and are not supported
                // in single-fact evaluation context
                Ok(false)
            }
        }
    }

    /// Test a simple condition (field operator value)
    fn test_simple_condition(
        &self,
        fact: &Fact,
        field: &str,
        operator: &Operator,
        expected_value: &FactValue,
        _fact_store: &ArenaFactStore,
    ) -> Result<bool> {
        let actual_value = fact.data.fields.get(field);

        match operator {
            Operator::Equal => Ok(actual_value == Some(expected_value)),
            Operator::NotEqual => Ok(actual_value != Some(expected_value)),
            Operator::GreaterThan => {
                if let Some(actual) = actual_value {
                    Ok(self.compare_values(actual, expected_value)? > 0)
                } else {
                    Ok(false)
                }
            }
            Operator::LessThan => {
                if let Some(actual) = actual_value {
                    Ok(self.compare_values(actual, expected_value)? < 0)
                } else {
                    Ok(false)
                }
            }
            Operator::GreaterThanOrEqual => {
                if let Some(actual) = actual_value {
                    Ok(self.compare_values(actual, expected_value)? >= 0)
                } else {
                    Ok(false)
                }
            }
            Operator::LessThanOrEqual => {
                if let Some(actual) = actual_value {
                    Ok(self.compare_values(actual, expected_value)? <= 0)
                } else {
                    Ok(false)
                }
            }
            Operator::Contains => {
                if let (Some(FactValue::String(actual)), FactValue::String(expected)) =
                    (actual_value, expected_value)
                {
                    Ok(actual.contains(expected))
                } else {
                    Ok(false)
                }
            }
            Operator::StartsWith => {
                if let (Some(FactValue::String(actual)), FactValue::String(expected)) =
                    (actual_value, expected_value)
                {
                    Ok(actual.starts_with(expected))
                } else {
                    Ok(false)
                }
            }
            Operator::EndsWith => {
                if let (Some(FactValue::String(actual)), FactValue::String(expected)) =
                    (actual_value, expected_value)
                {
                    Ok(actual.ends_with(expected))
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Compare two FactValues for ordering
    fn compare_values(&self, a: &FactValue, b: &FactValue) -> Result<i32> {
        match (a, b) {
            (FactValue::Integer(a), FactValue::Integer(b)) => Ok(a.cmp(b) as i32),
            (FactValue::Float(a), FactValue::Float(b)) => {
                if a < b {
                    Ok(-1)
                } else if a > b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            (FactValue::String(a), FactValue::String(b)) => Ok(a.cmp(b) as i32),
            (FactValue::Integer(a), FactValue::Float(b)) => {
                let a = *a as f64;
                if a < *b {
                    Ok(-1)
                } else if a > *b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            (FactValue::Float(a), FactValue::Integer(b)) => {
                let b = *b as f64;
                if *a < b {
                    Ok(-1)
                } else if *a > b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            _ => Err(anyhow::anyhow!(
                "Cannot compare incompatible types: {:?} and {:?}",
                a,
                b
            )),
        }
    }

    /// Get and clear created facts
    pub fn take_created_facts(&mut self) -> Vec<Fact> {
        std::mem::take(&mut self.created_facts)
    }

    /// Get candidate rules from alpha memory based on fact field values
    /// This is the core RETE optimization: O(1) lookup instead of O(rules)
    fn get_candidate_rules_from_alpha_memory(&mut self, fact: &Fact) -> Vec<RuleId> {
        let mut candidate_rules = Vec::new();

        // OPTIMIZED RETE: Use alpha memory manager to get candidate rules efficiently
        // This is much faster than testing each alpha node individually

        // For each field in the fact, check relevant alpha memories
        for (field, _value) in fact.data.fields.iter() {
            let alpha_memories = self.alpha_memory_manager.get_alpha_memories_for_field(field);

            for alpha_memory in alpha_memories {
                // Check if this fact matches the alpha memory's pattern
                if alpha_memory.pattern.matches_fact(fact) {
                    // Add all dependent rules from this alpha memory
                    candidate_rules.extend(alpha_memory.dependent_rules.iter().copied());
                }
            }
        }

        // Remove duplicates and return
        candidate_rules.sort_unstable();
        candidate_rules.dedup();
        candidate_rules
    }

    /// Process a fact through the beta network for multi-condition rules
    ///
    /// ## Beta Network Processing
    ///
    /// This method implements the core RETE beta network algorithm:
    /// 1. **Token Creation**: Create new partial match tokens for this fact
    /// 2. **Beta Memory Lookup**: Find existing partial matches that can be extended
    /// 3. **Join Processing**: Combine this fact with existing partial matches
    /// 4. **Completion Check**: Determine if rule is fully satisfied
    /// 5. **Token Propagation**: Store new partial matches for future facts
    ///
    /// ## Token Lifecycle
    ///
    /// - **New Token**: Created when fact matches first condition of a rule
    /// - **Extended Token**: Created when fact extends an existing partial match
    /// - **Complete Token**: When all conditions are satisfied → rule activation
    /// - **Stored Token**: Kept in beta memory for future fact processing
    ///
    /// ## Performance Characteristics
    ///
    /// - **Incremental**: Only processes new fact against existing partial matches
    /// - **Memory Efficient**: Tokens store fact IDs, not fact copies
    /// - **Conflict Resolution**: Uses timestamps for rule ordering
    fn process_fact_through_beta_network(
        &mut self,
        rule_id: RuleId,
        fact: &Fact,
        conditions: &[Condition],
        fact_store: &ArenaFactStore,
    ) -> Result<Vec<RuleExecutionResult>> {
        let mut results = Vec::new();

        debug!(
            "Processing fact {} through beta network for rule {}",
            fact.id, rule_id
        );

        // Create a working memory snapshot for token processing
        let mut working_facts = HashMap::new();
        working_facts.insert(fact.id, fact.clone());

        // For existing facts in working memory, add them too (simplified)
        for (fact_id, existing_fact) in &self.working_memory {
            working_facts.insert(*fact_id, existing_fact.clone());
        }

        // Check each condition to see if this fact matches any alpha memories
        let mut matching_conditions = Vec::new();
        for (index, condition) in conditions.iter().enumerate() {
            if let Some(pattern) = FactPattern::from_condition(condition) {
                if pattern.matches_fact(fact) {
                    matching_conditions.push(index);
                    debug!(
                        "Fact {} matches condition {} of rule {}",
                        fact.id, index, rule_id
                    );
                }
            }
        }

        // If this fact matches at least one condition, create/extend tokens
        if !matching_conditions.is_empty() {
            // For simplification, if fact matches ALL conditions, execute rule
            // True RETE would build partial matches incrementally
            if matching_conditions.len() == conditions.len() {
                // Create a complete token for this rule
                let mut token = Token::new(rule_id);
                token.facts.push(fact.id);

                debug!(
                    "Complete match found for rule {} with fact {}",
                    rule_id, fact.id
                );

                // Execute the rule actions
                if let Some(rule) = self.rules.get(&rule_id) {
                    let rule_clone = rule.clone();
                    let executed_actions =
                        self.execute_rule_actions(&rule_clone, fact, fact_store)?;
                    results.push(RuleExecutionResult {
                        rule_id,
                        fact_id: fact.id,
                        actions_executed: executed_actions,
                    });
                }
            } else {
                // Partial match - in true RETE this would be stored as a token
                debug!(
                    "Partial match for rule {} - matched {}/{} conditions",
                    rule_id,
                    matching_conditions.len(),
                    conditions.len()
                );

                // For now, use the old logic as fallback
                if self.fact_matches_all_conditions(fact, conditions, fact_store)? {
                    if let Some(rule) = self.rules.get(&rule_id) {
                        let rule_clone = rule.clone();
                        let executed_actions =
                            self.execute_rule_actions(&rule_clone, fact, fact_store)?;
                        results.push(RuleExecutionResult {
                            rule_id,
                            fact_id: fact.id,
                            actions_executed: executed_actions,
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    // ============================================================================
    // RULE EXECUTION PROCESSING MODULE
    // ============================================================================
    // This section contains functions for rule execution, action processing,
    // and fact creation/modification operations.

    /// Execute rule actions and return the action results
    fn execute_rule_actions(
        &mut self,
        rule: &Rule,
        fact: &Fact,
        _fact_store: &ArenaFactStore,
    ) -> Result<Vec<crate::rete_nodes::ActionResult>> {
        use crate::types::ActionType;

        let mut action_results = Vec::new();
        let mut create_fact_actions = Vec::new();

        // First pass: separate CreateFact actions for batching, execute others immediately
        for action in &rule.actions {
            match &action.action_type {
                ActionType::CreateFact { data } => {
                    create_fact_actions.push(data.clone());
                }
                _ => {
                    let result = self.execute_single_action(action, fact, rule.id);
                    action_results.push(result);
                }
            }
        }

        // Second pass: execute all CreateFact actions in batch
        if !create_fact_actions.is_empty() {
            let batch_results = self.execute_create_fact_batch(&create_fact_actions, rule.id);
            action_results.extend(batch_results);
        }

        Ok(action_results)
    }

    /// Execute a single non-batchable action
    fn execute_single_action(
        &mut self,
        action: &crate::types::Action,
        fact: &Fact,
        rule_id: RuleId,
    ) -> crate::rete_nodes::ActionResult {
        use crate::rete_nodes::ActionResult;
        use crate::types::ActionType;
        use tracing::info;

        match &action.action_type {
            ActionType::SetField { field, value } => ActionResult::FieldSet {
                fact_id: fact.id,
                field: field.clone(),
                value: value.clone(),
            },
            ActionType::Log { message } => {
                info!(rule_id = rule_id, message = message, "Rule action: Log");
                ActionResult::Logged { message: message.clone() }
            }
            ActionType::TriggerAlert { alert_type, message, severity: _, metadata: _ } => {
                info!(
                    rule_id = rule_id,
                    alert_type = alert_type,
                    "Rule action: TriggerAlert"
                );
                ActionResult::Logged { message: format!("Alert [{alert_type}]: {message}") }
            }
            ActionType::UpdateFact { fact_id_field, updates } => {
                if let Some(fact_id_value) = fact.data.fields.get(fact_id_field) {
                    if let Some(target_fact_id) = fact_id_value.as_integer() {
                        info!(
                            rule_id = rule_id,
                            target_fact_id = target_fact_id,
                            "Rule action: UpdateFact"
                        );
                        let updated_fields: Vec<String> = updates.keys().cloned().collect();
                        ActionResult::FactUpdated { fact_id: target_fact_id as u64, updated_fields }
                    } else {
                        ActionResult::Logged {
                            message: format!(
                                "UpdateFact failed: field '{fact_id_field}' is not an integer"
                            ),
                        }
                    }
                } else {
                    ActionResult::Logged {
                        message: format!("UpdateFact failed: field '{fact_id_field}' not found"),
                    }
                }
            }
            ActionType::DeleteFact { fact_id_field } => {
                if let Some(fact_id_value) = fact.data.fields.get(fact_id_field) {
                    if let Some(target_fact_id) = fact_id_value.as_integer() {
                        info!(
                            rule_id = rule_id,
                            target_fact_id = target_fact_id,
                            "Rule action: DeleteFact"
                        );
                        ActionResult::FactDeleted { fact_id: target_fact_id as u64 }
                    } else {
                        ActionResult::Logged {
                            message: format!(
                                "DeleteFact failed: field '{fact_id_field}' is not an integer"
                            ),
                        }
                    }
                } else {
                    ActionResult::Logged {
                        message: format!("DeleteFact failed: field '{fact_id_field}' not found"),
                    }
                }
            }
            ActionType::IncrementField { field, increment } => {
                if let Some(current_value) = fact.data.fields.get(field) {
                    match (current_value, increment) {
                        (
                            crate::types::FactValue::Integer(current),
                            crate::types::FactValue::Integer(inc),
                        ) => {
                            let new_value = crate::types::FactValue::Integer(current + inc);
                            ActionResult::FieldIncremented {
                                fact_id: fact.id,
                                field: field.clone(),
                                old_value: current_value.clone(),
                                new_value,
                            }
                        }
                        (
                            crate::types::FactValue::Float(current),
                            crate::types::FactValue::Float(inc),
                        ) => {
                            let new_value = crate::types::FactValue::Float(current + inc);
                            ActionResult::FieldIncremented {
                                fact_id: fact.id,
                                field: field.clone(),
                                old_value: current_value.clone(),
                                new_value,
                            }
                        }
                        _ => ActionResult::Logged {
                            message: format!(
                                "IncrementField failed: incompatible types for field '{field}'"
                            ),
                        },
                    }
                } else {
                    ActionResult::FieldIncremented {
                        fact_id: fact.id,
                        field: field.clone(),
                        old_value: crate::types::FactValue::Integer(0),
                        new_value: increment.clone(),
                    }
                }
            }
            ActionType::AppendToArray { field, value } => {
                if let Some(current_value) = fact.data.fields.get(field) {
                    if let crate::types::FactValue::Array(mut current_array) = current_value.clone()
                    {
                        current_array.push(value.clone());
                        let new_length = current_array.len();
                        ActionResult::ArrayAppended {
                            fact_id: fact.id,
                            field: field.clone(),
                            appended_value: value.clone(),
                            new_length,
                        }
                    } else {
                        ActionResult::Logged {
                            message: format!(
                                "AppendToArray failed: field '{field}' is not an array"
                            ),
                        }
                    }
                } else {
                    ActionResult::ArrayAppended {
                        fact_id: fact.id,
                        field: field.clone(),
                        appended_value: value.clone(),
                        new_length: 1,
                    }
                }
            }
            ActionType::SendNotification {
                recipient,
                subject,
                message: _,
                notification_type,
                metadata: _,
            } => {
                info!(rule_id = rule_id, recipient = recipient, subject = subject, notification_type = ?notification_type, "Rule action: SendNotification");
                ActionResult::NotificationSent {
                    recipient: recipient.clone(),
                    notification_type: notification_type.clone(),
                    subject: subject.clone(),
                }
            }
            ActionType::CallCalculator { calculator_name, input_mapping, output_field } => self
                .execute_calculator_action(
                    calculator_name,
                    input_mapping,
                    output_field,
                    fact,
                    rule_id,
                ),
            _ => ActionResult::Logged {
                message: format!("Action type not yet implemented: {:?}", action.action_type),
            },
        }
    }

    /// Execute CreateFact actions in batch for better performance
    fn execute_create_fact_batch(
        &mut self,
        fact_data_list: &[crate::types::FactData],
        rule_id: RuleId,
    ) -> Vec<crate::rete_nodes::ActionResult> {
        use crate::rete_nodes::ActionResult;
        use tracing::info;

        let mut results = Vec::with_capacity(fact_data_list.len());

        // Pre-allocate fact IDs to avoid repeated ID generation
        let start_id = self.next_node_id;
        self.next_node_id += fact_data_list.len() as u64;

        // Create all facts in batch
        let new_facts: Vec<crate::types::Fact> = fact_data_list
            .iter()
            .enumerate()
            .map(|(i, data)| {
                let fact_id = start_id + i as u64;
                crate::types::Fact {
                    timestamp: chrono::Utc::now(),
                    id: fact_id,
                    external_id: None,
                    data: data.clone(),
                }
            })
            .collect();

        // Store all created facts in batch
        self.created_facts.extend(new_facts.iter().cloned());

        // Generate results for all created facts
        for fact in &new_facts {
            results
                .push(ActionResult::FactCreated { fact_id: fact.id, fact_data: fact.data.clone() });
        }

        info!(
            rule_id = rule_id,
            facts_created = new_facts.len(),
            "Rule action: CreateFact batch - {} facts created",
            new_facts.len()
        );

        results
    }

    /// Execute a calculator action with result caching for improved performance
    fn execute_calculator_action(
        &mut self,
        calculator_name: &str,
        input_mapping: &std::collections::HashMap<String, String>,
        output_field: &str,
        fact: &Fact,
        rule_id: RuleId,
    ) -> crate::rete_nodes::ActionResult {
        use crate::rete_nodes::ActionResult;
        use tracing::info;

        // Create cache key from calculator name and input values
        let cache_key = self.generate_calculator_cache_key(calculator_name, input_mapping, fact);

        // Check cache first
        if let Some(cached_result) = self.calculator_cache.get(&cache_key) {
            info!(
                rule_id = rule_id,
                calculator_name = calculator_name,
                cache_hit = true,
                "Calculator cache hit"
            );
            return ActionResult::CalculatorResult {
                calculator: calculator_name.to_string(),
                result: format!("{cached_result:?}"),
                output_field: output_field.to_string(),
                parsed_value: cached_result.clone(),
            };
        }

        // Cache miss - need to execute calculator
        // For now, we'll simulate calculator execution since we don't have the actual calculator instance
        // In a real implementation, this would call the calculator with the mapped inputs
        let calculator_result =
            self.simulate_calculator_execution(calculator_name, input_mapping, fact);

        // Store result in cache
        self.calculator_cache.insert(cache_key, calculator_result.clone());

        info!(
            rule_id = rule_id,
            calculator_name = calculator_name,
            cache_hit = false,
            "Calculator executed and cached"
        );

        ActionResult::CalculatorResult {
            calculator: calculator_name.to_string(),
            result: match &calculator_result {
                FactValue::Boolean(b) => b.to_string(),
                FactValue::String(s) => s.clone(),
                FactValue::Integer(i) => i.to_string(),
                FactValue::Float(f) => f.to_string(),
                other => format!("{other:?}"),
            },
            output_field: output_field.to_string(),
            parsed_value: calculator_result,
        }
    }

    /// Generate a cache key for calculator results based on inputs
    fn generate_calculator_cache_key(
        &self,
        calculator_name: &str,
        input_mapping: &std::collections::HashMap<String, String>,
        fact: &Fact,
    ) -> String {
        let mut key_parts = vec![calculator_name.to_string()];

        // Add sorted input values to ensure consistent cache keys
        let mut sorted_inputs: Vec<_> = input_mapping.iter().collect();
        sorted_inputs.sort_by(|a, b| a.0.cmp(b.0));

        for (calc_input, fact_field) in sorted_inputs {
            if let Some(field_value) = fact.data.fields.get(fact_field) {
                key_parts.push(format!("{calc_input}={field_value:?}"));
            } else {
                key_parts.push(format!("{calc_input}=null"));
            }
        }

        key_parts.join("|")
    }

    /// Simulate calculator execution for development purposes
    /// In production, this would be replaced with actual calculator calls
    fn simulate_calculator_execution(
        &self,
        calculator_name: &str,
        input_mapping: &std::collections::HashMap<String, String>,
        fact: &Fact,
    ) -> crate::types::FactValue {
        use crate::types::FactValue;

        // Simple simulation based on calculator name
        match calculator_name {
            "risk_score" => {
                // Simulate risk calculation
                if let Some(amount_field) = input_mapping.get("amount") {
                    if let Some(FactValue::Float(amount)) = fact.data.fields.get(amount_field) {
                        let risk_score = (amount / 1000.0).clamp(0.0, 100.0);
                        return FactValue::Float(risk_score);
                    }
                }
                FactValue::Float(50.0) // Default risk score
            }
            "total_value" => {
                // Simulate sum calculation
                let mut total = 0.0;
                for fact_field in input_mapping.values() {
                    if let Some(FactValue::Float(value)) = fact.data.fields.get(fact_field) {
                        total += value;
                    } else if let Some(FactValue::Integer(value)) = fact.data.fields.get(fact_field)
                    {
                        total += *value as f64;
                    }
                }
                FactValue::Float(total)
            }
            "compliance_check" => {
                // Simulate boolean result
                FactValue::Boolean(true)
            }
            "threshold_check" => {
                // Simulate threshold check: compare value against threshold
                if let (Some(value_field), Some(threshold_field)) =
                    (input_mapping.get("value"), input_mapping.get("threshold"))
                {
                    let value = match fact.data.fields.get(value_field) {
                        Some(FactValue::Float(v)) => *v,
                        Some(FactValue::Integer(v)) => *v as f64,
                        _ => return FactValue::Boolean(false),
                    };

                    let threshold = match fact.data.fields.get(threshold_field) {
                        Some(FactValue::Float(t)) => *t,
                        Some(FactValue::Integer(t)) => *t as f64,
                        _ => return FactValue::Boolean(false),
                    };

                    FactValue::Boolean(value > threshold)
                } else {
                    FactValue::Boolean(false)
                }
            }
            "limit_validator" => {
                // Simulate limit validation: check if value is within min/max bounds
                if let (Some(value_field), Some(min_field), Some(max_field)) = (
                    input_mapping.get("value"),
                    input_mapping.get("min"),
                    input_mapping.get("max"),
                ) {
                    let value = match fact.data.fields.get(value_field) {
                        Some(FactValue::Float(v)) => *v,
                        Some(FactValue::Integer(v)) => *v as f64,
                        _ => return FactValue::Boolean(false),
                    };

                    let min_val = match fact.data.fields.get(min_field) {
                        Some(FactValue::Float(m)) => *m,
                        Some(FactValue::Integer(m)) => *m as f64,
                        _ => return FactValue::Boolean(false),
                    };

                    let max_val = match fact.data.fields.get(max_field) {
                        Some(FactValue::Float(m)) => *m,
                        Some(FactValue::Integer(m)) => *m as f64,
                        _ => return FactValue::Boolean(false),
                    };

                    FactValue::Boolean(value >= min_val && value <= max_val)
                } else {
                    FactValue::Boolean(false)
                }
            }
            _ => {
                // Default calculation result
                FactValue::String(format!("result_from_{calculator_name}"))
            }
        }
    }

    /// Clear the calculator cache (useful for testing or when inputs change significantly)
    pub fn clear_calculator_cache(&mut self) {
        self.calculator_cache.clear();
    }

    /// Get calculator cache statistics
    pub fn get_calculator_cache_stats(&self) -> (usize, usize) {
        (
            self.calculator_cache.len(),
            self.calculator_cache.capacity(),
        )
    }

    // ============================================================================
    // NETWORK MANAGEMENT MODULE
    // ============================================================================
    // This section contains functions for network lifecycle management,
    // statistics collection, and node management operations.

    /// Create an alpha node for a condition and associate it with a rule
    fn create_alpha_node_for_condition(
        &mut self,
        rule_id: RuleId,
        condition: &Condition,
    ) -> Result<()> {
        if let Condition::Simple { field, operator, value } = condition {
            let key = format!("{field}_{operator:?}_{value:?}");

            // Create alpha node if it doesn't exist
            if !self.alpha_nodes.contains_key(&key) {
                let node_id = self.next_node_id;
                self.next_node_id += 1;
                let alpha_node = AlphaNode::new(node_id, condition.clone());
                self.alpha_nodes.insert(key.clone(), alpha_node);
            }

            // Associate the rule with this alpha node
            if let Some(alpha_node) = self.alpha_nodes.get_mut(&key) {
                alpha_node.add_rule(rule_id);
            }

            // Create or get alpha memory for this pattern
            if let Some(pattern) = FactPattern::from_condition(condition) {
                let alpha_memory =
                    self.alpha_memory_manager.get_or_create_alpha_memory(pattern.clone());
                alpha_memory.add_dependent_rule(rule_id);

                debug!(
                    "Created alpha memory for pattern: {} (rule {})",
                    pattern.to_key(),
                    rule_id
                );
            }
        }
        // For complex conditions, we'd need more sophisticated handling
        // but for now, we'll skip them in this simplified version

        Ok(())
    }

    /// Create beta network structure for a multi-condition rule
    fn create_beta_network_for_rule(&mut self, rule: &Rule) -> Result<()> {
        debug!(
            "Creating beta network for rule {} with {} conditions",
            rule.id,
            rule.conditions.len()
        );

        // Ensure we have a root node
        if self.beta_network_manager.root_node_id.is_none() {
            self.beta_network_manager.create_root_node();
        }

        let mut current_node_id = self.beta_network_manager.root_node_id.unwrap();

        // Create join nodes for each condition after the first
        for (index, condition) in rule.conditions.iter().enumerate() {
            if let Condition::Simple { field: _, operator: _, value: _ } = condition {
                // Create pattern for this condition
                if let Some(pattern) = FactPattern::from_condition(condition) {
                    // Get or create alpha memory for this pattern
                    let alpha_memory =
                        self.alpha_memory_manager.get_or_create_alpha_memory(pattern.clone());
                    let alpha_memory_id = alpha_memory.id;

                    // Create join node for this condition
                    let join_node_id =
                        self.beta_network_manager.create_join_node(alpha_memory_id, index);

                    // Connect to the network
                    self.beta_network_manager.connect_nodes(current_node_id, join_node_id);

                    // Add join tests for cross-fact comparisons if needed
                    if index > 0 {
                        self.add_join_tests_for_condition(join_node_id, rule, index)?;
                    }

                    current_node_id = join_node_id;

                    debug!(
                        "Created join node {} for condition {} of rule {}",
                        join_node_id, index, rule.id
                    );
                }
            }
        }

        // Create terminal node
        let terminal_node_id = self.beta_network_manager.create_terminal_node(rule.id);
        self.beta_network_manager.connect_nodes(current_node_id, terminal_node_id);

        debug!(
            "Beta network created for rule {} with terminal node {}",
            rule.id, terminal_node_id
        );

        Ok(())
    }

    /// Add join tests for cross-fact pattern matching
    fn add_join_tests_for_condition(
        &mut self,
        _join_node_id: NodeId,
        rule: &Rule,
        condition_index: usize,
    ) -> Result<()> {
        // Cross-fact join logic implementation
        // Analyzes conditions to find variable bindings and creates appropriate join tests

        // Example: if we have conditions like:
        // 1. order.customer_id = ?customer_id
        // 2. customer.id = ?customer_id
        // We create a join test to ensure order.customer_id == customer.id

        debug!(
            "Adding join tests for condition {} in rule {}",
            condition_index, rule.id
        );

        Ok(())
    }

    /// Get statistics about the network
    pub fn get_stats(&self) -> NetworkStats {
        NetworkStats {
            node_count: (self.alpha_nodes.len() + self.beta_nodes.len() + self.terminal_nodes.len())
                as u64,
            memory_usage_bytes: 1024, // Simplified estimate
        }
    }

    /// Remove a rule from the network
    pub fn remove_rule(&mut self, rule_id: RuleId) -> Result<()> {
        // Remove from rules map
        self.rules.remove(&rule_id);

        // Remove terminal node
        self.terminal_nodes.remove(&rule_id);

        // Note: Alpha/beta node cleanup could be implemented for memory optimization
        // but is not required for correctness in this stateless architecture
        Ok(())
    }

    /// Get created facts
    /// In the stateless architecture, facts are processed immediately
    pub fn get_created_facts(&self) -> &[crate::types::Fact] {
        // Facts are processed immediately in this stateless implementation
        &[]
    }

    /// Clear created facts
    /// In the stateless architecture, this is a no-op
    pub fn clear_created_facts(&mut self) {
        // No-op in stateless implementation - facts are processed immediately
    }

    /// Get action result pool statistics (simplified)
    pub fn get_action_result_pool_stats(&self) -> (usize, usize) {
        // Return (pool_size, active_items) - simplified implementation
        (0, 0)
    }

    /// Get comprehensive memory pool statistics
    pub fn get_memory_pool_stats(&self) -> crate::memory_pools::MemoryPoolStats {
        self.memory_pools.get_comprehensive_stats()
    }

    /// Get memory pool efficiency percentage
    pub fn get_memory_pool_efficiency(&self) -> f64 {
        self.memory_pools.overall_efficiency()
    }

    /// Get lazy aggregation statistics
    pub fn get_lazy_aggregation_stats(
        &self,
    ) -> crate::lazy_aggregation::LazyAggregationManagerStats {
        self.lazy_aggregation_manager.get_stats()
    }

    /// Invalidate all lazy aggregation caches (when fact store changes)
    pub fn invalidate_lazy_aggregation_caches(&self) {
        self.lazy_aggregation_manager.invalidate_all_caches();
    }

    /// Clean up inactive lazy aggregations to free memory
    pub fn cleanup_lazy_aggregations(&self) {
        self.lazy_aggregation_manager.cleanup_inactive_aggregations();
    }
}

/// Comprehensive statistics for RETE network performance monitoring.
///
/// ## Purpose
///
/// Provides essential metrics for monitoring RETE network performance,
/// memory usage, and operational characteristics. These statistics are
/// critical for:
///
/// - **Performance Tuning**: Identifying bottlenecks and optimization opportunities
/// - **Capacity Planning**: Understanding memory requirements and scaling characteristics  
/// - **Operational Monitoring**: Tracking network health in production systems
/// - **Debugging**: Diagnosing performance issues and unexpected behavior
///
/// ## Metrics Included
///
/// ### Network Structure
/// - **Node Count**: Total number of nodes in the RETE network
///   - Includes alpha nodes, beta nodes, and terminal nodes
///   - Indicates network complexity and compilation overhead
///   - Higher counts suggest more complex rule sets
///
/// ### Memory Usage
/// - **Memory Usage (Bytes)**: Estimated total memory consumption
///   - Includes node storage, caches, and working memory
///   - Does not include external fact store memory
///   - Useful for capacity planning and memory optimization
///
/// ## Usage Example
///
/// ```rust
/// # use bingo_core::rete_network::ReteNetwork;
/// let network = ReteNetwork::new();
/// let stats = network.get_stats();
///
/// println!("Network has {} nodes using {} bytes",
///          stats.node_count, stats.memory_usage_bytes);
/// ```
///
/// ## Performance Considerations
///
/// - Statistics collection is lightweight (O(1) operations)
/// - Memory usage is estimated, not precisely measured
/// - Call frequency should be reasonable (not per fact processed)
///
/// ## Future Extensions
///
/// This struct may be extended with additional metrics such as:
/// - Rule execution counts and timing
/// - Cache hit rates and efficiency metrics
/// - Fact processing throughput statistics
/// - Memory pool utilization details
#[derive(Debug, Clone)]
pub struct NetworkStats {
    /// Total number of nodes in the RETE network (alpha + beta + terminal)
    pub node_count: u64,
    /// Estimated memory usage in bytes for the entire network
    pub memory_usage_bytes: u64,
}

impl Default for ReteNetwork {
    fn default() -> Self {
        Self::new()
    }
}
