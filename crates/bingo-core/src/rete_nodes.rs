use crate::calculator::{CalculatorResult, EvaluationContext};
use crate::calculator_cache::CachedCalculator;
use crate::types::{Action, ActionType, Condition, Fact, FactId, FactValue, Operator};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};

/// Unique identifier for nodes in the RETE network
pub type NodeId = u64;

/// Interned fact ID collection for shared token storage
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactIdSet {
    ids: Arc<Vec<FactId>>,
}

impl FactIdSet {
    /// Create a new fact ID set from a vector
    pub fn new(ids: Vec<FactId>) -> Self {
        Self { ids: Arc::new(ids) }
    }

    /// Create from a single fact ID
    pub fn single(id: FactId) -> Self {
        Self { ids: Arc::new(vec![id]) }
    }

    /// Get the fact IDs as a slice
    pub fn as_slice(&self) -> &[FactId] {
        &self.ids
    }

    /// Get an iterator over fact IDs
    pub fn iter(&self) -> std::slice::Iter<'_, FactId> {
        self.ids.iter()
    }

    /// Extend with another FactIdSet
    pub fn extend(&mut self, other: &FactIdSet) {
        // Since FactIdSet uses Arc<Vec<FactId>>, we need to create a new vector
        // In a more optimized implementation, this could use a more efficient data structure
        let mut new_ids = Vec::with_capacity(self.ids.len() + other.ids.len());
        new_ids.extend_from_slice(&self.ids);
        new_ids.extend_from_slice(&other.ids);
        self.ids = Arc::new(new_ids);
    }

    /// Join with another fact ID to create a new set
    pub fn join(&self, id: FactId) -> Self {
        let mut new_ids = Vec::with_capacity(self.ids.len() + 1);
        new_ids.extend_from_slice(&self.ids);
        new_ids.push(id);
        Self::new(new_ids)
    }

    /// Join with multiple fact IDs to create a new set
    pub fn join_many(&self, other_ids: &[FactId]) -> Self {
        let mut new_ids = Vec::with_capacity(self.ids.len() + other_ids.len());
        new_ids.extend_from_slice(&self.ids);
        new_ids.extend_from_slice(other_ids);
        Self::new(new_ids)
    }

    /// Join two fact ID sets
    pub fn join_sets(&self, other: &FactIdSet) -> Self {
        let mut new_ids = Vec::with_capacity(self.ids.len() + other.ids.len());
        new_ids.extend_from_slice(&self.ids);
        new_ids.extend_from_slice(&other.ids);
        Self::new(new_ids)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Check if the set contains a specific fact ID
    pub fn contains(&self, fact_id: &FactId) -> bool {
        self.ids.contains(fact_id)
    }
}

impl serde::Serialize for FactIdSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.ids.as_slice().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for FactIdSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ids = Vec::<FactId>::deserialize(deserializer)?;
        Ok(FactIdSet::new(ids))
    }
}

/// Lightweight reference to a fact for token propagation with memory sharing
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Token {
    pub fact_ids: FactIdSet,
}

impl Token {
    /// Create a new token with a single fact ID
    pub fn new(fact_id: FactId) -> Self {
        Self { fact_ids: FactIdSet::single(fact_id) }
    }

    /// Create a token from multiple fact IDs
    pub fn from_facts(fact_ids: impl Into<Vec<FactId>>) -> Self {
        Self { fact_ids: FactIdSet::new(fact_ids.into()) }
    }

    /// Join this token with another fact ID
    pub fn join(&self, fact_id: FactId) -> Self {
        Self { fact_ids: self.fact_ids.join(fact_id) }
    }

    /// Join this token with multiple fact IDs
    pub fn join_many(&self, other_fact_ids: &[FactId]) -> Self {
        Self { fact_ids: self.fact_ids.join_many(other_fact_ids) }
    }

    /// Join two tokens together
    pub fn join_tokens(&self, other: &Token) -> Self {
        Self { fact_ids: self.fact_ids.join_sets(&other.fact_ids) }
    }
}

/// Configuration for token pool adaptive behavior
#[derive(Debug, Clone)]
pub struct TokenPoolConfig {
    pub max_single_tokens: usize,
    pub max_multi_tokens: usize,
    pub resize_threshold: f64,
    pub burst_detection_window: std::time::Duration,
}

impl Default for TokenPoolConfig {
    fn default() -> Self {
        Self {
            max_single_tokens: 1000,
            max_multi_tokens: 500,
            resize_threshold: 0.8,
            burst_detection_window: std::time::Duration::from_millis(100),
        }
    }
}

