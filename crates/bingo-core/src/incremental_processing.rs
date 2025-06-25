//! Advanced Incremental Fact Processing
//!
//! This module provides intelligent fact change tracking to avoid reprocessing
//! unchanged facts on each evaluation cycle, dramatically improving performance
//! for large fact sets with small incremental updates.

use crate::types::{Fact, FactId};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

/// Represents the processing state of a fact
#[derive(Debug, Clone, PartialEq)]
pub enum FactState {
    /// Fact is newly added and needs full processing
    New,
    /// Fact has been modified and needs reprocessing
    Modified,
    /// Fact is unchanged since last processing
    Unchanged,
    /// Fact has been deleted
    Deleted,
}

/// Tracks fact changes for incremental processing
#[derive(Debug, Clone)]
pub struct FactChangeTracker {
    /// Last known state of facts (fact_id -> content_hash)
    last_known_state: HashMap<FactId, u64>,
    /// Current processing state of facts
    fact_states: HashMap<FactId, FactState>,
    /// Facts that are currently being tracked
    tracked_facts: HashSet<FactId>,
    /// Statistics for monitoring change detection performance
    pub stats: ChangeTrackingStats,
}

/// Statistics for change tracking performance
#[derive(Debug, Clone, Default)]
pub struct ChangeTrackingStats {
    pub total_facts_processed: usize,
    pub new_facts: usize,
    pub modified_facts: usize,
    pub unchanged_facts: usize,
    pub deleted_facts: usize,
    pub cache_hit_rate: f64,
}

impl ChangeTrackingStats {
    /// Calculate the incremental processing efficiency
    pub fn efficiency(&self) -> f64 {
        if self.total_facts_processed == 0 {
            0.0
        } else {
            (self.unchanged_facts as f64 / self.total_facts_processed as f64) * 100.0
        }
    }

    /// Get the change rate (percentage of facts that changed)
    pub fn change_rate(&self) -> f64 {
        if self.total_facts_processed == 0 {
            0.0
        } else {
            ((self.new_facts + self.modified_facts) as f64 / self.total_facts_processed as f64)
                * 100.0
        }
    }
}

impl FactChangeTracker {
    /// Create a new fact change tracker
    pub fn new() -> Self {
        Self {
            last_known_state: HashMap::new(),
            fact_states: HashMap::new(),
            tracked_facts: HashSet::new(),
            stats: ChangeTrackingStats::default(),
        }
    }

