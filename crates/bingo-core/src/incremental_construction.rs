//! Incremental Network Construction
//!
//! This module implements incremental network construction features that allow
//! the RETE network to be built and optimized dynamically as rules are added
//! and facts are processed, improving performance for large rule sets.

use crate::rete_nodes::NodeId;
use crate::types::{FactId, Rule};
use std::collections::{HashMap, HashSet};

/// Tracks the activation state of nodes in the network
#[derive(Debug, Clone, PartialEq)]
pub enum NodeActivationState {
    /// Node is inactive and not processing any facts
    Inactive,
    /// Node is active and processing facts
    Active,
    /// Node is dormant (was active but no recent activity)
    Dormant,
    /// Node is scheduled for activation when facts arrive
    Pending,
}

/// Incremental construction manager for RETE networks
#[derive(Debug)]
pub struct IncrementalConstructionManager {
    /// Track activation state of all nodes
    node_activation_states: HashMap<NodeId, NodeActivationState>,
    /// Track which facts activated which nodes
    fact_to_nodes: HashMap<FactId, HashSet<NodeId>>,
    /// Track which rules contributed to which nodes
    rule_to_nodes: HashMap<u64, HashSet<NodeId>>,
    /// Track join path statistics for optimization
    join_path_stats: HashMap<(NodeId, NodeId), JoinPathStats>,
    /// Network topology for path optimization
    network_topology: NetworkTopology,
    /// Statistics for monitoring incremental construction performance
    pub stats: IncrementalConstructionStats,
}

/// Statistics for join paths between nodes
#[derive(Debug, Clone)]
pub struct JoinPathStats {
    /// Number of successful joins between these nodes
    pub successful_joins: usize,
    /// Number of failed join attempts
    pub failed_joins: usize,
    /// Average tokens produced per successful join
    pub avg_tokens_produced: f64,
    /// Last time this path was used
    pub last_used: Option<std::time::Instant>,
}

/// Network topology information for optimization
#[derive(Debug)]
pub struct NetworkTopology {
    /// Adjacency list for efficient path finding
    adjacency: HashMap<NodeId, HashSet<NodeId>>,
    /// Reverse adjacency for backward traversal
    reverse_adjacency: HashMap<NodeId, HashSet<NodeId>>,
    /// Node depths from root for optimization
    node_depths: HashMap<NodeId, usize>,
    /// Critical paths that are frequently used
    critical_paths: Vec<Vec<NodeId>>,
}

/// Statistics for incremental construction performance
#[derive(Debug, Clone, Default)]
pub struct IncrementalConstructionStats {
    /// Number of nodes activated lazily
    pub lazy_activations: usize,
    /// Number of nodes deactivated due to inactivity
    pub deactivations: usize,
    /// Number of network paths optimized
    pub path_optimizations: usize,
    /// Number of join paths pruned
    pub paths_pruned: usize,
    /// Time saved through incremental construction (estimated)
    pub time_saved_ms: f64,
    /// Memory saved through lazy activation (estimated bytes)
    pub memory_saved_bytes: usize,
}

impl IncrementalConstructionManager {
    /// Create a new incremental construction manager
    pub fn new() -> Self {
        Self {
            node_activation_states: HashMap::new(),
            fact_to_nodes: HashMap::new(),
            rule_to_nodes: HashMap::new(),
            join_path_stats: HashMap::new(),
            network_topology: NetworkTopology::new(),
            stats: IncrementalConstructionStats::default(),
        }
    }

    /// Register a new node in the incremental construction system
    pub fn register_node(&mut self, node_id: NodeId, initial_state: NodeActivationState) {
        self.node_activation_states.insert(node_id, initial_state);
        self.network_topology.add_node(node_id);
    }

    /// Activate a node when facts arrive that could trigger it
    pub fn activate_node(&mut self, node_id: NodeId, triggered_by_fact: Option<FactId>) -> bool {
        let current_state = self
            .node_activation_states
            .get(&node_id)
            .cloned()
            .unwrap_or(NodeActivationState::Inactive);

        let activated = match current_state {
            NodeActivationState::Inactive
            | NodeActivationState::Dormant
            | NodeActivationState::Pending => {
                self.node_activation_states.insert(node_id, NodeActivationState::Active);
                self.stats.lazy_activations += 1;
                true
            }
            NodeActivationState::Active => false, // Already active
        };

        // Track fact-to-node relationship for optimization
        if let Some(fact_id) = triggered_by_fact {
            self.fact_to_nodes.entry(fact_id).or_insert_with(HashSet::new).insert(node_id);
        }

        activated
    }

    /// Deactivate a node when it's no longer needed
    pub fn deactivate_node(&mut self, node_id: NodeId) -> bool {
        if let Some(state) = self.node_activation_states.get_mut(&node_id) {
            if *state == NodeActivationState::Active {
                *state = NodeActivationState::Dormant;
                self.stats.deactivations += 1;
                return true;
            }
        }
        false
    }

