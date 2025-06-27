use crate::cache::CacheStats;
use crate::types::{Fact, FactId, FactValue};
use std::collections::HashMap;

/// Abstraction for fact storage that allows swapping allocation strategies
pub trait FactStore: Send + Sync {
    /// Insert a fact and return its ID
    fn insert(&mut self, fact: Fact) -> FactId;

    /// Get a fact by ID
    fn get(&self, id: FactId) -> Option<&Fact>;

    /// Extend with facts from a Vec (object-safe alternative to generic extend)
    fn extend_from_vec(&mut self, facts: Vec<Fact>);

    /// Get the number of stored facts
    fn len(&self) -> usize;

    /// Check if empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all facts
    fn clear(&mut self);

    /// Find facts by field value (optimized lookup)
    fn find_by_field(&self, field: &str, value: &FactValue) -> Vec<&Fact>;

    /// Find facts by multiple field criteria
    fn find_by_criteria(&self, criteria: &[(String, FactValue)]) -> Vec<&Fact>;

    /// Get cache statistics (if caching is enabled)
    fn cache_stats(&self) -> Option<CacheStats> {
        None
    }

    /// Clear cache (if caching is enabled)
    fn clear_cache(&mut self) {
        // Default implementation does nothing
    }
}

/// Standard Vec-based fact store for baseline implementation
#[derive(Debug, Default)]
pub struct VecFactStore {
    facts: Vec<Fact>,
    // Hash indexes for common lookup patterns
    field_indexes: HashMap<String, HashMap<String, Vec<FactId>>>,
}

impl VecFactStore {
    /// Create a new Vec-based fact store
    pub fn new() -> Self {
        Self { facts: Vec::new(), field_indexes: HashMap::new() }
    }

    /// Create with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self { facts: Vec::with_capacity(capacity), field_indexes: HashMap::new() }
    }

    /// Update indexes when a fact is added (only index commonly used fields for performance)
    fn update_indexes(&mut self, fact: &Fact) {
        // Only index fields that are commonly used for lookups
        let indexed_fields = ["entity_id", "id", "user_id", "customer_id", "status", "category"];

        for (field_name, field_value) in &fact.data.fields {
            if indexed_fields.contains(&field_name.as_str()) {
                let value_key = self.fact_value_to_index_key(field_value);

                self.field_indexes
                    .entry(field_name.clone())
                    .or_default()
                    .entry(value_key)
                    .or_default()
                    .push(fact.id);
            }
        }
    }

    /// Convert FactValue to string key for indexing
    fn fact_value_to_index_key(&self, value: &FactValue) -> String {
        value.as_string()
    }

    /// Get all facts as a slice
    pub fn facts(&self) -> &[Fact] {
        &self.facts
    }

    /// Insert a fact with a predetermined ID (for partitioned stores)
    pub fn insert_with_id(&mut self, mut fact: Fact, predetermined_id: FactId) -> FactId {
        fact.id = predetermined_id;
        self.update_indexes(&fact);
        self.facts.push(fact);
        predetermined_id
    }
}

impl FactStore for VecFactStore {
    fn insert(&mut self, fact: Fact) -> FactId {
        let id = self.facts.len() as FactId;

        // Set the ID based on position, then update indexes
        let mut indexed_fact = fact;
        indexed_fact.id = id;

        // Update indexes with the fact that has proper ID
        self.update_indexes(&indexed_fact);

        self.facts.push(indexed_fact);
        id
    }

    fn get(&self, id: FactId) -> Option<&Fact> {
        // First try the fast path: if ID matches array index, use direct access
        if let Some(fact) = self.facts.get(id as usize) {
            if fact.id == id {
                return Some(fact);
            }
        }

        // Fallback: search linearly for the correct fact ID
        // This is necessary when facts have non-sequential IDs (e.g., in partitioned stores)
        self.facts.iter().find(|fact| fact.id == id)
    }

    fn extend_from_vec(&mut self, facts: Vec<Fact>) {
        for fact in facts {
            self.insert(fact);
        }
    }

    fn len(&self) -> usize {
        self.facts.len()
    }

    fn clear(&mut self) {
        self.facts.clear();
        self.field_indexes.clear();
    }

    fn find_by_field(&self, field: &str, value: &FactValue) -> Vec<&Fact> {
        let value_key = self.fact_value_to_index_key(value);

        if let Some(field_index) = self.field_indexes.get(field) {
            if let Some(fact_ids) = field_index.get(&value_key) {
                return fact_ids.iter().filter_map(|&id| self.get(id)).collect();
            }
        }

        Vec::new()
    }

    fn find_by_criteria(&self, criteria: &[(String, FactValue)]) -> Vec<&Fact> {
        if criteria.is_empty() {
            return Vec::new();
        }

        // Start with facts matching the first criterion
        let (first_field, first_value) = &criteria[0];
        let mut candidates = self.find_by_field(first_field, first_value);

        // Filter by remaining criteria
        for (field, value) in &criteria[1..] {
            candidates.retain(|fact| {
                fact.data
                    .fields
                    .get(field)
                    .map(|fact_value| fact_value == value)
                    .unwrap_or(false)
            });
        }

        candidates
    }
}