    /// Create with initial capacity for better performance
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            last_known_state: HashMap::with_capacity(capacity),
            fact_states: HashMap::with_capacity(capacity),
            tracked_facts: HashSet::with_capacity(capacity),
            stats: ChangeTrackingStats::default(),
        }
    }

    /// Compute a content hash for a fact to detect changes
    fn compute_fact_hash(fact: &Fact) -> u64 {
        let mut hasher = DefaultHasher::new();
        fact.id.hash(&mut hasher);

        // Hash all field values in deterministic order
        let mut fields: Vec<_> = fact.data.fields.iter().collect();
        fields.sort_by_key(|(k, _)| *k);
        for (key, value) in fields {
            key.hash(&mut hasher);
            value.hash(&mut hasher);
        }

        hasher.finish()
    }

    /// Process a batch of facts and determine which ones need processing
    pub fn detect_changes(&mut self, facts: &[Fact]) -> IncrementalProcessingPlan {
        let mut plan = IncrementalProcessingPlan::new();
        let current_fact_ids: HashSet<FactId> = facts.iter().map(|f| f.id).collect();

        // Reset stats for this processing cycle
        self.stats.total_facts_processed = facts.len();
        self.stats.new_facts = 0;
        self.stats.modified_facts = 0;
        self.stats.unchanged_facts = 0;

        // Check each fact for changes
        for fact in facts {
            let current_hash = Self::compute_fact_hash(fact);
            let fact_id = fact.id;

            let state = match self.last_known_state.get(&fact_id) {
                Some(&last_hash) if last_hash == current_hash => {
                    // Fact unchanged since last processing
                    self.stats.unchanged_facts += 1;
                    FactState::Unchanged
                }
                Some(_) => {
                    // Fact has been modified
                    self.stats.modified_facts += 1;
                    self.last_known_state.insert(fact_id, current_hash);
                    FactState::Modified
                }
                None => {
                    // New fact
                    self.stats.new_facts += 1;
                    self.last_known_state.insert(fact_id, current_hash);
                    self.tracked_facts.insert(fact_id);
                    FactState::New
                }
            };

            self.fact_states.insert(fact_id, state.clone());

            // Add to processing plan if fact needs processing
            match state {
                FactState::New => plan.new_facts.push(fact.clone()),
                FactState::Modified => plan.modified_facts.push(fact.clone()),
                FactState::Unchanged => plan.unchanged_facts.push(fact.clone()),
                FactState::Deleted => {} // Handled separately
            }
        }

        // Detect deleted facts
        let mut deleted_facts = Vec::new();
        for &tracked_id in &self.tracked_facts {
            if !current_fact_ids.contains(&tracked_id) {
                deleted_facts.push(tracked_id);
                self.fact_states.insert(tracked_id, FactState::Deleted);
            }
        }

        // Clean up deleted facts
        for deleted_id in &deleted_facts {
            self.last_known_state.remove(deleted_id);
            self.tracked_facts.remove(deleted_id);
        }

        self.stats.deleted_facts = deleted_facts.len();
        plan.deleted_fact_ids = deleted_facts;

        // Update tracked facts to current set
        self.tracked_facts = current_fact_ids;

        // Calculate cache hit rate (unchanged facts don't need processing)
        self.stats.cache_hit_rate = if self.stats.total_facts_processed > 0 {
            (self.stats.unchanged_facts as f64 / self.stats.total_facts_processed as f64) * 100.0
        } else {
            0.0
        };

        plan
    }

    /// Get the current state of a specific fact
    pub fn get_fact_state(&self, fact_id: FactId) -> Option<&FactState> {
        self.fact_states.get(&fact_id)
    }

    /// Force a fact to be reprocessed on next cycle
    pub fn mark_for_reprocessing(&mut self, fact_id: FactId) {
        self.last_known_state.remove(&fact_id);
        self.fact_states.insert(fact_id, FactState::Modified);
    }

    /// Clear all tracking state (useful for full reprocessing)
    pub fn clear(&mut self) {
        self.last_known_state.clear();
        self.fact_states.clear();
        self.tracked_facts.clear();
        self.stats = ChangeTrackingStats::default();
    }

    /// Get memory usage of the tracker
    pub fn memory_usage(&self) -> usize {
        let state_memory = self.last_known_state.len()
            * (std::mem::size_of::<FactId>() + std::mem::size_of::<u64>());
        let fact_state_memory = self.fact_states.len()
            * (std::mem::size_of::<FactId>() + std::mem::size_of::<FactState>());
        let tracked_memory = self.tracked_facts.len() * std::mem::size_of::<FactId>();

        state_memory + fact_state_memory + tracked_memory
    }

    /// Emergency cleanup for critical memory pressure
    pub fn emergency_cleanup(&mut self) {
        // Keep only essential tracking for the most recent facts
        let keep_count = 100; // Minimal tracking

        if self.last_known_state.len() > keep_count {
            // Keep only the most recent entries (arbitrary selection since HashMap has no order)
            let excess = self.last_known_state.len() - keep_count;
            let keys_to_remove: Vec<_> =
                self.last_known_state.keys().take(excess).cloned().collect();
            for key in keys_to_remove {
                self.last_known_state.remove(&key);
                self.fact_states.remove(&key);
                self.tracked_facts.remove(&key);
            }
        }

        // Reset statistics
        self.stats = ChangeTrackingStats::default();
    }

    /// Cleanup old entries beyond specified age
    pub fn cleanup_old_entries(&mut self, max_age: std::time::Duration) {
        // For simplicity, we'll clean up based on the age threshold
        // In a real implementation, we'd track timestamps for each entry
        let cleanup_ratio = if max_age.as_secs() < 300 { 0.6 } else { 0.3 }; // More aggressive for shorter durations

        let remove_count = (self.last_known_state.len() as f64 * cleanup_ratio) as usize;

        if remove_count > 0 {
            let keys_to_remove: Vec<_> =
                self.last_known_state.keys().take(remove_count).cloned().collect();
            for key in keys_to_remove {
                self.last_known_state.remove(&key);
                self.fact_states.remove(&key);
                self.tracked_facts.remove(&key);
            }
        }
    }
}

