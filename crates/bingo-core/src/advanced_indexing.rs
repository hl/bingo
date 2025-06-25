//! Advanced indexing strategies for large-scale fact storage optimization
//!
//! This module provides enhanced indexing capabilities that go beyond the basic
//! field indexing to support high-performance queries on large datasets.

use crate::bloom_filter::{BloomFilter, BloomFilterStats};
use crate::types::{Fact, FactId, FactValue};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Represents different indexing strategies based on field characteristics
#[derive(Debug, Clone)]
pub enum IndexStrategy {
    /// HashMap-based index for high-cardinality fields (many unique values)
    HighCardinality(HashMap<String, Vec<FactId>>),
    /// Bitmap-based index for low-cardinality fields (few unique values)
    LowCardinality(Vec<(String, BitSet)>),
    /// B-tree based index for numeric fields requiring range queries
    Numeric(BTreeMap<OrderedValue, Vec<FactId>>),
    /// Hybrid index combining multiple strategies
    Hybrid { primary: Box<IndexStrategy>, secondary: HashMap<String, Vec<FactId>> },
}

/// Wrapper for values that can be ordered for B-tree indexing
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderedValue {
    Integer(i64),
    Float(OrderedFloat),
    String(String),
}

/// Wrapper for f64 to make it orderable (handles NaN consistently)
#[derive(Debug, Clone, PartialEq)]
pub struct OrderedFloat(f64);

impl Eq for OrderedFloat {}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// Compressed bit set for efficient storage of fact IDs in low-cardinality indexes
#[derive(Debug, Clone)]
pub struct BitSet {
    bits: Vec<u64>,
    max_fact_id: FactId,
}

impl BitSet {
    /// Create a new bit set with capacity for the given maximum fact ID
    pub fn new(max_fact_id: FactId) -> Self {
        let word_count = ((max_fact_id as usize + 63) / 64).max(1);
        Self { bits: vec![0u64; word_count], max_fact_id }
    }

    /// Set a bit for the given fact ID
    pub fn set(&mut self, fact_id: FactId) {
        if fact_id <= self.max_fact_id {
            let word_index = fact_id as usize / 64;
            let bit_index = fact_id as usize % 64;
            if word_index < self.bits.len() {
                self.bits[word_index] |= 1u64 << bit_index;
            }
        }
    }

    /// Check if a bit is set for the given fact ID
    pub fn get(&self, fact_id: FactId) -> bool {
        if fact_id > self.max_fact_id {
            return false;
        }
        let word_index = fact_id as usize / 64;
        let bit_index = fact_id as usize % 64;
        if word_index < self.bits.len() {
            (self.bits[word_index] & (1u64 << bit_index)) != 0
        } else {
            false
        }
    }

    /// Clear a bit for the given fact ID
    pub fn clear(&mut self, fact_id: FactId) {
        if fact_id <= self.max_fact_id {
            let word_index = fact_id as usize / 64;
            let bit_index = fact_id as usize % 64;
            if word_index < self.bits.len() {
                self.bits[word_index] &= !(1u64 << bit_index);
            }
        }
    }

    /// Convert to a vector of fact IDs (for compatibility)
    pub fn to_vec(&self) -> Vec<FactId> {
        let mut result = Vec::new();
        for (word_index, &word) in self.bits.iter().enumerate() {
            if word != 0 {
                for bit_index in 0..64 {
                    if (word & (1u64 << bit_index)) != 0 {
                        let fact_id = (word_index * 64 + bit_index) as FactId;
                        if fact_id <= self.max_fact_id {
                            result.push(fact_id);
                        }
                    }
                }
            }
        }
        result
    }

    /// Get the memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() + self.bits.len() * std::mem::size_of::<u64>()
    }
}