/// Tracks allocation rate for adaptive pool sizing
#[derive(Debug)]
pub struct AllocationRateTracker {
    recent_allocations: Vec<std::time::Instant>,
    window_size: std::time::Duration,
}

impl AllocationRateTracker {
    pub fn new(window_size: std::time::Duration) -> Self {
        Self { recent_allocations: Vec::new(), window_size }
    }

    pub fn record_allocation(&mut self) {
        let now = std::time::Instant::now();
        self.recent_allocations.push(now);

        // Remove old allocations outside the window
        let cutoff = now - self.window_size;
        self.recent_allocations.retain(|&time| time >= cutoff);
    }

    pub fn current_rate(&self) -> f64 {
        self.recent_allocations.len() as f64 / self.window_size.as_secs_f64()
    }
}

/// Shared token pool for memory optimization across all nodes
#[derive(Debug)]
pub struct TokenPool {
    /// Pool of reusable single-fact tokens
    single_tokens: Vec<Token>,
    /// Pool of reusable multi-fact tokens
    multi_tokens: Vec<Token>,
    /// Configuration for adaptive behavior
    config: TokenPoolConfig,
    /// Statistics for monitoring
    pub allocated_count: usize,
    pub returned_count: usize,
    pub pool_hits: usize,
    pub pool_misses: usize,
    /// Enhanced performance metrics
    allocation_requests_since_resize: usize,
    last_resize_time: std::time::Instant,
    peak_allocation_burst: usize,
    current_burst_count: usize,
    allocation_rate_tracker: AllocationRateTracker,
}

impl TokenPool {
    /// Create a new token pool with initial capacity
    pub fn new(capacity: usize) -> Self {
        let config = TokenPoolConfig::default();
        Self {
            single_tokens: Vec::with_capacity(capacity),
            multi_tokens: Vec::with_capacity(capacity / 2), // Multi-token pool is smaller
            config: config.clone(),
            allocated_count: 0,
            returned_count: 0,
            pool_hits: 0,
            pool_misses: 0,
            allocation_requests_since_resize: 0,
            last_resize_time: std::time::Instant::now(),
            peak_allocation_burst: 0,
            current_burst_count: 0,
            allocation_rate_tracker: AllocationRateTracker::new(config.burst_detection_window),
        }
    }

