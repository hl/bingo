//! Beta Network Implementation for RETE Network
//!
//! This module implements the beta network component of the RETE algorithm, which handles
//! multi-condition rule processing and cross-fact pattern matching. The beta network is
//! responsible for maintaining partial matches and performing joins between different
//! fact patterns.
//!
//! ## Beta Network Architecture
//!
//! ```text
//! Alpha Memory → Beta Network → Terminal Nodes
//!      ↓              ↓              ↓
//!   Single          Multi-         Rule
//!   Patterns        Patterns       Execution
//! ```
//!
//! ## Key Components
//!
//! - **Token**: Represents a partial match through the beta network
//! - **BetaNode**: Abstract base for all beta network nodes
//! - **JoinNode**: Performs joins between alpha memory and beta memory
//! - **BetaMemory**: Stores partial matches for incremental processing
//! - **TerminalNode**: Executes actions when all conditions are satisfied

use crate::memory_pools::MemoryPoolManager;
use crate::types::{Fact, FactId, FactValue, NodeId, Rule, RuleId};
use std::collections::{HashMap, HashSet};
use tracing::{debug, instrument};

/// Token represents a partial match in the RETE network
///
/// A token carries the current state of pattern matching for a rule,
/// including all facts that have matched so far and metadata for
/// conflict resolution and debugging.
#[derive(Debug, Clone)]
pub struct Token {
    /// Facts that contribute to this partial match
    pub facts: Vec<FactId>,
    /// Rule being matched
    pub rule_id: RuleId,
    /// Timestamp for conflict resolution
    pub timestamp: u64,
    /// Parent token this was derived from (for tracing)
    pub parent_token: Option<Box<Token>>,
    /// Current condition index being matched
    pub condition_index: usize,
}

impl Token {
    /// Create a new root token for a rule
    pub fn new(rule_id: RuleId) -> Self {
        Self {
            facts: Vec::new(),
            rule_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            parent_token: None,
            condition_index: 0,
        }
    }

    /// Create a new token by extending this one with a fact
    pub fn extend(&self, fact_id: FactId) -> Self {
        let mut new_token = self.clone();
        new_token.facts.push(fact_id);
        new_token.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        new_token.parent_token = Some(Box::new(self.clone()));
        new_token.condition_index += 1;
        new_token
    }

    /// Check if this token represents a complete match for the rule
    pub fn is_complete(&self, rule: &Rule) -> bool {
        // For single-fact matching: token is complete if condition_index >= total conditions
        // For cross-fact matching: token is complete if facts.len() == conditions.len()
        // We use condition_index as the primary indicator of completeness
        self.condition_index >= rule.conditions.len()
    }

    /// Get the fact at a specific condition index
    pub fn get_fact_for_condition(&self, index: usize) -> Option<FactId> {
        self.facts.get(index).copied()
    }

    /// Generate a unique key for this token combination
    pub fn to_key(&self) -> String {
        format!(
            "{}_{}",
            self.rule_id,
            self.facts.iter().map(|f| f.to_string()).collect::<Vec<_>>().join("_")
        )
    }
}

/// Beta node types in the RETE network
#[derive(Debug, Clone)]
pub enum BetaNodeType {
    /// Root node (no conditions)
    Root,
    /// Join node (combines alpha memory with beta memory)
    Join { alpha_memory_id: NodeId, condition_index: usize },
    /// Terminal node (rule execution)
    Terminal { rule_id: RuleId },
}

/// Beta node in the RETE network
///
/// Beta nodes maintain partial matches and perform joins between
/// different fact patterns. They form the backbone of multi-condition
/// rule processing.
#[derive(Debug, Clone)]
pub struct BetaNode {
    /// Unique identifier for this beta node
    pub id: NodeId,
    /// Type of beta node
    pub node_type: BetaNodeType,
    /// Child nodes in the beta network
    pub children: Vec<NodeId>,
    /// Parent node (None for root)
    pub parent: Option<NodeId>,
    /// Tokens stored in this node's memory
    pub tokens: HashSet<String>, // Using token keys for efficient lookup
    /// Performance statistics
    pub tokens_processed: u64,
    pub tokens_passed: u64,
    pub join_attempts: u64,
    pub successful_joins: u64,
}

