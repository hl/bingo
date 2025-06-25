//! Bloom filter implementation for fast fact existence checks
//!
//! This module provides probabilistic data structures for efficiently determining
//! if a fact might exist in the storage system, significantly reducing expensive
//! lookup operations for non-existent facts.

use crate::types::{Fact, FactId};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Bloom filter for probabilistic membership testing
///
/// A bloom filter is a space-efficient probabilistic data structure that can tell you
/// if an element is "definitely not in the set" or "might be in the set".
/// False positives are possible, but false negatives are not.
#[derive(Debug, Clone)]
pub struct BloomFilter {
    /// Bit array for the bloom filter
    bits: Vec<u64>,
    /// Number of hash functions to use
    hash_functions: usize,
    /// Number of bits in the filter
    size: usize,
    /// Number of elements added to the filter
    element_count: usize,
}

impl BloomFilter {
    /// Create a new bloom filter with the specified size and number of hash functions
    ///
    /// # Arguments
    /// * `size` - Number of bits in the filter (will be rounded up to nearest multiple of 64)
    /// * `hash_functions` - Number of hash functions to use (typically 2-8)
    pub fn new(size: usize, hash_functions: usize) -> Self {
        let word_count = size.div_ceil(64); // Round up to nearest multiple of 64
        let actual_size = word_count * 64;

        Self {
            bits: vec![0u64; word_count],
            hash_functions: hash_functions.clamp(1, 16), // Clamp to reasonable range
            size: actual_size,
            element_count: 0,
        }
    }

    /// Create a bloom filter optimized for the expected number of elements and desired false positive rate
    ///
    /// # Arguments
    /// * `expected_elements` - Expected number of elements to be added
    /// * `false_positive_rate` - Desired false positive rate (e.g., 0.01 for 1%)
    pub fn with_capacity(expected_elements: usize, false_positive_rate: f64) -> Self {
        let m = Self::optimal_size(expected_elements, false_positive_rate);
        let k = Self::optimal_hash_functions(expected_elements, m);
        Self::new(m, k)
    }

    /// Calculate optimal bloom filter size for given parameters
    fn optimal_size(expected_elements: usize, false_positive_rate: f64) -> usize {
        let ln2 = std::f64::consts::LN_2;
        let size = -(expected_elements as f64 * false_positive_rate.ln()) / (ln2 * ln2);
        size.ceil() as usize
    }

    /// Calculate optimal number of hash functions
    fn optimal_hash_functions(expected_elements: usize, size: usize) -> usize {
        let ln2 = std::f64::consts::LN_2;
        let k = (size as f64 / expected_elements as f64) * ln2;
        k.round().clamp(1.0, 16.0) as usize
    }

    /// Add a fact ID to the bloom filter
    pub fn add_fact_id(&mut self, fact_id: FactId) {
        let hashes = self.hash_fact_id(fact_id);

        for &hash in &hashes {
            let bit_index = (hash as usize) % self.size;
            let word_index = bit_index / 64;
            let bit_offset = bit_index % 64;

            if word_index < self.bits.len() {
                self.bits[word_index] |= 1u64 << bit_offset;
            }
        }

        self.element_count += 1;
    }

    /// Add a fact to the bloom filter (uses fact ID)
    pub fn add_fact(&mut self, fact: &Fact) {
        self.add_fact_id(fact.id);
    }

    /// Check if a fact ID might exist in the set
    ///
    /// Returns:
    /// - `true`: The fact might be in the set (could be false positive)
    /// - `false`: The fact is definitely not in the set
    pub fn might_contain_fact_id(&self, fact_id: FactId) -> bool {
        let hashes = self.hash_fact_id(fact_id);

        for &hash in &hashes {
            let bit_index = (hash as usize) % self.size;
            let word_index = bit_index / 64;
            let bit_offset = bit_index % 64;

            if word_index >= self.bits.len() {
                return false;
            }

            if (self.bits[word_index] & (1u64 << bit_offset)) == 0 {
                return false; // Definitely not in set
            }
        }

        true // Might be in set
    }

    /// Check if a fact might exist in the set
    pub fn might_contain_fact(&self, fact: &Fact) -> bool {
        self.might_contain_fact_id(fact.id)
    }

    /// Clear all bits in the filter
    pub fn clear(&mut self) {
        for word in &mut self.bits {
            *word = 0;
        }
        self.element_count = 0;
    }

    /// Get the current estimated false positive rate
    pub fn estimated_false_positive_rate(&self) -> f64 {
        if self.element_count == 0 {
            return 0.0;
        }

        let fraction_set = self.fraction_of_bits_set();
        fraction_set.powi(self.hash_functions as i32)
    }