    /// Check if a node is active
    pub fn is_node_active(&self, node_id: NodeId) -> bool {
        matches!(
            self.node_activation_states.get(&node_id),
            Some(NodeActivationState::Active)
        )
    }

    /// Get nodes that should be activated for a given fact
    pub fn get_nodes_for_fact(&self, fact_id: FactId) -> Option<&HashSet<NodeId>> {
        self.fact_to_nodes.get(&fact_id)
    }

    /// Register a rule's contribution to nodes for cleanup tracking
    pub fn register_rule_nodes(&mut self, rule_id: u64, node_ids: &[NodeId]) {
        self.rule_to_nodes.insert(rule_id, node_ids.iter().copied().collect());
    }

    /// Unregister nodes when a rule is removed
    pub fn unregister_rule_nodes(&mut self, rule_id: u64) -> Option<HashSet<NodeId>> {
        self.rule_to_nodes.remove(&rule_id)
    }

    /// Record join statistics for path optimization
    pub fn record_join_attempt(
        &mut self,
        left_node: NodeId,
        right_node: NodeId,
        successful: bool,
        tokens_produced: usize,
    ) {
        let stats =
            self.join_path_stats
                .entry((left_node, right_node))
                .or_insert_with(|| JoinPathStats {
                    successful_joins: 0,
                    failed_joins: 0,
                    avg_tokens_produced: 0.0,
                    last_used: None,
                });

        if successful {
            stats.successful_joins += 1;
            // Update running average
            let total_tokens = stats.avg_tokens_produced * (stats.successful_joins - 1) as f64
                + tokens_produced as f64;
            stats.avg_tokens_produced = total_tokens / stats.successful_joins as f64;
        } else {
            stats.failed_joins += 1;
        }

        stats.last_used = Some(std::time::Instant::now());
    }

    /// Get join path efficiency for optimization decisions
    pub fn get_join_path_efficiency(&self, left_node: NodeId, right_node: NodeId) -> f64 {
        if let Some(stats) = self.join_path_stats.get(&(left_node, right_node)) {
            let total_attempts = stats.successful_joins + stats.failed_joins;
            if total_attempts > 0 {
                stats.successful_joins as f64 / total_attempts as f64
            } else {
                0.0
            }
        } else {
            0.0 // No data available
        }
    }

    /// Optimize network paths based on usage patterns
    pub fn optimize_network_paths(&mut self) -> usize {
        let mut optimizations = 0;

        // Find inefficient paths (low success rate)
        let inefficient_paths: Vec<_> = self
            .join_path_stats
            .iter()
            .filter(|(_, stats)| {
                let total_attempts = stats.successful_joins + stats.failed_joins;
                total_attempts > 10 && // Only optimize paths with sufficient data
                (stats.successful_joins as f64 / total_attempts as f64) < 0.3 // Less than 30% success rate
            })
            .map(|((left, right), _)| (*left, *right))
            .collect();

        // Mark inefficient paths for pruning
        for (left, right) in inefficient_paths {
            self.network_topology.mark_path_for_pruning(left, right);
            optimizations += 1;
        }

        // Identify critical paths for prioritization
        let critical_paths = self.identify_critical_paths();
        self.network_topology.critical_paths = critical_paths;

        self.stats.path_optimizations += optimizations;
        optimizations
    }

    /// Identify critical paths in the network
    fn identify_critical_paths(&self) -> Vec<Vec<NodeId>> {
        let mut critical_paths = Vec::new();

        // Find paths with high usage frequency
        for ((left, right), stats) in &self.join_path_stats {
            if stats.successful_joins > 100 && stats.avg_tokens_produced > 1.0 {
                // This is a high-traffic path
                if let Some(path) = self.network_topology.find_path(*left, *right) {
                    critical_paths.push(path);
                }
            }
        }

        critical_paths
    }

    /// Estimate memory savings from incremental construction
    pub fn estimate_memory_savings(&self) -> usize {
        let inactive_nodes = self
            .node_activation_states
            .values()
            .filter(|state| {
                matches!(
                    state,
                    NodeActivationState::Inactive | NodeActivationState::Dormant
                )
            })
            .count();

        // Estimate average memory per inactive node (rough approximation)
        let avg_memory_per_node = 1024; // bytes
        inactive_nodes * avg_memory_per_node
    }

    /// Get comprehensive stats
    pub fn get_comprehensive_stats(&self) -> IncrementalConstructionStats {
        let mut stats = self.stats.clone();
        stats.memory_saved_bytes = self.estimate_memory_savings();
        stats
    }

    /// Clean up stale data
    pub fn cleanup_stale_data(&mut self, age_threshold: std::time::Duration) {
        let now = std::time::Instant::now();

        // Remove old join path statistics
        self.join_path_stats.retain(|_, stats| {
            stats.last_used.map_or(false, |last_used| {
                now.duration_since(last_used) < age_threshold
            })
        });

        // Clean up fact-to-node mappings for removed facts
        // This would need integration with fact lifecycle management
    }
}