    /// Estimate memory usage in bytes
    pub fn memory_usage_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.single_tokens.capacity() * std::mem::size_of::<Token>()
            + self.multi_tokens.capacity() * std::mem::size_of::<Token>()
            + self.allocation_rate_tracker.recent_allocations.capacity()
                * std::mem::size_of::<std::time::Instant>()
    }

    /// Get a token for a single fact ID (reuse if possible)
    pub fn get_single_token(&mut self, fact_id: FactId) -> Token {
        self.allocation_rate_tracker.record_allocation();
        self.allocated_count += 1;
        self.current_burst_count += 1;

        if let Some(mut token) = self.single_tokens.pop() {
            // Reuse existing token by replacing its fact_ids
            token.fact_ids = FactIdSet::single(fact_id);
            self.pool_hits += 1;
            token
        } else {
            self.pool_misses += 1;
            Token::new(fact_id)
        }
    }

    /// Return a token to the pool for reuse
    pub fn return_token(&mut self, token: Token) {
        self.current_burst_count = self.current_burst_count.saturating_sub(1);

        // Only pool single-fact tokens for now to keep it simple
        if token.fact_ids.len() == 1 && self.single_tokens.len() < self.config.max_single_tokens {
            self.single_tokens.push(token);
            self.returned_count += 1;
        } else if token.fact_ids.len() > 1 && self.multi_tokens.len() < self.config.max_multi_tokens
        {
            self.multi_tokens.push(token);
            self.returned_count += 1;
        }

        // Check if we need to resize the pool based on utilization
        self.allocation_requests_since_resize += 1;
        if self.allocation_requests_since_resize >= 1000 {
            self.maybe_resize_pool();
        }
    }

    /// Clear the pool
    pub fn clear(&mut self) {
        self.single_tokens.clear();
        self.multi_tokens.clear();
        self.allocated_count = 0;
        self.returned_count = 0;
        self.pool_hits = 0;
        self.pool_misses = 0;
        self.allocation_requests_since_resize = 0;
        self.current_burst_count = 0;
        self.peak_allocation_burst = 0;
    }

    /// Maybe resize the pool based on current utilization patterns
    fn maybe_resize_pool(&mut self) {
        let now = std::time::Instant::now();
        let time_since_last_resize = now.duration_since(self.last_resize_time);

        // Only resize if enough time has passed since last resize
        if time_since_last_resize < std::time::Duration::from_secs(10) {
            return;
        }

        let hit_rate = self.utilization();
        let allocation_rate = self.allocation_rate_tracker.current_rate();

        // Update peak burst tracking
        if self.current_burst_count > self.peak_allocation_burst {
            self.peak_allocation_burst = self.current_burst_count;
        }

        // Adaptive resizing based on performance benchmarks
        let should_increase = hit_rate < 20.0 && allocation_rate > 10000.0; // High allocation rate, low hit rate
        let should_decrease =
            hit_rate > 80.0 && self.single_tokens.capacity() > self.peak_allocation_burst * 2;

        if should_increase {
            // Increase pool size - benchmarks showed that larger pools (5000) are optimal
            let new_capacity = (self.config.max_single_tokens * 2).min(10_000);
            if new_capacity > self.config.max_single_tokens {
                self.config.max_single_tokens = new_capacity;
                self.config.max_multi_tokens = (new_capacity / 2).max(500);
                self.single_tokens.reserve(new_capacity - self.single_tokens.capacity());

                tracing::debug!(
                    old_capacity = self.single_tokens.capacity(),
                    new_capacity = new_capacity,
                    hit_rate = hit_rate,
                    allocation_rate = allocation_rate,
                    "Increased token pool capacity"
                );
            }
        } else if should_decrease {
            // Decrease pool size to save memory
            let new_capacity = (self.config.max_single_tokens / 2).max(1000);
            if new_capacity < self.config.max_single_tokens {
                self.config.max_single_tokens = new_capacity;
                self.config.max_multi_tokens = (new_capacity / 2).max(100);

                // Trim excess tokens
                self.single_tokens.truncate(new_capacity);

                tracing::debug!(
                    old_capacity = self.single_tokens.capacity(),
                    new_capacity = new_capacity,
                    hit_rate = hit_rate,
                    "Decreased token pool capacity"
                );
            }
        }

        self.last_resize_time = now;
        self.allocation_requests_since_resize = 0;
    }

    /// Get comprehensive pool statistics for monitoring and tuning
    pub fn get_comprehensive_stats(&self) -> TokenPoolComprehensiveStats {
        TokenPoolComprehensiveStats {
            pool_hits: self.pool_hits,
            pool_misses: self.pool_misses,
            utilization: self.utilization(),
            allocated_count: self.allocated_count,
            returned_count: self.returned_count,
            current_single_pool_size: self.single_tokens.len(),
            max_single_pool_size: self.config.max_single_tokens,
            current_multi_pool_size: self.multi_tokens.len(),
            max_multi_pool_size: self.config.max_multi_tokens,
            allocation_rate: self.allocation_rate_tracker.current_rate(),
            peak_allocation_burst: self.peak_allocation_burst,
            current_burst_count: self.current_burst_count,
            memory_usage_bytes: self.memory_usage_bytes(),
        }
    }

    /// Create a token pool with optimal settings based on performance benchmarks
    pub fn with_optimal_settings() -> Self {
        // Based on benchmark results: size 5000 achieved highest score (0.70)
        let config = TokenPoolConfig {
            max_single_tokens: 5000,
            max_multi_tokens: 2500,
            resize_threshold: 0.8,
            burst_detection_window: std::time::Duration::from_millis(100),
        };

        Self {
            single_tokens: Vec::with_capacity(5000),
            multi_tokens: Vec::with_capacity(2500),
            config: config.clone(),
            allocated_count: 0,
            returned_count: 0,
            pool_hits: 0,
            pool_misses: 0,
            allocation_requests_since_resize: 0,
            last_resize_time: std::time::Instant::now(),
            peak_allocation_burst: 0,
            current_burst_count: 0,
            allocation_rate_tracker: AllocationRateTracker::new(config.burst_detection_window),
        }
    }

    /// Emergency consolidation for critical memory pressure
    pub fn emergency_consolidate(&mut self) {
        // Clear all pooled tokens to free memory immediately
        self.single_tokens.clear();
        self.multi_tokens.clear();

        // Shrink to minimal capacity
        self.single_tokens.shrink_to_fit();
        self.multi_tokens.shrink_to_fit();

        // Reset pool to emergency capacity
        self.single_tokens.reserve(10); // Minimal emergency capacity
        self.multi_tokens.reserve(5);

        // Update configuration for emergency mode
        self.config.max_single_tokens = 50;
        self.config.max_multi_tokens = 25;

        // Reset statistics
        self.allocation_requests_since_resize = 0;
        self.last_resize_time = std::time::Instant::now();
    }

    /// Optimize token pool for memory pressure
    pub fn optimize_for_memory_pressure(&mut self) {
        // Reduce pool sizes by 50%
        let target_single = self.config.max_single_tokens / 2;
        let target_multi = self.config.max_multi_tokens / 2;

        // Trim pools to target size
        if self.single_tokens.len() > target_single {
            self.single_tokens.truncate(target_single);
        }
        if self.multi_tokens.len() > target_multi {
            self.multi_tokens.truncate(target_multi);
        }

        // Shrink underlying storage
        self.single_tokens.shrink_to_fit();
        self.multi_tokens.shrink_to_fit();

        // Update configuration
        self.config.max_single_tokens = target_single.max(100);
        self.config.max_multi_tokens = target_multi.max(50);
    }

    /// Release excess capacity during normal operation
    pub fn release_excess_capacity(&mut self) {
        let single_utilization = if self.config.max_single_tokens > 0 {
            self.single_tokens.len() as f64 / self.config.max_single_tokens as f64
        } else {
            0.0
        };

        let multi_utilization = if self.config.max_multi_tokens > 0 {
            self.multi_tokens.len() as f64 / self.config.max_multi_tokens as f64
        } else {
            0.0
        };

        // Only shrink if utilization is very low (< 25%)
        if single_utilization < 0.25 && self.config.max_single_tokens > 200 {
            let new_capacity = (self.config.max_single_tokens * 3 / 4).max(200);
            if self.single_tokens.len() > new_capacity {
                self.single_tokens.truncate(new_capacity);
            }
            self.single_tokens.shrink_to_fit();
            self.config.max_single_tokens = new_capacity;
        }

        if multi_utilization < 0.25 && self.config.max_multi_tokens > 100 {
            let new_capacity = (self.config.max_multi_tokens * 3 / 4).max(100);
            if self.multi_tokens.len() > new_capacity {
                self.multi_tokens.truncate(new_capacity);
            }
            self.multi_tokens.shrink_to_fit();
            self.config.max_multi_tokens = new_capacity;
        }
    }

    /// Optimize capacity during normal operation
    pub fn optimize_capacity(&mut self) {
        let current_time = std::time::Instant::now();

        // Check if pools are underutilized and have been stable for a while
        if current_time.duration_since(self.last_resize_time) > std::time::Duration::from_secs(60) {
            let single_utilization = self.utilization();

            // If utilization is low and no recent bursts, consider reducing capacity slightly
            if single_utilization < 0.4 && self.current_burst_count == 0 {
                let new_single_cap = (self.config.max_single_tokens * 9 / 10).max(500);
                let new_multi_cap = (self.config.max_multi_tokens * 9 / 10).max(250);

                if self.single_tokens.len() > new_single_cap {
                    self.single_tokens.truncate(new_single_cap);
                    self.single_tokens.shrink_to_fit();
                }
                if self.multi_tokens.len() > new_multi_cap {
                    self.multi_tokens.truncate(new_multi_cap);
                    self.multi_tokens.shrink_to_fit();
                }

                self.config.max_single_tokens = new_single_cap;
                self.config.max_multi_tokens = new_multi_cap;
                self.last_resize_time = current_time;
            }
        }
    }
}

