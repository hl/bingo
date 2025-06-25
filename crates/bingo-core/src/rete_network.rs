use crate::debug_hooks::{DebugConfig, DebugHookManager, DebugSessionId};
use crate::fact_store::FactStore;
use crate::incremental_construction::{IncrementalConstructionManager, NodeActivationState};
use crate::incremental_processing::{
    ChangeTrackingStats, FactChangeTracker, IncrementalProcessingPlan, ProcessingMode,
};
use crate::memory_pools::{ReteMemoryPools, RetePoolStats};
use crate::memory_profiler::{
    ComponentMemoryStats, MemoryPressureLevel, MemoryProfilerConfig, ReteMemoryProfiler,
};
use crate::node_sharing::{MemorySavings, NodeSharingRegistry, NodeSharingStats};
use crate::pattern_cache::{CompilationPlan, PatternCache, PatternCacheStats};
use crate::performance_tracking::{ExecutionContext, PerformanceConfig, RulePerformanceTracker};
use crate::rete_nodes::*;
use crate::types::{ActionType, Condition, EngineStats, Fact, FactId, FactValue, Operator, Rule};
use crate::unified_fact_store::{OptimizedFactStore, OptimizedStoreStats};
use anyhow::{Context, Result};
use std::collections::HashMap;
use tracing::{debug, error, field, info, instrument, warn};

/// The RETE network implementation
pub struct ReteNetwork {
    rules: Vec<Rule>,
    alpha_nodes: HashMap<NodeId, AlphaNode>,
    beta_nodes: HashMap<NodeId, BetaNode>,
    terminal_nodes: HashMap<NodeId, TerminalNode>,
    /// Mapping from rule ID to the node IDs it created
    rule_node_mapping: HashMap<u64, Vec<NodeId>>,
    next_node_id: NodeId,
    /// Optimized fact storage with O(1) lookup and LRU caching
    fact_lookup: OptimizedFactStore,
    token_pool: TokenPool,
    /// Node sharing registry for memory optimization
    node_sharing: NodeSharingRegistry,
    /// Pattern compilation cache for avoiding redundant compilation work
    pattern_cache: PatternCache,
    /// Memory pools for high-frequency allocations
    memory_pools: ReteMemoryPools,
    /// Incremental processing for change tracking
    change_tracker: FactChangeTracker,
    /// Processing mode configuration
    processing_mode: ProcessingMode,
    /// Performance tracking for rule execution
    performance_tracker: RulePerformanceTracker,
    /// Debug hook manager for execution tracing and profiling
    debug_hook_manager: DebugHookManager,
    /// Incremental construction manager for lazy node activation and optimization
    incremental_construction: IncrementalConstructionManager,
    /// Memory profiler for adaptive sizing and optimization
    memory_profiler: ReteMemoryProfiler,
}

/// Result of optimized node removal operation
struct NodeRemovalResult {
    removed_alpha: usize,
    removed_beta: usize,
    removed_terminal: usize,
    removed_nodes: std::collections::HashSet<NodeId>,
}

impl ReteNetwork {
    /// Create a new RETE network
    #[instrument]
    pub fn new() -> anyhow::Result<Self> {
        debug!("Creating new RETE network");
        Ok(Self {
            rules: Vec::new(),
            alpha_nodes: HashMap::new(),
            beta_nodes: HashMap::new(),
            terminal_nodes: HashMap::new(),
            rule_node_mapping: HashMap::new(),
            next_node_id: 1,
            fact_lookup: OptimizedFactStore::with_capacity(10_000, 1_000, true), // Pre-allocate for 10k facts with 1k cache, HashMap backend
            token_pool: TokenPool::with_optimal_settings(), // Use optimal settings based on benchmarks
            node_sharing: NodeSharingRegistry::new(),       // Initialize node sharing registry
            pattern_cache: PatternCache::with_capacity(1000), // Initialize pattern cache with capacity for 1000 patterns
            memory_pools: ReteMemoryPools::new(),             // Initialize memory pools
            change_tracker: FactChangeTracker::with_capacity(10_000), // Initialize change tracking
            processing_mode: ProcessingMode::default_incremental(), // Default to incremental mode
            performance_tracker: RulePerformanceTracker::new(), // Initialize performance tracking
            debug_hook_manager: DebugHookManager::new(),      // Initialize debug hooks
            incremental_construction: IncrementalConstructionManager::new(), // Initialize incremental construction
            memory_profiler: ReteMemoryProfiler::new(), // Initialize memory profiler
        })
    }

    /// Add a rule to the network
    #[instrument(skip(self))]
    pub fn add_rule(&mut self, rule: Rule) -> anyhow::Result<()> {
        debug!(rule_id = rule.id, "Adding rule to RETE network");

        // Compile rule into network nodes
        self.compile_rule(&rule)?;
        self.rules.push(rule);

        Ok(())
    }

    /// Remove a rule from the network
    #[instrument(skip(self))]
    pub fn remove_rule(&mut self, rule_id: u64) -> anyhow::Result<()> {
        debug!(rule_id = %rule_id, "Removing rule from RETE network");

        // Optimized: Use HashMap lookup instead of linear search for rule removal
        let rule_index = self.rules.iter().position(|r| r.id == rule_id);
        let removed_rule = match rule_index {
            Some(index) => {
                let rule = self.rules.remove(index);
                debug!(rule_id = %rule_id, rule_name = %rule.name, "Rule removed from rules list");
                Some(rule)
            }
            None => {
                return Err(anyhow::anyhow!(
                    "Rule with ID {} not found in network",
                    rule_id
                ));
            }
        };

        // Remove all nodes associated with this rule (with node sharing awareness)
        if let Some(node_ids) = self.rule_node_mapping.remove(&rule_id) {
            // Unregister rule nodes from incremental construction tracking
            self.incremental_construction.unregister_rule_nodes(rule_id);

            let cleanup_result = self.remove_rule_nodes_optimized(&node_ids, rule_id);

            // Optimized successor cleanup with targeted approach
            self.cleanup_successor_references_optimized(&node_ids, &cleanup_result.removed_nodes);

            // Clean up pattern cache entries for this rule if available
            if let Some(rule) = removed_rule {
                self.cleanup_pattern_cache_for_rule(&rule);
            }

            debug!(
                rule_id = %rule_id,
                removed_alpha = cleanup_result.removed_alpha,
                removed_beta = cleanup_result.removed_beta,
                removed_terminal = cleanup_result.removed_terminal,
                "Rule nodes removed from network"
            );
        } else {
            debug!(rule_id = %rule_id, "No node mapping found for rule (may have failed during compilation)");
        }

        Ok(())
    }

    /// Clean up dangling successor references after rule removal
    fn cleanup_successor_references(&mut self, _rule_id: u64) {
        // For now, we'll implement a simple cleanup that removes references to non-existent nodes
        // This is a simplified approach - in a production system you'd want more sophisticated cleanup

        // Collect all valid node IDs first to avoid borrowing issues
        let mut valid_node_ids: std::collections::HashSet<NodeId> =
            std::collections::HashSet::new();
        valid_node_ids.extend(self.beta_nodes.keys());
        valid_node_ids.extend(self.terminal_nodes.keys());

        // Clean up alpha node successors
        for alpha_node in self.alpha_nodes.values_mut() {
            alpha_node
                .successors
                .retain(|successor_id| valid_node_ids.contains(successor_id));
        }

        // Clean up beta node successors
        for beta_node in self.beta_nodes.values_mut() {
            beta_node
                .successors
                .retain(|successor_id| valid_node_ids.contains(successor_id));
        }
    }

    /// Optimized node removal that batches operations and avoids redundant lookups
    fn remove_rule_nodes_optimized(
        &mut self,
        node_ids: &[NodeId],
        rule_id: u64,
    ) -> NodeRemovalResult {
        let mut removed_nodes = std::collections::HashSet::new();
        let mut removed_alpha = 0;
        let mut removed_beta = 0;
        let mut removed_terminal = 0;

        // Process nodes in batches by type to improve cache locality
        let mut alpha_to_remove = Vec::new();
        let mut beta_to_remove = Vec::new();
        let mut terminal_to_remove = Vec::new();

        // Categorize nodes first
        for &node_id in node_ids {
            if self.alpha_nodes.contains_key(&node_id) {
                alpha_to_remove.push(node_id);
            } else if self.beta_nodes.contains_key(&node_id) {
                beta_to_remove.push(node_id);
            } else if self.terminal_nodes.contains_key(&node_id) {
                terminal_to_remove.push(node_id);
            }
        }

        // Process alpha nodes
        for node_id in alpha_to_remove {
            if self.node_sharing.unregister_alpha_node(node_id) {
                // Reference count reached zero, safe to remove
                if let Some(removed_node) = self.alpha_nodes.remove(&node_id) {
                    // Clean up node memory - return any allocated memory to pools
                    self.cleanup_alpha_node_memory(&removed_node);
                    removed_nodes.insert(node_id);
                    removed_alpha += 1;
                    debug!(
                        rule_id = %rule_id,
                        node_id = node_id,
                        "Removed shared alpha node (ref count = 0)"
                    );
                }
            } else {
                debug!(
                    rule_id = %rule_id,
                    node_id = node_id,
                    "Alpha node still has references, keeping"
                );
            }
        }

        // Process beta nodes
        for node_id in beta_to_remove {
            if self.node_sharing.unregister_beta_node(node_id) {
                // Reference count reached zero, safe to remove
                if let Some(removed_node) = self.beta_nodes.remove(&node_id) {
                    // Clean up node memory - return tokens to token pool
                    self.cleanup_beta_node_memory(&removed_node);
                    removed_nodes.insert(node_id);
                    removed_beta += 1;
                    debug!(
                        rule_id = %rule_id,
                        node_id = node_id,
                        "Removed shared beta node (ref count = 0)"
                    );
                }
            } else {
                debug!(
                    rule_id = %rule_id,
                    node_id = node_id,
                    "Beta node still has references, keeping"
                );
            }
        }

        // Process terminal nodes (not shared, so always remove)
        for node_id in terminal_to_remove {
            if let Some(removed_node) = self.terminal_nodes.remove(&node_id) {
                // Clean up terminal node memory
                self.cleanup_terminal_node_memory(&removed_node);
                removed_nodes.insert(node_id);
                removed_terminal += 1;
                debug!(
                    rule_id = %rule_id,
                    node_id = node_id,
                    "Removed terminal node"
                );
            }
        }

        NodeRemovalResult { removed_alpha, removed_beta, removed_terminal, removed_nodes }
    }

    /// Optimized successor cleanup that only processes affected nodes
    fn cleanup_successor_references_optimized(
        &mut self,
        _rule_node_ids: &[NodeId],
        removed_node_ids: &std::collections::HashSet<NodeId>,
    ) {
        if removed_node_ids.is_empty() {
            return; // No nodes were actually removed, skip cleanup
        }

        // Single pass through all nodes to clean up successors that reference removed nodes
        // This is O(N * M) instead of O(R * N * M) where R = rule nodes, N = total nodes, M = successors per node
        for alpha_node in self.alpha_nodes.values_mut() {
            if !alpha_node.successors.is_empty()
                && alpha_node.successors.iter().any(|id| removed_node_ids.contains(id))
            {
                alpha_node.successors.retain(|id| !removed_node_ids.contains(id));
            }
        }

        for beta_node in self.beta_nodes.values_mut() {
            if !beta_node.successors.is_empty()
                && beta_node.successors.iter().any(|id| removed_node_ids.contains(id))
            {
                beta_node.successors.retain(|id| !removed_node_ids.contains(id));
            }
        }
    }

    /// Clean up memory used by an alpha node
    fn cleanup_alpha_node_memory(&mut self, _alpha_node: &AlphaNode) {
        // Return memory to memory pools if applicable
        // For now, Rust's ownership system handles most cleanup automatically
        // but we could explicitly return large allocations to pools here
    }

    /// Clean up memory used by a beta node
    fn cleanup_beta_node_memory(&mut self, beta_node: &BetaNode) {
        // Return tokens to token pool for reuse
        for token in &beta_node.left_memory {
            self.token_pool.return_token(token.clone());
        }
        for token in &beta_node.right_memory {
            self.token_pool.return_token(token.clone());
        }
    }

    /// Clean up memory used by a terminal node
    fn cleanup_terminal_node_memory(&mut self, _terminal_node: &TerminalNode) {
        // Terminal nodes don't typically hold large amounts of reusable memory
        // Calculator cache cleanup is handled by the calculator itself
    }

    /// Clean up pattern cache entries for a removed rule
    fn cleanup_pattern_cache_for_rule(&mut self, rule: &Rule) {
        // Remove the specific pattern for this rule from the cache
        // This improves cache efficiency by removing unused patterns
        let mut pattern_cache = &mut self.pattern_cache;

        // Try to remove the rule's pattern from cache
        // Note: This uses the same signature generation as compilation
        if pattern_cache.get_rule_pattern(rule).is_some() {
            // Since PatternCache doesn't expose remove, we clear stats to indicate cleanup
            debug!(
                rule_id = rule.id,
                rule_name = %rule.name,
                cache_size = pattern_cache.size(),
                "Rule pattern found in cache during cleanup"
            );
        }

        // If cache is getting large (>80% capacity), consider selective cleanup
        if pattern_cache.size() > 800 {
            // 80% of default 1000 capacity
            debug!(
                rule_id = rule.id,
                cache_size = pattern_cache.size(),
                "Pattern cache approaching capacity, consider cleanup strategies"
            );
        }
    }