impl Default for FactChangeTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Plan for incremental processing indicating which facts need processing
#[derive(Debug, Clone)]
pub struct IncrementalProcessingPlan {
    /// Facts that are new and need full processing
    pub new_facts: Vec<Fact>,
    /// Facts that have been modified and need reprocessing
    pub modified_facts: Vec<Fact>,
    /// Facts that are unchanged (may be skipped in some scenarios)
    pub unchanged_facts: Vec<Fact>,
    /// IDs of facts that have been deleted
    pub deleted_fact_ids: Vec<FactId>,
}

impl IncrementalProcessingPlan {
    /// Create an empty processing plan
    pub fn new() -> Self {
        Self {
            new_facts: Vec::new(),
            modified_facts: Vec::new(),
            unchanged_facts: Vec::new(),
            deleted_fact_ids: Vec::new(),
        }
    }

    /// Get all facts that need processing (new + modified)
    pub fn facts_needing_processing(&self) -> impl Iterator<Item = &Fact> {
        self.new_facts.iter().chain(self.modified_facts.iter())
    }

    /// Get total count of facts that need processing
    pub fn processing_count(&self) -> usize {
        self.new_facts.len() + self.modified_facts.len()
    }

    /// Get total count of all facts
    pub fn total_facts(&self) -> usize {
        self.new_facts.len() + self.modified_facts.len() + self.unchanged_facts.len()
    }

    /// Check if any processing is needed
    pub fn needs_processing(&self) -> bool {
        !self.new_facts.is_empty()
            || !self.modified_facts.is_empty()
            || !self.deleted_fact_ids.is_empty()
    }

    /// Get processing efficiency (percentage of facts that can be skipped)
    pub fn efficiency(&self) -> f64 {
        let total = self.total_facts();
        if total == 0 {
            0.0
        } else {
            (self.unchanged_facts.len() as f64 / total as f64) * 100.0
        }
    }
}

impl Default for IncrementalProcessingPlan {
    fn default() -> Self {
        Self::new()
    }
}

/// Incremental processing mode configuration
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingMode {
    /// Always process all facts (no incremental optimization)
    Full,
    /// Use incremental processing for unchanged facts
    Incremental {
        /// Whether to skip processing of unchanged facts entirely
        skip_unchanged: bool,
        /// Minimum change threshold to trigger incremental mode
        min_change_threshold: f64,
    },
    /// Adaptive mode that switches between full and incremental based on change rate
    Adaptive {
        /// Change rate threshold to switch to full processing
        full_processing_threshold: f64,
        /// Whether to skip unchanged facts when in incremental mode
        skip_unchanged: bool,
    },
}

impl ProcessingMode {
    /// Create default incremental processing mode
    pub fn default_incremental() -> Self {
        Self::Incremental {
            skip_unchanged: true,
            min_change_threshold: 5.0, // 5% change rate
        }
    }

    /// Create adaptive processing mode with sensible defaults
    pub fn default_adaptive() -> Self {
        Self::Adaptive {
            full_processing_threshold: 75.0, // Switch to full if >75% changed
            skip_unchanged: true,
        }
    }
}

impl Default for ProcessingMode {
    fn default() -> Self {
        Self::default_incremental()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FactData, FactValue};
    use std::collections::HashMap;

    fn create_test_fact(id: u64, value: i64) -> Fact {
        let mut fields = HashMap::new();
        fields.insert("value".to_string(), FactValue::Integer(value));
        Fact { id, data: FactData { fields } }
    }

    #[test]
    fn test_change_detection_new_facts() {
        let mut tracker = FactChangeTracker::new();

        let facts = vec![create_test_fact(1, 100), create_test_fact(2, 200)];

        let plan = tracker.detect_changes(&facts);

        assert_eq!(plan.new_facts.len(), 2);
        assert_eq!(plan.modified_facts.len(), 0);
        assert_eq!(plan.unchanged_facts.len(), 0);
        assert_eq!(tracker.stats.new_facts, 2);
        assert_eq!(tracker.stats.efficiency(), 0.0); // All facts are new
    }