    /// Get the fraction of bits that are set
    pub fn fraction_of_bits_set(&self) -> f64 {
        let total_bits_set =
            self.bits.iter().map(|&word| word.count_ones() as usize).sum::<usize>();

        total_bits_set as f64 / self.size as f64
    }

    /// Get bloom filter statistics
    pub fn stats(&self) -> BloomFilterStats {
        BloomFilterStats {
            size_bits: self.size,
            hash_functions: self.hash_functions,
            element_count: self.element_count,
            bits_set: self.bits.iter().map(|&word| word.count_ones() as usize).sum(),
            estimated_false_positive_rate: self.estimated_false_positive_rate(),
            memory_usage_bytes: self.memory_usage(),
        }
    }

    /// Get memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() + self.bits.len() * std::mem::size_of::<u64>()
    }

    /// Generate multiple hash values for a fact ID using double hashing
    fn hash_fact_id(&self, fact_id: FactId) -> Vec<u64> {
        let mut hashes = Vec::with_capacity(self.hash_functions);

        // Primary hash
        let mut hasher1 = DefaultHasher::new();
        fact_id.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        // Secondary hash (using a different seed)
        let mut hasher2 = DefaultHasher::new();
        (fact_id ^ 0xAAAAAAAAAAAAAAAAu64).hash(&mut hasher2);
        let hash2 = hasher2.finish();

        // Generate k hash values using double hashing: h1 + i * h2
        for i in 0..self.hash_functions {
            let combined_hash = hash1.wrapping_add((i as u64).wrapping_mul(hash2));
            hashes.push(combined_hash);
        }

        hashes
    }

    /// Check if the bloom filter should be resized based on current load
    pub fn should_resize(&self) -> bool {
        let load_factor = self.fraction_of_bits_set();
        // Consider resizing if more than 50% of bits are set
        load_factor > 0.5
    }

    /// Create a new larger bloom filter and migrate existing data
    pub fn resize(&self, new_expected_elements: usize, target_false_positive_rate: f64) -> Self {
        // Note: We can't migrate existing elements since we don't store them
        // This is a limitation of bloom filters - they can't be resized while preserving data
        // The caller would need to re-add all elements to the new filter
        Self::with_capacity(new_expected_elements, target_false_positive_rate)
    }
}

/// Statistics about bloom filter performance and utilization
#[derive(Debug, Clone)]
pub struct BloomFilterStats {
    pub size_bits: usize,
    pub hash_functions: usize,
    pub element_count: usize,
    pub bits_set: usize,
    pub estimated_false_positive_rate: f64,
    pub memory_usage_bytes: usize,
}

impl BloomFilterStats {
    /// Get the utilization percentage of the filter
    pub fn utilization_percentage(&self) -> f64 {
        if self.size_bits == 0 {
            0.0
        } else {
            (self.bits_set as f64 / self.size_bits as f64) * 100.0
        }
    }

    /// Get the efficiency score (lower false positive rate is better)
    pub fn efficiency_score(&self) -> f64 {
        1.0 - self.estimated_false_positive_rate
    }

    /// Check if the filter is well-tuned (good balance of size vs false positive rate)
    pub fn is_well_tuned(&self) -> bool {
        let util = self.utilization_percentage();
        let fpr = self.estimated_false_positive_rate;

        // Good tuning: 30-70% utilization with <5% false positive rate
        (30.0..=70.0).contains(&util) && fpr < 0.05
    }
}

/// Bloom filter optimized for fact storage systems
///
/// This is a specialized bloom filter that can handle multiple types of queries
/// commonly used in fact storage systems.
#[derive(Debug)]
pub struct FactBloomFilter {
    /// Primary bloom filter for fact IDs
    fact_id_filter: BloomFilter,
    /// Secondary bloom filter for field existence (optional)
    field_filter: Option<BloomFilter>,
    /// Configuration for automatic maintenance
    config: FactBloomConfig,
    /// Statistics for monitoring
    query_count: usize,
    true_negatives: usize, // Queries that bloom filter correctly identified as non-existent
}

#[derive(Debug, Clone)]
pub struct FactBloomConfig {
    /// Whether to enable field-level bloom filtering
    pub enable_field_filtering: bool,
    /// Target false positive rate
    pub target_false_positive_rate: f64,
    /// Whether to automatically resize when overloaded
    pub auto_resize: bool,
    /// Threshold for triggering resize (fraction of bits set)
    pub resize_threshold: f64,
}

