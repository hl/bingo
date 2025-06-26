//! RETE Network Node Sharing Optimization
//!
//! This module implements node sharing to reduce memory usage by reusing
//! identical nodes across multiple rules instead of creating duplicates.

use crate::rete_nodes::{AlphaNode, BetaNode, JoinCondition, NodeId};
use crate::types::Condition;
use crate::unified_memory_coordinator::MemoryConsumer;
use std::collections::HashMap;

/// A canonical representation of an alpha node for sharing purposes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlphaNodeSignature {
    /// The condition this node tests
    condition: Condition,
}

impl AlphaNodeSignature {
    /// Create a signature from an alpha node condition
    pub fn new(condition: Condition) -> Self {
        Self { condition }
    }

    /// Create a signature from an existing alpha node
    pub fn from_alpha_node(node: &AlphaNode) -> Self {
        Self { condition: node.condition.clone() }
    }

    /// Check if this signature can be shared (only simple conditions for now)
    pub fn is_shareable(&self) -> bool {
        matches!(self.condition, Condition::Simple { .. })
    }
}

/// A canonical representation of a beta node for sharing purposes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BetaNodeSignature {
    /// The join conditions this node tests
    join_conditions: Vec<JoinCondition>,
}

impl BetaNodeSignature {
    /// Create a signature from join conditions
    pub fn new(join_conditions: Vec<JoinCondition>) -> Self {
        // Sort join conditions for consistent hashing
        let mut sorted_conditions = join_conditions;
        sorted_conditions.sort_by(|a, b| {
            a.left_field
                .cmp(&b.left_field)
                .then_with(|| a.right_field.cmp(&b.right_field))
                .then_with(|| format!("{:?}", a.operator).cmp(&format!("{:?}", b.operator)))
        });

        Self { join_conditions: sorted_conditions }
    }

    /// Create a signature from an existing beta node
    pub fn from_beta_node(node: &BetaNode) -> Self {
        Self::new(node.join_conditions.clone())
    }

    /// Check if this signature can be shared
    pub fn is_shareable(&self) -> bool {
        // Beta nodes can be shared regardless of whether they have join conditions
        // Empty join conditions are also shareable (simple cross product)
        true
    }
}

/// Node sharing registry to track and reuse identical nodes
#[derive(Debug)]
pub struct NodeSharingRegistry {
    /// Map from alpha node signatures to existing node IDs
    alpha_node_map: HashMap<AlphaNodeSignature, NodeId>,
    /// Map from beta node signatures to existing node IDs
    beta_node_map: HashMap<BetaNodeSignature, NodeId>,
    /// Reference counts for shared nodes
    alpha_ref_counts: HashMap<NodeId, usize>,
    beta_ref_counts: HashMap<NodeId, usize>,
    /// Statistics for monitoring
    pub alpha_shares_found: usize,
    pub beta_shares_found: usize,
    pub alpha_nodes_created: usize,
    pub beta_nodes_created: usize,
}

impl NodeSharingRegistry {
    /// Create a new node sharing registry
    pub fn memory_usage_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.alpha_node_map.capacity() * std::mem::size_of::<(AlphaNodeSignature, NodeId)>()
            + self.beta_node_map.capacity() * std::mem::size_of::<(BetaNodeSignature, NodeId)>()
            + self.alpha_ref_counts.capacity() * std::mem::size_of::<(NodeId, usize)>()
            + self.beta_ref_counts.capacity() * std::mem::size_of::<(NodeId, usize)>()
    }
}

impl MemoryConsumer for NodeSharingRegistry {
    fn memory_usage_bytes(&self) -> usize {
        self.memory_usage_bytes()
    }

    fn reduce_memory_usage(&mut self, _reduction_factor: f64) -> usize {
        // Node sharing registry doesn't have dynamic memory to reduce easily
        // Clearing it would defeat its purpose. No-op for now.
        0
    }

    fn get_stats(&self) -> HashMap<String, f64> {
        let stats = self.get_stats();
        let mut map = HashMap::new();
        map.insert(
            "alpha_nodes_total".to_string(),
            stats.alpha_nodes_total as f64,
        );
        map.insert(
            "beta_nodes_total".to_string(),
            stats.beta_nodes_total as f64,
        );
        map.insert(
            "alpha_shares_found".to_string(),
            stats.alpha_shares_found as f64,
        );
        map.insert(
            "beta_shares_found".to_string(),
            stats.beta_shares_found as f64,
        );
        map.insert(
            "alpha_nodes_active".to_string(),
            stats.alpha_nodes_active as f64,
        );
        map.insert(
            "beta_nodes_active".to_string(),
            stats.beta_nodes_active as f64,
        );
        map.insert("alpha_sharing_rate".to_string(), stats.alpha_sharing_rate);
        map.insert("beta_sharing_rate".to_string(), stats.beta_sharing_rate);
        map.insert(
            "overall_sharing_rate".to_string(),
            stats.overall_sharing_rate(),
        );
        map.insert(
            "nodes_without_sharing".to_string(),
            stats.nodes_without_sharing() as f64,
        );
        map
    }