/// Field characteristics analysis for determining optimal index strategy
#[derive(Debug, Clone)]
pub struct FieldAnalysis {
    pub field_name: String,
    pub total_values: usize,
    pub unique_values: usize,
    pub cardinality_ratio: f64, // unique_values / total_values
    pub is_numeric: bool,
    pub average_value_length: f64,
    pub recommended_strategy: IndexStrategyType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IndexStrategyType {
    HighCardinality,
    LowCardinality,
    Numeric,
    Hybrid,
}

impl FieldAnalysis {
    /// Analyze a field's characteristics to determine the best indexing strategy
    pub fn analyze_field(field_name: &str, values: &[&FactValue]) -> Self {
        let total_values = values.len();
        let mut unique_values = HashSet::new();
        let mut numeric_count = 0;
        let mut total_length = 0;

        for value in values {
            unique_values.insert(value.as_string());
            match value {
                FactValue::Integer(_) | FactValue::Float(_) => numeric_count += 1,
                _ => {}
            }
            total_length += value.as_string().len();
        }

        let unique_count = unique_values.len();
        let cardinality_ratio = if total_values > 0 {
            unique_count as f64 / total_values as f64
        } else {
            0.0
        };

        let is_numeric = numeric_count as f64 / total_values as f64 > 0.8;
        let average_value_length = if total_values > 0 {
            total_length as f64 / total_values as f64
        } else {
            0.0
        };

        let recommended_strategy = if is_numeric && cardinality_ratio > 0.1 {
            IndexStrategyType::Numeric
        } else if cardinality_ratio < 0.1 && unique_count < 100 {
            IndexStrategyType::LowCardinality
        } else if cardinality_ratio > 0.5 {
            IndexStrategyType::HighCardinality
        } else {
            IndexStrategyType::Hybrid
        };

        Self {
            field_name: field_name.to_string(),
            total_values,
            unique_values: unique_count,
            cardinality_ratio,
            is_numeric,
            average_value_length,
            recommended_strategy,
        }
    }
}

/// Advanced field indexer with adaptive strategies for large datasets
#[derive(Debug)]
pub struct AdvancedFieldIndexer {
    /// Field-specific indexes with adaptive strategies
    indexes: HashMap<String, IndexStrategy>,
    /// Field analysis cache for optimization decisions
    field_analyses: HashMap<String, FieldAnalysis>,
    /// Set of fields currently being indexed
    indexed_fields: HashSet<String>,
    /// Maximum fact ID seen (for bit set sizing)
    max_fact_id: FactId,
    /// Bloom filter for fast existence checks
    existence_filter: Option<BloomFilter>,
    /// Statistics for performance monitoring
    stats: AdvancedIndexingStats,
}

#[derive(Debug, Clone, Default)]
pub struct AdvancedIndexingStats {
    pub total_facts_indexed: usize,
    pub total_lookups: usize,
    pub cache_hits: usize,
    pub bloom_filter_saves: usize,
    pub index_memory_usage: usize,
    pub avg_lookup_time_micros: f64,
    pub strategy_distribution: HashMap<String, IndexStrategyType>,
    pub bloom_filter_stats: Option<BloomFilterStats>,
}

impl AdvancedFieldIndexer {
    /// Create a new advanced field indexer
    pub fn new() -> Self {
        Self::with_bloom_filter(None)
    }

    /// Create a new advanced field indexer with optional bloom filter
    pub fn with_bloom_filter(expected_facts: Option<usize>) -> Self {
        let existence_filter = expected_facts.map(|count| BloomFilter::with_capacity(count, 0.01));

        Self {
            indexes: HashMap::new(),
            field_analyses: HashMap::new(),
            indexed_fields: HashSet::new(),
            max_fact_id: 0,
            existence_filter,
            stats: AdvancedIndexingStats::default(),
        }
    }

    /// Add a field for indexing with automatic strategy selection
    pub fn add_field(&mut self, field_name: String) {
        self.indexed_fields.insert(field_name);
    }

    /// Add a field with explicit strategy selection
    pub fn add_field_with_strategy(&mut self, field_name: String, strategy: IndexStrategyType) {
        self.indexed_fields.insert(field_name.clone());
        // Create initial index based on strategy
        match strategy {
            IndexStrategyType::HighCardinality => {
                self.indexes.insert(
                    field_name.clone(),
                    IndexStrategy::HighCardinality(HashMap::new()),
                );
            }
            IndexStrategyType::LowCardinality => {
                self.indexes.insert(
                    field_name.clone(),
                    IndexStrategy::LowCardinality(Vec::new()),
                );
            }
            IndexStrategyType::Numeric => {
                self.indexes.insert(field_name.clone(), IndexStrategy::Numeric(BTreeMap::new()));
            }
            IndexStrategyType::Hybrid => {
                self.indexes.insert(
                    field_name.clone(),
                    IndexStrategy::Hybrid {
                        primary: Box::new(IndexStrategy::HighCardinality(HashMap::new())),
                        secondary: HashMap::new(),
                    },
                );
            }
        }
    }