impl NetworkTopology {
    /// Create a new network topology
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
            reverse_adjacency: HashMap::new(),
            node_depths: HashMap::new(),
            critical_paths: Vec::new(),
        }
    }

    /// Add a node to the topology
    pub fn add_node(&mut self, node_id: NodeId) {
        self.adjacency.entry(node_id).or_insert_with(HashSet::new);
        self.reverse_adjacency.entry(node_id).or_insert_with(HashSet::new);
        self.node_depths.insert(node_id, 0); // Will be updated when edges are added
    }

    /// Add an edge between nodes
    pub fn add_edge(&mut self, from: NodeId, to: NodeId) {
        self.adjacency.entry(from).or_insert_with(HashSet::new).insert(to);
        self.reverse_adjacency.entry(to).or_insert_with(HashSet::new).insert(from);

        // Update depths
        self.update_node_depths();
    }

    /// Mark a path for pruning
    pub fn mark_path_for_pruning(&mut self, _from: NodeId, _to: NodeId) {
        // In a full implementation, this would mark the path for removal
        // For now, it's a placeholder for the optimization framework
    }

    /// Find a path between two nodes
    pub fn find_path(&self, from: NodeId, to: NodeId) -> Option<Vec<NodeId>> {
        // Simple BFS path finding
        use std::collections::VecDeque;

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = HashMap::new();

        queue.push_back(from);
        visited.insert(from);

        while let Some(current) = queue.pop_front() {
            if current == to {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = to;
                while let Some(&p) = parent.get(&node) {
                    path.push(node);
                    node = p;
                }
                path.push(from);
                path.reverse();
                return Some(path);
            }

            if let Some(neighbors) = self.adjacency.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        parent.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        None
    }

    /// Update node depths based on network structure
    fn update_node_depths(&mut self) {
        // Simple topological depth calculation
        // This would be more sophisticated in a full implementation
        for node_id in self.adjacency.keys() {
            let depth = self.calculate_depth(*node_id);
            self.node_depths.insert(*node_id, depth);
        }
    }

    /// Calculate depth of a node from network roots
    fn calculate_depth(&self, node_id: NodeId) -> usize {
        if let Some(predecessors) = self.reverse_adjacency.get(&node_id) {
            if predecessors.is_empty() {
                0 // Root node
            } else {
                predecessors
                    .iter()
                    .map(|&pred| self.node_depths.get(&pred).unwrap_or(&0) + 1)
                    .max()
                    .unwrap_or(0)
            }
        } else {
            0
        }
    }
}

impl Default for IncrementalConstructionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for NetworkTopology {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_activation() {
        let mut manager = IncrementalConstructionManager::new();

        manager.register_node(1, NodeActivationState::Inactive);
        assert!(!manager.is_node_active(1));

        let activated = manager.activate_node(1, Some(100));
        assert!(activated);
        assert!(manager.is_node_active(1));

        // Activating again should return false
        let activated_again = manager.activate_node(1, Some(101));
        assert!(!activated_again);
    }

    #[test]
    fn test_node_deactivation() {
        let mut manager = IncrementalConstructionManager::new();

        manager.register_node(1, NodeActivationState::Active);
        assert!(manager.is_node_active(1));

        let deactivated = manager.deactivate_node(1);
        assert!(deactivated);
        assert!(!manager.is_node_active(1));
    }

    #[test]
    fn test_join_path_statistics() {
        let mut manager = IncrementalConstructionManager::new();

        // Record successful joins
        manager.record_join_attempt(1, 2, true, 5);
        manager.record_join_attempt(1, 2, true, 3);
        manager.record_join_attempt(1, 2, false, 0);

        let efficiency = manager.get_join_path_efficiency(1, 2);
        assert!((efficiency - 0.6667).abs() < 0.001); // 2/3 success rate

        let stats = manager.join_path_stats.get(&(1, 2)).unwrap();
        assert_eq!(stats.successful_joins, 2);
        assert_eq!(stats.failed_joins, 1);
        assert_eq!(stats.avg_tokens_produced, 4.0); // (5+3)/2
    }

    #[test]
    fn test_network_topology() {
        let mut topology = NetworkTopology::new();

        topology.add_node(1);
        topology.add_node(2);
        topology.add_node(3);

        topology.add_edge(1, 2);
        topology.add_edge(2, 3);

        let path = topology.find_path(1, 3);
        assert_eq!(path, Some(vec![1, 2, 3]));

        let no_path = topology.find_path(3, 1);
        assert_eq!(no_path, None);
    }

    #[test]
    fn test_rule_node_tracking() {
        let mut manager = IncrementalConstructionManager::new();

        let rule_nodes = vec![1, 2, 3];
        manager.register_rule_nodes(100, &rule_nodes);

        let tracked_nodes = manager.rule_to_nodes.get(&100).unwrap();
        assert_eq!(tracked_nodes.len(), 3);
        assert!(tracked_nodes.contains(&1));
        assert!(tracked_nodes.contains(&2));
        assert!(tracked_nodes.contains(&3));

        let unregistered = manager.unregister_rule_nodes(100);
        assert!(unregistered.is_some());
        assert!(manager.rule_to_nodes.get(&100).is_none());
    }
}
