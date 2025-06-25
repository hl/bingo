//! Shared field indexing utilities for fact storage systems
//!
//! This module consolidates the field indexing logic that was previously
//! duplicated across VecFactStore, ArenaFactStore, and OptimizedFactStore.

use crate::types::{Fact, FactId, FactValue};
use std::collections::HashMap;

/// Shared field indexing system for efficient fact lookups
#[derive(Debug, Clone, Default)]
pub struct FieldIndexer {
    /// Field indexes mapping field_name -> field_value -> fact_ids
    indexes: HashMap<String, HashMap<String, Vec<FactId>>>,
    /// Configurable list of fields to index
    indexed_fields: Vec<String>,
}

impl FieldIndexer {
    /// Create a new field indexer with default indexed fields
    pub fn new() -> Self {
        Self { indexes: HashMap::new(), indexed_fields: Self::default_indexed_fields() }
    }

    /// Create a field indexer with custom indexed fields
    pub fn with_fields(fields: Vec<String>) -> Self {
        Self { indexes: HashMap::new(), indexed_fields: fields }
    }

    /// Get the default list of commonly indexed fields
    pub fn default_indexed_fields() -> Vec<String> {
        vec![
            "entity_id".to_string(),
            "id".to_string(),
            "user_id".to_string(),
            "customer_id".to_string(),
            "status".to_string(),
            "category".to_string(),
        ]
    }

    /// Add a fact to the indexes
    pub fn index_fact(&mut self, fact: &Fact) {
        for (field_name, field_value) in &fact.data.fields {
            if self.indexed_fields.contains(field_name) {
                let value_key = self.fact_value_to_index_key(field_value);

                self.indexes
                    .entry(field_name.clone())
                    .or_default()
                    .entry(value_key)
                    .or_default()
                    .push(fact.id);
            }
        }
    }

    /// Remove a fact from the indexes
    pub fn remove_fact(&mut self, fact: &Fact) {
        for (field_name, field_value) in &fact.data.fields {
            if self.indexed_fields.contains(field_name) {
                let value_key = self.fact_value_to_index_key(field_value);

                if let Some(field_index) = self.indexes.get_mut(field_name) {
                    if let Some(fact_ids) = field_index.get_mut(&value_key) {
                        fact_ids.retain(|&id| id != fact.id);

                        // Clean up empty vectors
                        if fact_ids.is_empty() {
                            field_index.remove(&value_key);
                        }
                    }

                    // Clean up empty field indexes
                    if field_index.is_empty() {
                        self.indexes.remove(field_name);
                    }
                }
            }
        }
    }

    /// Find facts by field value
    pub fn find_by_field(&self, field: &str, value: &FactValue) -> Vec<FactId> {
        let value_key = self.fact_value_to_index_key(value);

        if let Some(field_index) = self.indexes.get(field) {
            if let Some(fact_ids) = field_index.get(&value_key) {
                return fact_ids.clone();
            }
        }

        Vec::new()
    }

    /// Find facts matching multiple criteria (AND logic)
    pub fn find_by_criteria(&self, criteria: &[(String, FactValue)]) -> Vec<FactId> {
        if criteria.is_empty() {
            return Vec::new();
        }

        // Start with facts matching the first criterion
        let (first_field, first_value) = &criteria[0];
        let mut candidates = self.find_by_field(first_field, first_value);

        // Filter by remaining criteria
        for (field, value) in &criteria[1..] {
            let matching_ids = self.find_by_field(field, value);
            candidates.retain(|id| matching_ids.contains(id));
        }

        candidates
    }

    /// Clear all indexes
    pub fn clear(&mut self) {
        self.indexes.clear();
    }

    /// Get statistics about the indexes
    pub fn stats(&self) -> FieldIndexStats {
        let total_indexes = self.indexes.len();
        let total_entries: usize = self.indexes.values().map(|field_index| field_index.len()).sum();
        let total_fact_refs: usize = self
            .indexes
            .values()
            .flat_map(|field_index| field_index.values())
            .map(|fact_ids| fact_ids.len())
            .sum();

        FieldIndexStats {
            indexed_fields: self.indexed_fields.len(),
            total_indexes,
            total_entries,
            total_fact_refs,
            memory_usage_bytes: self.estimate_memory_usage(),
        }
    }

    /// Get the list of currently indexed fields
    pub fn get_indexed_fields(&self) -> &[String] {
        &self.indexed_fields
    }

    /// Update the list of fields to index (clears existing indexes)
    pub fn set_indexed_fields(&mut self, fields: Vec<String>) {
        self.clear();
        self.indexed_fields = fields;
    }

    /// Convert FactValue to index key (consistent across all implementations)
    fn fact_value_to_index_key(&self, value: &FactValue) -> String {
        value.as_string()
    }

    /// Estimate memory usage of the indexes (rough calculation)
    fn estimate_memory_usage(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();

        for (field_name, field_index) in &self.indexes {
            size += field_name.len();
            size += std::mem::size_of::<HashMap<String, Vec<FactId>>>();

            for (value_key, fact_ids) in field_index {
                size += value_key.len();
                size += fact_ids.len() * std::mem::size_of::<FactId>();
                size += std::mem::size_of::<Vec<FactId>>();
            }
        }

        size
    }
}

/// Statistics about field indexing performance
#[derive(Debug, Clone)]
pub struct FieldIndexStats {
    pub indexed_fields: usize,
    pub total_indexes: usize,
    pub total_entries: usize,
    pub total_fact_refs: usize,
    pub memory_usage_bytes: usize,
}