    /// Index a fact using the appropriate strategy for each field
    pub fn index_fact(&mut self, fact: &Fact) {
        self.max_fact_id = self.max_fact_id.max(fact.id);
        self.stats.total_facts_indexed += 1;

        // Add to bloom filter if enabled
        if let Some(ref mut filter) = self.existence_filter {
            filter.add_fact(fact);
        }

        for (field_name, field_value) in &fact.data.fields {
            if self.indexed_fields.contains(field_name) {
                self.index_field_value(field_name, field_value, fact.id);
            }
        }
    }

    /// Remove a fact from all indexes
    pub fn remove_fact(&mut self, fact: &Fact) {
        // Collect fields to remove to avoid borrowing issues
        let fields_to_remove: Vec<(String, FactValue)> =
            fact.data.fields.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

        for (field_name, field_value) in fields_to_remove {
            if self.indexes.contains_key(&field_name) {
                // Remove using a separate method to avoid borrowing issues
                self.remove_field_value(&field_name, &field_value, fact.id);
            }
        }
    }

    /// Find facts by field value using the appropriate index strategy
    pub fn find_by_field(&mut self, field_name: &str, value: &FactValue) -> Vec<FactId> {
        self.stats.total_lookups += 1;
        let start_time = std::time::Instant::now();

        let result = if let Some(index) = self.indexes.get(field_name) {
            self.lookup_in_index(index, value)
        } else {
            Vec::new()
        };

        let elapsed = start_time.elapsed().as_micros() as f64;
        self.stats.avg_lookup_time_micros =
            (self.stats.avg_lookup_time_micros * (self.stats.total_lookups - 1) as f64 + elapsed)
                / self.stats.total_lookups as f64;

        result
    }

    /// Check if a fact might exist using bloom filter (fast negative lookup)
    pub fn might_contain_fact(&mut self, fact_id: FactId) -> bool {
        if let Some(ref filter) = self.existence_filter {
            if !filter.might_contain_fact_id(fact_id) {
                self.stats.bloom_filter_saves += 1;
                return false;
            }
        }
        true // Might exist or bloom filter disabled
    }

    /// Find facts matching multiple criteria with optimized intersection
    pub fn find_by_criteria(&mut self, criteria: &[(String, FactValue)]) -> Vec<FactId> {
        if criteria.is_empty() {
            return Vec::new();
        }

        // Sort criteria by expected selectivity (smallest result sets first)
        let mut sorted_criteria: Vec<_> = criteria.iter().collect();
        sorted_criteria
            .sort_by_key(|(field_name, value)| self.estimate_result_size(field_name, value));

        // Start with the most selective criterion
        let (first_field, first_value) = sorted_criteria[0];
        let mut candidates = self.find_by_field(first_field, first_value);

        // Intersect with remaining criteria
        for (field_name, value) in &sorted_criteria[1..] {
            if candidates.is_empty() {
                break; // Early termination if no candidates remain
            }

            let matching_ids = self.find_by_field(field_name, value);
            candidates = self.intersect_sorted(&candidates, &matching_ids);
        }

        candidates
    }

    /// Analyze field characteristics and optimize indexes
    pub fn optimize_indexes(&mut self, sample_facts: &[&Fact]) {
        // Collect field values for analysis
        let mut field_values: HashMap<String, Vec<&FactValue>> = HashMap::new();

        for fact in sample_facts {
            for (field_name, field_value) in &fact.data.fields {
                if self.indexed_fields.contains(field_name) {
                    field_values.entry(field_name.clone()).or_default().push(field_value);
                }
            }
        }

        // Analyze each field and rebuild indexes if needed
        for (field_name, values) in field_values {
            let analysis = FieldAnalysis::analyze_field(&field_name, &values);
            let current_strategy = self.get_current_strategy(&field_name);

            if current_strategy != Some(analysis.recommended_strategy.clone()) {
                // Rebuild index with new strategy
                self.rebuild_index_with_strategy(
                    &field_name,
                    analysis.recommended_strategy.clone(),
                    sample_facts,
                );
            }

            self.field_analyses.insert(field_name.clone(), analysis.clone());
            self.stats
                .strategy_distribution
                .insert(field_name, analysis.recommended_strategy);
        }
    }