    fn name(&self) -> &str {
        "NodeSharingRegistry"
    }
}

impl NodeSharingRegistry {
    /// Create a new node sharing registry
    pub fn new() -> Self {
        Self {
            alpha_node_map: HashMap::new(),
            beta_node_map: HashMap::new(),
            alpha_ref_counts: HashMap::new(),
            beta_ref_counts: HashMap::new(),
            alpha_shares_found: 0,
            beta_shares_found: 0,
            alpha_nodes_created: 0,
            beta_nodes_created: 0,
        }
    }

    /// Try to find an existing alpha node with the same signature
    pub fn find_shared_alpha_node(&mut self, condition: &Condition) -> Option<NodeId> {
        let signature = AlphaNodeSignature::new(condition.clone());

        if !signature.is_shareable() {
            return None;
        }

        if let Some(&node_id) = self.alpha_node_map.get(&signature) {
            self.alpha_shares_found += 1;
            // Increment reference count
            *self.alpha_ref_counts.entry(node_id).or_insert(0) += 1;
            Some(node_id)
        } else {
            None
        }
    }

    /// Register a new alpha node for sharing
    pub fn register_alpha_node(&mut self, node_id: NodeId, condition: &Condition) {
        let signature = AlphaNodeSignature::new(condition.clone());

        if signature.is_shareable() {
            self.alpha_node_map.insert(signature, node_id);
            self.alpha_ref_counts.insert(node_id, 1);
        }

        self.alpha_nodes_created += 1;
    }

    /// Try to find an existing beta node with the same signature
    pub fn find_shared_beta_node(&mut self, join_conditions: &[JoinCondition]) -> Option<NodeId> {
        let signature = BetaNodeSignature::new(join_conditions.to_vec());

        if !signature.is_shareable() {
            return None;
        }

        if let Some(&node_id) = self.beta_node_map.get(&signature) {
            self.beta_shares_found += 1;
            // Increment reference count
            *self.beta_ref_counts.entry(node_id).or_insert(0) += 1;
            Some(node_id)
        } else {
            None
        }
    }

    /// Register a new beta node for sharing
    pub fn register_beta_node(&mut self, node_id: NodeId, join_conditions: &[JoinCondition]) {
        let signature = BetaNodeSignature::new(join_conditions.to_vec());

        if signature.is_shareable() {
            self.beta_node_map.insert(signature, node_id);
            self.beta_ref_counts.insert(node_id, 1);
        }

        self.beta_nodes_created += 1;
    }

    /// Decrement reference count when a rule is removed
    pub fn unregister_alpha_node(&mut self, node_id: NodeId) -> bool {
        if let Some(count) = self.alpha_ref_counts.get_mut(&node_id) {
            *count -= 1;
            if *count == 0 {
                self.alpha_ref_counts.remove(&node_id);
                // Remove from signature map
                self.alpha_node_map.retain(|_, &mut id| id != node_id);
                return true; // Node can be deleted
            }
        }
        false // Node still has references
    }

    /// Decrement reference count when a rule is removed
    pub fn unregister_beta_node(&mut self, node_id: NodeId) -> bool {
        if let Some(count) = self.beta_ref_counts.get_mut(&node_id) {
            *count -= 1;
            if *count == 0 {
                self.beta_ref_counts.remove(&node_id);
                // Remove from signature map
                self.beta_node_map.retain(|_, &mut id| id != node_id);
                return true; // Node can be deleted
            }
        }
        false // Node still has references
    }

