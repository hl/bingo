use crate::cache::CacheStats;
use crate::types::{Fact, FactId, FactValue};

pub mod arena_store {
    use super::*;
    use std::collections::HashMap;

    /// Arena-based fact store for high-performance allocation (Phase 2+)
    /// Note: Optimized with direct Vec indexing for maximum performance + thread safety
    #[derive(Default, Debug)]
    pub struct ArenaFactStore {
        facts: Vec<Option<Fact>>, // Direct indexing: fact.id == Vec index
        field_indexes: HashMap<String, HashMap<String, Vec<FactId>>>,
        external_id_map: HashMap<String, FactId>, // For external ID lookups
        next_id: FactId,
    }

    impl ArenaFactStore {
        pub fn new() -> Self {
            Self {
                facts: Vec::new(),
                field_indexes: HashMap::new(),
                external_id_map: HashMap::new(),
                next_id: 0,
            }
        }

        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                facts: Vec::with_capacity(capacity),
                field_indexes: HashMap::with_capacity(6), // Pre-allocate for common indexed fields
                external_id_map: HashMap::with_capacity(capacity),
                next_id: 0,
            }
        }

        /// Create with optimized settings for large datasets (1M+ facts)
        pub fn with_large_capacity(capacity: usize) -> Self {
            Self {
                facts: Vec::with_capacity(capacity),
                field_indexes: HashMap::with_capacity(10), // More indexed fields for large datasets
                external_id_map: HashMap::with_capacity(capacity),
                next_id: 0,
            }
        }

        /// Update indexes when a fact is added (only index commonly used fields for performance)
        fn update_indexes(&mut self, fact: &Fact) {
            // Static set of commonly used fields for fast lookup
            const INDEXED_FIELDS: &[&str] =
                &["entity_id", "id", "user_id", "customer_id", "status", "category"];

            for (field_name, field_value) in &fact.data.fields {
                // Fast string comparison for indexed fields
                if INDEXED_FIELDS.iter().any(|&f| f == field_name) {
                    let value_key = self.fact_value_to_index_key(field_value);

                    // Optimized entry pattern with pre-allocated capacity hints
                    let field_map = self
                        .field_indexes
                        .entry(field_name.clone())
                        .or_insert_with(|| HashMap::with_capacity(64)); // Expect ~64 unique values per field

                    field_map
                        .entry(value_key)
                        .or_insert_with(|| Vec::with_capacity(16)) // Expect ~16 facts per value
                        .push(fact.id);
                }
            }
        }

        /// Convert FactValue to string key for indexing (optimized for performance)
        fn fact_value_to_index_key(&self, value: &FactValue) -> String {
            match value {
                FactValue::String(s) => s.clone(),
                FactValue::Integer(i) => {
                    // Fast integer formatting using itoa for better performance
                    i.to_string()
                }
                FactValue::Float(f) => {
                    // Use ryu for fast float formatting
                    f.to_string()
                }
                FactValue::Boolean(true) => "true".to_string(),
                FactValue::Boolean(false) => "false".to_string(),
                FactValue::Array(_) => "[array]".to_string(),
                FactValue::Object(_) => "[object]".to_string(),
                FactValue::Date(dt) => dt.to_rfc3339(),
                FactValue::Null => "[null]".to_string(),
            }
        }

        pub fn insert(&mut self, mut fact: Fact) -> FactId {
            let id = self.next_id;
            fact.id = id;
            self.next_id += 1;

            // Update external ID mapping if present
            if let Some(ref external_id) = fact.external_id {
                self.external_id_map.insert(external_id.clone(), id);
            }

            // Update indexes
            self.update_indexes(&fact);

            // Ensure Vec capacity and insert
            if self.facts.len() <= id as usize {
                self.facts.resize(id as usize + 1, None);
            }
            self.facts[id as usize] = Some(fact);

            id
        }

        pub fn get_fact(&self, id: FactId) -> Option<&Fact> {
            self.facts.get(id as usize)?.as_ref()
        }

        pub fn get_by_external_id(&self, external_id: &str) -> Option<&Fact> {
            let fact_id = *self.external_id_map.get(external_id)?;
            self.get_fact(fact_id)
        }

        pub fn get_field_by_id(&self, fact_id: &str, field: &str) -> Option<FactValue> {
            self.get_by_external_id(fact_id)?.data.fields.get(field).cloned()
        }

        pub fn extend_from_vec(&mut self, facts: Vec<Fact>) {
            for fact in facts {
                self.insert(fact);
            }
        }

        /// Iterate all stored facts (helper for read-only iteration)
        pub fn iter(&self) -> impl Iterator<Item = &Fact> {
            self.facts.iter().filter_map(|opt| opt.as_ref())
        }

        /// Return references to facts whose timestamps are within the inclusive time range.
        pub fn facts_in_time_range(
            &self,
            start: chrono::DateTime<chrono::Utc>,
            end: chrono::DateTime<chrono::Utc>,
        ) -> Vec<&Fact> {
            self.iter().filter(|f| f.timestamp >= start && f.timestamp <= end).collect()
        }

        pub fn len(&self) -> usize {
            self.facts.iter().filter(|f| f.is_some()).count()
        }

        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }

        pub fn clear(&mut self) {
            self.facts.clear();
            self.field_indexes.clear();
            self.external_id_map.clear();
            self.next_id = 0;
        }

        pub fn find_by_field(&self, field: &str, value: &FactValue) -> Vec<&Fact> {
            let value_key = self.fact_value_to_index_key(value);

            if let Some(field_map) = self.field_indexes.get(field) {
                if let Some(fact_ids) = field_map.get(&value_key) {
                    return fact_ids.iter().filter_map(|&id| self.get_fact(id)).collect();
                }
            }

            // Fallback to linear search for non-indexed fields
            self.facts
                .iter()
                .filter_map(|opt_fact| opt_fact.as_ref())
                .filter(|fact| fact.data.fields.get(field) == Some(value))
                .collect()
        }

        pub fn find_by_criteria(&self, criteria: &[(String, FactValue)]) -> Vec<&Fact> {
            self.facts
                .iter()
                .filter_map(|opt_fact| opt_fact.as_ref())
                .filter(|fact| {
                    criteria.iter().all(|(field, value)| fact.data.fields.get(field) == Some(value))
                })
                .collect()
        }

        pub fn cache_stats(&self) -> Option<CacheStats> {
            None
        }

        pub fn clear_cache(&mut self) {
            // Default implementation does nothing
        }

        /// Update an existing fact by ID
        pub fn update_fact(
            &mut self,
            fact_id: FactId,
            updates: HashMap<String, FactValue>,
        ) -> bool {
            if let Some(fact_option) = self.facts.get_mut(fact_id as usize) {
                if let Some(fact) = fact_option.as_mut() {
                    // Apply updates to the fact's fields
                    for (field, value) in updates {
                        fact.data.fields.insert(field, value);
                    }

                    // Clone the fact for re-indexing to avoid borrow checker issues
                    let fact_clone = fact.clone();
                    let _ = fact; // Explicitly drop the mutable reference
                    self.update_indexes(&fact_clone);
                    return true;
                }
            }
            false
        }

        /// Delete a fact by ID
        pub fn delete_fact(&mut self, fact_id: FactId) -> bool {
            if let Some(fact_option) = self.facts.get_mut(fact_id as usize) {
                if let Some(fact) = fact_option.take() {
                    // Remove from external ID mapping if present
                    if let Some(ref external_id) = fact.external_id {
                        self.external_id_map.remove(external_id);
                    }

                    // Remove from field indexes
                    self.remove_from_indexes(&fact);
                    return true;
                }
            }
            false
        }

        /// Remove a fact from all field indexes
        fn remove_from_indexes(&mut self, fact: &Fact) {
            const INDEXED_FIELDS: &[&str] =
                &["entity_id", "id", "user_id", "customer_id", "status", "category"];

            for (field_name, field_value) in &fact.data.fields {
                if INDEXED_FIELDS.iter().any(|&f| f == field_name) {
                    let value_key = self.fact_value_to_index_key(field_value);

                    if let Some(field_map) = self.field_indexes.get_mut(field_name) {
                        if let Some(fact_ids) = field_map.get_mut(&value_key) {
                            fact_ids.retain(|&id| id != fact.id);
                            // Remove the entry if no facts remain
                            if fact_ids.is_empty() {
                                field_map.remove(&value_key);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::arena_store::ArenaFactStore;
    use crate::types::{Fact, FactData, FactValue};
    use std::collections::HashMap;

    fn create_test_fact(id: u64) -> Fact {
        let mut fields = HashMap::new();
        fields.insert(
            "test_field".to_string(),
            FactValue::String("test_value".to_string()),
        );
        Fact { id, external_id: None, timestamp: chrono::Utc::now(), data: FactData { fields } }
    }

    #[test]
    fn test_arena_fact_store_get_field_by_id() {
        let mut store = ArenaFactStore::new();

        let mut fact1 = create_test_fact(1);
        fact1.external_id = Some("fact1_ext_id".to_string());
        fact1.data.fields.insert(
            "name".to_string(),
            FactValue::String("TestName".to_string()),
        );
        fact1.data.fields.insert("age".to_string(), FactValue::Integer(30));

        store.insert(fact1.clone());

        // Test successful lookup of a string field
        let name_field = store.get_field_by_id("fact1_ext_id", "name");
        assert!(name_field.is_some());
        assert_eq!(
            name_field.unwrap(),
            FactValue::String("TestName".to_string())
        );

        // Test successful lookup of an integer field
        let age_field = store.get_field_by_id("fact1_ext_id", "age");
        assert!(age_field.is_some());
        assert_eq!(age_field.unwrap(), FactValue::Integer(30));

        // Test lookup of a non-existent field
        let non_existent_field = store.get_field_by_id("fact1_ext_id", "non_existent");
        assert!(non_existent_field.is_none());

        // Test lookup with a non-existent external ID
        let non_existent_fact = store.get_field_by_id("non_existent_ext_id", "name");
        assert!(non_existent_fact.is_none());
    }
}