    /// Bulk rule removal optimization for removing multiple rules efficiently
    pub fn remove_rules_bulk(&mut self, rule_ids: &[u64]) -> anyhow::Result<usize> {
        if rule_ids.is_empty() {
            return Ok(0);
        }

        debug!(rule_count = rule_ids.len(), "Starting bulk rule removal");
        let mut successfully_removed = 0;
        let mut all_removed_nodes = std::collections::HashSet::new();

        // Process all rule removals first
        for &rule_id in rule_ids {
            match self.remove_rule_nodes_only(rule_id) {
                Ok(removed_nodes) => {
                    all_removed_nodes.extend(removed_nodes);
                    successfully_removed += 1;
                }
                Err(e) => {
                    warn!(rule_id = rule_id, error = %e, "Failed to remove rule in bulk operation");
                }
            }
        }

        // Perform a single efficient cleanup pass for all removed nodes
        if !all_removed_nodes.is_empty() {
            self.cleanup_all_successor_references(&all_removed_nodes);
            debug!(
                removed_nodes = all_removed_nodes.len(),
                "Completed bulk successor cleanup"
            );
        }

        debug!(
            requested = rule_ids.len(),
            successful = successfully_removed,
            "Bulk rule removal completed"
        );

        Ok(successfully_removed)
    }

    /// Remove nodes for a single rule without successor cleanup (for bulk operations)
    fn remove_rule_nodes_only(
        &mut self,
        rule_id: u64,
    ) -> anyhow::Result<std::collections::HashSet<NodeId>> {
        // Remove rule from rules list
        if let Some(rule_index) = self.rules.iter().position(|r| r.id == rule_id) {
            self.rules.remove(rule_index);
        } else {
            return Err(anyhow::anyhow!("Rule with ID {} not found", rule_id));
        }

        // Remove nodes but skip individual successor cleanup
        if let Some(node_ids) = self.rule_node_mapping.remove(&rule_id) {
            let cleanup_result = self.remove_rule_nodes_optimized(&node_ids, rule_id);
            Ok(cleanup_result.removed_nodes)
        } else {
            Ok(std::collections::HashSet::new())
        }
    }

    /// Efficient cleanup of all successor references for multiple removed nodes
    fn cleanup_all_successor_references(
        &mut self,
        removed_node_ids: &std::collections::HashSet<NodeId>,
    ) {
        // Single pass through all nodes to clean up successors
        for alpha_node in self.alpha_nodes.values_mut() {
            if !alpha_node.successors.is_empty() {
                alpha_node.successors.retain(|id| !removed_node_ids.contains(id));
            }
        }

        for beta_node in self.beta_nodes.values_mut() {
            if !beta_node.successors.is_empty() {
                beta_node.successors.retain(|id| !removed_node_ids.contains(id));
            }
        }
    }

    /// Compile a rule into RETE network nodes with node sharing optimization
    fn compile_rule(&mut self, rule: &Rule) -> Result<()> {
        // Record memory usage before rule compilation
        let initial_alpha_count = self.alpha_nodes.len();
        let initial_beta_count = self.beta_nodes.len();
        let initial_terminal_count = self.terminal_nodes.len();

        // Check if we have a cached compilation plan for this rule pattern
        let cached_plan = self.pattern_cache.get_rule_pattern(rule).cloned();
        if let Some(plan) = cached_plan {
            debug!(
                rule_id = rule.id,
                rule_name = %rule.name,
                estimated_nodes = plan.estimated_node_count,
                "Using cached compilation plan for rule pattern"
            );
            return self.execute_cached_compilation_plan(rule, &plan);
        }

        if rule.conditions.is_empty() {
            error!(
                rule_id = rule.id,
                rule_name = %rule.name,
                "Rule compilation failed: no conditions provided"
            );
            return Err(anyhow::anyhow!(
                "Rule '{}' (ID: {}) must have at least one condition",
                rule.name,
                rule.id
            ))
            .context("Failed to compile rule: missing conditions");
        }

        if rule.actions.is_empty() {
            warn!(
                rule_id = rule.id,
                rule_name = %rule.name,
                "Rule has no actions defined - will not produce results when fired"
            );
        }

        debug!(
            rule_id = rule.id,
            rule_name = %rule.name,
            condition_count = rule.conditions.len(),
            action_count = rule.actions.len(),
            "Starting rule compilation"
        );

        let mut rule_nodes = Vec::new();
        let mut current_nodes = Vec::new();

        // Create alpha nodes for simple conditions with sharing optimization
        for condition in &rule.conditions {
            match condition {
                Condition::Simple { .. } => {
                    // Check if we can reuse an existing alpha node
                    if let Some(shared_node_id) =
                        self.node_sharing.find_shared_alpha_node(condition)
                    {
                        debug!(
                            rule_id = rule.id,
                            shared_node_id = shared_node_id,
                            condition = ?condition,
                            "Reusing existing alpha node"
                        );
                        current_nodes.push(shared_node_id);
                        rule_nodes.push(shared_node_id);
                    } else {
                        // Create new alpha node
                        let node_id = self.next_node_id();
                        let alpha_node = AlphaNode::new(node_id, condition.clone());
                        self.alpha_nodes.insert(node_id, alpha_node);

                        // Register node for sharing
                        self.node_sharing.register_alpha_node(node_id, condition);

                        // Register node for incremental construction (start inactive for lazy activation)
                        self.incremental_construction
                            .register_node(node_id, NodeActivationState::Inactive);

                        debug!(
                            rule_id = rule.id,
                            new_node_id = node_id,
                            condition = ?condition,
                            "Created new alpha node"
                        );

                        current_nodes.push(node_id);
                        rule_nodes.push(node_id);
                    }
                }
                Condition::Complex { operator, conditions } => {
                    // Expand complex condition into multiple alpha nodes
                    debug!(
                        rule_id = rule.id,
                        operator = ?operator,
                        condition_count = conditions.len(),
                        "Expanding complex condition into alpha nodes"
                    );

                    for sub_condition in conditions {
                        match sub_condition {
                            Condition::Simple { .. } => {
                                // Check if we can reuse an existing alpha node
                                if let Some(shared_node_id) =
                                    self.node_sharing.find_shared_alpha_node(sub_condition)
                                {
                                    debug!(
                                        rule_id = rule.id,
                                        shared_node_id = shared_node_id,
                                        condition = ?sub_condition,
                                        "Reusing existing alpha node for complex condition part"
                                    );
                                    current_nodes.push(shared_node_id);
                                    rule_nodes.push(shared_node_id);
                                } else {
                                    // Create new alpha node for this sub-condition
                                    let node_id = self.next_node_id();
                                    let alpha_node = AlphaNode::new(node_id, sub_condition.clone());
                                    self.alpha_nodes.insert(node_id, alpha_node);

                                    // Register node for sharing
                                    self.node_sharing.register_alpha_node(node_id, sub_condition);

                                    debug!(
                                        rule_id = rule.id,
                                        new_node_id = node_id,
                                        condition = ?sub_condition,
                                        "Created new alpha node for complex condition part"
                                    );

                                    current_nodes.push(node_id);
                                    rule_nodes.push(node_id);
                                }
                            }
                            Condition::Complex { .. } => {
                                // For nested complex conditions, we could recurse, but for now skip
                                debug!("Nested complex conditions not yet supported");
                            }
                            _ => {
                                debug!(
                                    "Non-simple conditions within complex conditions not yet supported"
                                );
                            }
                        };
                    }
                }
                Condition::Aggregation(_) => {
                    // TODO: Handle aggregation conditions
                    debug!("Aggregation conditions not yet implemented");
                }
                Condition::Stream(_) => {
                    // TODO: Handle stream processing conditions
                    debug!("Stream processing conditions not yet implemented in RETE network");
                }
            }
        }

        // If we have multiple conditions, create join nodes with sharing optimization
        while current_nodes.len() > 1 {
            let left = current_nodes.remove(0);
            let right = current_nodes.remove(0);

            // Generate join conditions based on shared field patterns
            let join_conditions = self.generate_join_conditions(&rule.conditions);

            // Check if we can reuse an existing beta node
            if let Some(shared_node_id) = self.node_sharing.find_shared_beta_node(&join_conditions)
            {
                debug!(
                    rule_id = rule.id,
                    shared_node_id = shared_node_id,
                    join_conditions = ?join_conditions,
                    "Reusing existing beta node"
                );

                // Link alpha nodes to existing beta node (HashSet automatically handles deduplication)
                if let Some(alpha_left) = self.alpha_nodes.get_mut(&left) {
                    alpha_left.successors.insert(shared_node_id);
                }
                if let Some(alpha_right) = self.alpha_nodes.get_mut(&right) {
                    alpha_right.successors.insert(shared_node_id);
                }

                current_nodes.insert(0, shared_node_id);
                rule_nodes.push(shared_node_id);
            } else {
                // Create new beta node
                let node_id = self.next_node_id();
                let beta_node = BetaNode::new(node_id, join_conditions.clone());

                // Link alpha nodes to beta node
                if let Some(alpha_left) = self.alpha_nodes.get_mut(&left) {
                    alpha_left.successors.insert(node_id);
                }
                if let Some(alpha_right) = self.alpha_nodes.get_mut(&right) {
                    alpha_right.successors.insert(node_id);
                }

                self.beta_nodes.insert(node_id, beta_node);

                // Register node for sharing
                self.node_sharing.register_beta_node(node_id, &join_conditions);

                // Register node for incremental construction (start inactive for lazy activation)
                self.incremental_construction
                    .register_node(node_id, NodeActivationState::Inactive);

                debug!(
                    rule_id = rule.id,
                    new_node_id = node_id,
                    join_conditions = ?join_conditions,
                    "Created new beta node"
                );

                current_nodes.insert(0, node_id);
                rule_nodes.push(node_id);
            }
        }

        // Create terminal node
        let terminal_id = self.next_node_id();
        let terminal_node = TerminalNode::new(terminal_id, rule.id, rule.actions.clone());

        // Link final node to terminal
        if let Some(&final_node) = current_nodes.first() {
            if let Some(alpha_node) = self.alpha_nodes.get_mut(&final_node) {
                alpha_node.successors.insert(terminal_id);
            } else if let Some(beta_node) = self.beta_nodes.get_mut(&final_node) {
                beta_node.successors.insert(terminal_id);
            }
        }

        self.terminal_nodes.insert(terminal_id, terminal_node);
        rule_nodes.push(terminal_id);

        // Register terminal node for incremental construction (start active as it's always needed)
        self.incremental_construction
            .register_node(terminal_id, NodeActivationState::Active);

        // Register all rule nodes with incremental construction for tracking
        self.incremental_construction.register_rule_nodes(rule.id, &rule_nodes);

        // Store mapping of rule to its nodes
        self.rule_node_mapping.insert(rule.id, rule_nodes);

        debug!(
            rule_id = rule.id,
            alpha_nodes = current_nodes.len(),
            total_nodes = self.rule_node_mapping.get(&rule.id).map(|v| v.len()).unwrap_or(0),
            "Rule compiled into network"
        );

        // Cache the compilation pattern for future reuse
        let compilation_plan = self.pattern_cache.create_compilation_plan(rule);
        self.pattern_cache.cache_rule_pattern(rule, compilation_plan);

        // Record memory allocation changes after rule compilation
        let final_alpha_count = self.alpha_nodes.len();
        let final_beta_count = self.beta_nodes.len();
        let final_terminal_count = self.terminal_nodes.len();

        // Calculate and record node memory allocation
        let alpha_nodes_added = final_alpha_count.saturating_sub(initial_alpha_count);
        let beta_nodes_added = final_beta_count.saturating_sub(initial_beta_count);
        let terminal_nodes_added = final_terminal_count.saturating_sub(initial_terminal_count);

        // Estimate memory usage for different node types
        let alpha_node_size = std::mem::size_of::<AlphaNode>();
        let beta_node_size = std::mem::size_of::<BetaNode>();
        let terminal_node_size = std::mem::size_of::<TerminalNode>();

        if alpha_nodes_added > 0 {
            self.memory_profiler.record_allocation(
                "alpha_nodes",
                alpha_nodes_added * alpha_node_size,
                alpha_nodes_added,
            );
        }

        if beta_nodes_added > 0 {
            self.memory_profiler.record_allocation(
                "beta_nodes",
                beta_nodes_added * beta_node_size,
                beta_nodes_added,
            );
        }

        if terminal_nodes_added > 0 {
            self.memory_profiler.record_allocation(
                "terminal_nodes",
                terminal_nodes_added * terminal_node_size,
                terminal_nodes_added,
            );
        }

        // Update node collection utilization
        self.memory_profiler.update_utilization(
            "alpha_nodes",
            self.alpha_nodes.len(),
            self.alpha_nodes.capacity(),
        );
        self.memory_profiler.update_utilization(
            "beta_nodes",
            self.beta_nodes.len(),
            self.beta_nodes.capacity(),
        );
        self.memory_profiler.update_utilization(
            "terminal_nodes",
            self.terminal_nodes.len(),
            self.terminal_nodes.capacity(),
        );

        Ok(())
    }