    /// Get statistics about the indexing system
    pub fn get_stats(&self) -> AdvancedIndexingStats {
        let mut stats = self.stats.clone();
        stats.index_memory_usage = self.calculate_memory_usage();
        stats.bloom_filter_stats = self.existence_filter.as_ref().map(|f| f.stats());
        stats
    }

    /// Clear all indexes
    pub fn clear(&mut self) {
        self.indexes.clear();
        self.field_analyses.clear();
        self.max_fact_id = 0;

        if let Some(ref mut filter) = self.existence_filter {
            filter.clear();
        }

        self.stats = AdvancedIndexingStats::default();
    }

    // Private helper methods

    fn index_field_value(&mut self, field_name: &str, field_value: &FactValue, fact_id: FactId) {
        // Ensure index exists for this field
        if !self.indexes.contains_key(field_name) {
            // Create default high-cardinality index
            self.indexes.insert(
                field_name.to_string(),
                IndexStrategy::HighCardinality(HashMap::new()),
            );
        }

        // Use separate method to avoid borrowing issues
        self.add_field_value(field_name, field_value, fact_id);
    }

    fn add_field_value(&mut self, field_name: &str, field_value: &FactValue, fact_id: FactId) {
        if let Some(index) = self.indexes.get_mut(field_name) {
            match index {
                IndexStrategy::HighCardinality(map) => {
                    let key = field_value.as_string();
                    map.entry(key).or_default().push(fact_id);
                }
                IndexStrategy::LowCardinality(vec) => {
                    let key = field_value.as_string();
                    // Find or create entry for this value
                    if let Some((_, bitset)) = vec.iter_mut().find(|(k, _)| k == &key) {
                        bitset.set(fact_id);
                    } else {
                        let mut bitset = BitSet::new(self.max_fact_id);
                        bitset.set(fact_id);
                        vec.push((key, bitset));
                    }
                }
                IndexStrategy::Numeric(btree) => {
                    let ordered_value = Self::value_to_ordered(field_value);
                    btree.entry(ordered_value).or_default().push(fact_id);
                }
                IndexStrategy::Hybrid { primary, secondary } => {
                    // Handle primary index
                    match primary.as_mut() {
                        IndexStrategy::HighCardinality(map) => {
                            let key = field_value.as_string();
                            map.entry(key).or_default().push(fact_id);
                        }
                        IndexStrategy::LowCardinality(vec) => {
                            let key = field_value.as_string();
                            if let Some((_, bitset)) = vec.iter_mut().find(|(k, _)| k == &key) {
                                bitset.set(fact_id);
                            } else {
                                let mut bitset = BitSet::new(self.max_fact_id);
                                bitset.set(fact_id);
                                vec.push((key, bitset));
                            }
                        }
                        IndexStrategy::Numeric(btree) => {
                            let ordered_value = Self::value_to_ordered(field_value);
                            btree.entry(ordered_value).or_default().push(fact_id);
                        }
                        IndexStrategy::Hybrid { .. } => {
                            // Nested hybrid not supported
                        }
                    }
                    // Handle secondary index
                    let key = field_value.as_string();
                    secondary.entry(key).or_default().push(fact_id);
                }
            }
        }
    }