impl BetaNode {
    /// Create a new beta node
    pub fn new(id: NodeId, node_type: BetaNodeType) -> Self {
        Self {
            id,
            node_type,
            children: Vec::new(),
            parent: None,
            tokens: HashSet::new(),
            tokens_processed: 0,
            tokens_passed: 0,
            join_attempts: 0,
            successful_joins: 0,
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child_id: NodeId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    /// Set the parent node
    pub fn set_parent(&mut self, parent_id: NodeId) {
        self.parent = Some(parent_id);
    }

    /// Add a token to this node's memory
    pub fn add_token(&mut self, token: &Token) -> bool {
        let token_key = token.to_key();
        let was_new = self.tokens.insert(token_key);
        if was_new {
            self.tokens_processed += 1;
        }
        was_new
    }

    /// Remove a token from this node's memory
    pub fn remove_token(&mut self, token: &Token) -> bool {
        let token_key = token.to_key();
        self.tokens.remove(&token_key)
    }

    /// Check if this node contains a specific token
    pub fn contains_token(&self, token: &Token) -> bool {
        let token_key = token.to_key();
        self.tokens.contains(&token_key)
    }

    /// Get the number of tokens in this node
    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    /// Clear all tokens from this node
    pub fn clear_tokens(&mut self) {
        self.tokens.clear();
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> BetaNodeStats {
        BetaNodeStats {
            id: self.id,
            node_type: self.node_type.clone(),
            token_count: self.tokens.len(),
            tokens_processed: self.tokens_processed,
            tokens_passed: self.tokens_passed,
            join_attempts: self.join_attempts,
            successful_joins: self.successful_joins,
            join_success_rate: if self.join_attempts > 0 {
                (self.successful_joins as f64 / self.join_attempts as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Statistics for a beta node
#[derive(Debug, Clone)]
pub struct BetaNodeStats {
    pub id: NodeId,
    pub node_type: BetaNodeType,
    pub token_count: usize,
    pub tokens_processed: u64,
    pub tokens_passed: u64,
    pub join_attempts: u64,
    pub successful_joins: u64,
    pub join_success_rate: f64,
}

/// Join node implementation for combining alpha and beta memories
#[derive(Debug, Clone)]
pub struct JoinNode {
    /// Base beta node
    pub beta_node: BetaNode,
    /// Alpha memory this node joins with
    pub alpha_memory_id: NodeId,
    /// Condition index this node is responsible for
    pub condition_index: usize,
    /// Join tests for cross-fact comparisons
    pub join_tests: Vec<JoinTest>,
}

impl JoinNode {
    /// Create a new join node
    pub fn new(id: NodeId, alpha_memory_id: NodeId, condition_index: usize) -> Self {
        Self {
            beta_node: BetaNode::new(id, BetaNodeType::Join { alpha_memory_id, condition_index }),
            alpha_memory_id,
            condition_index,
            join_tests: Vec::new(),
        }
    }

    /// Add a join test for cross-fact comparisons
    pub fn add_join_test(&mut self, test: JoinTest) {
        self.join_tests.push(test);
    }

    /// Perform join operation between a token and fact
    pub fn perform_join(
        &mut self,
        token: &Token,
        fact_id: FactId,
        facts: &HashMap<FactId, Fact>,
    ) -> Option<Token> {
        self.beta_node.join_attempts += 1;

        // Check if all join tests pass
        for test in &self.join_tests {
            if !test.evaluate(token, fact_id, facts) {
                return None;
            }
        }

        // Create extended token
        let new_token = token.extend(fact_id);
        self.beta_node.successful_joins += 1;
        Some(new_token)
    }

    /// Get the alpha memory ID this node joins with
    pub fn get_alpha_memory_id(&self) -> NodeId {
        self.alpha_memory_id
    }

    /// Get the condition index this node handles
    pub fn get_condition_index(&self) -> usize {
        self.condition_index
    }
}

/// Join test for cross-fact comparisons
#[derive(Debug, Clone)]
pub struct JoinTest {
    /// Field from the current fact
    pub current_field: String,
    /// Field from a previous fact (identified by condition index)
    pub previous_field: String,
    /// Condition index of the previous fact
    pub previous_condition_index: usize,
    /// Comparison operator
    pub operator: JoinOperator,
}

impl JoinTest {
    /// Create a new join test
    pub fn new(
        current_field: String,
        previous_field: String,
        previous_condition_index: usize,
        operator: JoinOperator,
    ) -> Self {
        Self { current_field, previous_field, previous_condition_index, operator }
    }

    /// Evaluate this join test against a token and fact
    pub fn evaluate(&self, token: &Token, fact_id: FactId, facts: &HashMap<FactId, Fact>) -> bool {
        // Get the current fact
        let current_fact = match facts.get(&fact_id) {
            Some(fact) => fact,
            None => return false,
        };

        // Get the previous fact
        let previous_fact_id = match token.get_fact_for_condition(self.previous_condition_index) {
            Some(id) => id,
            None => return false,
        };

        let previous_fact = match facts.get(&previous_fact_id) {
            Some(fact) => fact,
            None => return false,
        };

        // Get field values
        let current_value = match current_fact.data.fields.get(&self.current_field) {
            Some(value) => value,
            None => return false,
        };

        let previous_value = match previous_fact.data.fields.get(&self.previous_field) {
            Some(value) => value,
            None => return false,
        };

        // Apply comparison
        self.operator.compare(current_value, previous_value)
    }
}

/// Join operators for cross-fact comparisons
#[derive(Debug, Clone)]
pub enum JoinOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

impl JoinOperator {
    /// Compare two fact values using this operator
    pub fn compare(&self, left: &FactValue, right: &FactValue) -> bool {
        match self {
            JoinOperator::Equal => left == right,
            JoinOperator::NotEqual => left != right,
            JoinOperator::GreaterThan => {
                if let (Some(left_num), Some(right_num)) =
                    (left.to_comparable(), right.to_comparable())
                {
                    left_num > right_num
                } else {
                    false
                }
            }
            JoinOperator::LessThan => {
                if let (Some(left_num), Some(right_num)) =
                    (left.to_comparable(), right.to_comparable())
                {
                    left_num < right_num
                } else {
                    false
                }
            }
            JoinOperator::GreaterThanOrEqual => {
                if let (Some(left_num), Some(right_num)) =
                    (left.to_comparable(), right.to_comparable())
                {
                    left_num >= right_num
                } else {
                    false
                }
            }
            JoinOperator::LessThanOrEqual => {
                if let (Some(left_num), Some(right_num)) =
                    (left.to_comparable(), right.to_comparable())
                {
                    left_num <= right_num
                } else {
                    false
                }
            }
        }
    }
}

/// Beta memory for storing partial matches
#[derive(Debug)]
pub struct BetaMemory {
    /// Tokens stored in this memory
    pub tokens: HashMap<String, Token>,
    /// Performance statistics
    pub tokens_added: u64,
    pub tokens_removed: u64,
    pub total_activations: u64,
}

impl BetaMemory {
    /// Create a new beta memory
    pub fn new() -> Self {
        Self { tokens: HashMap::new(), tokens_added: 0, tokens_removed: 0, total_activations: 0 }
    }

    /// Add a token to the memory
    pub fn add_token(&mut self, token: Token) -> bool {
        let token_key = token.to_key();
        let was_new = self.tokens.insert(token_key, token).is_none();
        if was_new {
            self.tokens_added += 1;
        }
        was_new
    }

    /// Remove a token from the memory
    pub fn remove_token(&mut self, token: &Token) -> bool {
        let token_key = token.to_key();
        let was_present = self.tokens.remove(&token_key).is_some();
        if was_present {
            self.tokens_removed += 1;
        }
        was_present
    }

    /// Get all tokens in the memory
    pub fn get_tokens(&self) -> Vec<&Token> {
        self.tokens.values().collect()
    }

    /// Get a specific token by key
    pub fn get_token(&self, key: &str) -> Option<&Token> {
        self.tokens.get(key)
    }

    /// Check if memory contains a token
    pub fn contains_token(&self, token: &Token) -> bool {
        let token_key = token.to_key();
        self.tokens.contains_key(&token_key)
    }

    /// Get the number of tokens
    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    /// Clear all tokens
    pub fn clear(&mut self) {
        self.tokens.clear();
    }

    /// Record an activation
    pub fn record_activation(&mut self) {
        self.total_activations += 1;
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> BetaMemoryStats {
        BetaMemoryStats {
            token_count: self.tokens.len(),
            tokens_added: self.tokens_added,
            tokens_removed: self.tokens_removed,
            total_activations: self.total_activations,
        }
    }
}

impl Default for BetaMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for beta memory
#[derive(Debug, Clone)]
pub struct BetaMemoryStats {
    pub token_count: usize,
    pub tokens_added: u64,
    pub tokens_removed: u64,
    pub total_activations: u64,
}

/// Beta network manager
#[derive(Debug)]
pub struct BetaNetworkManager {
    /// Beta nodes indexed by ID
    pub beta_nodes: HashMap<NodeId, BetaNode>,
    /// Join nodes indexed by ID
    pub join_nodes: HashMap<NodeId, JoinNode>,
    /// Beta memories indexed by node ID
    pub beta_memories: HashMap<NodeId, BetaMemory>,
    /// Root node ID
    pub root_node_id: Option<NodeId>,
    /// Next node ID
    pub next_node_id: NodeId,
    /// Memory pool manager for token vector allocation
    pub memory_pools: MemoryPoolManager,
    /// Performance statistics
    pub total_tokens_processed: u64,
    pub total_joins_performed: u64,
    pub total_activations: u64,
}

impl BetaNetworkManager {
    /// Create a new beta network manager
    pub fn new() -> Self {
        Self {
            beta_nodes: HashMap::new(),
            join_nodes: HashMap::new(),
            beta_memories: HashMap::new(),
            root_node_id: None,
            next_node_id: 1,
            memory_pools: MemoryPoolManager::new(),
            total_tokens_processed: 0,
            total_joins_performed: 0,
            total_activations: 0,
        }
    }

    /// Create a new beta network manager with high-throughput configuration
    pub fn with_high_throughput_config() -> Self {
        Self {
            beta_nodes: HashMap::new(),
            join_nodes: HashMap::new(),
            beta_memories: HashMap::new(),
            root_node_id: None,
            next_node_id: 1,
            memory_pools: MemoryPoolManager::with_high_throughput_config(),
            total_tokens_processed: 0,
            total_joins_performed: 0,
            total_activations: 0,
        }
    }

    /// Create the root node
    pub fn create_root_node(&mut self) -> NodeId {
        let node_id = self.next_node_id;
        self.next_node_id += 1;

        let root_node = BetaNode::new(node_id, BetaNodeType::Root);
        self.beta_nodes.insert(node_id, root_node);
        self.beta_memories.insert(node_id, BetaMemory::new());

        self.root_node_id = Some(node_id);
        node_id
    }

    /// Create a join node
    pub fn create_join_node(&mut self, alpha_memory_id: NodeId, condition_index: usize) -> NodeId {
        let node_id = self.next_node_id;
        self.next_node_id += 1;

        let join_node = JoinNode::new(node_id, alpha_memory_id, condition_index);
        self.join_nodes.insert(node_id, join_node);
        self.beta_memories.insert(node_id, BetaMemory::new());

        node_id
    }

    /// Create a terminal node
    pub fn create_terminal_node(&mut self, rule_id: RuleId) -> NodeId {
        let node_id = self.next_node_id;
        self.next_node_id += 1;

        let terminal_node = BetaNode::new(node_id, BetaNodeType::Terminal { rule_id });
        self.beta_nodes.insert(node_id, terminal_node);

        node_id
    }

    /// Connect two nodes (parent -> child)
    pub fn connect_nodes(&mut self, parent_id: NodeId, child_id: NodeId) {
        // Update parent's children
        if let Some(parent_node) = self.beta_nodes.get_mut(&parent_id) {
            parent_node.add_child(child_id);
        } else if let Some(parent_join) = self.join_nodes.get_mut(&parent_id) {
            parent_join.beta_node.add_child(child_id);
        }

        // Update child's parent
        if let Some(child_node) = self.beta_nodes.get_mut(&child_id) {
            child_node.set_parent(parent_id);
        } else if let Some(child_join) = self.join_nodes.get_mut(&child_id) {
            child_join.beta_node.set_parent(parent_id);
        }
    }

    /// Process a token through the beta network
    #[instrument(skip(self, token, facts))]
    pub fn process_token(&mut self, token: Token, facts: &HashMap<FactId, Fact>) -> Vec<Token> {
        self.total_tokens_processed += 1;
        let mut completed_tokens = self.memory_pools.token_vecs.get();
        let mut tokens_to_process = self.memory_pools.token_vecs.get();
        tokens_to_process.push(token);

        debug!("Processing token through beta network");

        while let Some(current_token) = tokens_to_process.pop() {
            debug!(
                "Processing token with {} facts for rule {}",
                current_token.facts.len(),
                current_token.rule_id
            );

            // Find the current node in the beta network based on the token's progress
            let current_node_id = self.find_current_beta_node(&current_token);

            if let Some(node_id) = current_node_id {
                // Get the node and process the token
                if let Some(node) = self.beta_nodes.get(&node_id) {
                    match &node.node_type {
                        BetaNodeType::Root => {
                            // Root node - pass token to children
                            for &_child_id in &node.children {
                                tokens_to_process.push(current_token.clone());
                            }
                        }
                        BetaNodeType::Terminal { .. } => {
                            // Terminal node - token is complete
                            completed_tokens.push(current_token);
                            self.total_activations += 1;
                        }
                        BetaNodeType::Join { alpha_memory_id: _, condition_index } => {
                            // Join node - this would be handled by join_nodes HashMap
                            debug!("Token reached join node for condition {}", condition_index);
                        }
                    }
                } else if self.join_nodes.contains_key(&node_id) {
                    // Process through join node - need to handle borrow checker
                    let (processed_tokens, child_node_ids, joins_performed) = {
                        let join_node = self.join_nodes.get_mut(&node_id).unwrap();
                        let (processed_tokens, joins_performed) =
                            Self::process_token_through_join_node(&current_token, join_node, facts);
                        let child_node_ids = join_node.beta_node.children.clone();
                        (processed_tokens, child_node_ids, joins_performed)
                    };

                    // Update statistics
                    self.total_joins_performed += joins_performed;

                    // Add resulting tokens to processing queue
                    for new_token in processed_tokens {
                        // Store token in beta memory
                        if let Some(memory) = self.beta_memories.get_mut(&node_id) {
                            memory.add_token(new_token.clone());
                        }

                        // Pass to children if not complete
                        if !self.is_token_complete(&new_token, facts) {
                            for &_child_id in &child_node_ids {
                                tokens_to_process.push(new_token.clone());
                            }
                        } else {
                            completed_tokens.push(new_token);
                            self.total_activations += 1;
                        }
                    }
                }
            }
        }

        debug!(
            "Beta network processing completed: {} tokens out",
            completed_tokens.len()
        );

        // Return the tokens_to_process vector to the pool before returning results
        self.memory_pools.token_vecs.return_vec(tokens_to_process);

        // Convert completed_tokens from pooled to regular Vec for return
        let result = completed_tokens.clone();
        self.memory_pools.token_vecs.return_vec(completed_tokens);
        result
    }

    /// Find the current beta node for a token based on its progress
    fn find_current_beta_node(&self, token: &Token) -> Option<NodeId> {
        // In a simple implementation, use the root node if token has no facts
        // Otherwise, find the appropriate join node based on condition index
        if token.facts.is_empty() {
            self.root_node_id
        } else {
            // Find join node for the current condition index
            let condition_index = token.condition_index;
            for (&node_id, join_node) in &self.join_nodes {
                if join_node.condition_index == condition_index {
                    return Some(node_id);
                }
            }
            None
        }
    }

    /// Process a token through a specific join node
    fn process_token_through_join_node(
        token: &Token,
        join_node: &mut JoinNode,
        facts: &HashMap<FactId, Fact>,
    ) -> (Vec<Token>, u64) {
        let mut resulting_tokens = Vec::new();
        let mut joins_performed = 0;

        // For each fact in alpha memory that matches this join node's condition
        // attempt to perform a join with the current token

        // This is a simplified implementation - in a full RETE network,
        // we would get facts from the specific alpha memory associated
        // with this join node
        for &fact_id in facts.keys() {
            if let Some(joined_token) = join_node.perform_join(token, fact_id, facts) {
                resulting_tokens.push(joined_token);
                joins_performed += 1;
            }
        }

        (resulting_tokens, joins_performed)
    }

    /// Check if a token is complete (has satisfied all conditions)
    fn is_token_complete(&self, token: &Token, _facts: &HashMap<FactId, Fact>) -> bool {
        // A token is complete when it has reached a terminal node in the beta network.
        // We determine this by finding the current beta node and checking if it's terminal.
        if let Some(current_node_id) = self.find_current_beta_node(token) {
            if let Some(node) = self.beta_nodes.get(&current_node_id) {
                matches!(node.node_type, BetaNodeType::Terminal { .. })
            } else {
                // If we can't find the node, assume incomplete
                false
            }
        } else {
            // If we can't determine current position, assume incomplete
            false
        }
    }

    /// Retract tokens containing a specific fact
    pub fn retract_tokens_containing_fact(&mut self, fact_id: FactId) -> usize {
        let mut tokens_removed = 0;

        // Remove tokens from all beta memories
        for memory in self.beta_memories.values_mut() {
            let initial_count = memory.token_count();

            // Find tokens containing this fact
            let tokens_to_remove: Vec<String> = memory
                .tokens
                .iter()
                .filter(|(_, token)| token.facts.contains(&fact_id))
                .map(|(key, _)| key.clone())
                .collect();

            // Remove the tokens
            for key in tokens_to_remove {
                memory.tokens.remove(&key);
                memory.tokens_removed += 1;
            }

            tokens_removed += initial_count - memory.token_count();
        }

        // Remove tokens from beta nodes
        for node in self.beta_nodes.values_mut() {
            // For nodes that store tokens directly, we would remove them here
            // This is a simplified approach
            node.clear_tokens();
        }

        for join_node in self.join_nodes.values_mut() {
            join_node.beta_node.clear_tokens();
        }

        debug!(
            "Retracted {} tokens containing fact {}",
            tokens_removed, fact_id
        );
        tokens_removed
    }

    /// Process fact addition through beta network
    pub fn process_fact_addition(
        &mut self,
        fact_id: FactId,
        _fact: &Fact,
        matching_rules: &[RuleId],
    ) -> Vec<Token> {
        let mut new_tokens = Vec::new();

        // For each rule that might be affected by this fact
        for &rule_id in matching_rules {
            // Create a new root token for this rule
            let mut root_token = Token::new(rule_id);
            root_token.facts.push(fact_id);
            root_token.condition_index = 1; // First condition matched

            new_tokens.push(root_token);
        }

        debug!(
            "Created {} new tokens for fact {} addition",
            new_tokens.len(),
            fact_id
        );
        new_tokens
    }

    /// Get detailed beta network state for debugging
    pub fn get_network_state(&self) -> BetaNetworkState {
        BetaNetworkState {
            total_nodes: self.beta_nodes.len() + self.join_nodes.len(),
            total_memories: self.beta_memories.len(),
            total_tokens: self.beta_memories.values().map(|m| m.token_count()).sum(),
            root_node_id: self.root_node_id,
            next_node_id: self.next_node_id,
        }
    }

    /// Get statistics for the beta network
    pub fn get_statistics(&self) -> BetaNetworkStats {
        let beta_node_stats: Vec<BetaNodeStats> =
            self.beta_nodes.values().map(|node| node.get_stats()).collect();

        let join_node_stats: Vec<BetaNodeStats> =
            self.join_nodes.values().map(|node| node.beta_node.get_stats()).collect();

        let beta_memory_stats: Vec<BetaMemoryStats> =
            self.beta_memories.values().map(|memory| memory.get_stats()).collect();

        BetaNetworkStats {
            total_beta_nodes: self.beta_nodes.len() + self.join_nodes.len(),
            total_tokens_processed: self.total_tokens_processed,
            total_joins_performed: self.total_joins_performed,
            total_activations: self.total_activations,
            beta_node_stats,
            join_node_stats,
            beta_memory_stats,
        }
    }

    /// Clear all tokens from the network
    pub fn clear_all_tokens(&mut self) {
        for memory in self.beta_memories.values_mut() {
            memory.clear();
        }

        for node in self.beta_nodes.values_mut() {
            node.clear_tokens();
        }

        for join_node in self.join_nodes.values_mut() {
            join_node.beta_node.clear_tokens();
        }
    }

    /// Get memory usage estimate
    pub fn estimate_memory_usage(&self) -> usize {
        let mut total_size = std::mem::size_of::<Self>();

        // Beta nodes
        total_size += self.beta_nodes.len() * std::mem::size_of::<BetaNode>();

        // Join nodes
        total_size += self.join_nodes.len() * std::mem::size_of::<JoinNode>();

        // Beta memories
        for memory in self.beta_memories.values() {
            total_size += memory.tokens.len() * std::mem::size_of::<Token>();
        }

        total_size
    }
}

impl Default for BetaNetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive statistics for the beta network
#[derive(Debug, Clone)]
pub struct BetaNetworkStats {
    pub total_beta_nodes: usize,
    pub total_tokens_processed: u64,
    pub total_joins_performed: u64,
    pub total_activations: u64,
    pub beta_node_stats: Vec<BetaNodeStats>,
    pub join_node_stats: Vec<BetaNodeStats>,
    pub beta_memory_stats: Vec<BetaMemoryStats>,
}

/// Beta network state for debugging and monitoring
#[derive(Debug, Clone)]
pub struct BetaNetworkState {
    pub total_nodes: usize,
    pub total_memories: usize,
    pub total_tokens: usize,
    pub root_node_id: Option<NodeId>,
    pub next_node_id: NodeId,
}

impl std::fmt::Display for BetaNetworkStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Beta Network Statistics ===")?;
        writeln!(f, "Total Beta Nodes: {}", self.total_beta_nodes)?;
        writeln!(f, "Total Tokens Processed: {}", self.total_tokens_processed)?;
        writeln!(f, "Total Joins Performed: {}", self.total_joins_performed)?;
        writeln!(f, "Total Activations: {}", self.total_activations)?;

        if self.total_tokens_processed > 0 {
            let activation_rate =
                (self.total_activations as f64 / self.total_tokens_processed as f64) * 100.0;
            writeln!(f, "Activation Rate: {activation_rate:.2}%")?;
        }

        writeln!(f, "\nTop Beta Node Usage:")?;
        let mut sorted_nodes = self.beta_node_stats.clone();
        sorted_nodes.sort_by(|a, b| b.tokens_processed.cmp(&a.tokens_processed));

        for (i, stats) in sorted_nodes.iter().take(5).enumerate() {
            writeln!(
                f,
                "  {}. Node {} -> {} tokens processed ({:.1}% success rate)",
                i + 1,
                stats.id,
                stats.tokens_processed,
                stats.join_success_rate
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FactData;
    use std::collections::HashMap;

    fn create_test_fact(id: FactId, user_id: i64, status: &str) -> Fact {
        let mut fields = HashMap::new();
        fields.insert("user_id".to_string(), FactValue::Integer(user_id));
        fields.insert("status".to_string(), FactValue::String(status.to_string()));

        Fact {
            id,
            external_id: Some(format!("fact_{id}")),
            timestamp: chrono::Utc::now(),
            data: FactData { fields },
        }
    }

    #[test]
    fn test_token_creation() {
        let token = Token::new(1);
        assert_eq!(token.rule_id, 1);
        assert_eq!(token.facts.len(), 0);
        assert_eq!(token.condition_index, 0);
        assert!(token.parent_token.is_none());
    }

    #[test]
    fn test_token_extension() {
        let token = Token::new(1);
        let extended = token.extend(42);

        assert_eq!(extended.rule_id, 1);
        assert_eq!(extended.facts.len(), 1);
        assert_eq!(extended.facts[0], 42);
        assert_eq!(extended.condition_index, 1);
        assert!(extended.parent_token.is_some());
    }

    #[test]
    fn test_beta_node_creation() {
        let node = BetaNode::new(1, BetaNodeType::Root);
        assert_eq!(node.id, 1);
        assert!(matches!(node.node_type, BetaNodeType::Root));
        assert_eq!(node.children.len(), 0);
        assert!(node.parent.is_none());
    }

    #[test]
    fn test_join_node_creation() {
        let join_node = JoinNode::new(1, 2, 0);
        assert_eq!(join_node.beta_node.id, 1);
        assert_eq!(join_node.alpha_memory_id, 2);
        assert_eq!(join_node.condition_index, 0);
        assert_eq!(join_node.join_tests.len(), 0);
    }

    #[test]
    fn test_join_test_evaluation() {
        let mut facts = HashMap::new();
        let fact1 = create_test_fact(1, 100, "active");
        let fact2 = create_test_fact(2, 100, "inactive");
        facts.insert(1, fact1);
        facts.insert(2, fact2);

        let mut token = Token::new(1);
        token.facts.push(1); // Add first fact to token

        let join_test = JoinTest::new(
            "user_id".to_string(),
            "user_id".to_string(),
            0,
            JoinOperator::Equal,
        );

        // Should match because both facts have user_id = 100
        assert!(join_test.evaluate(&token, 2, &facts));

        // Create a fact with different user_id
        let fact3 = create_test_fact(3, 200, "active");
        facts.insert(3, fact3);

        // Should not match because user_ids are different
        assert!(!join_test.evaluate(&token, 3, &facts));
    }

    #[test]
    fn test_beta_memory_operations() {
        let mut memory = BetaMemory::new();
        let token1 = Token::new(1);
        let token2 = Token::new(2);

        // Test adding tokens
        assert!(memory.add_token(token1.clone()));
        assert!(!memory.add_token(token1.clone())); // duplicate
        assert!(memory.add_token(token2.clone()));
        assert_eq!(memory.token_count(), 2);

        // Test removing tokens
        assert!(memory.remove_token(&token1));
        assert!(!memory.remove_token(&token1)); // already removed
        assert_eq!(memory.token_count(), 1);

        // Test contains
        assert!(memory.contains_token(&token2));
        assert!(!memory.contains_token(&token1));
    }

    #[test]
    fn test_beta_network_manager() {
        let mut manager = BetaNetworkManager::new();

        // Create root node
        let root_id = manager.create_root_node();
        assert_eq!(manager.root_node_id, Some(root_id));

        // Create join node
        let join_id = manager.create_join_node(10, 0);
        assert!(manager.join_nodes.contains_key(&join_id));

        // Create terminal node
        let terminal_id = manager.create_terminal_node(1);
        assert!(manager.beta_nodes.contains_key(&terminal_id));

        // Connect nodes
        manager.connect_nodes(root_id, join_id);
        manager.connect_nodes(join_id, terminal_id);

        // Verify connections
        let root_node = manager.beta_nodes.get(&root_id).unwrap();
        assert!(root_node.children.contains(&join_id));

        let join_node = manager.join_nodes.get(&join_id).unwrap();
        assert_eq!(join_node.beta_node.parent, Some(root_id));
        assert!(join_node.beta_node.children.contains(&terminal_id));
    }

    #[test]
    fn test_join_operators() {
        let value1 = FactValue::Integer(100);
        let value2 = FactValue::Integer(200);
        let value3 = FactValue::Integer(100);

        assert!(JoinOperator::Equal.compare(&value1, &value3));
        assert!(!JoinOperator::Equal.compare(&value1, &value2));

        assert!(JoinOperator::LessThan.compare(&value1, &value2));
        assert!(!JoinOperator::LessThan.compare(&value2, &value1));

        assert!(JoinOperator::GreaterThan.compare(&value2, &value1));
        assert!(!JoinOperator::GreaterThan.compare(&value1, &value2));
    }
}