impl VecFactStore {
    /// Extend with any iterator (convenience method for VecFactStore)
    pub fn extend<I: IntoIterator<Item = Fact>>(&mut self, facts: I) {
        self.facts.extend(facts);
    }
}

mod arena_store {
    use super::*;
    use std::collections::HashMap;

    /// Arena-based fact store for high-performance allocation (Phase 2+)
    /// Note: Optimized with direct Vec indexing for maximum performance + thread safety
    #[derive(Default)]
    pub struct ArenaFactStore {
        facts: Vec<Option<Fact>>, // Direct indexing: fact.id == Vec index
        field_indexes: HashMap<String, HashMap<String, Vec<FactId>>>,
        next_id: FactId,
    }

    impl ArenaFactStore {
        pub fn new() -> Self {
            Self { facts: Vec::new(), field_indexes: HashMap::new(), next_id: 0 }
        }

        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                facts: Vec::with_capacity(capacity),
                field_indexes: HashMap::with_capacity(6), // Pre-allocate for common indexed fields
                next_id: 0,
            }
        }

        /// Create with optimized settings for large datasets (1M+ facts)
        pub fn with_large_capacity(capacity: usize) -> Self {
            Self {
                facts: Vec::with_capacity(capacity),
                field_indexes: HashMap::with_capacity(10), // More indexed fields for large datasets
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
                FactValue::Date(date) => date.to_rfc3339(),
                FactValue::Null => "[null]".to_string(),
            }
        }
    }

    impl FactStore for ArenaFactStore {
        fn insert(&mut self, fact: Fact) -> FactId {
            // Use Vec length as direct ID for O(1) access
            let id = self.facts.len() as FactId;

            // Set the fact ID and update indexes
            let mut indexed_fact = fact;
            indexed_fact.id = id;
            self.update_indexes(&indexed_fact);

            // Direct Vec storage: fact.id == Vec index
            self.facts.push(Some(indexed_fact));
            self.next_id = id + 1;
            id
        }

        fn get(&self, id: FactId) -> Option<&Fact> {
            // O(1) direct access - no HashMap lookup overhead
            self.facts.get(id as usize)?.as_ref()
        }

        fn extend_from_vec(&mut self, facts: Vec<Fact>) {
            for fact in facts {
                self.insert(fact);
            }
        }

        fn len(&self) -> usize {
            // Count actual facts (exclude None slots)
            self.facts.iter().filter(|fact| fact.is_some()).count()
        }

        fn is_empty(&self) -> bool {
            self.facts.iter().all(|fact| fact.is_none())
        }

        fn clear(&mut self) {
            self.facts.clear();
            self.field_indexes.clear();
            self.next_id = 0;
        }

        fn find_by_field(&self, field: &str, value: &FactValue) -> Vec<&Fact> {
            let value_key = self.fact_value_to_index_key(value);

            if let Some(field_index) = self.field_indexes.get(field) {
                if let Some(fact_ids) = field_index.get(&value_key) {
                    return fact_ids.iter().filter_map(|&id| self.get(id)).collect();
                }
            }

            Vec::new()
        }

        fn find_by_criteria(&self, criteria: &[(String, FactValue)]) -> Vec<&Fact> {
            if criteria.is_empty() {
                return Vec::new();
            }

            // Start with facts matching the first criterion
            let (first_field, first_value) = &criteria[0];
            let mut candidates = self.find_by_field(first_field, first_value);

            // Filter by remaining criteria
            for (field, value) in &criteria[1..] {
                candidates.retain(|fact| {
                    fact.data
                        .fields
                        .get(field)
                        .map(|fact_value| fact_value == value)
                        .unwrap_or(false)
                });
            }

            candidates
        }
    }
}

pub use arena_store::ArenaFactStore;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FactData, FactValue};
    use std::collections::HashMap;

    fn create_test_fact(id: u64) -> Fact {
        let mut fields = HashMap::new();
        fields.insert("test_field".to_string(), FactValue::Integer(id as i64));

        Fact { id, data: FactData { fields } }
    }

    #[test]
    fn test_vec_fact_store() {
        let mut store = VecFactStore::new();

        let fact1 = create_test_fact(1);
        let fact2 = create_test_fact(2);

        let id1 = store.insert(fact1.clone());
        let id2 = store.insert(fact2.clone());

        assert_eq!(store.len(), 2);
        assert_eq!(store.get(id1).unwrap().id, id1); // ID gets overridden to index
        assert_eq!(store.get(id2).unwrap().id, id2); // ID gets overridden to index

        let facts = vec![create_test_fact(3), create_test_fact(4)];
        store.extend_from_vec(facts);

        assert_eq!(store.len(), 4);
    }

    #[test]
    fn test_vec_fact_store_with_capacity() {
        let store = VecFactStore::with_capacity(100);
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }
}