/// Token pool statistics for monitoring and optimization
#[derive(Debug, Clone)]
pub struct TokenPoolStats {
    pub pool_hits: usize,
    pub pool_misses: usize,
    pub utilization: f64,
    pub allocated_count: usize,
    pub returned_count: usize,
    pub memory_usage_bytes: usize,
}

/// Comprehensive token pool statistics for detailed monitoring and tuning
#[derive(Debug, Clone)]
pub struct TokenPoolComprehensiveStats {
    pub pool_hits: usize,
    pub pool_misses: usize,
    pub utilization: f64,
    pub allocated_count: usize,
    pub returned_count: usize,
    pub current_single_pool_size: usize,
    pub max_single_pool_size: usize,
    pub current_multi_pool_size: usize,
    pub max_multi_pool_size: usize,
    pub allocation_rate: f64,
    pub peak_allocation_burst: usize,
    pub current_burst_count: usize,
    pub memory_usage_bytes: usize,
}

impl TokenPoolStats {
    /// Get hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.pool_hits + self.pool_misses;
        if total == 0 {
            0.0
        } else {
            (self.pool_hits as f64 / total as f64) * 100.0
        }
    }

    /// Estimate memory usage in bytes
    pub fn memory_usage_bytes(&self) -> usize {
        // Rough estimate based on the size of the struct and its fields
        std::mem::size_of::<Self>()
    }
}