impl Default for FactBloomConfig {
    fn default() -> Self {
        Self {
            enable_field_filtering: false,
            target_false_positive_rate: 0.01, // 1%
            auto_resize: true,
            resize_threshold: 0.5,
        }
    }
}

impl FactBloomFilter {
    /// Create a new fact bloom filter with the given configuration
    pub fn new(expected_facts: usize, config: FactBloomConfig) -> Self {
        let fact_id_filter =
            BloomFilter::with_capacity(expected_facts, config.target_false_positive_rate);

        let field_filter = if config.enable_field_filtering {
            // Estimate field count as roughly 10x fact count (conservative)
            let estimated_fields = expected_facts * 10;
            Some(BloomFilter::with_capacity(
                estimated_fields,
                config.target_false_positive_rate,
            ))
        } else {
            None
        };

        Self { fact_id_filter, field_filter, config, query_count: 0, true_negatives: 0 }
    }

    /// Create with default configuration
    pub fn with_capacity(expected_facts: usize) -> Self {
        Self::new(expected_facts, FactBloomConfig::default())
    }

    /// Add a fact to the bloom filters
    pub fn add_fact(&mut self, fact: &Fact) {
        self.fact_id_filter.add_fact(fact);

        if let Some(ref mut field_filter) = self.field_filter {
            // Add field names to the field filter
            for field_name in fact.data.fields.keys() {
                let mut hasher = DefaultHasher::new();
                field_name.hash(&mut hasher);
                field_filter.add_fact_id(hasher.finish());
            }
        }
    }

    /// Check if a fact might exist (fast negative lookup)
    pub fn might_contain_fact(&mut self, fact_id: FactId) -> bool {
        self.query_count += 1;

        let might_exist = self.fact_id_filter.might_contain_fact_id(fact_id);

        if !might_exist {
            self.true_negatives += 1;
        }

        might_exist
    }

    /// Check if a field might exist on any fact
    pub fn might_contain_field(&mut self, field_name: &str) -> bool {
        if let Some(ref field_filter) = self.field_filter {
            self.query_count += 1;

            let mut hasher = DefaultHasher::new();
            field_name.hash(&mut hasher);
            let might_exist = field_filter.might_contain_fact_id(hasher.finish());

            if !might_exist {
                self.true_negatives += 1;
            }

            might_exist
        } else {
            true // If field filtering is disabled, assume field might exist
        }
    }

    /// Clear all bloom filters
    pub fn clear(&mut self) {
        self.fact_id_filter.clear();
        if let Some(ref mut field_filter) = self.field_filter {
            field_filter.clear();
        }
        self.query_count = 0;
        self.true_negatives = 0;
    }

    /// Get comprehensive statistics
    pub fn stats(&self) -> FactBloomStats {
        FactBloomStats {
            fact_id_stats: self.fact_id_filter.stats(),
            field_stats: self.field_filter.as_ref().map(|f| f.stats()),
            query_count: self.query_count,
            true_negatives: self.true_negatives,
            effectiveness: self.effectiveness(),
            total_memory_usage: self.memory_usage(),
        }
    }

    /// Get the effectiveness of the bloom filter (percentage of true negatives)
    pub fn effectiveness(&self) -> f64 {
        if self.query_count == 0 {
            0.0
        } else {
            (self.true_negatives as f64 / self.query_count as f64) * 100.0
        }
    }

    /// Get total memory usage
    pub fn memory_usage(&self) -> usize {
        let mut total = std::mem::size_of::<Self>() + self.fact_id_filter.memory_usage();

        if let Some(ref field_filter) = self.field_filter {
            total += field_filter.memory_usage();
        }

        total
    }

    /// Check if the filter should be resized and do so if configured
    pub fn check_and_resize(&mut self, new_expected_facts: usize) -> bool {
        if !self.config.auto_resize {
            return false;
        }

        if self.fact_id_filter.fraction_of_bits_set() > self.config.resize_threshold {
            let new_filter = BloomFilter::with_capacity(
                new_expected_facts,
                self.config.target_false_positive_rate,
            );
            self.fact_id_filter = new_filter;

            if self.config.enable_field_filtering {
                let estimated_fields = new_expected_facts * 10;
                let new_field_filter = BloomFilter::with_capacity(
                    estimated_fields,
                    self.config.target_false_positive_rate,
                );
                self.field_filter = Some(new_field_filter);
            }

            return true;
        }

        false
    }
}

/// Comprehensive statistics for fact bloom filters
#[derive(Debug, Clone)]
pub struct FactBloomStats {
    pub fact_id_stats: BloomFilterStats,
    pub field_stats: Option<BloomFilterStats>,
    pub query_count: usize,
    pub true_negatives: usize,
    pub effectiveness: f64,
    pub total_memory_usage: usize,
}