    /// Get sharing statistics
    pub fn get_stats(&self) -> NodeSharingStats {
        NodeSharingStats {
            alpha_nodes_total: self.alpha_nodes_created,
            beta_nodes_total: self.beta_nodes_created,
            alpha_shares_found: self.alpha_shares_found,
            beta_shares_found: self.beta_shares_found,
            alpha_nodes_active: self.alpha_ref_counts.len(),
            beta_nodes_active: self.beta_ref_counts.len(),
            alpha_sharing_rate: if self.alpha_nodes_created > 0 {
                (self.alpha_shares_found as f64 / self.alpha_nodes_created as f64) * 100.0
            } else {
                0.0
            },
            beta_sharing_rate: if self.beta_nodes_created > 0 {
                (self.beta_shares_found as f64 / self.beta_nodes_created as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Calculate memory savings from node sharing
    pub fn calculate_memory_savings(&self) -> MemorySavings {
        let alpha_memory_saved = self.alpha_shares_found * std::mem::size_of::<AlphaNode>();
        let beta_memory_saved = self.beta_shares_found * std::mem::size_of::<BetaNode>();

        MemorySavings {
            alpha_nodes_saved: self.alpha_shares_found,
            beta_nodes_saved: self.beta_shares_found,
            alpha_memory_saved_bytes: alpha_memory_saved,
            beta_memory_saved_bytes: beta_memory_saved,
            total_memory_saved_bytes: alpha_memory_saved + beta_memory_saved,
        }
    }

    /// Clear all sharing data
    pub fn clear(&mut self) {
        self.alpha_node_map.clear();
        self.beta_node_map.clear();
        self.alpha_ref_counts.clear();
        self.beta_ref_counts.clear();
        self.alpha_shares_found = 0;
        self.beta_shares_found = 0;
        self.alpha_nodes_created = 0;
        self.beta_nodes_created = 0;
    }
}

/// Statistics for node sharing performance
#[derive(Debug, Clone)]
pub struct NodeSharingStats {
    pub alpha_nodes_total: usize,
    pub beta_nodes_total: usize,
    pub alpha_shares_found: usize,
    pub beta_shares_found: usize,
    pub alpha_nodes_active: usize,
    pub beta_nodes_active: usize,
    pub alpha_sharing_rate: f64,
    pub beta_sharing_rate: f64,
}

impl NodeSharingStats {
    /// Get overall sharing efficiency
    pub fn overall_sharing_rate(&self) -> f64 {
        let total_nodes = self.alpha_nodes_total + self.beta_nodes_total;
        let total_shares = self.alpha_shares_found + self.beta_shares_found;

        if total_nodes > 0 {
            (total_shares as f64 / total_nodes as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get total nodes that would have been created without sharing
    pub fn nodes_without_sharing(&self) -> usize {
        self.alpha_nodes_total
            + self.alpha_shares_found
            + self.beta_nodes_total
            + self.beta_shares_found
    }
}

/// Memory savings calculation from node sharing
#[derive(Debug, Clone)]
pub struct MemorySavings {
    pub alpha_nodes_saved: usize,
    pub beta_nodes_saved: usize,
    pub alpha_memory_saved_bytes: usize,
    pub beta_memory_saved_bytes: usize,
    pub total_memory_saved_bytes: usize,
}

impl MemorySavings {
    /// Get memory savings as a human-readable string
    pub fn to_human_readable(&self) -> String {
        let total_kb = self.total_memory_saved_bytes as f64 / 1024.0;
        if total_kb < 1.0 {
            format!("{} bytes", self.total_memory_saved_bytes)
        } else if total_kb < 1024.0 {
            format!("{:.2} KB", total_kb)
        } else {
            format!("{:.2} MB", total_kb / 1024.0)
        }
    }

    /// Get total nodes saved
    pub fn total_nodes_saved(&self) -> usize {
        self.alpha_nodes_saved + self.beta_nodes_saved
    }
}

impl Default for NodeSharingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Condition, FactValue, Operator};

    #[test]
    fn test_alpha_node_signature_creation() {
        let condition = Condition::Simple {
            field: "age".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(18),
        };

        let signature = AlphaNodeSignature::new(condition.clone());
        assert!(signature.is_shareable());
        assert_eq!(signature.condition, condition);
    }

    #[test]
    fn test_beta_node_signature_ordering() {
        let conditions1 = vec![
            JoinCondition {
                left_field: "b".to_string(),
                right_field: "y".to_string(),
                operator: Operator::Equal,
            },
            JoinCondition {
                left_field: "a".to_string(),
                right_field: "x".to_string(),
                operator: Operator::Equal,
            },
        ];

        let conditions2 = vec![
            JoinCondition {
                left_field: "a".to_string(),
                right_field: "x".to_string(),
                operator: Operator::Equal,
            },
            JoinCondition {
                left_field: "b".to_string(),
                right_field: "y".to_string(),
                operator: Operator::Equal,
            },
        ];

        let sig1 = BetaNodeSignature::new(conditions1);
        let sig2 = BetaNodeSignature::new(conditions2);

        // Should be equal despite different input order
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_node_sharing_registry_alpha_nodes() {
        let mut registry = NodeSharingRegistry::new();

        let condition = Condition::Simple {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("active".to_string()),
        };

        // First lookup should find nothing
        assert!(registry.find_shared_alpha_node(&condition).is_none());

        // Register a node
        registry.register_alpha_node(1, &condition);

        // Second lookup should find the registered node
        assert_eq!(registry.find_shared_alpha_node(&condition), Some(1));

        // Third lookup should also find it (and increment stats)
        assert_eq!(registry.find_shared_alpha_node(&condition), Some(1));

        let stats = registry.get_stats();
        assert_eq!(stats.alpha_nodes_total, 1);
        assert_eq!(stats.alpha_shares_found, 2);
        assert_eq!(stats.alpha_sharing_rate, 200.0); // 2 shares / 1 created * 100
    }

    #[test]
    fn test_node_sharing_registry_beta_nodes() {
        let mut registry = NodeSharingRegistry::new();

        let join_conditions = vec![JoinCondition {
            left_field: "user_id".to_string(),
            right_field: "user_id".to_string(),
            operator: Operator::Equal,
        }];

        // First lookup should find nothing
        assert!(registry.find_shared_beta_node(&join_conditions).is_none());

        // Register a node
        registry.register_beta_node(2, &join_conditions);

        // Second lookup should find the registered node
        assert_eq!(registry.find_shared_beta_node(&join_conditions), Some(2));

        let stats = registry.get_stats();
        assert_eq!(stats.beta_nodes_total, 1);
        assert_eq!(stats.beta_shares_found, 1);
    }

    #[test]
    fn test_reference_counting() {
        let mut registry = NodeSharingRegistry::new();

        let condition = Condition::Simple {
            field: "type".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("test".to_string()),
        };

        // Register and share a node multiple times
        registry.register_alpha_node(1, &condition);
        registry.find_shared_alpha_node(&condition); // ref count = 2
        registry.find_shared_alpha_node(&condition); // ref count = 3

        // Unregister should not delete until ref count reaches 0
        assert!(!registry.unregister_alpha_node(1)); // ref count = 2
        assert!(!registry.unregister_alpha_node(1)); // ref count = 1
        assert!(registry.unregister_alpha_node(1)); // ref count = 0, can delete
    }

    #[test]
    fn test_memory_savings_calculation() {
        let mut registry = NodeSharingRegistry::new();

        // Add some sharing activity
        registry.alpha_shares_found = 5;
        registry.beta_shares_found = 3;

        let savings = registry.calculate_memory_savings();
        assert_eq!(savings.alpha_nodes_saved, 5);
        assert_eq!(savings.beta_nodes_saved, 3);
        assert_eq!(savings.total_nodes_saved(), 8);
        assert!(savings.total_memory_saved_bytes > 0);

        // Test human readable format
        let readable = savings.to_human_readable();
        assert!(readable.contains("bytes") || readable.contains("KB") || readable.contains("MB"));
    }

    #[test]
    fn test_sharing_statistics() {
        let mut registry = NodeSharingRegistry::new();

        registry.alpha_nodes_created = 10;
        registry.beta_nodes_created = 5;
        registry.alpha_shares_found = 3;
        registry.beta_shares_found = 2;

        let stats = registry.get_stats();
        assert_eq!(stats.alpha_sharing_rate, 30.0); // 3/10 * 100
        assert_eq!(stats.beta_sharing_rate, 40.0); // 2/5 * 100
        assert_eq!(stats.overall_sharing_rate(), (5.0 / 15.0) * 100.0); // (3+2)/(10+5) * 100
        assert_eq!(stats.nodes_without_sharing(), 20); // 10+3+5+2
    }

    #[test]
    fn test_clear_registry() {
        let mut registry = NodeSharingRegistry::new();

        let condition = Condition::Simple {
            field: "test".to_string(),
            operator: Operator::Equal,
            value: FactValue::Boolean(true),
        };

        registry.register_alpha_node(1, &condition);
        registry.find_shared_alpha_node(&condition);

        assert!(registry.get_stats().alpha_nodes_total > 0);

        registry.clear();

        let stats = registry.get_stats();
        assert_eq!(stats.alpha_nodes_total, 0);
        assert_eq!(stats.alpha_shares_found, 0);
        assert!(registry.find_shared_alpha_node(&condition).is_none());
    }
}