impl TokenPoolComprehensiveStats {
    /// Get hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.pool_hits + self.pool_misses;
        if total == 0 {
            0.0
        } else {
            (self.pool_hits as f64 / total as f64) * 100.0
        }
    }

    /// Estimate memory usage in bytes
    pub fn memory_usage_bytes(&self) -> usize {
        // Rough estimate based on the size of the struct and its fields
        std::mem::size_of::<Self>()
    }
}

impl TokenPool {
    /// Get pool utilization as a percentage
    pub fn utilization(&self) -> f64 {
        let total_requests = self.pool_hits + self.pool_misses;
        if total_requests == 0 {
            0.0
        } else {
            (self.pool_hits as f64 / total_requests as f64) * 100.0
        }
    }

    /// Get pool statistics for monitoring and optimization
    pub fn get_stats(&self) -> TokenPoolStats {
        TokenPoolStats {
            pool_hits: self.pool_hits,
            pool_misses: self.pool_misses,
            utilization: self.utilization(),
            allocated_count: self.allocated_count,
            returned_count: self.returned_count,
            memory_usage_bytes: self.memory_usage_bytes(),
        }
    }
}
#[derive(Debug)]
pub struct AlphaNode {
    pub node_id: NodeId,
    pub condition: Condition,
    pub memory: RwLock<Vec<FactId>>,
    pub successors: RwLock<HashSet<NodeId>>,
}

impl AlphaNode {
    pub fn new(node_id: NodeId, condition: Condition) -> Self {
        Self {
            node_id,
            condition,
            memory: RwLock::new(Vec::new()),
            successors: RwLock::new(HashSet::new()),
        }
    }

    pub fn with_capacity(node_id: NodeId, condition: Condition, capacity: usize) -> Self {
        Self {
            node_id,
            condition,
            memory: RwLock::new(Vec::with_capacity(capacity)),
            successors: RwLock::new(HashSet::new()),
        }
    }

    /// Test if a fact matches this alpha node's condition
    pub fn test_fact(&self, fact: &Fact) -> bool {
        match &self.condition {
            Condition::Simple { field, operator, value } => {
                if let Some(fact_value) = fact.data.fields.get(field) {
                    test_condition(fact_value, operator, value)
                } else {
                    false
                }
            }
            Condition::Complex { .. } => {
                // Complex conditions should be handled by a different node type
                // For now, return false as alpha nodes are meant for simple conditions
                false
            }
            Condition::Aggregation(_) => {
                // Aggregation conditions are handled by aggregation nodes
                false
            }
            Condition::Stream(_) => {
                // Stream conditions are handled by stream processing nodes
                false
            }
        }
    }

    /// Process a fact and return tokens if it matches (using shared token pool)
    pub fn process_fact(&self, fact: &Fact, token_pool: &mut TokenPool) -> Vec<Token> {
        if self.test_fact(fact) {
            self.memory.write().unwrap().push(fact.id);
            vec![token_pool.get_single_token(fact.id)]
        } else {
            Vec::new()
        }
    }

    /// Get all facts currently in this node's memory as tokens
    pub fn get_tokens(&self) -> Vec<Token> {
        self.memory.read().unwrap().iter().map(|&id| Token::new(id)).collect()
    }
}

/// Beta nodes perform joins between multiple facts
#[derive(Debug)]
pub struct BetaNode {
    pub node_id: NodeId,
    pub left_memory: RwLock<Vec<Token>>,
    pub right_memory: RwLock<Vec<Token>>,
    pub join_conditions: Vec<JoinCondition>,
    pub successors: RwLock<HashSet<NodeId>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JoinCondition {
    pub left_field: String,
    pub right_field: String,
    pub operator: Operator,
}

impl BetaNode {
    pub fn new(node_id: NodeId, join_conditions: Vec<JoinCondition>) -> Self {
        Self {
            node_id,
            left_memory: RwLock::new(Vec::new()),
            right_memory: RwLock::new(Vec::new()),
            join_conditions,
            successors: RwLock::new(HashSet::new()),
        }
    }