    fn remove_field_value(&mut self, field_name: &str, field_value: &FactValue, fact_id: FactId) {
        if let Some(index) = self.indexes.get_mut(field_name) {
            match index {
                IndexStrategy::HighCardinality(map) => {
                    let key = field_value.as_string();
                    if let Some(ids) = map.get_mut(&key) {
                        ids.retain(|&id| id != fact_id);
                        if ids.is_empty() {
                            map.remove(&key);
                        }
                    }
                }
                IndexStrategy::LowCardinality(vec) => {
                    let key = field_value.as_string();
                    if let Some((_, bitset)) = vec.iter_mut().find(|(k, _)| k == &key) {
                        bitset.clear(fact_id);
                    }
                }
                IndexStrategy::Numeric(btree) => {
                    let ordered_value = Self::value_to_ordered(field_value);
                    if let Some(ids) = btree.get_mut(&ordered_value) {
                        ids.retain(|&id| id != fact_id);
                        if ids.is_empty() {
                            btree.remove(&ordered_value);
                        }
                    }
                }
                IndexStrategy::Hybrid { primary, secondary } => {
                    // Handle primary index
                    match primary.as_mut() {
                        IndexStrategy::HighCardinality(map) => {
                            let key = field_value.as_string();
                            if let Some(ids) = map.get_mut(&key) {
                                ids.retain(|&id| id != fact_id);
                                if ids.is_empty() {
                                    map.remove(&key);
                                }
                            }
                        }
                        IndexStrategy::LowCardinality(vec) => {
                            let key = field_value.as_string();
                            if let Some((_, bitset)) = vec.iter_mut().find(|(k, _)| k == &key) {
                                bitset.clear(fact_id);
                            }
                        }
                        IndexStrategy::Numeric(btree) => {
                            let ordered_value = Self::value_to_ordered(field_value);
                            if let Some(ids) = btree.get_mut(&ordered_value) {
                                ids.retain(|&id| id != fact_id);
                                if ids.is_empty() {
                                    btree.remove(&ordered_value);
                                }
                            }
                        }
                        IndexStrategy::Hybrid { .. } => {
                            // Nested hybrid not supported
                        }
                    }
                    // Handle secondary index
                    let key = field_value.as_string();
                    if let Some(ids) = secondary.get_mut(&key) {
                        ids.retain(|&id| id != fact_id);
                        if ids.is_empty() {
                            secondary.remove(&key);
                        }
                    }
                }
            }
        }
    }

    fn lookup_in_index(&self, index: &IndexStrategy, value: &FactValue) -> Vec<FactId> {
        match index {
            IndexStrategy::HighCardinality(map) => {
                let key = value.as_string();
                map.get(&key).cloned().unwrap_or_default()
            }
            IndexStrategy::LowCardinality(vec) => {
                let key = value.as_string();
                if let Some((_, bitset)) = vec.iter().find(|(k, _)| k == &key) {
                    bitset.to_vec()
                } else {
                    Vec::new()
                }
            }
            IndexStrategy::Numeric(btree) => {
                let ordered_value = Self::value_to_ordered(value);
                btree.get(&ordered_value).cloned().unwrap_or_default()
            }
            IndexStrategy::Hybrid { primary, .. } => self.lookup_in_index(primary, value),
        }
    }

    fn value_to_ordered(value: &FactValue) -> OrderedValue {
        match value {
            FactValue::Integer(i) => OrderedValue::Integer(*i),
            FactValue::Float(f) => OrderedValue::Float(OrderedFloat(*f)),
            FactValue::String(s) => OrderedValue::String(s.clone()),
            FactValue::Boolean(b) => OrderedValue::String(b.to_string()),
            FactValue::Array(_) => OrderedValue::String(value.as_string()),
            FactValue::Object(_) => OrderedValue::String(value.as_string()),
            FactValue::Date(d) => OrderedValue::String(d.to_rfc3339()),
            FactValue::Null => OrderedValue::String("null".to_string()),
        }
    }

    fn estimate_result_size(&self, field_name: &str, _value: &FactValue) -> usize {
        if let Some(analysis) = self.field_analyses.get(field_name) {
            // Estimate based on cardinality
            if analysis.cardinality_ratio > 0.0 {
                (analysis.total_values as f64 / analysis.unique_values as f64) as usize
            } else {
                analysis.total_values
            }
        } else {
            // Conservative estimate
            1000
        }
    }