impl FactBloomStats {
    /// Get the overall false positive rate across all filters
    pub fn overall_false_positive_rate(&self) -> f64 {
        let mut total_rate = self.fact_id_stats.estimated_false_positive_rate;

        if let Some(ref field_stats) = self.field_stats {
            // For compound queries, false positive rate multiplies
            total_rate *= field_stats.estimated_false_positive_rate;
        }

        total_rate
    }

    /// Check if the bloom filter system is performing well
    pub fn is_performing_well(&self) -> bool {
        self.effectiveness > 50.0 && // At least 50% of queries are true negatives
        self.fact_id_stats.estimated_false_positive_rate < 0.05 && // Less than 5% false positive rate
        self.fact_id_stats.is_well_tuned()
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
    fn test_bloom_filter_basic_operations() {
        let mut filter = BloomFilter::new(1000, 3);

        // Add some fact IDs
        filter.add_fact_id(1);
        filter.add_fact_id(2);
        filter.add_fact_id(100);

        // Check membership
        assert!(filter.might_contain_fact_id(1));
        assert!(filter.might_contain_fact_id(2));
        assert!(filter.might_contain_fact_id(100));

        // Check non-existent (should be false for most cases)
        // Note: There's a small chance of false positives
        let mut false_positives = 0;
        for i in 1000..1100 {
            if filter.might_contain_fact_id(i) {
                false_positives += 1;
            }
        }

        // With good parameters, false positive rate should be low
        let fp_rate = false_positives as f64 / 100.0;
        assert!(fp_rate < 0.1); // Less than 10% false positive rate
    }

    #[test]
    fn test_bloom_filter_with_capacity() {
        let filter = BloomFilter::with_capacity(1000, 0.01);
        let stats = filter.stats();

        assert!(stats.size_bits > 0);
        assert!(stats.hash_functions > 0);
        assert_eq!(stats.element_count, 0);
    }

    #[test]
    fn test_fact_bloom_filter() {
        let mut filter = FactBloomFilter::with_capacity(100);

        // Add some facts
        let fact1 = create_test_fact(1, "name", FactValue::String("Alice".to_string()));
        let fact2 = create_test_fact(2, "age", FactValue::Integer(30));

        filter.add_fact(&fact1);
        filter.add_fact(&fact2);

        // Check existence
        assert!(filter.might_contain_fact(1));
        assert!(filter.might_contain_fact(2));

        // Check non-existent fact
        let _might_exist = filter.might_contain_fact(999);
        // Could be false positive, but likely false

        let stats = filter.stats();
        assert!(stats.query_count > 0);
    }

    #[test]
    fn test_bloom_filter_statistics() {
        let mut filter = BloomFilter::new(1000, 4);

        // Add many elements
        for i in 0..500 {
            filter.add_fact_id(i);
        }

        let stats = filter.stats();
        assert_eq!(stats.element_count, 500);
        assert!(stats.bits_set > 0);
        assert!(stats.estimated_false_positive_rate > 0.0);
        assert!(stats.memory_usage_bytes > 0);
        assert!(stats.utilization_percentage() > 0.0);
    }

    #[test]
    fn test_bloom_filter_clear() {
        let mut filter = BloomFilter::new(100, 2);

        filter.add_fact_id(1);
        filter.add_fact_id(2);
        assert!(filter.might_contain_fact_id(1));

        filter.clear();

        // After clear, nothing should be found (no false positives for cleared filter)
        assert!(!filter.might_contain_fact_id(1));
        assert!(!filter.might_contain_fact_id(2));

        let stats = filter.stats();
        assert_eq!(stats.element_count, 0);
        assert_eq!(stats.bits_set, 0);
    }

    #[test]
    fn test_fact_bloom_filter_effectiveness() {
        let mut filter = FactBloomFilter::with_capacity(100);

        // Add facts with IDs 1-50
        for i in 1..=50 {
            let fact = create_test_fact(i, "test", FactValue::Integer(i as i64));
            filter.add_fact(&fact);
        }

        // Query for non-existent facts (IDs 100-150)
        let mut true_negatives = 0;
        for i in 100..=150 {
            if !filter.might_contain_fact(i) {
                true_negatives += 1;
            }
        }

        // Should have good effectiveness (most queries correctly identified as negative)
        let effectiveness = (true_negatives as f64 / 51.0) * 100.0;
        assert!(effectiveness > 80.0); // At least 80% effectiveness
    }
}