    /// Process tokens from left input
    pub fn process_left_tokens(&self, tokens: Vec<Token>, facts: &[Fact]) -> Vec<Token> {
        let mut results = Vec::new();
        let mut left_mem = self.left_memory.write().unwrap();
        let right_mem = self.right_memory.read().unwrap();

        for token in tokens {
            left_mem.push(token.clone());

            // Try to join with existing right memory
            for right_token in right_mem.iter() {
                if self.tokens_match(&token, right_token, facts) {
                    results.push(self.join_tokens(&token, right_token));
                }
            }
        }

        results
    }

    /// Process tokens from right input
    pub fn process_right_tokens(&self, tokens: Vec<Token>, facts: &[Fact]) -> Vec<Token> {
        let mut results = Vec::new();
        let mut right_mem = self.right_memory.write().unwrap();
        let left_mem = self.left_memory.read().unwrap();

        for token in tokens {
            right_mem.push(token.clone());

            // Try to join with existing left memory
            for left_token in left_mem.iter() {
                if self.tokens_match(left_token, &token, facts) {
                    results.push(self.join_tokens(left_token, &token));
                }
            }
        }

        results
    }

    fn tokens_match(&self, left_token: &Token, right_token: &Token, facts: &[Fact]) -> bool {
        // If no join conditions specified, just check tokens are valid
        if self.join_conditions.is_empty() {
            return !left_token.fact_ids.is_empty() && !right_token.fact_ids.is_empty();
        }

        // Get facts for both tokens
        let left_facts = self.get_facts_for_token(left_token, facts);
        let right_facts = self.get_facts_for_token(right_token, facts);

        if left_facts.is_empty() || right_facts.is_empty() {
            return false;
        }

        // Test all join conditions - all must be satisfied
        for join_condition in &self.join_conditions {
            let mut condition_satisfied = false;

            // Test all combinations of left and right facts
            for left_fact in &left_facts {
                for right_fact in &right_facts {
                    if self.test_join_condition(join_condition, left_fact, right_fact) {
                        condition_satisfied = true;
                        break;
                    }
                }
                if condition_satisfied {
                    break;
                }
            }

            if !condition_satisfied {
                return false;
            }
        }

        true
    }

    fn get_facts_for_token<'a>(&self, token: &Token, facts: &'a [Fact]) -> Vec<&'a Fact> {
        let mut result = Vec::with_capacity(token.fact_ids.len());
        for &fact_id in token.fact_ids.as_slice() {
            // Optimize lookup: facts are often accessed by ID which is their index
            if let Some(fact) = facts.get(fact_id as usize) {
                if fact.id == fact_id {
                    result.push(fact);
                    continue;
                }
            }
            // Fallback to linear search if index doesn't match
            if let Some(fact) = facts.iter().find(|f| f.id == fact_id) {
                result.push(fact);
            }
        }
        result
    }

    fn test_join_condition(
        &self,
        join_condition: &JoinCondition,
        left_fact: &Fact,
        right_fact: &Fact,
    ) -> bool {
        let left_value = left_fact.data.fields.get(&join_condition.left_field);
        let right_value = right_fact.data.fields.get(&join_condition.right_field);

        match (left_value, right_value) {
            (Some(left_val), Some(right_val)) => {
                test_condition(left_val, &join_condition.operator, right_val)
            }
            _ => false, // Missing fields fail the join condition
        }
    }

    fn join_tokens(&self, left_token: &Token, right_token: &Token) -> Token {
        left_token.join_tokens(right_token)
    }
}

/// Terminal nodes represent rule conclusions and execute actions
#[derive(Debug)]
pub struct TerminalNode {
    pub node_id: NodeId,
    pub rule_id: u64,
    pub actions: Vec<Action>,
    pub memory: RwLock<Vec<Token>>,
    pub calculator: Mutex<CachedCalculator>,
}

impl TerminalNode {
    pub fn new(node_id: NodeId, rule_id: u64, actions: Vec<Action>) -> Self {
        Self {
            node_id,
            rule_id,
            actions,
            memory: RwLock::new(Vec::new()),
            calculator: Mutex::new(CachedCalculator::with_default_caches()),
        }
    }