    fn intersect_sorted(&self, a: &[FactId], b: &[FactId]) -> Vec<FactId> {
        let mut result = Vec::new();
        let mut i = 0;
        let mut j = 0;

        while i < a.len() && j < b.len() {
            match a[i].cmp(&b[j]) {
                std::cmp::Ordering::Equal => {
                    result.push(a[i]);
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Greater => j += 1,
            }
        }

        result
    }

    fn get_current_strategy(&self, field_name: &str) -> Option<IndexStrategyType> {
        self.indexes.get(field_name).map(|index| match index {
            IndexStrategy::HighCardinality(_) => IndexStrategyType::HighCardinality,
            IndexStrategy::LowCardinality(_) => IndexStrategyType::LowCardinality,
            IndexStrategy::Numeric(_) => IndexStrategyType::Numeric,
            IndexStrategy::Hybrid { .. } => IndexStrategyType::Hybrid,
        })
    }

    fn rebuild_index_with_strategy(
        &mut self,
        field_name: &str,
        strategy: IndexStrategyType,
        facts: &[&Fact],
    ) {
        // Clear existing index
        self.indexes.remove(field_name);

        // Create new index with optimal strategy
        self.add_field_with_strategy(field_name.to_string(), strategy);

        // Re-index all facts
        for fact in facts {
            if let Some(field_value) = fact.data.fields.get(field_name) {
                self.index_field_value(field_name, field_value, fact.id);
            }
        }
    }

    fn calculate_memory_usage(&self) -> usize {
        let mut total = std::mem::size_of::<Self>();

        for (field_name, index) in &self.indexes {
            total += field_name.len();
            total += match index {
                IndexStrategy::HighCardinality(map) => {
                    map.len() * (std::mem::size_of::<String>() + std::mem::size_of::<Vec<FactId>>())
                        + map
                            .values()
                            .map(|v| v.len() * std::mem::size_of::<FactId>())
                            .sum::<usize>()
                }
                IndexStrategy::LowCardinality(vec) => {
                    vec.iter().map(|(k, bitset)| k.len() + bitset.memory_usage()).sum::<usize>()
                }
                IndexStrategy::Numeric(btree) => {
                    btree.len()
                        * (std::mem::size_of::<OrderedValue>() + std::mem::size_of::<Vec<FactId>>())
                        + btree
                            .values()
                            .map(|v| v.len() * std::mem::size_of::<FactId>())
                            .sum::<usize>()
                }
                IndexStrategy::Hybrid { primary, secondary } => {
                    self.calculate_strategy_memory_usage(primary)
                        + secondary.len()
                            * (std::mem::size_of::<String>() + std::mem::size_of::<Vec<FactId>>())
                        + secondary
                            .values()
                            .map(|v| v.len() * std::mem::size_of::<FactId>())
                            .sum::<usize>()
                }
            };
        }

        total
    }

    fn calculate_strategy_memory_usage(&self, strategy: &IndexStrategy) -> usize {
        match strategy {
            IndexStrategy::HighCardinality(map) => {
                map.len() * (std::mem::size_of::<String>() + std::mem::size_of::<Vec<FactId>>())
                    + map.values().map(|v| v.len() * std::mem::size_of::<FactId>()).sum::<usize>()
            }
            IndexStrategy::LowCardinality(vec) => {
                vec.iter().map(|(k, bitset)| k.len() + bitset.memory_usage()).sum::<usize>()
            }
            IndexStrategy::Numeric(btree) => {
                btree.len()
                    * (std::mem::size_of::<OrderedValue>() + std::mem::size_of::<Vec<FactId>>())
                    + btree.values().map(|v| v.len() * std::mem::size_of::<FactId>()).sum::<usize>()
            }
            IndexStrategy::Hybrid { primary, secondary } => {
                self.calculate_strategy_memory_usage(primary)
                    + secondary.len()
                        * (std::mem::size_of::<String>() + std::mem::size_of::<Vec<FactId>>())
                    + secondary
                        .values()
                        .map(|v| v.len() * std::mem::size_of::<FactId>())
                        .sum::<usize>()
            }
        }
    }
}

impl Default for AdvancedFieldIndexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FactData, FactValue};
    use std::collections::HashMap;

    fn create_test_fact(id: FactId, field_name: &str, field_value: FactValue) -> Fact {
        let mut fields = HashMap::new();
        fields.insert(field_name.to_string(), field_value);
        Fact { id, data: FactData { fields } }
    }