impl FieldIndexStats {
    /// Get average entries per index
    pub fn avg_entries_per_index(&self) -> f64 {
        if self.total_indexes == 0 {
            0.0
        } else {
            self.total_entries as f64 / self.total_indexes as f64
        }
    }

    /// Get average fact references per entry
    pub fn avg_facts_per_entry(&self) -> f64 {
        if self.total_entries == 0 {
            0.0
        } else {
            self.total_fact_refs as f64 / self.total_entries as f64
        }
    }

    /// Get index efficiency (lower is better - indicates good selectivity)
    pub fn index_efficiency(&self) -> f64 {
        self.avg_facts_per_entry()
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
    fn test_field_indexer_basic_operations() {
        let mut indexer = FieldIndexer::new();

        // Create test facts
        let fact1 = create_test_fact(1, "status", FactValue::String("active".to_string()));
        let fact2 = create_test_fact(2, "status", FactValue::String("inactive".to_string()));
        let fact3 = create_test_fact(3, "status", FactValue::String("active".to_string()));

        // Index facts
        indexer.index_fact(&fact1);
        indexer.index_fact(&fact2);
        indexer.index_fact(&fact3);

        // Test single field lookup
        let active_facts =
            indexer.find_by_field("status", &FactValue::String("active".to_string()));
        assert_eq!(active_facts.len(), 2);
        assert!(active_facts.contains(&1));
        assert!(active_facts.contains(&3));

        let inactive_facts =
            indexer.find_by_field("status", &FactValue::String("inactive".to_string()));
        assert_eq!(inactive_facts.len(), 1);
        assert!(inactive_facts.contains(&2));

        // Test non-indexed field
        let non_indexed =
            indexer.find_by_field("non_indexed", &FactValue::String("test".to_string()));
        assert!(non_indexed.is_empty());
    }

    #[test]
    fn test_field_indexer_multiple_criteria() {
        let mut indexer = FieldIndexer::new();

        // Create facts with multiple indexed fields
        let mut fields1 = HashMap::new();
        fields1.insert(
            "status".to_string(),
            FactValue::String("active".to_string()),
        );
        fields1.insert(
            "category".to_string(),
            FactValue::String("premium".to_string()),
        );
        let fact1 = Fact { id: 1, data: FactData { fields: fields1 } };

        let mut fields2 = HashMap::new();
        fields2.insert(
            "status".to_string(),
            FactValue::String("active".to_string()),
        );
        fields2.insert(
            "category".to_string(),
            FactValue::String("basic".to_string()),
        );
        let fact2 = Fact { id: 2, data: FactData { fields: fields2 } };

        let mut fields3 = HashMap::new();
        fields3.insert(
            "status".to_string(),
            FactValue::String("inactive".to_string()),
        );
        fields3.insert(
            "category".to_string(),
            FactValue::String("premium".to_string()),
        );
        let fact3 = Fact { id: 3, data: FactData { fields: fields3 } };

        indexer.index_fact(&fact1);
        indexer.index_fact(&fact2);
        indexer.index_fact(&fact3);

        // Test multiple criteria
        let criteria = vec![
            (
                "status".to_string(),
                FactValue::String("active".to_string()),
            ),
            (
                "category".to_string(),
                FactValue::String("premium".to_string()),
            ),
        ];

        let matching_facts = indexer.find_by_criteria(&criteria);
        assert_eq!(matching_facts.len(), 1);
        assert!(matching_facts.contains(&1));
    }

    #[test]
    fn test_field_indexer_removal() {
        let mut indexer = FieldIndexer::new();

        let fact = create_test_fact(1, "status", FactValue::String("active".to_string()));

        // Index and then remove
        indexer.index_fact(&fact);
        let before_removal =
            indexer.find_by_field("status", &FactValue::String("active".to_string()));
        assert_eq!(before_removal.len(), 1);

        indexer.remove_fact(&fact);
        let after_removal =
            indexer.find_by_field("status", &FactValue::String("active".to_string()));
        assert!(after_removal.is_empty());
    }

    #[test]
    fn test_field_indexer_custom_fields() {
        let custom_fields = vec!["custom_field".to_string(), "another_field".to_string()];
        let mut indexer = FieldIndexer::with_fields(custom_fields);

        let fact = create_test_fact(1, "custom_field", FactValue::String("value".to_string()));
        indexer.index_fact(&fact);

        let results =
            indexer.find_by_field("custom_field", &FactValue::String("value".to_string()));
        assert_eq!(results.len(), 1);

        // Default fields should not be indexed
        let fact2 = create_test_fact(2, "status", FactValue::String("active".to_string()));
        indexer.index_fact(&fact2);
        let no_results = indexer.find_by_field("status", &FactValue::String("active".to_string()));
        assert!(no_results.is_empty());
    }

    #[test]
    fn test_field_indexer_stats() {
        let mut indexer = FieldIndexer::new();

        // Add some test data
        for i in 1..=10 {
            let fact =
                create_test_fact(i, "status", FactValue::String(format!("status_{}", i % 3)));
            indexer.index_fact(&fact);
        }

        let stats = indexer.stats();
        assert_eq!(stats.indexed_fields, 6); // Default fields count
        assert_eq!(stats.total_indexes, 1); // Only "status" field was used
        assert_eq!(stats.total_entries, 3); // 3 different status values
        assert_eq!(stats.total_fact_refs, 10); // 10 facts indexed
        assert!(stats.memory_usage_bytes > 0);

        // Test calculated metrics
        assert!(stats.avg_entries_per_index() > 0.0);
        assert!(stats.avg_facts_per_entry() > 0.0);
    }
}