    /// Process tokens and execute actions (optimized to avoid fact cloning)
    pub fn process_tokens(&self, tokens: Vec<Token>, facts: &[Fact]) -> anyhow::Result<Vec<Fact>> {
        let mut results = Vec::new();
        let mut mem = self.memory.write().unwrap();

        // Pre-allocate result capacity based on tokens and actions
        let capacity_hint = tokens.len() * self.actions.len();
        results.reserve(capacity_hint.min(1000));

        for token in tokens {
            mem.push(token.clone());

            // Execute actions for this token
            for action in &self.actions {
                match &action.action_type {
                    ActionType::Log { message } => {
                        tracing::info!(rule_id = self.rule_id, message = %message, "Rule fired");
                    }
                    ActionType::SetField { field, value } => {
                        // Create a modified copy without mutating the original facts
                        if let Some(&fact_id) = token.fact_ids.as_slice().first() {
                            if let Some(original_fact) = self.find_fact_by_id(fact_id, facts) {
                                let mut modified_fact = original_fact.clone();
                                modified_fact.data.fields.insert(field.clone(), value.clone());
                                results.push(modified_fact);
                            }
                        }
                    }
                    ActionType::CreateFact { data } => {
                        let new_fact = Fact {
                            id: facts.len() as u64 + 1000 + results.len() as u64, // Unique ID generation
                            data: data.clone(),
                        };
                        results.push(new_fact);
                    }
                    ActionType::Formula { target_field, expression, source_calculator: _ } => {
                        // Evaluate formula using calculator DSL
                        if let Some(&fact_id) = token.fact_ids.as_slice().first() {
                            if let Some(original_fact) = self.find_fact_by_id(fact_id, facts) {
                                // Collect facts referenced by the token for multi-fact context
                                let mut context_facts = Vec::new();
                                for &token_fact_id in token.fact_ids.as_slice() {
                                    if let Some(fact) = self.find_fact_by_id(token_fact_id, facts) {
                                        context_facts.push(fact.clone());
                                    }
                                }

                                // Create evaluation context with multi-fact support
                                let context = EvaluationContext {
                                    current_fact: original_fact,
                                    facts: &context_facts,
                                    globals: HashMap::new(),
                                };

                                // Evaluate the expression using cached calculator
                                match self
                                    .calculator
                                    .lock()
                                    .unwrap()
                                    .eval_cached(expression, &context)
                                {
                                    Ok(CalculatorResult::Value(computed_value)) => {
                                        let mut modified_fact = original_fact.clone();
                                        modified_fact
                                            .data
                                            .fields
                                            .insert(target_field.clone(), computed_value);
                                        results.push(modified_fact);

                                        tracing::info!(
                                            rule_id = self.rule_id,
                                            target_field = %target_field,
                                            expression = %expression,
                                            "Formula action executed successfully"
                                        );
                                    }
                                    Ok(other_result) => {
                                        tracing::warn!(
                                            rule_id = self.rule_id,
                                            target_field = %target_field,
                                            expression = %expression,
                                            result = ?other_result,
                                            "Formula returned non-value result"
                                        );
                                    }
                                    Err(error) => {
                                        tracing::error!(
                                            rule_id = self.rule_id,
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
                    ActionType::ConditionalSet { target_field, conditions, source_calculator } => {
                        // TODO: Implement conditional set logic (Phase 3)
                        tracing::warn!(
                            rule_id = self.rule_id,
                            target_field = %target_field,
                            condition_count = conditions.len(),
                            calculator = ?source_calculator,
                            "ConditionalSet action not yet implemented"
                        );
                    }
                    ActionType::EmitWindow { window_name, fields } => {
                        // TODO: Implement window emission for stream processing
                        tracing::info!(
                            rule_id = self.rule_id,
                            window_name = %window_name,
                            field_count = fields.len(),
                            "EmitWindow action not yet implemented"
                        );
                    }
                    ActionType::TriggerAlert { alert_type, message, severity, metadata } => {
                        // TODO: Implement alert triggering for stream processing
                        tracing::warn!(
                            rule_id = self.rule_id,
                            alert_type = %alert_type,
                            message = %message,
                            severity = ?severity,
                            metadata_count = metadata.len(),
                            "Alert triggered: {}", message
                        );
                    }
                    ActionType::CallCalculator {
                        calculator_name,
                        input_mapping: _,
                        output_field: _,
                    } => {
                        // TODO: Implement CallCalculator action
                        tracing::warn!(
                            "CallCalculator action not yet implemented in rete_nodes: {}",
                            calculator_name
                        );
                    }
                }
            }
        }

        Ok(results)
    }

    /// Optimized fact lookup by ID
    fn find_fact_by_id<'a>(&self, fact_id: FactId, facts: &'a [Fact]) -> Option<&'a Fact> {
        // Try index-based lookup first (common case where ID = index)
        if let Some(fact) = facts.get(fact_id as usize) {
            if fact.id == fact_id {
                return Some(fact);
            }
        }

        // Fallback to linear search
        facts.iter().find(|f| f.id == fact_id)
    }

    /// Get calculator cache statistics for monitoring
    pub fn get_calculator_cache_stats(&self) -> crate::calculator_cache::CalculatorCacheStats {
        self.calculator.lock().unwrap().cache_stats()
    }

    /// Get calculator cache utilization
    pub fn get_calculator_cache_utilization(&self) -> crate::calculator_cache::CacheUtilization {
        self.calculator.lock().unwrap().cache_utilization()
    }
}

/// Test a condition against a fact value using modern pattern matching
pub fn test_condition(
    fact_value: &FactValue,
    operator: &Operator,
    expected_value: &FactValue,
) -> bool {
    use {FactValue::*, Operator::*};

    match (fact_value, expected_value, operator) {
        // Integer comparisons
        (Integer(a), Integer(b), op) => match op {
            Equal => a == b,
            NotEqual => a != b,
            GreaterThan => a > b,
            LessThan => a < b,
            GreaterThanOrEqual => a >= b,
            LessThanOrEqual => a <= b,
            Contains => false, // Not applicable for integers
        },

        // Float comparisons with epsilon handling
        (Float(a), Float(b), op) => match op {
            Equal => (a - b).abs() < f64::EPSILON,
            NotEqual => (a - b).abs() >= f64::EPSILON,
            GreaterThan => a > b,
            LessThan => a < b,
            GreaterThanOrEqual => a >= b,
            LessThanOrEqual => a <= b,
            Contains => false, // Not applicable for floats
        },

        // Cross-numeric comparisons (Integer vs Float)
        (Integer(a), Float(_b), _op) => {
            let a_float = *a as f64;
            test_condition(&Float(a_float), operator, expected_value)
        }
        (Float(_a), Integer(b), _op) => {
            let b_float = *b as f64;
            test_condition(fact_value, operator, &Float(b_float))
        }

        // String comparisons
        (String(a), String(b), op) => match op {
            Equal => a == b,
            NotEqual => a != b,
            GreaterThan => a > b,
            LessThan => a < b,
            GreaterThanOrEqual => a >= b,
            LessThanOrEqual => a <= b,
            Contains => a.contains(b),
        },

        // Boolean comparisons
        (Boolean(a), Boolean(b), op) => match op {
            Equal => a == b,
            NotEqual => a != b,
            _ => false, // Other operators not applicable for booleans
        },

        // Type mismatch - return false
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Condition, FactData, FactValue, Operator};
    use std::collections::HashMap;

    #[test]
    fn test_alpha_node_simple_condition() {
        let condition = Condition::Simple {
            field: "age".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(18),
        };

        let alpha_node = AlphaNode::new(1, condition);

        let mut fields = HashMap::new();
        fields.insert("age".to_string(), FactValue::Integer(25));

        let fact = Fact { id: 1, data: FactData { fields } };

        let mut token_pool = TokenPool::new(100);
        let tokens = alpha_node.process_fact(&fact, &mut token_pool);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].fact_ids.as_slice(), &[1]);
        assert_eq!(alpha_node.memory.read().unwrap().len(), 1);
    }

    #[test]
    fn test_condition_matching() {
        use {FactValue::*, Operator::*};

        // Integer comparisons
        assert!(test_condition(&Integer(25), &GreaterThan, &Integer(18)));
        assert!(!test_condition(&Integer(15), &GreaterThan, &Integer(18)));

        // String operations
        assert!(test_condition(
            &String("hello world".to_string()),
            &Contains,
            &String("world".to_string())
        ));

        // Cross-type numeric comparisons (new in 2024)
        assert!(test_condition(&Integer(25), &GreaterThan, &Float(24.5)));
        assert!(test_condition(&Float(25.5), &GreaterThan, &Integer(25)));

        // Boolean operations
        assert!(test_condition(&Boolean(true), &Equal, &Boolean(true)));
        assert!(!test_condition(&Boolean(true), &Equal, &Boolean(false)));
    }
}