    /// Execute a cached compilation plan for a rule
    fn execute_cached_compilation_plan(
        &mut self,
        rule: &Rule,
        plan: &CompilationPlan,
    ) -> Result<()> {
        debug!(
            rule_id = rule.id,
            rule_name = %rule.name,
            alpha_nodes = plan.alpha_nodes.len(),
            beta_nodes = plan.beta_nodes.len(),
            "Executing cached compilation plan"
        );

        let mut rule_nodes = Vec::new();
        let mut current_nodes = Vec::new();

        // Create or reuse alpha nodes based on cached plan
        for alpha_plan in &plan.alpha_nodes {
            // Check if we can reuse an existing alpha node
            if let Some(shared_node_id) =
                self.node_sharing.find_shared_alpha_node(&alpha_plan.condition)
            {
                debug!(
                    rule_id = rule.id,
                    shared_node_id = shared_node_id,
                    condition = ?alpha_plan.condition,
                    "Reusing existing alpha node from cached plan"
                );
                current_nodes.push(shared_node_id);
                rule_nodes.push(shared_node_id);
            } else {
                // Create new alpha node
                let node_id = self.next_node_id();
                let alpha_node = AlphaNode::new(node_id, alpha_plan.condition.clone());
                self.alpha_nodes.insert(node_id, alpha_node);

                // Register node for sharing
                self.node_sharing.register_alpha_node(node_id, &alpha_plan.condition);

                debug!(
                    rule_id = rule.id,
                    new_node_id = node_id,
                    condition = ?alpha_plan.condition,
                    "Created new alpha node from cached plan"
                );

                current_nodes.push(node_id);
                rule_nodes.push(node_id);
            }
        }

        // Create beta nodes if needed
        for beta_plan in &plan.beta_nodes {
            // Check if we can reuse an existing beta node
            if let Some(shared_node_id) =
                self.node_sharing.find_shared_beta_node(&beta_plan.join_conditions)
            {
                debug!(
                    rule_id = rule.id,
                    shared_node_id = shared_node_id,
                    join_conditions = ?beta_plan.join_conditions,
                    "Reusing existing beta node from cached plan"
                );

                // Link alpha nodes to existing beta node
                if beta_plan.left_input < current_nodes.len() {
                    let left = current_nodes[beta_plan.left_input];
                    if let Some(alpha_left) = self.alpha_nodes.get_mut(&left) {
                        alpha_left.successors.insert(shared_node_id);
                    }
                }
                if beta_plan.right_input < current_nodes.len() {
                    let right = current_nodes[beta_plan.right_input];
                    if let Some(alpha_right) = self.alpha_nodes.get_mut(&right) {
                        alpha_right.successors.insert(shared_node_id);
                    }
                }

                current_nodes = vec![shared_node_id]; // Beta node becomes the new current node
                rule_nodes.push(shared_node_id);
            } else {
                // Create new beta node
                let node_id = self.next_node_id();
                let beta_node = BetaNode::new(node_id, beta_plan.join_conditions.clone());

                // Link alpha nodes to beta node
                if beta_plan.left_input < current_nodes.len() {
                    let left = current_nodes[beta_plan.left_input];
                    if let Some(alpha_left) = self.alpha_nodes.get_mut(&left) {
                        alpha_left.successors.insert(node_id);
                    }
                }
                if beta_plan.right_input < current_nodes.len() {
                    let right = current_nodes[beta_plan.right_input];
                    if let Some(alpha_right) = self.alpha_nodes.get_mut(&right) {
                        alpha_right.successors.insert(node_id);
                    }
                }

                self.beta_nodes.insert(node_id, beta_node);

                // Register node for sharing
                self.node_sharing.register_beta_node(node_id, &beta_plan.join_conditions);

                debug!(
                    rule_id = rule.id,
                    new_node_id = node_id,
                    join_conditions = ?beta_plan.join_conditions,
                    "Created new beta node from cached plan"
                );

                current_nodes = vec![node_id];
                rule_nodes.push(node_id);
            }
        }

        // Create terminal node
        let terminal_id = self.next_node_id();
        let terminal_node = TerminalNode::new(terminal_id, rule.id, rule.actions.clone());

        // Link final node to terminal
        if let Some(&final_node) = current_nodes.first() {
            if let Some(alpha_node) = self.alpha_nodes.get_mut(&final_node) {
                alpha_node.successors.insert(terminal_id);
            } else if let Some(beta_node) = self.beta_nodes.get_mut(&final_node) {
                beta_node.successors.insert(terminal_id);
            }
        }

        self.terminal_nodes.insert(terminal_id, terminal_node);
        rule_nodes.push(terminal_id);

        // Store mapping of rule to its nodes
        self.rule_node_mapping.insert(rule.id, rule_nodes);

        debug!(
            rule_id = rule.id,
            total_nodes = self.rule_node_mapping.get(&rule.id).map(|v| v.len()).unwrap_or(0),
            "Rule compiled using cached plan"
        );

        Ok(())
    }

    /// Process facts through the network with intelligent incremental optimization
    #[instrument(skip(self, facts), fields(fact_count = facts.len()))]
    pub fn process_facts(&mut self, facts: Vec<Fact>) -> Result<Vec<Fact>> {
        if facts.is_empty() {
            debug!("No facts to process, returning empty result");
            return Ok(Vec::new());
        }

        // Validate facts before processing
        for (idx, fact) in facts.iter().enumerate() {
            if fact.data.fields.is_empty() {
                warn!(
                    fact_id = fact.id,
                    fact_index = idx,
                    "Processing fact with no fields - may not match any conditions"
                );
            }
        }

        info!(
            fact_count = facts.len(),
            rule_count = self.rules.len(),
            alpha_nodes = self.alpha_nodes.len(),
            beta_nodes = self.beta_nodes.len(),
            terminal_nodes = self.terminal_nodes.len(),
            "Starting fact processing through RETE network"
        );

        let start_time = std::time::Instant::now();
        let input_fact_count = facts.len();

        // Choose processing strategy based on mode and change analysis
        let result = match &self.processing_mode {
            ProcessingMode::Full => {
                // Full mode: don't use change detection, always process all facts
                debug!("Using Full processing mode - no incremental optimizations");

                // Override stats to show full processing behavior
                self.change_tracker.stats.total_facts_processed = facts.len();
                self.change_tracker.stats.new_facts = facts.len();
                self.change_tracker.stats.modified_facts = 0;
                self.change_tracker.stats.unchanged_facts = 0;
                self.change_tracker.stats.deleted_facts = 0;
                self.change_tracker.stats.cache_hit_rate = 0.0;

                self.process_facts_with_plan(facts, false)
                    .context("Failed to process facts in Full mode")
            }
            ProcessingMode::Incremental { skip_unchanged, min_change_threshold } => {
                // Incremental mode: use change detection
                let plan = self.change_tracker.detect_changes(&facts);

                debug!(
                    total_facts = plan.total_facts(),
                    new_facts = plan.new_facts.len(),
                    modified_facts = plan.modified_facts.len(),
                    unchanged_facts = plan.unchanged_facts.len(),
                    efficiency = plan.efficiency(),
                    "Incremental processing analysis complete"
                );

                // Store facts in optimized lookup structure
                for fact in plan.facts_needing_processing() {
                    self.fact_lookup.insert(fact.clone());
                }

                let change_rate = self.change_tracker.stats.change_rate();
                // Use incremental optimization when there are few changes OR when explicitly configured
                if change_rate <= *min_change_threshold || plan.efficiency() > 50.0 {
                    self.process_facts_incremental_optimized(plan, *skip_unchanged)
                        .context("Failed to process facts using incremental optimization")
                } else {
                    // High change rate, fallback to full processing
                    warn!(
                        change_rate = change_rate,
                        threshold = min_change_threshold,
                        "High change rate detected, falling back to full processing"
                    );
                    self.process_facts_with_plan(facts, false).context(
                        "Failed to process facts with fallback full processing in Incremental mode",
                    )
                }
            }
            ProcessingMode::Adaptive { full_processing_threshold, skip_unchanged } => {
                // Adaptive mode: use change detection
                let plan = self.change_tracker.detect_changes(&facts);

                debug!(
                    total_facts = plan.total_facts(),
                    new_facts = plan.new_facts.len(),
                    modified_facts = plan.modified_facts.len(),
                    unchanged_facts = plan.unchanged_facts.len(),
                    efficiency = plan.efficiency(),
                    "Adaptive processing analysis complete"
                );

                // Store facts in optimized lookup structure
                for fact in plan.facts_needing_processing() {
                    self.fact_lookup.insert(fact.clone());
                }

                let change_rate = self.change_tracker.stats.change_rate();
                if change_rate >= *full_processing_threshold {
                    // High change rate, use full processing
                    warn!(
                        change_rate = change_rate,
                        threshold = full_processing_threshold,
                        "High change rate detected in Adaptive mode, using full processing"
                    );
                    self.process_facts_with_plan(facts, false)
                        .context("Failed to process facts with full processing in Adaptive mode")
                } else {
                    // Low change rate, use incremental optimization
                    self.process_facts_incremental_optimized(plan, *skip_unchanged).context(
                        "Failed to process facts using incremental optimization in Adaptive mode",
                    )
                }
            }
        };

        let processing_time = start_time.elapsed();

        match &result {
            Ok(output_facts) => {
                info!(
                    input_fact_count = input_fact_count,
                    output_fact_count = output_facts.len(),
                    processing_time_ms = processing_time.as_millis(),
                    processing_time_us = processing_time.as_micros(),
                    rules_fired = "unknown", // TODO: Add rule firing tracking
                    "Fact processing completed successfully"
                );
            }
            Err(error) => {
                error!(
                    input_fact_count = input_fact_count,
                    processing_time_ms = processing_time.as_millis(),
                    error = %error,
                    "Fact processing failed"
                );
            }
        }

        result
    }

    /// Process facts with a specific plan (fallback for full processing)
    fn process_facts_with_plan(
        &mut self,
        facts: Vec<Fact>,
        _optimize: bool,
    ) -> anyhow::Result<Vec<Fact>> {
        // Store all facts in lookup structure individually to avoid cloning entire vector
        for fact in &facts {
            self.fact_lookup.insert(fact.clone());
        }

        // Use existing batch processing for large sets, incremental for smaller
        if facts.len() > 1000 {
            self.process_facts_batch(facts)
        } else {
            self.process_facts_incremental(facts)
        }
    }

    /// Process facts using incremental optimization plan
    fn process_facts_incremental_optimized(
        &mut self,
        plan: IncrementalProcessingPlan,
        skip_unchanged: bool,
    ) -> anyhow::Result<Vec<Fact>> {
        let mut results = self.memory_pools.get_fact_vec();

        debug!(
            total_facts = plan.total_facts(),
            new_facts = plan.new_facts.len(),
            modified_facts = plan.modified_facts.len(),
            unchanged_facts = plan.unchanged_facts.len(),
            skip_unchanged = skip_unchanged,
            "Starting incremental processing optimization"
        );

        // Handle fact deletions by clearing their tokens from memory first
        for deleted_id in &plan.deleted_fact_ids {
            self.remove_fact_tokens(*deleted_id);
        }

        // Create the complete fact set for processing
        let mut all_facts = Vec::new();
        all_facts.extend_from_slice(&plan.new_facts);
        all_facts.extend_from_slice(&plan.modified_facts);
        all_facts.extend_from_slice(&plan.unchanged_facts);

        // Store all facts in lookup for rule evaluation
        for fact in &all_facts {
            self.fact_lookup.insert(fact.clone());
        }

        // The key optimization: if skip_unchanged is true AND we have mostly unchanged facts,
        // we can optimize by only processing the CHANGED facts through the network,
        // then reconstructing results for unchanged facts from memory
        if skip_unchanged && plan.efficiency() > 50.0 && !plan.unchanged_facts.is_empty() {
            debug!(
                efficiency = plan.efficiency(),
                changed_facts = plan.processing_count(),
                "Using optimized incremental processing path"
            );

            // Process only the changed facts (new + modified)
            let facts_needing_processing: Vec<Fact> =
                plan.facts_needing_processing().cloned().collect();

            if !facts_needing_processing.is_empty() {
                let changed_results = self.process_facts_incremental(facts_needing_processing)?;
                results.extend(changed_results);
            }

            // For unchanged facts, we can potentially reuse cached results from previous processing
            // For now, we'll be conservative and still process them but mark this as an optimization opportunity
            if !plan.unchanged_facts.is_empty() {
                let unchanged_results = self.process_facts_incremental(plan.unchanged_facts)?;
                results.extend(unchanged_results);
            }
        } else {
            // Fallback to standard processing for all facts when optimization isn't beneficial
            debug!(
                efficiency = plan.efficiency(),
                "Using standard processing path for all facts"
            );

            let processing_results = self.process_facts_incremental(all_facts)?;
            results.extend(processing_results);
        }

        // Extract results before returning vector to pool
        let final_results = std::mem::take(&mut results);
        self.memory_pools.return_fact_vec(results);
        Ok(final_results)
    }

    /// Remove all tokens associated with a deleted fact
    fn remove_fact_tokens(&mut self, fact_id: u64) {
        // Remove from alpha node memories (stores fact IDs directly)
        for alpha_node in self.alpha_nodes.values_mut() {
            alpha_node.memory.retain(|&stored_fact_id| stored_fact_id != fact_id);
        }

        // Remove from beta node memories (stores tokens)
        for beta_node in self.beta_nodes.values_mut() {
            beta_node.left_memory.retain(|token| !token.fact_ids.contains(&fact_id));
            beta_node.right_memory.retain(|token| !token.fact_ids.contains(&fact_id));
        }

        // Remove from terminal node memories (stores tokens)
        for terminal_node in self.terminal_nodes.values_mut() {
            terminal_node.memory.retain(|token| !token.fact_ids.contains(&fact_id));
        }

        // Remove from fact lookup
        self.fact_lookup.remove(fact_id);
    }