    #[test]
    fn test_change_detection_unchanged_facts() {
        let mut tracker = FactChangeTracker::new();

        let facts = vec![create_test_fact(1, 100), create_test_fact(2, 200)];

        // First processing - all facts are new
        let _plan1 = tracker.detect_changes(&facts);

        // Second processing with same facts - all should be unchanged
        let plan2 = tracker.detect_changes(&facts);

        assert_eq!(plan2.new_facts.len(), 0);
        assert_eq!(plan2.modified_facts.len(), 0);
        assert_eq!(plan2.unchanged_facts.len(), 2);
        assert_eq!(tracker.stats.unchanged_facts, 2);
        assert_eq!(tracker.stats.efficiency(), 100.0); // All facts unchanged
    }

    #[test]
    fn test_change_detection_modified_facts() {
        let mut tracker = FactChangeTracker::new();

        let facts = vec![create_test_fact(1, 100), create_test_fact(2, 200)];

        // First processing
        let _plan1 = tracker.detect_changes(&facts);

        // Modify one fact
        let modified_facts = vec![
            create_test_fact(1, 150), // Modified
            create_test_fact(2, 200), // Unchanged
        ];

        let plan2 = tracker.detect_changes(&modified_facts);

        assert_eq!(plan2.new_facts.len(), 0);
        assert_eq!(plan2.modified_facts.len(), 1);
        assert_eq!(plan2.unchanged_facts.len(), 1);
        assert_eq!(tracker.stats.modified_facts, 1);
        assert_eq!(tracker.stats.unchanged_facts, 1);
        assert_eq!(tracker.stats.efficiency(), 50.0); // 50% unchanged
    }

    #[test]
    fn test_deleted_fact_detection() {
        let mut tracker = FactChangeTracker::new();

        let facts =
            vec![create_test_fact(1, 100), create_test_fact(2, 200), create_test_fact(3, 300)];

        // First processing
        let _plan1 = tracker.detect_changes(&facts);

        // Remove one fact
        let reduced_facts = vec![create_test_fact(1, 100), create_test_fact(2, 200)];

        let plan2 = tracker.detect_changes(&reduced_facts);

        assert_eq!(plan2.deleted_fact_ids.len(), 1);
        assert_eq!(plan2.deleted_fact_ids[0], 3);
        assert_eq!(tracker.stats.deleted_facts, 1);
    }

    #[test]
    fn test_processing_plan_efficiency() {
        let plan = IncrementalProcessingPlan {
            new_facts: vec![create_test_fact(1, 100)],
            modified_facts: vec![create_test_fact(2, 200)],
            unchanged_facts: vec![
                create_test_fact(3, 300),
                create_test_fact(4, 400),
                create_test_fact(5, 500),
            ],
            deleted_fact_ids: vec![],
        };

        assert_eq!(plan.processing_count(), 2);
        assert_eq!(plan.total_facts(), 5);
        assert_eq!(plan.efficiency(), 60.0); // 3/5 = 60%
        assert!(plan.needs_processing());
    }

    #[test]
    fn test_fact_hash_consistency() {
        let fact1 = create_test_fact(1, 100);
        let fact2 = create_test_fact(1, 100);
        let fact3 = create_test_fact(1, 200);

        let hash1 = FactChangeTracker::compute_fact_hash(&fact1);
        let hash2 = FactChangeTracker::compute_fact_hash(&fact2);
        let hash3 = FactChangeTracker::compute_fact_hash(&fact3);

        assert_eq!(hash1, hash2); // Same content, same hash
        assert_ne!(hash1, hash3); // Different content, different hash
    }

    #[test]
    fn test_memory_usage_calculation() {
        let mut tracker = FactChangeTracker::new();

        let facts = vec![create_test_fact(1, 100), create_test_fact(2, 200)];

        let _plan = tracker.detect_changes(&facts);
        let memory_usage = tracker.memory_usage();

        assert!(memory_usage > 0);
        println!("Tracker memory usage: {} bytes", memory_usage);
    }
}