    #[test]
    fn test_bitset_operations() {
        let mut bitset = BitSet::new(100);

        // Test setting and getting bits
        bitset.set(5);
        bitset.set(10);
        bitset.set(95);

        assert!(bitset.get(5));
        assert!(bitset.get(10));
        assert!(bitset.get(95));
        assert!(!bitset.get(6));

        // Test clearing bits
        bitset.clear(10);
        assert!(!bitset.get(10));

        // Test conversion to vec
        let vec = bitset.to_vec();
        assert_eq!(vec, vec![5, 95]);
    }

    #[test]
    fn test_field_analysis() {
        let value1 = FactValue::String("status_1".to_string());
        let value2 = FactValue::String("status_2".to_string());
        let value3 = FactValue::String("status_1".to_string());
        let value4 = FactValue::String("status_3".to_string());
        let values = vec![&value1, &value2, &value3, &value4];

        let analysis = FieldAnalysis::analyze_field("status", &values);
        assert_eq!(analysis.total_values, 4);
        assert_eq!(analysis.unique_values, 3);
        assert_eq!(analysis.cardinality_ratio, 0.75);
        assert!(!analysis.is_numeric);
    }

    #[test]
    fn test_advanced_indexer_basic_operations() {
        let mut indexer = AdvancedFieldIndexer::new();
        indexer.add_field("status".to_string());

        let fact1 = create_test_fact(1, "status", FactValue::String("active".to_string()));
        let fact2 = create_test_fact(2, "status", FactValue::String("inactive".to_string()));
        let fact3 = create_test_fact(3, "status", FactValue::String("active".to_string()));

        indexer.index_fact(&fact1);
        indexer.index_fact(&fact2);
        indexer.index_fact(&fact3);

        let active_facts =
            indexer.find_by_field("status", &FactValue::String("active".to_string()));
        assert_eq!(active_facts.len(), 2);
        assert!(active_facts.contains(&1));
        assert!(active_facts.contains(&3));
    }

    #[test]
    fn test_advanced_indexer_multiple_criteria() {
        let mut indexer = AdvancedFieldIndexer::new();
        indexer.add_field("status".to_string());
        indexer.add_field("priority".to_string());

        let mut fields1 = HashMap::new();
        fields1.insert(
            "status".to_string(),
            FactValue::String("active".to_string()),
        );
        fields1.insert(
            "priority".to_string(),
            FactValue::String("high".to_string()),
        );
        let fact1 = Fact { id: 1, data: FactData { fields: fields1 } };

        let mut fields2 = HashMap::new();
        fields2.insert(
            "status".to_string(),
            FactValue::String("active".to_string()),
        );
        fields2.insert("priority".to_string(), FactValue::String("low".to_string()));
        let fact2 = Fact { id: 2, data: FactData { fields: fields2 } };

        indexer.index_fact(&fact1);
        indexer.index_fact(&fact2);

        let criteria = vec![
            (
                "status".to_string(),
                FactValue::String("active".to_string()),
            ),
            (
                "priority".to_string(),
                FactValue::String("high".to_string()),
            ),
        ];

        let matching_facts = indexer.find_by_criteria(&criteria);
        assert_eq!(matching_facts.len(), 1);
        assert!(matching_facts.contains(&1));
    }

    #[test]
    fn test_numeric_indexing() {
        let mut indexer = AdvancedFieldIndexer::new();
        indexer.add_field_with_strategy("score".to_string(), IndexStrategyType::Numeric);

        let fact1 = create_test_fact(1, "score", FactValue::Integer(100));
        let fact2 = create_test_fact(2, "score", FactValue::Integer(85));
        let fact3 = create_test_fact(3, "score", FactValue::Integer(100));

        indexer.index_fact(&fact1);
        indexer.index_fact(&fact2);
        indexer.index_fact(&fact3);

        let high_scores = indexer.find_by_field("score", &FactValue::Integer(100));
        assert_eq!(high_scores.len(), 2);
        assert!(high_scores.contains(&1));
        assert!(high_scores.contains(&3));
    }
}