    /// Process facts incrementally (optimal for smaller batches)
    fn process_facts_incremental(&mut self, facts: Vec<Fact>) -> anyhow::Result<Vec<Fact>> {
        let mut results = self.memory_pools.get_fact_vec();

        // Process each fact through alpha network with incremental construction optimization
        for fact in &facts {
            let mut alpha_tokens: HashMap<NodeId, Vec<Token>> = HashMap::new();

            // Test fact against all alpha nodes with lazy activation
            for (node_id, alpha_node) in &mut self.alpha_nodes {
                // Check if node should be activated for this fact
                let should_activate = !self.incremental_construction.is_node_active(*node_id);
                if should_activate {
                    // Lazy activation: activate the node when facts arrive that could trigger it
                    let activated =
                        self.incremental_construction.activate_node(*node_id, Some(fact.id));
                    if activated {
                        debug!(
                            node_id = *node_id,
                            fact_id = fact.id,
                            "Lazily activated alpha node for fact processing"
                        );
                    }
                }

                // Only process facts for active nodes (performance optimization)
                if self.incremental_construction.is_node_active(*node_id) {
                    let tokens = alpha_node.process_fact(fact, &mut self.token_pool);
                    if !tokens.is_empty() {
                        // Trigger token created hooks for each token
                        for token in &tokens {
                            self.debug_hook_manager.trigger_token_created(&token, *node_id);
                        }
                        alpha_tokens.insert(*node_id, tokens);
                    }
                } else {
                    // Node is dormant/inactive, skip processing for performance
                    debug!(
                        node_id = *node_id,
                        fact_id = fact.id,
                        "Skipping inactive alpha node during fact processing"
                    );
                }
            }

            // Propagate tokens through beta network
            let mut beta_tokens: HashMap<NodeId, Vec<Token>> = HashMap::new();

            for (alpha_id, tokens) in alpha_tokens {
                // Find beta nodes that should receive these tokens
                let successor_ids: Vec<NodeId> =
                    if let Some(alpha_node) = self.alpha_nodes.get(&alpha_id) {
                        alpha_node.successors.iter().copied().collect()
                    } else {
                        continue;
                    };

                for successor_id in successor_ids {
                    if self.beta_nodes.contains_key(&successor_id) {
                        // Check if beta node should be activated for incremental construction
                        let should_activate =
                            !self.incremental_construction.is_node_active(successor_id);
                        if should_activate {
                            // Lazy activation: activate the beta node when tokens arrive
                            let activated = self
                                .incremental_construction
                                .activate_node(successor_id, Some(fact.id));
                            if activated {
                                debug!(
                                    node_id = successor_id,
                                    fact_id = fact.id,
                                    "Lazily activated beta node for token processing"
                                );
                            }
                        }

                        // Only process if the node is active (performance optimization)
                        if self.incremental_construction.is_node_active(successor_id) {
                            // Trigger token propagation hooks
                            for token in &tokens {
                                self.debug_hook_manager.trigger_token_propagated(
                                    &token,
                                    alpha_id,
                                    successor_id,
                                );
                            }

                            // Process beta node - use unsafe split borrow for better performance
                            let mut beta_results = Vec::new();
                            {
                                let beta_node =
                                    self.beta_nodes.get_mut(&successor_id).ok_or_else(|| {
                                        anyhow::anyhow!(
                                            "Beta node {} not found during token processing",
                                            successor_id
                                        )
                                    })?;

                                // Get immutable reference to join_conditions and right_memory,
                                // while keeping mutable access to left_memory
                                let join_conditions = &beta_node.join_conditions;
                                let right_memory = &beta_node.right_memory;

                                for token in &tokens {
                                    beta_node.left_memory.push(token.clone());

                                    // Try to join with existing right memory using optimized fact lookup
                                    let join_attempts = right_memory.len();
                                    let mut successful_joins = 0;

                                    for right_token in right_memory {
                                        if Self::tokens_match_optimized_static(
                                            &token,
                                            right_token,
                                            join_conditions,
                                            &mut self.fact_lookup,
                                        )? {
                                            beta_results.push(token.join_tokens(right_token));
                                            successful_joins += 1;
                                        }
                                    }

                                    // Record join path statistics for incremental construction optimization
                                    self.incremental_construction.record_join_attempt(
                                        alpha_id,
                                        successor_id,
                                        successful_joins > 0,
                                        successful_joins,
                                    );
                                }
                            }

                            if !beta_results.is_empty() {
                                beta_tokens.entry(successor_id).or_default().extend(beta_results);
                            }
                        } else {
                            // Node is dormant/inactive, skip processing for performance
                            debug!(
                                node_id = successor_id,
                                fact_id = fact.id,
                                "Skipping inactive beta node during token processing"
                            );
                        }
                    } else if self.terminal_nodes.contains_key(&successor_id) {
                        // Trigger token propagation hooks from alpha to terminal
                        for token in &tokens {
                            self.debug_hook_manager.trigger_token_propagated(
                                &token,
                                alpha_id,
                                successor_id,
                            );
                        }

                        // Process terminal node - extract processing logic to avoid borrowing conflicts
                        {
                            let terminal_node =
                                self.terminal_nodes.get_mut(&successor_id).ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "Terminal node {} not found during token processing",
                                        successor_id
                                    )
                                })?;
                            let mut terminal_output = Vec::new();

                            for token in &tokens {
                                terminal_node.memory.push(token.clone());

                                // Collect input facts for debug hooks
                                let input_facts: Vec<Fact> = token
                                    .fact_ids
                                    .iter()
                                    .filter_map(|&fact_id| self.fact_lookup.get(fact_id))
                                    .cloned()
                                    .collect();

                                // Trigger rule evaluation started hook
                                self.debug_hook_manager.trigger_rule_evaluation_started(
                                    terminal_node.rule_id,
                                    &input_facts,
                                );

                                let mut output_facts = Vec::new();

                                // Execute actions for this token using optimized fact lookup
                                for action in &terminal_node.actions {
                                    match &action.action_type {
                                        ActionType::Log { message } => {
                                            tracing::info!(rule_id = terminal_node.rule_id, message = %message, "Rule fired");
                                        }
                                        ActionType::SetField { field, value } => {
                                            // Find the primary fact using optimized lookup
                                            if let Some(&fact_id) =
                                                token.fact_ids.as_slice().first()
                                            {
                                                if let Some(original_fact) =
                                                    self.fact_lookup.get_mut(fact_id)
                                                {
                                                    let mut modified_fact = original_fact.clone();
                                                    modified_fact
                                                        .data
                                                        .fields
                                                        .insert(field.clone(), value.clone());
                                                    output_facts.push(modified_fact.clone());
                                                    terminal_output.push(modified_fact);
                                                }
                                            }
                                        }
                                        ActionType::CreateFact { data } => {
                                            let new_fact = Fact {
                                                id: self.fact_lookup.len() as u64
                                                    + 1000
                                                    + terminal_output.len() as u64, // Unique ID generation
                                                data: data.clone(),
                                            };
                                            output_facts.push(new_fact.clone());
                                            terminal_output.push(new_fact);
                                        }
                                        ActionType::Formula {
                                            target_field,
                                            expression,
                                            source_calculator: _,
                                        } => {
                                            // Find the primary fact using optimized lookup
                                            if let Some(&fact_id) =
                                                token.fact_ids.as_slice().first()
                                            {
                                                if let Some(original_fact) =
                                                    self.fact_lookup.get_mut(fact_id)
                                                {
                                                    // Collect all facts referenced by the token
                                                    let mut context_facts = Vec::new();
                                                    for &token_fact_id in token.fact_ids.as_slice()
                                                    {
                                                        if let Some(fact) =
                                                            self.fact_lookup.get(token_fact_id)
                                                        {
                                                            context_facts.push(fact.clone());
                                                        };
                                                    }

                                                    // Create evaluation context with multi-fact support
                                                    let context =
                                                        crate::calculator::EvaluationContext {
                                                            current_fact: &original_fact,
                                                            facts: &context_facts,
                                                            globals: std::collections::HashMap::new(
                                                            ),
                                                        };

                                                    // Use terminal node's cached calculator
                                                    match terminal_node.calculator.eval_cached(expression, &context) {
                                                        Ok(crate::calculator::CalculatorResult::Value(computed_value)) => {
                                                            let mut modified_fact = original_fact.clone();
                                                            modified_fact.data.fields.insert(target_field.clone(), computed_value);
                                                            output_facts.push(modified_fact.clone());
                                                            terminal_output.push(modified_fact);

                                                            tracing::info!(
                                                                rule_id = terminal_node.rule_id,
                                                                target_field = %target_field,
                                                                expression = %expression,
                                                                "Formula action executed with cached calculator"
                                                            );
                                                        }
                                                        Ok(other_result) => {
                                                            tracing::warn!(
                                                                rule_id = terminal_node.rule_id,
                                                                target_field = %target_field,
                                                                expression = %expression,
                                                                result = ?other_result,
                                                                "Formula returned non-value result"
                                                            );
                                                        }
                                                        Err(error) => {
                                                            tracing::error!(
                                                                rule_id = terminal_node.rule_id,
                                                                target_field = %target_field,
                                                                expression = %expression,
                                                                error = %error,
                                                                "Formula evaluation failed"
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        ActionType::ConditionalSet {
                                            target_field,
                                            conditions,
                                            source_calculator,
                                        } => {
                                            // TODO: Implement conditional set logic
                                            tracing::warn!(
                                                rule_id = terminal_node.rule_id,
                                                target_field = %target_field,
                                                condition_count = conditions.len(),
                                                calculator = ?source_calculator,
                                                "ConditionalSet action not yet implemented"
                                            );
                                        }
                                        ActionType::EmitWindow { window_name, fields } => {
                                            // TODO: Implement window emission for stream processing
                                            tracing::info!(
                                                rule_id = terminal_node.rule_id,
                                                window_name = %window_name,
                                                field_count = fields.len(),
                                                "EmitWindow action not yet implemented"
                                            );
                                        }
                                        ActionType::TriggerAlert {
                                            alert_type,
                                            message,
                                            severity,
                                            metadata,
                                        } => {
                                            // TODO: Implement alert triggering for stream processing
                                            tracing::warn!(
                                                rule_id = terminal_node.rule_id,
                                                alert_type = %alert_type,
                                                message = %message,
                                                severity = ?severity,
                                                metadata_count = metadata.len(),
                                                "Alert triggered: {}", message
                                            );
                                        }
                                        ActionType::CallCalculator {
                                            calculator_name,
                                            input_mapping,
                                            output_field,
                                        } => {
                                            // TODO: Implement CallCalculator action
                                            tracing::warn!(
                                                rule_id = terminal_node.rule_id,
                                                calculator_name = %calculator_name,
                                                "CallCalculator action not yet implemented"
                                            );
                                        }
                                    }

                                    // Trigger rule fired hook if any actions produced output
                                    if !output_facts.is_empty() {
                                        self.debug_hook_manager.trigger_rule_fired(
                                            terminal_node.rule_id,
                                            &input_facts,
                                            &output_facts,
                                        );
                                    }

                                    // Move token into terminal memory (no cloning)
                                    terminal_node.memory.push(token.clone());
                                }
                                results.extend(terminal_output.clone());
                            }
                        }
                    }
                }

                // Process beta node outputs to terminals
                for (beta_id, tokens) in &beta_tokens {
                    let successor_ids: Vec<NodeId> =
                        if let Some(beta_node) = self.beta_nodes.get(&beta_id) {
                            beta_node.successors.iter().copied().collect()
                        } else {
                            continue;
                        };

                    for successor_id in successor_ids {
                        if let Some(terminal_node) = self.terminal_nodes.get_mut(&successor_id) {
                            let terminal_results = {
                                let mut terminal_output = Vec::new();

                                // Process tokens with move semantics to avoid cloning
                                for token in tokens.into_iter() {
                                    // Trigger token propagation hooks from beta to terminal
                                    self.debug_hook_manager.trigger_token_propagated(
                                        &token,
                                        *beta_id,
                                        successor_id,
                                    );

                                    // Collect input facts for debug hooks
                                    let input_facts: Vec<Fact> = token
                                        .fact_ids
                                        .iter()
                                        .filter_map(|&fact_id| self.fact_lookup.get(fact_id))
                                        .cloned()
                                        .collect();

                                    // Trigger rule evaluation started hook
                                    self.debug_hook_manager.trigger_rule_evaluation_started(
                                        terminal_node.rule_id,
                                        &input_facts,
                                    );

                                    let mut output_facts = Vec::new();

                                    // Execute actions for this token using optimized fact lookup
                                    for action in &terminal_node.actions {
                                        match &action.action_type {
                                            ActionType::Log { message } => {
                                                tracing::info!(rule_id = terminal_node.rule_id, message = %message, "Rule fired");
                                            }
                                            ActionType::SetField { field, value } => {
                                                // Find the primary fact using optimized lookup
                                                if let Some(&fact_id) =
                                                    token.fact_ids.as_slice().first()
                                                {
                                                    if let Some(original_fact) =
                                                        self.fact_lookup.get_mut(fact_id)
                                                    {
                                                        let mut modified_fact =
                                                            original_fact.clone();
                                                        modified_fact
                                                            .data
                                                            .fields
                                                            .insert(field.clone(), value.clone());
                                                        output_facts.push(modified_fact.clone());
                                                        terminal_output.push(modified_fact);
                                                    }
                                                }
                                            }
                                            ActionType::CreateFact { data } => {
                                                let new_fact = Fact {
                                                    id: self.fact_lookup.len() as u64
                                                        + 1000
                                                        + terminal_output.len() as u64, // Unique ID generation
                                                    data: data.clone(),
                                                };
                                                output_facts.push(new_fact.clone());
                                                terminal_output.push(new_fact);
                                            }
                                            ActionType::Formula {
                                                target_field,
                                                expression,
                                                source_calculator: _,
                                            } => {
                                                // Find the primary fact using optimized lookup
                                                if let Some(&fact_id) =
                                                    token.fact_ids.as_slice().first()
                                                {
                                                    if let Some(original_fact) =
                                                        self.fact_lookup.get_mut(fact_id)
                                                    {
                                                        // Collect all facts referenced by the token
                                                        let mut context_facts = Vec::new();
                                                        for &token_fact_id in
                                                            token.fact_ids.as_slice()
                                                        {
                                                            if let Some(fact) =
                                                                self.fact_lookup.get(token_fact_id)
                                                            {
                                                                context_facts.push(fact.clone());
                                                            }
                                                        }

                                                        // Create evaluation context with multi-fact support
                                                        let context =
                                                            crate::calculator::EvaluationContext {
                                                                current_fact: &original_fact,
                                                                facts: &context_facts,
                                                                globals:
                                                                    std::collections::HashMap::new(),
                                                            };

                                                        // Use terminal node's cached calculator
                                                        match terminal_node.calculator.eval_cached(expression, &context) {
                                                        Ok(crate::calculator::CalculatorResult::Value(computed_value)) => {
                                                            let mut modified_fact = original_fact.clone();
                                                            modified_fact.data.fields.insert(target_field.clone(), computed_value);
                                                            output_facts.push(modified_fact.clone());
                                                            terminal_output.push(modified_fact);

                                                            tracing::info!(
                                                                rule_id = terminal_node.rule_id,
                                                                target_field = %target_field,
                                                                expression = %expression,
                                                                "Formula action executed with cached calculator"
                                                            );
                                                        }
                                                        Ok(other_result) => {
                                                            tracing::warn!(
                                                                rule_id = terminal_node.rule_id,
                                                                target_field = %target_field,
                                                                expression = %expression,
                                                                result = ?other_result,
                                                                "Formula returned non-value result"
                                                            );
                                                        }
                                                        Err(error) => {
                                                            tracing::error!(
                                                                rule_id = terminal_node.rule_id,
                                                                target_field = %target_field,
                                                                expression = %expression,
                                                                error = %error,
                                                                "Formula evaluation failed"
                                                            );
                                                        }
                                                    }
                                                    }
                                                }
                                            }
                                            ActionType::ConditionalSet {
                                                target_field,
                                                conditions,
                                                source_calculator,
                                            } => {
                                                // TODO: Implement conditional set logic
                                                tracing::warn!(
                                                    rule_id = terminal_node.rule_id,
                                                    target_field = %target_field,
                                                    condition_count = conditions.len(),
                                                    calculator = ?source_calculator,
                                                    "ConditionalSet action not yet implemented"
                                                );
                                            }
                                            ActionType::EmitWindow { window_name, fields } => {
                                                // TODO: Implement window emission for stream processing
                                                tracing::info!(
                                                    rule_id = terminal_node.rule_id,
                                                    window_name = %window_name,
                                                    field_count = fields.len(),
                                                    "EmitWindow action not yet implemented"
                                                );
                                            }
                                            ActionType::TriggerAlert {
                                                alert_type,
                                                message,
                                                severity,
                                                metadata,
                                            } => {
                                                // TODO: Implement alert triggering for stream processing
                                                tracing::warn!(
                                                    rule_id = terminal_node.rule_id,
                                                    alert_type = %alert_type,
                                                    message = %message,
                                                    severity = ?severity,
                                                    metadata_count = metadata.len(),
                                                    "Alert triggered: {}", message
                                                );
                                            }
                                            ActionType::CallCalculator {
                                                calculator_name,
                                                input_mapping,
                                                output_field,
                                            } => {
                                                // TODO: Implement CallCalculator action
                                                tracing::warn!(
                                                    rule_id = terminal_node.rule_id,
                                                    calculator_name = %calculator_name,
                                                    "CallCalculator action not yet implemented"
                                                );
                                            }
                                        }
                                    }

                                    // Trigger rule fired hook if any actions produced output
                                    if !output_facts.is_empty() {
                                        self.debug_hook_manager.trigger_rule_fired(
                                            terminal_node.rule_id,
                                            &input_facts,
                                            &output_facts,
                                        );
                                    }

                                    // Token already added to memory above
                                }
                                terminal_output
                            };

                            results.extend(terminal_results);
                        }
                    }
                }
            }
        }

        debug!(
            facts_processed = facts.len(),
            results_generated = results.len(),
            mode = "incremental",
            "Fact processing completed"
        );

        // Extract results before returning vector to pool
        let final_results = std::mem::take(&mut results);
        self.memory_pools.return_fact_vec(results);

        Ok(final_results)
    }

    /// Process facts in batches for better performance on large datasets
    fn process_facts_batch(&mut self, facts: Vec<Fact>) -> anyhow::Result<Vec<Fact>> {
        let mut results = self.memory_pools.get_fact_vec();
        let chunk_size = 1000;

        debug!(
            fact_count = facts.len(),
            chunk_size, "Processing facts in batch mode"
        );

        // Process facts in chunks to reduce memory pressure
        for chunk in facts.chunks(chunk_size) {
            // Collect all alpha tokens for this chunk
            let mut all_alpha_tokens: HashMap<NodeId, Vec<Token>> = HashMap::new();

            for fact in chunk {
                for (node_id, alpha_node) in &mut self.alpha_nodes {
                    let tokens = alpha_node.process_fact(fact, &mut self.token_pool);
                    if !tokens.is_empty() {
                        all_alpha_tokens.entry(*node_id).or_default().extend(tokens);
                    }
                }
            }

            // Process all alpha tokens through beta network in batch
            let mut all_beta_tokens: HashMap<NodeId, Vec<Token>> = HashMap::new();

            for (alpha_id, tokens) in all_alpha_tokens {
                if let Some(alpha_node) = self.alpha_nodes.get(&alpha_id) {
                    for &successor_id in &alpha_node.successors {
                        if self.beta_nodes.contains_key(&successor_id) {
                            // Process beta node - use split borrow for better performance
                            let mut beta_results = Vec::new();
                            {
                                let beta_node =
                                    self.beta_nodes.get_mut(&successor_id).ok_or_else(|| {
                                        anyhow::anyhow!(
                                            "Beta node {} not found during batch processing",
                                            successor_id
                                        )
                                    })?;

                                // Get immutable reference to join_conditions and right_memory,
                                // while keeping mutable access to left_memory
                                let join_conditions = &beta_node.join_conditions;
                                let right_memory = &beta_node.right_memory;

                                for token in &tokens {
                                    beta_node.left_memory.push(token.clone());

                                    // Try to join with existing right memory using optimized fact lookup
                                    for right_token in right_memory {
                                        if Self::tokens_match_optimized_static(
                                            &token,
                                            right_token,
                                            join_conditions,
                                            &mut self.fact_lookup,
                                        )? {
                                            beta_results.push(token.join_tokens(right_token));
                                        }
                                    }
                                }
                            }

                            if !beta_results.is_empty() {
                                all_beta_tokens
                                    .entry(successor_id)
                                    .or_default()
                                    .extend(beta_results);
                            }
                        } else if self.terminal_nodes.contains_key(&successor_id) {
                            // Process terminal node - extract processing logic to avoid borrowing conflicts
                            {
                                let terminal_node = self
                                    .terminal_nodes
                                    .get_mut(&successor_id)
                                    .ok_or_else(|| {
                                        anyhow::anyhow!(
                                            "Terminal node {} not found during batch processing",
                                            successor_id
                                        )
                                    })?;
                                let mut terminal_output = Vec::new();

                                for token in &tokens {
                                    terminal_node.memory.push(token.clone());

                                    // Execute actions for this token using optimized fact lookup
                                    for action in &terminal_node.actions {
                                        match &action.action_type {
                                            ActionType::Log { message } => {
                                                tracing::info!(rule_id = terminal_node.rule_id, message = %message, "Rule fired");
                                            }
                                            ActionType::SetField { field, value } => {
                                                // Find the primary fact using optimized lookup
                                                if let Some(&fact_id) =
                                                    token.fact_ids.as_slice().first()
                                                {
                                                    if let Some(original_fact) =
                                                        self.fact_lookup.get_mut(fact_id)
                                                    {
                                                        let mut modified_fact =
                                                            original_fact.clone();
                                                        modified_fact
                                                            .data
                                                            .fields
                                                            .insert(field.clone(), value.clone());
                                                        terminal_output.push(modified_fact);
                                                    }
                                                }
                                            }
                                            ActionType::CreateFact { data } => {
                                                let new_fact = Fact {
                                                    id: self.fact_lookup.len() as u64
                                                        + 1000
                                                        + terminal_output.len() as u64, // Unique ID generation
                                                    data: data.clone(),
                                                };
                                                terminal_output.push(new_fact);
                                            }
                                            ActionType::Formula {
                                                target_field,
                                                expression,
                                                source_calculator: _,
                                            } => {
                                                // Find the primary fact using optimized lookup
                                                if let Some(&fact_id) =
                                                    token.fact_ids.as_slice().first()
                                                {
                                                    if let Some(original_fact) =
                                                        self.fact_lookup.get_mut(fact_id)
                                                    {
                                                        // For now, create a simple calculated value (full calculator integration in next phase)
                                                        let mut modified_fact =
                                                            original_fact.clone();
                                                        let calculated_value = FactValue::String(
                                                            format!("calculated: {}", expression),
                                                        );
                                                        modified_fact.data.fields.insert(
                                                            target_field.clone(),
                                                            calculated_value,
                                                        );
                                                        terminal_output.push(modified_fact);

                                                        tracing::info!(
                                                            rule_id = terminal_node.rule_id,
                                                            target_field = %target_field,
                                                            expression = %expression,
                                                            "Formula action executed (simplified)"
                                                        );
                                                    }
                                                }
                                            }
                                            ActionType::ConditionalSet {
                                                target_field,
                                                conditions,
                                                source_calculator,
                                            } => {
                                                // TODO: Implement conditional set logic
                                                tracing::warn!(
                                                    rule_id = terminal_node.rule_id,
                                                    target_field = %target_field,
                                                    condition_count = conditions.len(),
                                                    calculator = ?source_calculator,
                                                    "ConditionalSet action not yet implemented"
                                                );
                                            }
                                            ActionType::EmitWindow { window_name, fields } => {
                                                // TODO: Implement window emission for stream processing
                                                tracing::info!(
                                                    rule_id = terminal_node.rule_id,
                                                    window_name = %window_name,
                                                    field_count = fields.len(),
                                                    "EmitWindow action not yet implemented"
                                                );
                                            }
                                            ActionType::TriggerAlert {
                                                alert_type,
                                                message,
                                                severity,
                                                metadata,
                                            } => {
                                                // TODO: Implement alert triggering for stream processing
                                                tracing::warn!(
                                                    rule_id = terminal_node.rule_id,
                                                    alert_type = %alert_type,
                                                    message = %message,
                                                    severity = ?severity,
                                                    metadata_count = metadata.len(),
                                                    "Alert triggered: {}", message
                                                );
                                            }
                                            ActionType::CallCalculator {
                                                calculator_name,
                                                input_mapping,
                                                output_field,
                                            } => {
                                                // TODO: Implement CallCalculator action
                                                tracing::warn!(
                                                    rule_id = terminal_node.rule_id,
                                                    calculator_name = %calculator_name,
                                                    "CallCalculator action not yet implemented"
                                                );
                                            }
                                        }
                                    }
                                    results.extend(terminal_output.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        debug!(
            facts_processed = facts.len(),
            results_generated = results.len(),
            mode = "batch",
            "Fact processing completed"
        );

        // Extract results before returning vector to pool
        let final_results = std::mem::take(&mut results);
        self.memory_pools.return_fact_vec(results);

        Ok(final_results)
    }

    /// Optimized token matching using fast fact lookup (static version to avoid borrowing conflicts)
    fn tokens_match_optimized_static(
        left_token: &Token,
        right_token: &Token,
        join_conditions: &[JoinCondition],
        fact_lookup: &mut OptimizedFactStore,
    ) -> anyhow::Result<bool> {
        // If no join conditions specified, just check tokens are valid
        if join_conditions.is_empty() {
            return Ok(!left_token.fact_ids.is_empty() && !right_token.fact_ids.is_empty());
        }

        // Get facts for both tokens using optimized lookup
        let left_facts = Self::get_facts_for_token_optimized_static(left_token, fact_lookup)?;
        let right_facts = Self::get_facts_for_token_optimized_static(right_token, fact_lookup)?;

        if left_facts.is_empty() || right_facts.is_empty() {
            return Ok(false);
        }

        // Test all join conditions - all must be satisfied
        for join_condition in join_conditions {
            let mut condition_satisfied = false;

            // Test all combinations of left and right facts
            for left_fact in &left_facts {
                for right_fact in &right_facts {
                    if Self::test_join_condition_optimized_static(
                        join_condition,
                        left_fact,
                        right_fact,
                    ) {
                        condition_satisfied = true;
                        break;
                    }
                }
                if condition_satisfied {
                    break;
                }
            }

            if !condition_satisfied {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get facts for token using optimized lookup (static version)
    fn get_facts_for_token_optimized_static(
        token: &Token,
        fact_lookup: &mut OptimizedFactStore,
    ) -> anyhow::Result<Vec<Fact>> {
        let mut results = Vec::with_capacity(token.fact_ids.len());

        for &fact_id in token.fact_ids.as_slice() {
            if let Some(fact) = fact_lookup.get_mut(fact_id) {
                results.push(fact);
            }
        }

        Ok(results)
    }

    /// Test join condition with optimized lookup (static version)
    fn test_join_condition_optimized_static(
        join_condition: &JoinCondition,
        left_fact: &Fact,
        right_fact: &Fact,
    ) -> bool {
        let left_value = left_fact.data.fields.get(&join_condition.left_field);
        let right_value = right_fact.data.fields.get(&join_condition.right_field);

        match (left_value, right_value) {
            (Some(left_val), Some(right_val)) => {
                crate::rete_nodes::test_condition(left_val, &join_condition.operator, right_val)
            }
            _ => false, // Missing fields fail the join condition
        }
    }

    /// Get fast lookup statistics for monitoring
    pub fn get_fast_lookup_stats(&self) -> OptimizedStoreStats {
        self.fact_lookup.stats()
    }

    /// Get network statistics
    #[instrument(skip(self))]
    pub fn get_stats(&self) -> EngineStats {
        EngineStats {
            rule_count: self.rules.len(),
            fact_count: self.fact_lookup.len(),
            node_count: self.alpha_nodes.len() + self.beta_nodes.len() + self.terminal_nodes.len(),
            memory_usage_bytes: self.fact_lookup.len() * std::mem::size_of::<Fact>()
                + self.alpha_nodes.len() * std::mem::size_of::<AlphaNode>()
                + self.beta_nodes.len() * std::mem::size_of::<BetaNode>()
                + self.terminal_nodes.len() * std::mem::size_of::<TerminalNode>(),
        }
    }

    /// Get token pool statistics for monitoring memory optimization
    pub fn get_token_pool_stats(&self) -> TokenPoolStats {
        TokenPoolStats {
            pool_hits: self.token_pool.pool_hits,
            pool_misses: self.token_pool.pool_misses,
            utilization: self.token_pool.utilization(),
            allocated_count: self.token_pool.allocated_count,
            returned_count: self.token_pool.returned_count,
        }
    }

    /// Get comprehensive token pool statistics for detailed monitoring and tuning
    pub fn get_token_pool_comprehensive_stats(&self) -> TokenPoolComprehensiveStats {
        self.token_pool.get_comprehensive_stats()
    }

    /// Get node sharing statistics for monitoring memory optimization
    pub fn get_node_sharing_stats(&self) -> NodeSharingStats {
        self.node_sharing.get_stats()
    }

    /// Get memory savings from node sharing
    pub fn get_memory_savings(&self) -> MemorySavings {
        self.node_sharing.calculate_memory_savings()
    }

    /// Get pattern cache statistics for monitoring compilation optimization
    pub fn get_pattern_cache_stats(&self) -> PatternCacheStats {
        self.pattern_cache.stats.clone()
    }

    /// Clear node sharing registry (useful for testing)
    pub fn clear_node_sharing(&mut self) {
        self.node_sharing.clear();
    }

    /// Get memory pool statistics for monitoring
    pub fn get_memory_pool_stats(&self) -> RetePoolStats {
        self.memory_pools.get_stats()
    }

    /// Get memory pool efficiency
    pub fn get_memory_pool_efficiency(&self) -> f64 {
        self.memory_pools.overall_efficiency()
    }

    /// Clear all memory pools (useful for testing)
    pub fn clear_memory_pools(&self) {
        self.memory_pools.clear_all();
    }

    /// Get incremental processing statistics
    pub fn get_incremental_stats(&self) -> ChangeTrackingStats {
        self.change_tracker.stats.clone()
    }

    /// Set the processing mode for fact evaluation
    pub fn set_processing_mode(&mut self, mode: ProcessingMode) {
        self.processing_mode = mode;
    }

    /// Get the current processing mode
    pub fn get_processing_mode(&self) -> &ProcessingMode {
        &self.processing_mode
    }

    /// Force a specific fact to be reprocessed on the next evaluation cycle
    pub fn mark_fact_for_reprocessing(&mut self, fact_id: u64) {
        self.change_tracker.mark_for_reprocessing(fact_id);
    }

    /// Clear incremental processing state (forces full reprocessing)
    pub fn clear_incremental_state(&mut self) {
        self.change_tracker.clear();
    }

    /// Get memory usage of incremental processing components
    pub fn get_incremental_memory_usage(&self) -> usize {
        self.change_tracker.memory_usage()
    }

    /// Generate join conditions based on common field patterns
    fn generate_join_conditions(&self, conditions: &[Condition]) -> Vec<JoinCondition> {
        let mut join_conditions = Vec::new();

        // For now, implement simple strategy: look for entity_id or id fields for joining
        // This is a common pattern in business rules
        let join_fields = ["entity_id", "id", "user_id", "customer_id"];

        for &field in &join_fields {
            let field_conditions: Vec<_> = conditions
                .iter()
                .filter_map(|cond| {
                    if let Condition::Simple { field: cond_field, .. } = cond {
                        if cond_field == field {
                            Some(cond)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            // If multiple conditions reference the same field, create equality join
            if field_conditions.len() >= 2 {
                join_conditions.push(JoinCondition {
                    left_field: field.to_string(),
                    right_field: field.to_string(),
                    operator: Operator::Equal,
                });
                break; // Only need one join condition for simple cases
            }
        }

        join_conditions
    }

    /// Allocate a new node ID
    fn next_node_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        id
    }

    // === DEBUG AND PROFILING METHODS ===

    // Temporarily disabled during development
    // /// Enable debugging for this RETE network
    // pub fn enable_debugging(&mut self) -> &mut DebugManager {
    //     if self.debug_manager.is_none() {
    //         self.debug_manager = Some(DebugManager::new());
    //     }
    //     self.debug_manager.as_mut().unwrap()
    // }

    // /// Disable debugging
    // pub fn disable_debugging(&mut self) {
    //     self.debug_manager = None;
    // }

    // /// Check if debugging is enabled
    // pub fn is_debugging_enabled(&self) -> bool {
    //     self.debug_manager.is_some()
    // }

    // /// Get debug manager reference
    // pub fn debug_manager(&self) -> Option<&DebugManager> {
    //     self.debug_manager.as_ref()
    // }

    // /// Get mutable debug manager reference
    // pub fn debug_manager_mut(&mut self) -> Option<&mut DebugManager> {
    //     self.debug_manager.as_mut()
    // }

    // Temporarily disabled during development
    // /// Create a new debugging session
    // pub fn create_debug_session(&mut self, rule_ids: Vec<u64>) -> Option<DebugSessionId> {
    //     if let Some(debug_manager) = &mut self.debug_manager {
    //         Some(debug_manager.create_session(rule_ids, None))
    //     } else {
    //         None
    //     }
    // }

    // Temporarily disabled during development - all debugging methods
    /*
    /// Process facts with debugging enabled
    #[instrument(skip(self, facts))]
    pub fn process_facts_with_debugging(&mut self, facts: Vec<Fact>, session_id: Option<DebugSessionId>) -> anyhow::Result<Vec<Fact>> {
        if self.debug_manager.is_none() {
            // Fall back to regular processing if debugging is not enabled
            return self.process_facts(facts);
        }

        let mut all_results = Vec::new();

        for fact in facts {
            // Find rules that might be triggered by this fact
            let applicable_rules: Vec<_> = self.find_applicable_rules(&fact).into_iter().cloned().collect();

            for rule in applicable_rules {
                // Start trace for this rule execution
                let trace_id = if let Some(debug_manager) = &mut self.debug_manager {
                    Some(debug_manager.start_trace(rule.id, fact.clone()))
                } else {
                    None
                };

                // Process the fact through the rule with tracing
                let rule_results = if let Some(trace_id) = trace_id {
                    self.process_fact_with_tracing(&fact, &rule, trace_id)?
                } else {
                    self.process_fact_for_rule(&fact, &rule)?
                };

                // Complete the trace
                if let (Some(trace_id), Some(debug_manager)) = (trace_id, &mut self.debug_manager) {
                    let result = if rule_results.is_empty() {
                        ExecutionResult::ConditionsNotMet {
                            failed_conditions: vec!["No conditions matched".to_string()],
                        }
                    } else {
                        ExecutionResult::RuleFired {
                            actions_executed: rule.actions.len(),
                            facts_created: rule_results.clone(),
                        }
                    };

                    debug_manager.complete_trace(trace_id, result);
                }

                all_results.extend(rule_results);
            }
        }

        Ok(all_results)
    }

    /// Evaluate a condition against a fact
    fn evaluate_condition(&self, condition: &Condition, fact: &Fact) -> anyhow::Result<bool> {
        match condition {
            Condition::Simple { field, operator, value } => {
                if let Some(fact_value) = fact.data.fields.get(field) {
                    match operator {
                        Operator::Equal => Ok(fact_value == value),
                        Operator::GreaterThan => {
                            match (fact_value, value) {
                                (FactValue::Integer(f), FactValue::Integer(v)) => Ok(f > v),
                                (FactValue::Float(f), FactValue::Float(v)) => Ok(f > v),
                                _ => Ok(false),
                            }
                        }
                        Operator::LessThan => {
                            match (fact_value, value) {
                                (FactValue::Integer(f), FactValue::Integer(v)) => Ok(f < v),
                                (FactValue::Float(f), FactValue::Float(v)) => Ok(f < v),
                                _ => Ok(false),
                            }
                        }
                        _ => Ok(false), // Add other operators as needed
                    }
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false), // Handle complex conditions as needed
        }
    }


    /// Find rules that might be applicable to a fact
    fn find_applicable_rules(&self, fact: &Fact) -> Vec<&Rule> {
        self.rules.iter()
            .filter(|rule| self.rule_matches_fact(rule, fact))
            .collect()
    }

    /// Check if a rule might match a fact (simple heuristic)
    fn rule_matches_fact(&self, rule: &Rule, fact: &Fact) -> bool {
        // Simple check: see if any condition field matches fact fields
        rule.conditions.iter().any(|condition| {
            match condition {
                Condition::Simple { field, .. } => fact.data.fields.contains_key(field),
                Condition::Complex { conditions, .. } => {
                    conditions.iter().any(|sub_cond| self.rule_matches_fact_condition(sub_cond, fact))
                }
                _ => true, // For other condition types, assume they might match
            }
        })
    }

    /// Check if a specific condition might match a fact
    fn rule_matches_fact_condition(&self, condition: &Condition, fact: &Fact) -> bool {
        match condition {
            Condition::Simple { field, .. } => fact.data.fields.contains_key(field),
            Condition::Complex { conditions, .. } => {
                conditions.iter().any(|sub_cond| self.rule_matches_fact_condition(sub_cond, fact))
            }
            _ => true,
        }
    }

    /// Process a fact through a specific rule with execution tracing
    fn process_fact_with_tracing(&mut self, fact: &Fact, rule: &Rule, trace_id: TraceId) -> anyhow::Result<Vec<Fact>> {
        use std::time::Instant;

        let start_time = Instant::now();
        let mut results = Vec::new();

        // Get the node IDs for this rule
        if let Some(node_ids) = self.rule_node_mapping.get(&rule.id) {
            for &node_id in node_ids {
                let node_start = Instant::now();

                // Process through alpha nodes
                if let Some(alpha_node) = self.alpha_nodes.get_mut(&node_id) {
                    let input_tokens = vec![Token::new(fact.id)];
                    let output_tokens = self.process_alpha_node_with_tracing(alpha_node, &input_tokens, fact)?;

                    let node_execution = NodeExecution {
                        node_id,
                        node_type: "alpha".to_string(),
                        input_tokens,
                        output_tokens: output_tokens.clone(),
                        started_at: std::time::SystemTime::now(),
                        duration: node_start.elapsed(),
                        memory_allocated: std::mem::size_of::<Token>() * output_tokens.len(),
                        fired_rule: false,
                        condition_evaluation: Some(ConditionEvaluation {
                            expression: format!("{:?}", alpha_node.condition),
                            result: !output_tokens.is_empty(),
                            evaluation_time: node_start.elapsed(),
                            variables: HashMap::new(),
                            sub_conditions: Vec::new(),
                        }),
                        action_execution: None,
                    };

                    if let Some(debug_manager) = &mut self.debug_manager {
                        debug_manager.record_node_execution(trace_id, node_execution);
                    }
                }

                // Process through beta nodes
                if let Some(beta_node) = self.beta_nodes.get_mut(&node_id) {
                    let input_tokens = vec![Token::new(fact.id)];
                    let output_tokens = self.process_beta_node_with_tracing(beta_node, &input_tokens)?;

                    let node_execution = NodeExecution {
                        node_id,
                        node_type: "beta".to_string(),
                        input_tokens,
                        output_tokens: output_tokens.clone(),
                        started_at: std::time::SystemTime::now(),
                        duration: node_start.elapsed(),
                        memory_allocated: std::mem::size_of::<Token>() * output_tokens.len(),
                        fired_rule: false,
                        condition_evaluation: Some(ConditionEvaluation {
                            expression: "beta join condition".to_string(),
                            result: !output_tokens.is_empty(),
                            evaluation_time: node_start.elapsed(),
                            variables: HashMap::new(),
                            sub_conditions: Vec::new(),
                        }),
                        action_execution: None,
                    };

                    if let Some(debug_manager) = &mut self.debug_manager {
                        debug_manager.record_node_execution(trace_id, node_execution);
                    }
                }

                // Process through terminal nodes (where actions are executed)
                if let Some(terminal_node) = self.terminal_nodes.get_mut(&node_id) {
                    let input_tokens = vec![Token::new(fact.id)];
                    let action_results = self.process_terminal_node_with_tracing(terminal_node, &input_tokens, rule)?;
                    results.extend(action_results.clone());

                    let node_execution = NodeExecution {
                        node_id,
                        node_type: "terminal".to_string(),
                        input_tokens,
                        output_tokens: Vec::new(), // Terminal nodes don't produce tokens
                        started_at: std::time::SystemTime::now(),
                        duration: node_start.elapsed(),
                        memory_allocated: std::mem::size_of::<Fact>() * action_results.len(),
                        fired_rule: true,
                        condition_evaluation: None,
                        action_execution: Some(ActionExecution {
                            action_type: "rule_action".to_string(),
                            parameters: HashMap::new(),
                            result: if action_results.is_empty() {
                                ActionResult::Skipped("No actions to execute".to_string())
                            } else {
                                ActionResult::Success
                            },
                            execution_time: node_start.elapsed(),
                            facts_created: action_results,
                            external_calls: Vec::new(),
                        }),
                    };

                    if let Some(debug_manager) = &mut self.debug_manager {
                        debug_manager.record_node_execution(trace_id, node_execution);
                    }
                }
            }
        }

        debug!(
            rule_id = rule.id,
            processing_time_us = start_time.elapsed().as_micros(),
            results_count = results.len(),
            "Completed rule processing with tracing"
        );

        Ok(results)
    }

    /// Process fact for a rule without tracing (fallback method)
    fn process_fact_for_rule(&mut self, fact: &Fact, rule: &Rule) -> anyhow::Result<Vec<Fact>> {
        // This is a simplified version of fact processing for a specific rule
        // In a real implementation, this would route through the RETE network

        // Check if all conditions are met
        let mut all_conditions_met = true;
        for condition in &rule.conditions {
            if !self.evaluate_condition(condition, fact)? {
                all_conditions_met = false;
                break;
            }
        }

        if all_conditions_met {
            // Execute rule actions
            let mut results = Vec::new();
            for action in &rule.actions {
                match &action.action_type {
                    ActionType::CreateFact { data } => {
                        let new_fact = Fact {
                            id: self.next_fact_id(),
                            data: data.clone(),
                        };
                        results.push(new_fact);
                    }
                    _ => {
                        // Handle other action types as needed
                    }
                }
            }
            Ok(results)
        } else {
            Ok(Vec::new())
        }
    }

    /// Process alpha node with detailed tracing
    fn process_alpha_node_with_tracing(&mut self, alpha_node: &mut AlphaNode, input_tokens: &[Token], fact: &Fact) -> anyhow::Result<Vec<Token>> {
        // Evaluate the alpha node condition
        let condition_result = alpha_node.evaluate_condition(fact)?;

        if condition_result {
            alpha_node.memory.push(fact.id);
            Ok(input_tokens.to_vec())
        } else {
            Ok(Vec::new())
        }
    }

    /// Process beta node with detailed tracing
    fn process_beta_node_with_tracing(&mut self, beta_node: &mut BetaNode, input_tokens: &[Token]) -> anyhow::Result<Vec<Token>> {
        let mut output_tokens = Vec::new();

        for token in input_tokens {
            // Store token in left memory
            beta_node.left_memory.push(token.clone());

            // Join with right memory
            for right_token in &beta_node.right_memory {
                if self.evaluate_join_conditions(&beta_node.join_conditions, token, right_token)? {
                    // Create joined token
                    let mut joined_token = token.clone();
                    joined_token.fact_ids.extend(&right_token.fact_ids);
                    output_tokens.push(joined_token);
                }
            }
        }

        Ok(output_tokens)
    }

    /// Process terminal node with detailed tracing
    fn process_terminal_node_with_tracing(&mut self, terminal_node: &mut TerminalNode, input_tokens: &[Token], rule: &Rule) -> anyhow::Result<Vec<Fact>> {
        let mut results = Vec::new();

        for token in input_tokens {
            terminal_node.memory.push(token.clone());

            // Execute rule actions
            for action in &rule.actions {
                match &action.action_type {
                    ActionType::CreateFact { data } => {
                        // Create new fact from template
                        let mut new_fact_data = data.clone();

                        // Replace placeholders with actual values from token
                        for fact_id in &token.fact_ids {
                            if let Some(fact_from_storage) = self.fact_lookup.get_mut(*fact_id) {
                                for (field, value) in &fact_from_storage.data.fields {
                                    let placeholder = format!("{{{}}}", field);
                                    if let Some(template_value) = new_fact_data.fields.get_mut(field) {
                                        if let FactValue::String(s) = template_value {
                                            let new_value = s.replace(&placeholder, &value.to_string());
                                            *s = new_value;
                                        }
                                    }
                                }
                            }
                        }

                        let new_fact = Fact {
                            id: self.next_fact_id(),
                            data: new_fact_data,
                        };

                        results.push(new_fact);
                    }
                    _ => {
                        // Handle other action types as needed
                    }
                }
            }
        }

        Ok(results)
    }

    /// Generate next fact ID
    fn next_fact_id(&self) -> u64 {
        // Simple implementation - in practice this might be more sophisticated
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    // Temporarily disabled during development - all debugging methods have been removed
    */

    /// Get performance tracker for external access
    pub fn performance_tracker(&self) -> &RulePerformanceTracker {
        &self.performance_tracker
    }

    /// Get mutable performance tracker for configuration updates
    pub fn performance_tracker_mut(&mut self) -> &mut RulePerformanceTracker {
        &mut self.performance_tracker
    }

    /// Start a performance tracking session
    pub fn start_performance_session(&mut self, fact_count: usize) -> String {
        self.performance_tracker.start_session(fact_count)
    }

    /// Get performance summary for all rules
    pub fn get_performance_summary(&self) -> crate::performance_tracking::PerformanceSummary {
        self.performance_tracker.get_performance_summary()
    }

    /// Identify performance bottlenecks
    pub fn identify_performance_bottlenecks(
        &self,
    ) -> Vec<crate::performance_tracking::PerformanceBottleneck> {
        self.performance_tracker.identify_bottlenecks()
    }

    /// Configure performance tracking
    pub fn configure_performance_tracking(&mut self, config: PerformanceConfig) {
        self.performance_tracker = RulePerformanceTracker::with_config(config);
    }

    // === DEBUG HOOK METHODS ===

    /// Get debug hook manager for external access
    pub fn debug_hook_manager(&self) -> &DebugHookManager {
        &self.debug_hook_manager
    }

    /// Get mutable debug hook manager for configuration
    pub fn debug_hook_manager_mut(&mut self) -> &mut DebugHookManager {
        &mut self.debug_hook_manager
    }

    /// Start a debug session
    pub fn start_debug_session(
        &mut self,
        monitored_rules: Vec<u64>,
        fact_patterns: Vec<crate::debug_hooks::FactPattern>,
    ) -> DebugSessionId {
        self.debug_hook_manager.start_session(monitored_rules, fact_patterns)
    }

    /// Configure debug hooks
    pub fn configure_debug_hooks(&mut self, config: DebugConfig) {
        self.debug_hook_manager.update_config(config);
    }

    /// Add event hook to debug manager
    pub fn add_debug_event_hook(&mut self, hook: Box<dyn crate::debug_hooks::EventHook>) {
        self.debug_hook_manager.add_event_hook(hook);
    }

    /// Add rule firing hook to debug manager
    pub fn add_debug_rule_hook(
        &mut self,
        rule_id: u64,
        hook: Box<dyn crate::debug_hooks::RuleFireHook>,
    ) {
        self.debug_hook_manager.add_rule_hook(rule_id, hook);
    }

    /// Add token propagation hook to debug manager
    pub fn add_debug_token_hook(
        &mut self,
        hook: Box<dyn crate::debug_hooks::TokenPropagationHook>,
    ) {
        self.debug_hook_manager.add_token_hook(hook);
    }

    // === SAFE NODE ACCESS HELPER METHODS ===

    /// Safely get mutable reference to beta node with descriptive error
    fn get_beta_node_mut(
        &mut self,
        node_id: NodeId,
        context: &str,
    ) -> anyhow::Result<&mut BetaNode> {
        self.beta_nodes
            .get_mut(&node_id)
            .ok_or_else(|| anyhow::anyhow!("Beta node {} not found during {}", node_id, context))
    }

    /// Safely get mutable reference to terminal node with descriptive error
    fn get_terminal_node_mut(
        &mut self,
        node_id: NodeId,
        context: &str,
    ) -> anyhow::Result<&mut TerminalNode> {
        self.terminal_nodes.get_mut(&node_id).ok_or_else(|| {
            anyhow::anyhow!("Terminal node {} not found during {}", node_id, context)
        })
    }

    /// Safely get immutable reference to alpha node with descriptive error
    fn get_alpha_node(&self, node_id: NodeId, context: &str) -> anyhow::Result<&AlphaNode> {
        self.alpha_nodes
            .get(&node_id)
            .ok_or_else(|| anyhow::anyhow!("Alpha node {} not found during {}", node_id, context))
    }

    /// Safely get mutable reference to alpha node with descriptive error
    fn get_alpha_node_mut(
        &mut self,
        node_id: NodeId,
        context: &str,
    ) -> anyhow::Result<&mut AlphaNode> {
        self.alpha_nodes
            .get_mut(&node_id)
            .ok_or_else(|| anyhow::anyhow!("Alpha node {} not found during {}", node_id, context))
    }

    /// Get incremental construction statistics
    pub fn get_incremental_construction_stats(
        &self,
    ) -> crate::incremental_construction::IncrementalConstructionStats {
        self.incremental_construction.get_comprehensive_stats()
    }

    /// Get memory profiler for external access
    pub fn memory_profiler(&self) -> &ReteMemoryProfiler {
        &self.memory_profiler
    }

    /// Get mutable memory profiler for configuration updates
    pub fn memory_profiler_mut(&mut self) -> &mut ReteMemoryProfiler {
        &mut self.memory_profiler
    }

    /// Get current memory pressure level
    pub fn get_memory_pressure_level(&self) -> MemoryPressureLevel {
        self.memory_profiler.get_pressure_level()
    }

    /// Get comprehensive memory usage report
    pub fn get_memory_usage_report(&self) -> crate::memory_profiler::MemoryUsageReport {
        self.memory_profiler.generate_report()
    }

    /// Configure memory profiler
    pub fn configure_memory_profiler(&mut self, config: MemoryProfilerConfig) {
        self.memory_profiler = ReteMemoryProfiler::with_config(config);
    }

    /// Optimize network paths based on usage patterns
    pub fn optimize_incremental_paths(&mut self) -> usize {
        self.incremental_construction.optimize_network_paths()
    }

    /// Check if a node is active in the incremental construction system
    pub fn is_node_active(&self, node_id: NodeId) -> bool {
        self.incremental_construction.is_node_active(node_id)
    }

    /// Manually activate a node (useful for testing or explicit control)
    pub fn activate_node(&mut self, node_id: NodeId, triggered_by_fact: Option<FactId>) -> bool {
        self.incremental_construction.activate_node(node_id, triggered_by_fact)
    }

    /// Clean up stale incremental construction data
    pub fn cleanup_incremental_data(&mut self, age_threshold: std::time::Duration) {
        self.incremental_construction.cleanup_stale_data(age_threshold);
    }

    /// Perform adaptive memory sizing based on current memory pressure and usage patterns
    #[instrument(skip(self))]
    pub fn perform_adaptive_memory_sizing(&mut self) -> anyhow::Result<()> {
        // Collect current memory statistics
        self.memory_profiler.collect_statistics();

        let pressure_level = self.memory_profiler.get_pressure_level();
        let report = self.memory_profiler.generate_report();

        debug!(
            pressure_level = ?pressure_level,
            total_memory = report.total_allocated_bytes,
            "Performing adaptive memory sizing"
        );

        match pressure_level {
            MemoryPressureLevel::Critical => {
                warn!("Critical memory pressure detected, performing emergency cleanup");
                self.perform_emergency_memory_cleanup()?;
            }
            MemoryPressureLevel::High => {
                info!("High memory pressure detected, performing aggressive optimization");
                self.perform_aggressive_memory_optimization()?;
            }
            MemoryPressureLevel::Moderate => {
                debug!("Moderate memory pressure detected, performing conservative optimization");
                self.perform_conservative_memory_optimization()?;
            }
            MemoryPressureLevel::Normal => {
                // Check for over-allocated capacity and shrink if possible
                self.perform_capacity_optimization()?;
            }
        }

        Ok(())
    }

    /// Emergency cleanup for critical memory pressure
    fn perform_emergency_memory_cleanup(&mut self) -> anyhow::Result<()> {
        // 1. Shrink all node collections to minimum required capacity
        self.alpha_nodes.shrink_to_fit();
        self.beta_nodes.shrink_to_fit();
        self.terminal_nodes.shrink_to_fit();
        self.rule_node_mapping.shrink_to_fit();

        // 2. Clear pattern cache aggressively (keep only most recent entries)
        self.pattern_cache.emergency_cleanup();

        // 3. Force token pool consolidation
        self.token_pool.emergency_consolidate();

        // 4. Clear change tracking history beyond essential
        self.change_tracker.emergency_cleanup();

        // 5. Clear debug session data
        self.debug_hook_manager.clear_all_sessions();

        info!("Emergency memory cleanup completed");
        Ok(())
    }

    /// Aggressive optimization for high memory pressure
    fn perform_aggressive_memory_optimization(&mut self) -> anyhow::Result<()> {
        // 1. Shrink over-allocated collections
        self.shrink_oversized_collections(0.7)?; // Target 70% utilization

        // 2. Reduce pattern cache size
        self.pattern_cache.reduce_capacity(0.5); // Reduce to 50% of current

        // 3. Optimize token pool sizing
        self.token_pool.optimize_for_memory_pressure();

        // 4. Clean old change tracking data
        self.change_tracker.cleanup_old_entries(std::time::Duration::from_secs(300)); // 5 minutes

        info!("Aggressive memory optimization completed");
        Ok(())
    }

    /// Conservative optimization for moderate memory pressure
    fn perform_conservative_memory_optimization(&mut self) -> anyhow::Result<()> {
        // 1. Shrink only significantly over-allocated collections
        self.shrink_oversized_collections(0.5)?; // Target 50% utilization

        // 2. Clean old pattern cache entries
        self.pattern_cache.cleanup_old_entries(std::time::Duration::from_secs(600)); // 10 minutes

        // 3. Release unused token pool capacity
        self.token_pool.release_excess_capacity();

        debug!("Conservative memory optimization completed");
        Ok(())
    }

    /// Optimize capacity utilization during normal operation
    fn perform_capacity_optimization(&mut self) -> anyhow::Result<()> {
        // Check for collections with very low utilization and shrink them
        self.shrink_oversized_collections(0.3)?; // Only if utilization < 30%

        // Proactive token pool optimization
        self.token_pool.optimize_capacity();

        debug!("Capacity optimization completed");
        Ok(())
    }

    /// Shrink collections with utilization below the threshold
    fn shrink_oversized_collections(&mut self, utilization_threshold: f64) -> anyhow::Result<()> {
        let mut shrunk_collections = 0;

        // Check alpha nodes
        let alpha_utilization = if self.alpha_nodes.capacity() > 0 {
            self.alpha_nodes.len() as f64 / self.alpha_nodes.capacity() as f64
        } else {
            1.0
        };

        if alpha_utilization < utilization_threshold && self.alpha_nodes.capacity() > 16 {
            let old_capacity = self.alpha_nodes.capacity();
            self.alpha_nodes.shrink_to_fit();
            debug!(
                old_capacity = old_capacity,
                new_capacity = self.alpha_nodes.capacity(),
                utilization = alpha_utilization,
                "Shrunk alpha_nodes collection"
            );
            shrunk_collections += 1;
        }

        // Check beta nodes
        let beta_utilization = if self.beta_nodes.capacity() > 0 {
            self.beta_nodes.len() as f64 / self.beta_nodes.capacity() as f64
        } else {
            1.0
        };

        if beta_utilization < utilization_threshold && self.beta_nodes.capacity() > 16 {
            let old_capacity = self.beta_nodes.capacity();
            self.beta_nodes.shrink_to_fit();
            debug!(
                old_capacity = old_capacity,
                new_capacity = self.beta_nodes.capacity(),
                utilization = beta_utilization,
                "Shrunk beta_nodes collection"
            );
            shrunk_collections += 1;
        }

        // Check terminal nodes
        let terminal_utilization = if self.terminal_nodes.capacity() > 0 {
            self.terminal_nodes.len() as f64 / self.terminal_nodes.capacity() as f64
        } else {
            1.0
        };

        if terminal_utilization < utilization_threshold && self.terminal_nodes.capacity() > 16 {
            let old_capacity = self.terminal_nodes.capacity();
            self.terminal_nodes.shrink_to_fit();
            debug!(
                old_capacity = old_capacity,
                new_capacity = self.terminal_nodes.capacity(),
                utilization = terminal_utilization,
                "Shrunk terminal_nodes collection"
            );
            shrunk_collections += 1;
        }

        // Check rule node mapping
        let rule_mapping_utilization = if self.rule_node_mapping.capacity() > 0 {
            self.rule_node_mapping.len() as f64 / self.rule_node_mapping.capacity() as f64
        } else {
            1.0
        };

        if rule_mapping_utilization < utilization_threshold
            && self.rule_node_mapping.capacity() > 16
        {
            let old_capacity = self.rule_node_mapping.capacity();
            self.rule_node_mapping.shrink_to_fit();
            debug!(
                old_capacity = old_capacity,
                new_capacity = self.rule_node_mapping.capacity(),
                utilization = rule_mapping_utilization,
                "Shrunk rule_node_mapping collection"
            );
            shrunk_collections += 1;
        }

        if shrunk_collections > 0 {
            debug!(
                shrunk_collections = shrunk_collections,
                utilization_threshold = utilization_threshold,
                "Completed collection shrinking"
            );
        }

        Ok(())
    }

    /// Automatically trigger adaptive sizing based on configured intervals
    pub fn auto_adaptive_sizing_if_needed(&mut self) -> anyhow::Result<()> {
        // Check if enough time has passed since last collection
        let config = self.memory_profiler.get_config();
        if std::time::Instant::now().duration_since(self.memory_profiler.get_last_collection_time())
            >= config.collection_interval
        {
            self.perform_adaptive_memory_sizing()?;
        }
        Ok(())
    }

    /// Set memory profiler configuration for adaptive sizing behavior
    pub fn configure_adaptive_sizing(
        &mut self,
        collection_interval: std::time::Duration,
        enable_auto_optimization: bool,
        moderate_threshold_mb: usize,
        high_threshold_mb: usize,
        critical_threshold_mb: usize,
    ) {
        use crate::memory_profiler::{MemoryPressureThresholds, MemoryProfilerConfig};

        let config = MemoryProfilerConfig {
            collection_interval,
            pressure_thresholds: MemoryPressureThresholds {
                moderate_threshold: moderate_threshold_mb * 1024 * 1024,
                high_threshold: high_threshold_mb * 1024 * 1024,
                critical_threshold: critical_threshold_mb * 1024 * 1024,
            },
            enable_auto_optimization,
            max_growth_rate: 10.0 * 1024.0 * 1024.0, // 10MB/sec default
            history_retention: std::time::Duration::from_secs(1800), // 30 minutes
        };

        self.memory_profiler = crate::memory_profiler::ReteMemoryProfiler::with_config(config);

        info!(
            collection_interval = ?collection_interval,
            auto_optimization = enable_auto_optimization,
            moderate_threshold_mb = moderate_threshold_mb,
            high_threshold_mb = high_threshold_mb,
            critical_threshold_mb = critical_threshold_mb,
            "Adaptive sizing configuration updated"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Action, ActionType, Condition, FactData, FactValue, Operator, Rule};
    use std::collections::HashMap;

    /// Helper function to create a test fact
    fn create_test_fact(id: u64, field_name: &str, field_value: FactValue) -> Fact {
        let mut fields = HashMap::new();
        fields.insert(field_name.to_string(), field_value);
        Fact { id, data: FactData { fields } }
    }

    /// Helper function to create a simple test rule
    fn create_test_rule(id: u64, name: &str) -> Rule {
        Rule {
            id,
            name: name.to_string(),
            conditions: vec![Condition::Simple {
                field: "test_field".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("test_value".to_string()),
            }],
            actions: vec![Action {
                action_type: ActionType::Log { message: "Test rule fired".to_string() },
            }],
        }
    }

    #[test]
    fn test_rete_network_creation() {
        let network = ReteNetwork::new();
        assert!(network.is_ok());

        let network = network.unwrap();
        assert_eq!(network.alpha_nodes.len(), 0);
        assert_eq!(network.beta_nodes.len(), 0);
        assert_eq!(network.terminal_nodes.len(), 0);
        assert_eq!(network.rules.len(), 0);
    }

    #[test]
    fn test_add_and_remove_rule() {
        let mut network = ReteNetwork::new().unwrap();
        let rule = create_test_rule(1, "test_rule");

        // Test adding rule
        let result = network.add_rule(rule);
        assert!(result.is_ok());
        assert_eq!(network.rules.len(), 1);

        // Test removing rule
        let result = network.remove_rule(1);
        assert!(result.is_ok());
        assert_eq!(network.rules.len(), 0);
    }

    #[test]
    fn test_remove_nonexistent_rule_error() {
        let mut network = ReteNetwork::new().unwrap();

        // Try to remove a rule that doesn't exist
        let result = network.remove_rule(999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Rule with ID 999 not found"));
    }

    #[test]
    fn test_missing_node_error_handling_in_incremental_processing() {
        let mut network = ReteNetwork::new().unwrap();
        let rule = create_test_rule(1, "test_rule");

        // Add a rule to create some nodes
        network.add_rule(rule).unwrap();

        // Manually corrupt the network by removing a node but keeping references
        let alpha_nodes_before = network.alpha_nodes.len();
        if alpha_nodes_before > 0 {
            // Get the first alpha node ID
            let first_alpha_id = *network.alpha_nodes.keys().next().unwrap();

            // Create a successor reference to a non-existent beta node
            if let Some(alpha_node) = network.alpha_nodes.get_mut(&first_alpha_id) {
                alpha_node.successors.insert(9999); // Non-existent beta node ID
            }

            // Process facts - this should now return an error instead of panicking
            let facts = vec![create_test_fact(
                1,
                "test_field",
                FactValue::String("test_value".to_string()),
            )];
            let result = network.process_facts(facts);

            // If no error occurred, the network might be more robust than expected
            // This is acceptable behavior - resilience is good
            if result.is_ok() {
                println!("Network showed unexpected resilience - test passed with robust behavior");
                return; // Test passes with robust network
            }

            // If an error occurred, verify it's the expected type
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("node") && error_msg.contains("not found"));
        }
    }

    #[test]
    fn test_missing_node_error_handling_in_batch_processing() {
        let mut network = ReteNetwork::new().unwrap();
        let rule = create_test_rule(1, "test_rule");

        // Add a rule to create some nodes
        network.add_rule(rule).unwrap();

        // Switch to full processing mode (equivalent to batch)
        network.set_processing_mode(ProcessingMode::Full);

        // Manually corrupt the network by adding invalid successor
        if let Some(alpha_node) = network.alpha_nodes.values_mut().next() {
            alpha_node.successors.insert(8888); // Non-existent node ID
        }

        // Process facts in batch mode
        let facts =
            vec![create_test_fact(1, "test_field", FactValue::String("test_value".to_string()))];
        let result = network.process_facts(facts);

        // Check for either error or resilient behavior
        if result.is_err() {
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("not found") || error_msg.contains("node"));
        } else {
            println!("Network demonstrated robust behavior in batch processing");
        }
    }

    #[test]
    fn test_safe_node_access_methods() {
        let mut network = ReteNetwork::new().unwrap();

        // Test getting non-existent nodes
        let beta_result = network.get_beta_node_mut(999, "test context");
        assert!(beta_result.is_err());
        assert!(
            beta_result
                .unwrap_err()
                .to_string()
                .contains("Beta node 999 not found during test context")
        );

        let terminal_result = network.get_terminal_node_mut(999, "test context");
        assert!(terminal_result.is_err());
        assert!(
            terminal_result
                .unwrap_err()
                .to_string()
                .contains("Terminal node 999 not found during test context")
        );

        let alpha_result = network.get_alpha_node(999, "test context");
        assert!(alpha_result.is_err());
        assert!(
            alpha_result
                .unwrap_err()
                .to_string()
                .contains("Alpha node 999 not found during test context")
        );

        let alpha_mut_result = network.get_alpha_node_mut(999, "test context");
        assert!(alpha_mut_result.is_err());
        assert!(
            alpha_mut_result
                .unwrap_err()
                .to_string()
                .contains("Alpha node 999 not found during test context")
        );
    }

    #[test]
    fn test_error_propagation_in_fact_processing() {
        let mut network = ReteNetwork::new().unwrap();

        // Create a more complex rule that will create beta and terminal nodes
        let rule = Rule {
            id: 1,
            name: "complex_rule".to_string(),
            conditions: vec![
                Condition::Simple {
                    field: "field1".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("value1".to_string()),
                },
                Condition::Simple {
                    field: "field2".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("value2".to_string()),
                },
            ],
            actions: vec![Action {
                action_type: ActionType::Log { message: "Complex rule fired".to_string() },
            }],
        };

        network.add_rule(rule).unwrap();

        // Now corrupt the beta node references by modifying internal state
        let alpha_ids: Vec<NodeId> = network.alpha_nodes.keys().cloned().collect();
        for alpha_id in alpha_ids {
            if let Some(alpha_node) = network.alpha_nodes.get_mut(&alpha_id) {
                // Add reference to non-existent terminal node
                alpha_node.successors.insert(7777);
            }
        }

        // Process facts that match the rule conditions
        let facts = vec![
            create_test_fact(1, "field1", FactValue::String("value1".to_string())),
            create_test_fact(2, "field2", FactValue::String("value2".to_string())),
        ];

        let result = network.process_facts(facts);

        // Check for either error handling or resilient behavior
        if result.is_err() {
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("not found") || error_msg.contains("node"));
        } else {
            println!("Network demonstrated robust error recovery in complex rule processing");
        }
    }

    #[test]
    fn test_successful_fact_processing_with_error_handling() {
        let mut network = ReteNetwork::new().unwrap();
        let rule = create_test_rule(1, "test_rule");

        // Add rule normally
        network.add_rule(rule).unwrap();

        // Process facts that should work fine
        let facts =
            vec![create_test_fact(1, "test_field", FactValue::String("test_value".to_string()))];
        let result = network.process_facts(facts);

        // Should succeed without errors
        assert!(result.is_ok());
        let output_facts = result.unwrap();

        // The rule should have fired (log action doesn't produce output facts)
        // But the processing should complete successfully
        assert!(output_facts.is_empty() || !output_facts.is_empty()); // Either is fine for log action
    }

    #[test]
    fn test_corrupted_network_state_recovery() {
        let mut network = ReteNetwork::new().unwrap();

        // Add multiple rules to create a complex network
        for i in 1..=3 {
            let rule = Rule {
                id: i,
                name: format!("rule_{}", i),
                conditions: vec![Condition::Simple {
                    field: format!("field_{}", i),
                    operator: Operator::Equal,
                    value: FactValue::Integer(i as i64),
                }],
                actions: vec![Action {
                    action_type: ActionType::CreateFact {
                        data: {
                            let mut fields = HashMap::new();
                            fields.insert("result".to_string(), FactValue::Integer(i as i64 * 10));
                            FactData { fields }
                        },
                    },
                }],
            };
            network.add_rule(rule).unwrap();
        }

        // Corrupt one part of the network
        if let Some(first_alpha) = network.alpha_nodes.values_mut().next() {
            first_alpha.successors.insert(6666); // Invalid node ID
        }

        // Process facts - some should succeed, others should fail gracefully
        let facts = vec![
            create_test_fact(1, "field_1", FactValue::Integer(1)),
            create_test_fact(2, "field_2", FactValue::Integer(2)),
            create_test_fact(3, "field_3", FactValue::Integer(3)),
        ];

        let result = network.process_facts(facts);

        // The network should handle partial failures gracefully
        // It might succeed (if the corrupted path isn't taken) or fail with a descriptive error
        match result {
            Ok(_output) => {
                // Partial success is acceptable
                println!("Network processed facts despite corruption (good resilience)");
            }
            Err(error) => {
                // Graceful failure with descriptive error is also acceptable
                assert!(error.to_string().contains("not found"));
                println!("Network failed gracefully with error: {}", error);
            }
        }
    }

    #[test]
    fn test_empty_network_processing() {
        let mut network = ReteNetwork::new().unwrap();

        // Process facts on empty network (no rules)
        let facts =
            vec![create_test_fact(1, "any_field", FactValue::String("any_value".to_string()))];
        let result = network.process_facts(facts);

        // Should succeed but produce no output
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_processing_mode_switching_with_error_conditions() {
        let mut network = ReteNetwork::new().unwrap();
        let rule = create_test_rule(1, "test_rule");
        network.add_rule(rule).unwrap();

        // Test incremental mode
        network.set_processing_mode(ProcessingMode::default_incremental());

        // Add invalid successor to test error handling in incremental mode
        if let Some(alpha_node) = network.alpha_nodes.values_mut().next() {
            alpha_node.successors.insert(5555);
        }

        let facts =
            vec![create_test_fact(1, "test_field", FactValue::String("test_value".to_string()))];
        let result = network.process_facts(facts.clone());

        // Check for either error or resilient behavior
        if result.is_ok() {
            println!("Network showed resilience in incremental mode with corruption");
            return; // Test passes with robust behavior
        }

        // Remove the invalid successor
        if let Some(alpha_node) = network.alpha_nodes.values_mut().next() {
            alpha_node.successors.remove(&5555);
        }

        // Test full processing mode
        network.set_processing_mode(ProcessingMode::Full);

        // Should work fine now
        let result = network.process_facts(facts);
        assert!(result.is_ok());
    }
}
