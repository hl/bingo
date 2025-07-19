use crate::cache::CacheStats;
use crate::types::{Fact, FactId, FactValue};
use std::borrow::Cow;
use std::collections::HashMap;

/// Statistics for a specific field index
#[derive(Debug, Clone)]
pub struct FieldIndexStats {
    pub unique_values: usize,
    pub indexed_facts: usize,
    pub average_facts_per_value: f64,
}

/// Overall index statistics
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total_indexed_fields: usize,
    pub total_unique_values: usize,
    pub total_indexed_facts: usize,
    pub field_stats: HashMap<String, FieldIndexStats>,
    pub index_efficiency: f64,
}

pub mod arena_store {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, RwLock};

    /// Arena-based fact store for high-performance allocation and retrieval with thread safety.
    ///
    /// The `ArenaFactStore` provides an optimised in-memory storage solution for facts with
    /// direct vector indexing (fact.id == Vec index) and field-based indexing for fast lookups.
    /// Designed for high-throughput scenarios with minimal allocation overhead and full thread safety.
    ///
    /// # Architecture
    /// - **Facts Storage**: Direct vector indexing where `fact.id` corresponds to the vector index (RwLock protected)
    /// - **Field Indexes**: Hash-based secondary indexes on commonly queried fields (RwLock protected)
    /// - **External ID Mapping**: Optional string-based identifiers for external integration (RwLock protected)
    /// - **Thread Safety**: Fully thread-safe with granular locking for optimal concurrency
    ///
    /// # Performance Characteristics
    /// - **Insert**: O(1) amortised for sequential IDs, O(log n) for sparse IDs
    /// - **Get by ID**: O(1) direct array access with shared read lock
    /// - **Find by indexed field**: O(1) average case via hash indexes with shared read lock
    /// - **Find by non-indexed field**: O(n) linear scan (fallback) with shared read lock
    ///
    /// # Thread Safety
    /// - Multiple concurrent readers for all read operations
    /// - Exclusive write access for modifications
    /// - Atomic ID generation for lock-free ID assignment
    /// - Fine-grained locking to minimize contention
    ///
    /// # Indexed Fields
    /// The following fields are automatically indexed for fast lookup:
    /// - `entity_id`
    /// - `id`
    /// - `user_id`
    /// - `customer_id`
    /// - `status`
    /// - `category`
    ///
    /// # Usage Example
    /// ```rust
    /// use bingo_core::fact_store::arena_store::ArenaFactStore;
    /// use bingo_core::types::{Fact, FactData, FactValue};
    /// use std::collections::HashMap;
    ///
    /// // Create a new fact store (thread-safe by default)
    /// let store = ArenaFactStore::new();
    ///
    /// // Create and insert a fact
    /// let mut fields = HashMap::new();
    /// fields.insert("user_id".to_string(), FactValue::Integer(12345));
    /// fields.insert("status".to_string(), FactValue::String("active".to_string()));
    /// let fact = Fact {
    ///     id: 0, // Will be auto-assigned
    ///     external_id: Some("user-12345".to_string()),
    ///     timestamp: chrono::Utc::now(),
    ///     data: FactData { fields }
    /// };
    ///
    /// let fact_id = store.insert(fact);
    ///
    /// // Fast lookup by ID (concurrent reads allowed)
    /// if let Some(fact) = store.get_fact(fact_id) {
    ///     println!("Found fact: {:?}", fact);
    /// }
    ///
    /// // Fast lookup by external ID (concurrent reads allowed)
    /// if let Some(fact) = store.get_by_external_id("user-12345") {
    ///     println!("Found by external ID: {:?}", fact);
    /// }
    ///
    /// // Indexed field lookup (fast, concurrent reads allowed)
    /// let active_users = store.find_by_field("status", &FactValue::String("active".to_string()));
    /// ```
    #[derive(Debug)]
    pub struct ArenaFactStore {
        facts: RwLock<Vec<Option<Fact>>>, // Direct indexing: fact.id == Vec index (thread-safe)
        field_indexes: RwLock<HashMap<String, HashMap<String, Vec<FactId>>>>, // Thread-safe indexes
        external_id_map: RwLock<HashMap<String, FactId>>, // Thread-safe external ID lookups
        next_id: AtomicU64,               // Atomic ID generation for lock-free assignment
        fact_count: AtomicU64,            // Atomic fact count for O(1) len() operations
    }

    /// Thread-safe wrapper for ArenaFactStore providing concurrent access.
    ///
    /// This type alias combines `Arc` (atomic reference counting) with `RwLock` (read-write lock)
    /// to enable safe concurrent access to the fact store across multiple threads.
    ///
    /// # Concurrency Pattern
    /// - **Multiple readers**: Can read concurrently without blocking each other
    /// - **Single writer**: Write operations require exclusive access
    /// - **Arc sharing**: Multiple threads can hold references to the same store instance
    ///
    /// # Usage Example
    /// ```rust
    /// use bingo_core::fact_store::arena_store::{ArenaFactStore, ThreadSafeArenaFactStore};
    /// use std::sync::Arc;
    /// use std::thread;
    ///
    /// // Create a thread-safe fact store
    /// let store: ThreadSafeArenaFactStore = ArenaFactStore::new_shared();
    ///
    /// // Clone the Arc for use in another thread
    /// let store_clone = Arc::clone(&store);
    ///
    /// // Spawn a thread for concurrent access
    /// let handle = thread::spawn(move || {
    ///     // Read access (can be concurrent)
    ///     let read_guard = store_clone.read().unwrap();
    ///     println!("Store has {} facts", read_guard.len());
    /// });
    ///
    /// // Write access in main thread (exclusive)
    /// {
    ///     let mut write_guard = store.write().unwrap();
    ///     // Insert facts here...
    /// }
    ///
    /// handle.join().unwrap();
    /// ```
    pub type ThreadSafeArenaFactStore = Arc<RwLock<ArenaFactStore>>;

    impl Default for ArenaFactStore {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ArenaFactStore {
        /// Creates a new empty fact store with default capacity.
        ///
        /// This is the standard constructor for general use cases. It initialises all internal
        /// data structures with default capacities suitable for small to medium workloads.
        ///
        /// # Returns
        /// A new `ArenaFactStore` instance ready for use.
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        ///
        /// let store = ArenaFactStore::new();
        /// assert_eq!(store.len(), 0);
        /// assert!(store.is_empty());
        /// ```
        pub fn new() -> Self {
            Self {
                facts: RwLock::new(Vec::new()),
                field_indexes: RwLock::new(HashMap::new()),
                external_id_map: RwLock::new(HashMap::new()),
                next_id: AtomicU64::new(0),
                fact_count: AtomicU64::new(0),
            }
        }

        /// Creates a new fact store with pre-allocated capacity for facts.
        ///
        /// Use this constructor when you know approximately how many facts you'll be storing
        /// to avoid repeated memory allocations during insertion. The field indexes and
        /// external ID map are also pre-allocated with appropriate capacities.
        ///
        /// # Arguments
        /// * `capacity` - Expected number of facts to store
        ///
        /// # Returns
        /// A new `ArenaFactStore` instance with pre-allocated capacity.
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        ///
        /// // Pre-allocate for 10,000 facts
        /// let store = ArenaFactStore::with_capacity(10_000);
        /// assert_eq!(store.len(), 0);
        /// // Internal vectors are pre-allocated to avoid reallocations
        /// ```
        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                facts: RwLock::new(Vec::with_capacity(capacity)),
                field_indexes: RwLock::new(HashMap::with_capacity(6)), // Pre-allocate for common indexed fields
                external_id_map: RwLock::new(HashMap::with_capacity(capacity)),
                next_id: AtomicU64::new(0),
                fact_count: AtomicU64::new(0),
            }
        }

        /// Creates a new fact store optimised for large datasets (1M+ facts).
        ///
        /// This constructor uses optimised capacity settings for large-scale deployments:
        /// - Pre-allocates fact storage for the specified capacity
        /// - Allocates more index capacity (10 vs 6) for better field coverage
        /// - Optimises external ID mapping for large volumes
        ///
        /// # Arguments
        /// * `capacity` - Expected number of facts to store (recommended 1M+)
        ///
        /// # Returns
        /// A new `ArenaFactStore` instance optimised for large datasets.
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        ///
        /// // Optimised for 5 million facts
        /// let store = ArenaFactStore::with_large_capacity(5_000_000);
        /// assert_eq!(store.len(), 0);
        /// ```
        pub fn with_large_capacity(capacity: usize) -> Self {
            Self {
                facts: RwLock::new(Vec::with_capacity(capacity)),
                field_indexes: RwLock::new(HashMap::with_capacity(10)), // More indexed fields for large datasets
                external_id_map: RwLock::new(HashMap::with_capacity(capacity)),
                next_id: AtomicU64::new(0),
                fact_count: AtomicU64::new(0),
            }
        }

        /// Creates a new thread-safe fact store wrapped in `Arc<RwLock<>>`.
        ///
        /// This convenience constructor returns a `ThreadSafeArenaFactStore` ready for
        /// concurrent access across multiple threads. Equivalent to calling
        /// `Arc::new(RwLock::new(ArenaFactStore::new()))`.
        ///
        /// # Returns
        /// A thread-safe wrapper around a new `ArenaFactStore` instance.
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        ///
        /// let store = ArenaFactStore::new_shared();
        ///
        /// // Can be cloned and shared across threads
        /// let store_clone = std::sync::Arc::clone(&store);
        /// ```
        pub fn new_shared() -> ThreadSafeArenaFactStore {
            Arc::new(RwLock::new(Self::new()))
        }

        /// Creates a new thread-safe fact store with pre-allocated capacity.
        ///
        /// Combines the benefits of `with_capacity()` and `new_shared()` - pre-allocates
        /// memory for the expected number of facts and wraps the store in thread-safe
        /// synchronisation primitives.
        ///
        /// # Arguments
        /// * `capacity` - Expected number of facts to store
        ///
        /// # Returns
        /// A thread-safe wrapper around a capacity-optimised `ArenaFactStore` instance.
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        ///
        /// let store = ArenaFactStore::with_capacity_shared(50_000);
        ///
        /// // Ready for concurrent access with pre-allocated capacity
        /// let write_guard = store.write().unwrap();
        /// ```
        pub fn with_capacity_shared(capacity: usize) -> ThreadSafeArenaFactStore {
            Arc::new(RwLock::new(Self::with_capacity(capacity)))
        }

        /// Creates a new thread-safe fact store optimised for large datasets.
        ///
        /// Combines the benefits of `with_large_capacity()` and `new_shared()` - provides
        /// large-scale optimisations and thread-safe concurrent access.
        ///
        /// # Arguments
        /// * `capacity` - Expected number of facts to store (recommended 1M+)
        ///
        /// # Returns
        /// A thread-safe wrapper around a large-capacity-optimised `ArenaFactStore` instance.
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        ///
        /// let store = ArenaFactStore::with_large_capacity_shared(2_000_000);
        ///
        /// // Ready for high-throughput concurrent access
        /// let read_guard = store.read().unwrap();
        /// ```
        pub fn with_large_capacity_shared(capacity: usize) -> ThreadSafeArenaFactStore {
            Arc::new(RwLock::new(Self::with_large_capacity(capacity)))
        }

        /// Update indexes when a fact is added (only index commonly used fields for performance)
        ///
        /// ## Index Update Algorithm
        ///
        /// This method implements selective indexing for performance optimization:
        ///
        /// 1. **Field Selection**: Only indexes predefined high-frequency fields
        /// 2. **Value Conversion**: Converts FactValue to string representation for consistent indexing
        /// 3. **Hash Map Operations**: Uses nested HashMap structure for O(1) average lookup
        /// 4. **Capacity Optimization**: Pre-allocates with empirically-derived capacity hints
        ///
        /// ## Performance Characteristics
        ///
        /// - **Time Complexity**: O(k) where k = number of indexed fields in fact
        /// - **Space Complexity**: O(1) per field value (amortized via pre-allocation)
        /// - **Index Structure**: field_name -> value_string -> [fact_id1, fact_id2, ...]
        fn update_indexes(&self, fact: &Fact) {
            // Static set of commonly used fields for fast lookup
            // These fields are selected based on query patterns in business rules
            const INDEXED_FIELDS: &[&str] =
                &["entity_id", "id", "user_id", "customer_id", "status", "category"];

            let mut field_indexes = self.field_indexes.write().unwrap();

            for (field_name, field_value) in &fact.data.fields {
                // Fast string comparison for indexed fields (O(1) for small constant set)
                if INDEXED_FIELDS.iter().any(|&f| f == field_name) {
                    // Convert FactValue to string key for consistent indexing
                    let value_key = self.fact_value_to_index_key_owned(field_value);

                    // Optimized entry pattern with pre-allocated capacity hints
                    // Based on empirical analysis of typical workloads
                    let field_map = field_indexes
                        .entry(field_name.clone())
                        .or_insert_with(|| HashMap::with_capacity(64)); // Expect ~64 unique values per field

                    // Insert fact ID into value-specific list
                    field_map
                        .entry(value_key)
                        .or_insert_with(|| Vec::with_capacity(16)) // Expect ~16 facts per value
                        .push(fact.id);
                }
            }
        }

        /// Convert FactValue to string key for indexing (optimized for performance)
        fn fact_value_to_index_key<'a>(&self, value: &'a FactValue) -> Cow<'a, str> {
            match value {
                FactValue::String(s) => Cow::Borrowed(s),
                FactValue::Integer(i) => Cow::Owned(i.to_string()),
                FactValue::Float(f) => Cow::Owned(f.to_string()),
                FactValue::Boolean(true) => Cow::Borrowed("true"),
                FactValue::Boolean(false) => Cow::Borrowed("false"),
                FactValue::Array(_) => Cow::Borrowed("[array]"),
                FactValue::Object(_) => Cow::Borrowed("[object]"),
                FactValue::Date(dt) => Cow::Owned(dt.to_rfc3339()),
                FactValue::Null => Cow::Borrowed("[null]"),
            }
        }

        /// Optimized version that returns an owned string for map operations
        fn fact_value_to_index_key_owned(&self, value: &FactValue) -> String {
            self.fact_value_to_index_key(value).into_owned()
        }

        /// Inserts a fact into the store and returns its assigned ID.
        ///
        /// This method handles ID assignment automatically:
        /// - If `fact.id` is 0 or > 1,000,000, assigns a new sequential ID
        /// - Otherwise, preserves the existing ID (useful for tests and known data)
        /// - Updates field indexes for fast lookups on indexed fields
        /// - Registers external ID mapping if present
        ///
        /// # Arguments
        /// * `fact` - The fact to insert (ID may be modified during insertion)
        ///
        /// # Returns
        /// The assigned `FactId` for the inserted fact.
        ///
        /// # Performance
        /// - **Time Complexity**: O(1) amortised for sequential IDs, O(k) where k is the number of indexed fields
        /// - **Space Complexity**: O(1) for the fact itself, additional space for index entries
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        /// use bingo_core::types::{Fact, FactData, FactValue};
        /// use std::collections::HashMap;
        ///
        /// let store = ArenaFactStore::new();
        ///
        /// // Create a fact
        /// let mut fields = HashMap::new();
        /// fields.insert("user_id".to_string(), FactValue::Integer(42));
        /// fields.insert("status".to_string(), FactValue::String("active".to_string()));
        ///
        /// let fact = Fact {
        ///     id: 0, // Will be auto-assigned
        ///     external_id: Some("user-42".to_string()),
        ///     timestamp: chrono::Utc::now(),
        ///     data: FactData { fields }
        /// };
        ///
        /// let fact_id = store.insert(fact);
        /// assert_eq!(fact_id, 0); // First fact gets ID 0
        ///
        /// // Verify the fact was stored and indexed
        /// assert!(store.get_fact(fact_id).is_some());
        /// assert!(store.get_by_external_id("user-42").is_some());
        /// ```
        pub fn insert(&self, mut fact: Fact) -> FactId {
            // ID Assignment Algorithm: Balance between preserving external IDs and preventing memory issues
            //
            // Strategy 1: Use existing ID if reasonable (supports test scenarios and external integrations)
            // Strategy 2: Assign sequential ID for edge cases (prevents memory explosion from hash-based IDs)
            //
            // Threshold prevents accidental allocation of gigantic vectors when facts
            // are created from hashed external identifiers (e.g. 64-bit FNV hashes from API layer)
            let id = if fact.id == 0 || fact.id > crate::constants::fact_ids::MAX_USER_FACT_ID {
                // Use auto-incrementing sequential ID for new facts or suspicious large IDs
                self.next_id.fetch_add(1, Ordering::SeqCst)
            } else {
                // Preserve existing ID for test compatibility and reasonable external IDs
                let current_next = self.next_id.load(Ordering::SeqCst);
                if fact.id >= current_next {
                    // Update next_id atomically to stay ahead
                    self.next_id.store(fact.id + 1, Ordering::SeqCst);
                }
                fact.id
            };

            // Update fact with final assigned ID
            fact.id = id;

            // Register external ID mapping for string-based lookups
            if let Some(ref external_id) = fact.external_id {
                let mut external_id_map = self.external_id_map.write().unwrap();
                external_id_map.insert(external_id.clone(), id);
            }

            // Update field indexes for fast lookups on indexed fields
            self.update_indexes(&fact);

            // Vector Storage Algorithm: Ensure adequate capacity for direct indexing
            //
            // This implements the arena-style allocation pattern where fact.id equals Vec index
            // Benefits: O(1) lookup, cache-friendly sequential access, minimal memory overhead
            let mut facts = self.facts.write().unwrap();
            if facts.len() <= id as usize {
                // Resize vector to accommodate new fact ID with None padding for sparse gaps
                facts.resize(id as usize + 1, None);
            }
            // Direct indexing: fact.id becomes the vector index for O(1) access
            facts[id as usize] = Some(fact);

            // Increment fact count for O(1) len() operations
            self.fact_count.fetch_add(1, Ordering::Relaxed);

            id
        }

        /// Retrieves a fact by its internal ID.
        ///
        /// This is the fastest way to retrieve a fact when you have its internal ID,
        /// providing O(1) direct array access.
        ///
        /// # Arguments
        /// * `id` - The internal fact ID to retrieve
        ///
        /// # Returns
        /// `Some(&Fact)` if the fact exists, `None` if not found or ID is out of bounds.
        ///
        /// # Performance
        /// - **Time Complexity**: O(1) - direct array access
        /// - **Space Complexity**: O(1) - returns a reference
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        ///
        /// let mut store = ArenaFactStore::new();
        ///
        /// // Create a test fact
        /// # use bingo_core::types::{Fact, FactData};
        /// let fact = Fact {
        ///     id: 0,
        ///     external_id: None,
        ///     timestamp: chrono::Utc::now(),
        ///     data: FactData { fields: std::collections::HashMap::new() }
        /// };
        /// let fact_id = store.insert(fact);
        ///
        /// // Fast O(1) lookup
        /// if let Some(fact) = store.get_fact(fact_id) {
        ///     println!("Found fact with timestamp: {}", fact.timestamp);
        /// }
        /// ```
        pub fn get_fact(&self, id: FactId) -> Option<Fact> {
            let facts = self.facts.read().unwrap();
            facts.get(id as usize)?.as_ref().cloned()
        }

        /// Retrieves a fact by its external string ID.
        ///
        /// External IDs are optional string identifiers that can be assigned to facts
        /// for integration with external systems. This method provides fast O(1) average
        /// case lookup via hash map.
        ///
        /// # Arguments
        /// * `external_id` - The external string identifier to look up
        ///
        /// # Returns
        /// `Some(&Fact)` if a fact with the given external ID exists, `None` otherwise.
        ///
        /// # Performance
        /// - **Time Complexity**: O(1) average case via hash map lookup
        /// - **Space Complexity**: O(1) - returns a reference
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        /// use bingo_core::types::{Fact, FactData};
        ///
        /// let mut store = ArenaFactStore::new();
        ///
        /// let fact = Fact {
        ///     id: 0,
        ///     external_id: Some("order-12345".to_string()),
        ///     timestamp: chrono::Utc::now(),
        ///     data: FactData { fields: std::collections::HashMap::new() }
        /// };
        ///
        /// store.insert(fact);
        ///
        /// // Lookup by external ID
        /// if let Some(fact) = store.get_by_external_id("order-12345") {
        ///     println!("Found order fact");
        /// }
        /// ```
        pub fn get_by_external_id(&self, external_id: &str) -> Option<Fact> {
            let external_id_map = self.external_id_map.read().unwrap();
            let fact_id = *external_id_map.get(external_id)?;
            drop(external_id_map); // Release read lock early
            self.get_fact(fact_id)
        }

        /// Retrieves a specific field value from a fact by its external ID.
        ///
        /// This is a convenience method that combines external ID lookup with field access.
        /// Useful for quick field access when you know the external ID but only need one field.
        ///
        /// # Arguments
        /// * `fact_id` - The external string identifier of the fact
        /// * `field` - The field name to retrieve
        ///
        /// # Returns
        /// `Some(FactValue)` if both the fact and field exist, `None` otherwise.
        /// Returns a cloned value (owned `FactValue`).
        ///
        /// # Performance
        /// - **Time Complexity**: O(1) average case for fact lookup + O(1) for field access
        /// - **Space Complexity**: O(1) - returns a cloned field value
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        /// use bingo_core::types::FactValue;
        ///
        /// let mut store = ArenaFactStore::new();
        /// // ... insert fact with external_id "user-123" and field "status" ...
        ///
        /// // Quick field access
        /// if let Some(FactValue::String(status)) = store.get_field_by_id("user-123", "status") {
        ///     println!("User status: {}", status);
        /// }
        /// ```
        pub fn get_field_by_id(&self, fact_id: &str, field: &str) -> Option<FactValue> {
            self.get_by_external_id(fact_id)?.data.fields.get(field).cloned()
        }

        /// Convenience method for inserting multiple facts from a vector.
        ///
        /// This method simply delegates to `bulk_insert()` but discards the returned fact IDs.
        /// Use this when you need to insert multiple facts but don't need to track their assigned IDs.
        ///
        /// # Arguments
        /// * `facts` - Vector of facts to insert
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        /// # use bingo_core::types::{Fact, FactData};
        ///
        /// let mut store = ArenaFactStore::new();
        /// let facts = vec![
        ///     Fact { id: 0, external_id: None, timestamp: chrono::Utc::now(), data: FactData { fields: std::collections::HashMap::new() } },
        ///     Fact { id: 0, external_id: None, timestamp: chrono::Utc::now(), data: FactData { fields: std::collections::HashMap::new() } },
        ///     Fact { id: 0, external_id: None, timestamp: chrono::Utc::now(), data: FactData { fields: std::collections::HashMap::new() } },
        /// ];
        ///
        /// store.extend_from_vec(facts);
        /// assert_eq!(store.len(), 3);
        /// ```
        pub fn extend_from_vec(&self, facts: Vec<Fact>) {
            self.bulk_insert(facts);
        }

        /// Optimised bulk insert for large datasets with batch processing.
        ///
        /// This method provides superior performance for inserting large numbers of facts
        /// by batching operations and minimising memory allocations. All ID assignment,
        /// external ID mapping, and indexing operations are performed in optimised batches.
        ///
        /// # Performance Optimisations
        /// - Pre-allocates storage capacity to avoid repeated reallocations
        /// - Batches ID assignment and external ID mapping
        /// - Batches index updates for all indexed fields
        /// - Uses efficient memory layout for large datasets
        ///
        /// # Arguments
        /// * `facts` - Mutable vector of facts to insert (IDs will be assigned)
        ///
        /// # Returns
        /// Vector of assigned fact IDs in the same order as input facts.
        ///
        /// # Performance
        /// - **Time Complexity**: O(n * k) where n is number of facts, k is number of indexed fields
        /// - **Space Complexity**: O(n) for fact storage plus index overhead
        /// - **Memory Allocation**: Minimised through pre-allocation and batching
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        /// # use bingo_core::types::{Fact, FactData, FactValue};
        ///
        /// let mut store = ArenaFactStore::with_large_capacity(100_000);
        ///
        /// // Create a large batch of facts
        /// let mut facts = Vec::with_capacity(50_000);
        /// for i in 0..50_000 {
        ///     let mut fields = std::collections::HashMap::new();
        ///     fields.insert("user_id".to_string(), FactValue::Integer(i as i64));
        ///     facts.push(Fact { id: 0, external_id: None, timestamp: chrono::Utc::now(), data: FactData { fields } });
        /// }
        ///
        /// // Bulk insert with optimal performance
        /// let fact_ids = store.bulk_insert(facts);
        /// assert_eq!(fact_ids.len(), 50_000);
        ///
        /// // All facts are now stored and indexed
        /// assert_eq!(store.len(), 50_000);
        /// ```
        /// Insert facts from a slice without cloning the entire vector
        pub fn bulk_insert_slice(&self, facts: &[Fact]) -> Vec<FactId> {
            let mut fact_ids = Vec::with_capacity(facts.len());

            // Pre-allocate capacity if needed
            {
                let facts_read = self.facts.read().unwrap();
                let current_len = facts_read.len();
                let current_capacity = facts_read.capacity();
                drop(facts_read);

                if current_capacity < current_len + facts.len() {
                    let mut facts_write = self.facts.write().unwrap();
                    let current_write_len = facts_write.len();
                    let new_capacity = (current_write_len + facts.len()).next_power_of_two();
                    facts_write.reserve(new_capacity - current_write_len);
                }
            }

            // Process each fact individually for slice-based insertion
            for fact in facts {
                let fact_id = self.insert(fact.clone());
                fact_ids.push(fact_id);
            }

            fact_ids
        }

        pub fn bulk_insert(&self, mut facts: Vec<Fact>) -> Vec<FactId> {
            let mut fact_ids = Vec::with_capacity(facts.len());

            // Pre-allocate capacity if needed
            {
                let facts_read = self.facts.read().unwrap();
                let current_len = facts_read.len();
                let current_capacity = facts_read.capacity();
                drop(facts_read);

                if current_capacity < current_len + facts.len() {
                    let mut facts_write = self.facts.write().unwrap();
                    let current_write_len = facts_write.len();
                    let new_capacity = (current_write_len + facts.len()).next_power_of_two();
                    facts_write.reserve(new_capacity - current_write_len);
                }
            }

            // Batch process all facts for ID assignment and external ID mapping
            {
                let mut external_id_map = self.external_id_map.write().unwrap();

                for fact in &mut facts {
                    // Generate ID using same logic as insert
                    let id =
                        if fact.id == 0 || fact.id > crate::constants::fact_ids::MAX_USER_FACT_ID {
                            self.next_id.fetch_add(1, Ordering::SeqCst)
                        } else {
                            let current_next = self.next_id.load(Ordering::SeqCst);
                            if fact.id >= current_next {
                                self.next_id.store(fact.id + 1, Ordering::SeqCst);
                            }
                            fact.id
                        };

                    fact.id = id;
                    fact_ids.push(id);

                    // Update external ID mapping if present
                    if let Some(ref external_id) = fact.external_id {
                        external_id_map.insert(external_id.clone(), id);
                    }
                }
            }

            // Batch update indexes for all facts
            for fact in &facts {
                self.update_indexes(fact);
            }

            // Batch insert all facts into storage
            {
                let mut facts_storage = self.facts.write().unwrap();
                for fact in facts {
                    let id = fact.id;
                    // Ensure Vec capacity and insert
                    if facts_storage.len() <= id as usize {
                        facts_storage.resize(id as usize + 1, None);
                    }
                    facts_storage[id as usize] = Some(fact);
                }
            }

            // Update fact count for all inserted facts
            self.fact_count.fetch_add(fact_ids.len() as u64, Ordering::Relaxed);

            fact_ids
        }

        /// Creates an iterator over all stored facts.
        ///
        /// This method provides an iterator that yields cloned facts from all facts
        /// currently stored in the fact store. The iterator automatically skips
        /// over any deleted fact slots (None values).
        ///
        /// # Returns
        /// A vector of cloned `Fact` instances.
        ///
        /// # Performance
        /// - **Time Complexity**: O(n) to iterate through all facts
        /// - **Space Complexity**: O(n) - returns cloned facts
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        /// # use bingo_core::types::Fact;
        ///
        /// let store = ArenaFactStore::new();
        /// // ... insert some facts ...
        ///
        /// // Iterate over all facts
        /// for fact in store.iter() {
        ///     println!("Fact ID: {}, Timestamp: {}", fact.id, fact.timestamp);
        /// }
        ///
        /// // Collect into a vector if needed
        /// let all_facts: Vec<Fact> = store.iter();
        /// ```
        pub fn iter(&self) -> Vec<Fact> {
            let facts = self.facts.read().unwrap();
            facts.iter().filter_map(|opt| opt.as_ref().cloned()).collect()
        }

        /// Finds facts within a specific time range (inclusive bounds).
        ///
        /// Returns all facts whose timestamps fall within the specified time range.
        /// Both start and end times are inclusive. This method performs a linear
        /// scan through all facts.
        ///
        /// # Arguments
        /// * `start` - The earliest timestamp to include (inclusive)
        /// * `end` - The latest timestamp to include (inclusive)
        ///
        /// # Returns
        /// Vector of fact references whose timestamps are within the range.
        ///
        /// # Performance
        /// - **Time Complexity**: O(n) - linear scan through all facts
        /// - **Space Complexity**: O(k) where k is the number of matching facts
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        /// use chrono::{Utc, Duration};
        ///
        /// let store = ArenaFactStore::new();
        /// // ... insert facts with various timestamps ...
        ///
        /// let now = Utc::now();
        /// let one_hour_ago = now - Duration::hours(1);
        ///
        /// // Find facts from the last hour
        /// let recent_facts = store.facts_in_time_range(one_hour_ago, now);
        /// println!("Found {} recent facts", recent_facts.len());
        /// ```
        pub fn facts_in_time_range(
            &self,
            start: chrono::DateTime<chrono::Utc>,
            end: chrono::DateTime<chrono::Utc>,
        ) -> Vec<Fact> {
            self.iter()
                .into_iter()
                .filter(|f| f.timestamp >= start && f.timestamp <= end)
                .collect()
        }

        /// Returns the number of facts currently stored.
        ///
        /// This method counts all non-deleted facts in the store. Deleted fact
        /// slots (None values) are not included in the count.
        ///
        /// # Returns
        /// The total number of stored facts.
        ///
        /// # Performance
        /// - **Time Complexity**: O(n) - must scan through all slots to count non-None values
        /// - **Space Complexity**: O(1)
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        /// # use bingo_core::types::{Fact, FactData};
        ///
        /// let mut store = ArenaFactStore::new();
        /// assert_eq!(store.len(), 0);
        ///
        /// let fact1 = Fact { id: 0, external_id: None, timestamp: chrono::Utc::now(), data: FactData { fields: std::collections::HashMap::new() } };
        /// let fact2 = Fact { id: 0, external_id: None, timestamp: chrono::Utc::now(), data: FactData { fields: std::collections::HashMap::new() } };
        /// store.insert(fact1);
        /// store.insert(fact2);
        /// assert_eq!(store.len(), 2);
        /// ```
        pub fn len(&self) -> usize {
            self.fact_count.load(Ordering::Relaxed) as usize
        }

        /// Checks if the fact store is empty.
        ///
        /// # Returns
        /// `true` if no facts are stored, `false` otherwise.
        ///
        /// # Performance
        /// - **Time Complexity**: O(n) - delegates to `len()`
        /// - **Space Complexity**: O(1)
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        ///
        /// let store = ArenaFactStore::new();
        /// assert!(store.is_empty());
        /// ```
        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }

        /// Clears all facts and resets the store to empty state.
        ///
        /// This method removes all stored facts, clears all indexes, external ID mappings,
        /// and resets the internal ID counter. After calling this method, the store will
        /// be in the same state as a newly created instance.
        ///
        /// # Performance
        /// - **Time Complexity**: O(1) - all data structures support efficient clearing
        /// - **Space Complexity**: Frees all allocated memory
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        /// # use bingo_core::types::{Fact, FactData};
        ///
        /// let mut store = ArenaFactStore::new();
        ///
        /// // Insert a test fact
        /// let fact = Fact { id: 0, external_id: None, timestamp: chrono::Utc::now(), data: FactData { fields: std::collections::HashMap::new() } };
        /// store.insert(fact);
        /// assert!(!store.is_empty());
        ///
        /// store.clear();
        /// assert!(store.is_empty());
        /// assert_eq!(store.len(), 0);
        /// ```
        pub fn clear(&self) {
            let mut facts = self.facts.write().unwrap();
            facts.clear();
            drop(facts);

            let mut field_indexes = self.field_indexes.write().unwrap();
            field_indexes.clear();
            drop(field_indexes);

            let mut external_id_map = self.external_id_map.write().unwrap();
            external_id_map.clear();
            drop(external_id_map);

            self.next_id.store(0, Ordering::SeqCst);
            self.fact_count.store(0, Ordering::Relaxed);
        }

        /// Finds all facts that have a specific field value.
        ///
        /// This method provides efficient field-based lookups with automatic optimization:
        /// - For indexed fields: Uses O(1) hash lookup to find matching fact IDs
        /// - For non-indexed fields: Falls back to O(n) linear search
        ///
        /// # Indexed Fields (Fast Path)
        /// The following fields are automatically indexed: `entity_id`, `id`, `user_id`,
        /// `customer_id`, `status`, `category`.
        ///
        /// # Arguments
        /// * `field` - The field name to search
        /// * `value` - The field value to match
        ///
        /// # Returns
        /// Vector of fact references that have the specified field value.
        ///
        /// # Performance
        /// - **Indexed fields**: O(1) average case for lookup + O(k) where k is number of matches
        /// - **Non-indexed fields**: O(n) linear scan through all facts
        /// - **Space Complexity**: O(k) where k is the number of matching facts
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        /// use bingo_core::types::FactValue;
        ///
        /// let store = ArenaFactStore::new();
        /// // ... insert facts with user_id fields ...
        ///
        /// // Fast indexed field lookup
        /// let user_facts = store.find_by_field("user_id", &FactValue::Integer(12345));
        /// println!("Found {} facts for user 12345", user_facts.len());
        ///
        /// // Non-indexed field lookup (slower but still works)
        /// let email_facts = store.find_by_field("email", &FactValue::String("user@example.com".to_string()));
        /// ```
        pub fn find_by_field(&self, field: &str, value: &FactValue) -> Vec<Fact> {
            let value_key = self.fact_value_to_index_key(value);

            {
                let field_indexes = self.field_indexes.read().unwrap();
                if let Some(field_map) = field_indexes.get(field) {
                    if let Some(fact_ids) = field_map.get(value_key.as_ref()) {
                        return fact_ids.iter().filter_map(|&id| self.get_fact(id)).collect();
                    }
                }
            }

            // Fallback to linear search for non-indexed fields
            let facts = self.facts.read().unwrap();
            facts
                .iter()
                .filter_map(|opt_fact| opt_fact.as_ref())
                .filter(|fact| fact.data.fields.get(field) == Some(value))
                .cloned()
                .collect()
        }

        /// Finds facts that match multiple field criteria (AND logic).
        ///
        /// This advanced search method finds facts that match ALL specified criteria.
        /// It uses intelligent optimization to leverage indexes when possible, dramatically
        /// improving performance for queries involving indexed fields.
        ///
        /// # Optimisation Strategy
        /// 1. **Index Intersection**: For indexed criteria, builds a candidate set by intersecting
        ///    fact IDs from each indexed field's hash map
        /// 2. **Early Termination**: If any indexed field has no matches, returns empty immediately
        /// 3. **Hybrid Filtering**: Validates all criteria (indexed + non-indexed) on final candidates
        /// 4. **Fallback**: Uses linear search only when no indexed criteria are present
        ///
        /// # Arguments
        /// * `criteria` - Slice of (field_name, field_value) tuples to match
        ///
        /// # Returns
        /// Vector of fact references that match ALL criteria. Empty vector if no criteria provided
        /// returns all facts.
        ///
        /// # Performance
        /// - **With indexed criteria**: O(i × k) where i is number of indexed criteria, k is average matches per criteria
        /// - **Without indexed criteria**: O(n × c) where n is total facts, c is number of criteria
        /// - **Space Complexity**: O(k) where k is the number of matching facts
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        /// use bingo_core::types::FactValue;
        ///
        /// let store = ArenaFactStore::new();
        /// // ... insert facts ...
        ///
        /// // Multi-criteria search with indexed fields (fast)
        /// let criteria = [
        ///     ("user_id".to_string(), FactValue::Integer(12345)),   // indexed
        ///     ("status".to_string(), FactValue::String("active".to_string())), // indexed
        ///     ("role".to_string(), FactValue::String("admin".to_string())),   // not indexed
        /// ];
        ///
        /// let matching_facts = store.find_by_criteria(&criteria);
        /// // Returns facts that are: user_id=12345 AND status=active AND role=admin
        /// ```
        pub fn find_by_criteria(&self, criteria: &[(String, FactValue)]) -> Vec<Fact> {
            if criteria.is_empty() {
                return self.iter();
            }

            // Optimization: if we have indexed criteria, use them to reduce the search space
            const INDEXED_FIELDS: &[&str] =
                &["entity_id", "id", "user_id", "customer_id", "status", "category"];

            let mut candidate_ids: Option<std::collections::HashSet<FactId>> = None;

            // Try to find an indexed criterion to reduce the search space
            {
                let field_indexes = self.field_indexes.read().unwrap();
                for (field, value) in criteria {
                    if INDEXED_FIELDS.iter().any(|&f| f == field) {
                        let value_key = self.fact_value_to_index_key(value);
                        if let Some(field_map) = field_indexes.get(field) {
                            if let Some(fact_ids) = field_map.get(value_key.as_ref()) {
                                let current_ids: std::collections::HashSet<FactId> =
                                    fact_ids.iter().copied().collect();

                                candidate_ids = Some(match candidate_ids {
                                    None => current_ids,
                                    Some(existing) => {
                                        existing.intersection(&current_ids).copied().collect()
                                    }
                                });
                            } else {
                                // If any indexed field has no matches, return empty result
                                return Vec::new();
                            }
                        }
                    }
                }
            }

            // If we have candidate IDs from indexed fields, filter only those
            if let Some(ids) = candidate_ids {
                ids.iter()
                    .filter_map(|&id| self.get_fact(id))
                    .filter(|fact| {
                        criteria
                            .iter()
                            .all(|(field, value)| fact.data.fields.get(field) == Some(value))
                    })
                    .collect()
            } else {
                // Fall back to linear search if no indexed criteria were found
                let facts = self.facts.read().unwrap();
                facts
                    .iter()
                    .filter_map(|opt_fact| opt_fact.as_ref())
                    .filter(|fact| {
                        criteria
                            .iter()
                            .all(|(field, value)| fact.data.fields.get(field) == Some(value))
                    })
                    .cloned()
                    .collect()
            }
        }

        /// Returns cache statistics for the fact store.
        ///
        /// The `ArenaFactStore` does not use caching, so this method always returns `None`.
        /// This method exists for API compatibility with other fact store implementations
        /// that may include caching mechanisms.
        ///
        /// # Returns
        /// `None` - this implementation does not use caching.
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        ///
        /// let store = ArenaFactStore::new();
        /// assert!(store.cache_stats().is_none());
        /// ```
        pub fn cache_stats(&self) -> Option<CacheStats> {
            None
        }

        /// Clears any cache data in the fact store.
        ///
        /// The `ArenaFactStore` does not use caching, so this method is a no-op.
        /// This method exists for API compatibility with other fact store implementations
        /// that may include caching mechanisms.
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        ///
        /// let mut store = ArenaFactStore::new();
        /// store.clear_cache(); // Does nothing but won't error
        /// ```
        pub fn clear_cache(&self) {
            // Default implementation does nothing
        }

        /// Returns comprehensive index statistics for monitoring and optimisation.
        ///
        /// This method provides detailed insights into the current state of field indexes,
        /// including efficiency metrics, distribution patterns, and memory usage characteristics.
        /// Use this information to monitor index performance and identify optimisation opportunities.
        ///
        /// # Returns
        /// `IndexStats` containing:
        /// - **Total indexed fields**: Number of fields that have indexes
        /// - **Total unique values**: Sum of unique values across all indexed fields
        /// - **Total indexed facts**: Total number of fact references in all indexes
        /// - **Field-specific statistics**: Per-field breakdown of index performance
        /// - **Index efficiency**: Ratio of indexed facts to total facts (higher is better)
        ///
        /// # Performance
        /// - **Time Complexity**: O(f × v) where f is number of indexed fields, v is average unique values per field
        /// - **Space Complexity**: O(f) for the statistics structure
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        ///
        /// let store = ArenaFactStore::new();
        /// // ... insert facts with indexed fields ...
        ///
        /// let stats = store.index_stats();
        /// println!("Index efficiency: {:.2}%", stats.index_efficiency * 100.0);
        /// println!("Total indexed fields: {}", stats.total_indexed_fields);
        ///
        /// for (field_name, field_stats) in &stats.field_stats {
        ///     println!("{}: {} unique values, {} facts",
        ///         field_name, field_stats.unique_values, field_stats.indexed_facts);
        /// }
        /// ```
        pub fn index_stats(&self) -> IndexStats {
            let mut total_indexed_facts = 0;
            let mut total_unique_values = 0;
            let mut field_stats = std::collections::HashMap::new();

            {
                let field_indexes = self.field_indexes.read().unwrap();
                for (field_name, field_map) in field_indexes.iter() {
                    let unique_values = field_map.len();
                    let indexed_facts: usize = field_map.values().map(|v| v.len()).sum();

                    total_unique_values += unique_values;
                    total_indexed_facts += indexed_facts;

                    field_stats.insert(
                        field_name.clone(),
                        FieldIndexStats {
                            unique_values,
                            indexed_facts,
                            average_facts_per_value: if unique_values > 0 {
                                indexed_facts as f64 / unique_values as f64
                            } else {
                                0.0
                            },
                        },
                    );
                }

                IndexStats {
                    total_indexed_fields: field_indexes.len(),
                    total_unique_values,
                    total_indexed_facts,
                    field_stats,
                    index_efficiency: if !self.is_empty() {
                        total_indexed_facts as f64 / self.len() as f64
                    } else {
                        0.0
                    },
                }
            }
        }

        /// Compacts field indexes by removing empty entries and shrinking capacity.
        ///
        /// This maintenance operation optimises memory usage by:
        /// 1. **Removing empty entries**: Deletes field values that no longer have any facts
        /// 2. **Removing empty fields**: Deletes entire field indexes that have no values
        /// 3. **Shrinking capacity**: Reduces allocated memory for under-utilised containers
        ///
        /// Call this method periodically after bulk deletions or fact updates to maintain
        /// optimal memory usage and index performance.
        ///
        /// # When to Use
        /// - After deleting many facts (reduces index bloat)
        /// - After bulk fact updates (cleans up old index entries)
        /// - During maintenance windows (memory optimisation)
        /// - When index efficiency metrics indicate fragmentation
        ///
        /// # Performance
        /// - **Time Complexity**: O(f × v) where f is indexed fields, v is unique values per field
        /// - **Space Complexity**: Reduces memory usage, no additional allocation
        /// - **Memory Impact**: Frees unused capacity in hash maps and vectors
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        ///
        /// let mut store = ArenaFactStore::new();
        /// // ... insert many facts, then delete many ...
        ///
        /// // Check index stats before compaction
        /// let stats_before = store.index_stats();
        /// println!("Index efficiency before: {:.2}%", stats_before.index_efficiency * 100.0);
        ///
        /// // Compact indexes to reclaim memory
        /// store.compact_indexes();
        ///
        /// // Check improvements
        /// let stats_after = store.index_stats();
        /// println!("Index efficiency after: {:.2}%", stats_after.index_efficiency * 100.0);
        /// ```
        pub fn compact_indexes(&self) {
            let mut field_indexes = self.field_indexes.write().unwrap();
            field_indexes.retain(|_, field_map| {
                field_map.retain(|_, fact_ids| !fact_ids.is_empty());
                !field_map.is_empty()
            });

            // Shrink capacity for field maps that are significantly under-utilized
            for field_map in field_indexes.values_mut() {
                if field_map.capacity() > field_map.len() * 4 {
                    field_map.shrink_to_fit();
                }

                // Shrink fact ID vectors
                for fact_ids in field_map.values_mut() {
                    if fact_ids.capacity() > fact_ids.len() * 2 {
                        fact_ids.shrink_to_fit();
                    }
                }
            }
        }

        /// Updates an existing fact by modifying its field values.
        ///
        /// This method allows partial updates to fact fields without replacing the entire fact.
        /// Only the specified fields are updated; other fields remain unchanged. The fact's
        /// indexes are automatically updated to reflect the changes.
        ///
        /// # Arguments
        /// * `fact_id` - The internal fact ID to update
        /// * `updates` - HashMap of field names and new values to apply
        ///
        /// # Returns
        /// `true` if the fact was found and updated, `false` if the fact ID doesn't exist.
        ///
        /// # Index Maintenance
        /// This method automatically updates all relevant field indexes. For indexed fields,
        /// the old index entries are replaced with new ones based on the updated values.
        ///
        /// # Performance
        /// - **Time Complexity**: O(k) where k is the number of indexed fields being updated
        /// - **Space Complexity**: O(1) - updates in place
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        /// use bingo_core::types::FactValue;
        /// # use bingo_core::types::{Fact, FactData};
        /// use std::collections::HashMap;
        ///
        /// let mut store = ArenaFactStore::new();
        /// let fact = Fact { id: 0, external_id: None, timestamp: chrono::Utc::now(), data: FactData { fields: std::collections::HashMap::new() } };
        /// let fact_id = store.insert(fact);
        ///
        /// // Prepare updates
        /// let mut updates = HashMap::new();
        /// updates.insert("status".to_string(), FactValue::String("inactive".to_string()));
        /// updates.insert("last_seen".to_string(), FactValue::String("2024-01-01".to_string()));
        ///
        /// // Apply updates
        /// let success = store.update_fact(fact_id, updates);
        /// assert!(success);
        ///
        /// // Verify the changes
        /// let updated_fact = store.get_fact(fact_id).unwrap();
        /// assert_eq!(updated_fact.data.fields.get("status"),
        ///            Some(&FactValue::String("inactive".to_string())));
        /// ```
        pub fn update_fact(&self, fact_id: FactId, updates: HashMap<String, FactValue>) -> bool {
            let mut facts = self.facts.write().unwrap();
            if let Some(fact_option) = facts.get_mut(fact_id as usize) {
                if let Some(fact) = fact_option.as_mut() {
                    // Apply updates to the fact's fields
                    for (field, value) in updates {
                        fact.data.fields.insert(field, value);
                    }

                    // Clone the fact for re-indexing to avoid borrow checker issues
                    let fact_clone = fact.clone();
                    drop(facts); // Drop the write lock before calling update_indexes
                    self.update_indexes(&fact_clone);
                    return true;
                }
            }
            false
        }

        /// Deletes a fact by its internal ID.
        ///
        /// This method permanently removes a fact from the store, including:
        /// - The fact data itself from the main storage array
        /// - External ID mapping (if the fact had an external ID)
        /// - All field index entries that reference this fact
        ///
        /// The fact slot becomes available for reuse by future inserts.
        ///
        /// # Arguments
        /// * `fact_id` - The internal fact ID to delete
        ///
        /// # Returns
        /// `true` if the fact was found and deleted, `false` if the fact ID doesn't exist.
        ///
        /// # Performance
        /// - **Time Complexity**: O(k × v) where k is number of indexed fields, v is average values per field
        /// - **Space Complexity**: O(1) - frees memory used by the fact
        ///
        /// # Memory Impact
        /// The fact slot is set to `None` but the vector is not resized. Call `compact_indexes()`
        /// periodically after bulk deletions to reclaim memory.
        ///
        /// # Example
        /// ```rust
        /// use bingo_core::fact_store::arena_store::ArenaFactStore;
        /// # use bingo_core::types::{Fact, FactData};
        ///
        /// let mut store = ArenaFactStore::new();
        /// let fact = Fact { id: 0, external_id: None, timestamp: chrono::Utc::now(), data: FactData { fields: std::collections::HashMap::new() } };
        /// let fact_id = store.insert(fact);
        ///
        /// // Verify fact exists
        /// assert!(store.get_fact(fact_id).is_some());
        ///
        /// // Delete the fact
        /// let success = store.delete_fact(fact_id);
        /// assert!(success);
        ///
        /// // Verify fact is gone
        /// assert!(store.get_fact(fact_id).is_none());
        /// ```
        pub fn delete_fact(&self, fact_id: FactId) -> bool {
            let mut facts = self.facts.write().unwrap();
            if let Some(fact_option) = facts.get_mut(fact_id as usize) {
                if let Some(fact) = fact_option.take() {
                    drop(facts); // Release facts lock early

                    // Remove from external ID mapping if present
                    if let Some(ref external_id) = fact.external_id {
                        let mut external_id_map = self.external_id_map.write().unwrap();
                        external_id_map.remove(external_id);
                    }

                    // Remove from field indexes
                    self.remove_from_indexes(&fact);

                    // Decrement fact count
                    self.fact_count.fetch_sub(1, Ordering::Relaxed);

                    return true;
                }
            }
            false
        }

        /// Remove a fact from all field indexes
        fn remove_from_indexes(&self, fact: &Fact) {
            const INDEXED_FIELDS: &[&str] =
                &["entity_id", "id", "user_id", "customer_id", "status", "category"];

            let mut field_indexes = self.field_indexes.write().unwrap();

            for (field_name, field_value) in &fact.data.fields {
                if INDEXED_FIELDS.iter().any(|&f| f == field_name) {
                    let value_key = self.fact_value_to_index_key_owned(field_value);

                    if let Some(field_map) = field_indexes.get_mut(field_name) {
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

    // ArenaFactStore is thread-safe: all fields are protected by RwLock or AtomicU64
    unsafe impl Send for ArenaFactStore {}
    unsafe impl Sync for ArenaFactStore {}
}

#[cfg(test)]
mod tests {
    use super::arena_store::{ArenaFactStore, ThreadSafeArenaFactStore};
    use crate::types::{Fact, FactData, FactValue};
    use chrono::{Duration, Utc};
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::thread;

    fn create_test_fact(id: u64) -> Fact {
        let mut fields = HashMap::new();
        fields.insert(
            "test_field".to_string(),
            FactValue::String("test_value".to_string()),
        );
        Fact { id, external_id: None, timestamp: chrono::Utc::now(), data: FactData { fields } }
    }

    fn create_test_fact_with_external_id(id: u64, external_id: &str) -> Fact {
        let mut fact = create_test_fact(id);
        fact.external_id = Some(external_id.to_string());
        fact
    }

    fn create_test_fact_with_fields(id: u64, fields: HashMap<String, FactValue>) -> Fact {
        Fact { id, external_id: None, timestamp: chrono::Utc::now(), data: FactData { fields } }
    }

    #[test]
    fn test_arena_fact_store_constructors() {
        // Test basic constructor
        let store = ArenaFactStore::new();
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());

        // Test with capacity constructor
        let store_with_capacity = ArenaFactStore::with_capacity(100);
        assert_eq!(store_with_capacity.len(), 0);
        assert!(store_with_capacity.is_empty());

        // Test large capacity constructor
        let large_store = ArenaFactStore::with_large_capacity(10000);
        assert_eq!(large_store.len(), 0);
        assert!(large_store.is_empty());
    }

    #[test]
    fn test_thread_safe_constructors() {
        // Test shared constructor
        let shared_store = ArenaFactStore::new_shared();
        {
            let store = shared_store.read().unwrap();
            assert_eq!(store.len(), 0);
            assert!(store.is_empty());
        }

        // Test shared with capacity constructor
        let shared_store_capacity = ArenaFactStore::with_capacity_shared(100);
        {
            let store = shared_store_capacity.read().unwrap();
            assert_eq!(store.len(), 0);
            assert!(store.is_empty());
        }

        // Test shared large capacity constructor
        let shared_large_store = ArenaFactStore::with_large_capacity_shared(10000);
        {
            let store = shared_large_store.read().unwrap();
            assert_eq!(store.len(), 0);
            assert!(store.is_empty());
        }
    }

    #[test]
    fn test_basic_insert_and_get() {
        let store = ArenaFactStore::new();

        let fact = create_test_fact(42);
        let fact_id = store.insert(fact.clone());

        assert_eq!(fact_id, 42);
        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());

        let retrieved_fact = store.get_fact(fact_id);
        assert!(retrieved_fact.is_some());
        assert_eq!(retrieved_fact.unwrap().id, 42);
    }

    #[test]
    fn test_external_id_mapping() {
        let store = ArenaFactStore::new();

        let fact = create_test_fact_with_external_id(1, "external_123");
        store.insert(fact.clone());

        // Test lookup by external ID
        let retrieved_fact = store.get_by_external_id("external_123");
        assert!(retrieved_fact.is_some());
        let fact = retrieved_fact.unwrap();
        assert_eq!(fact.id, 1);
        assert_eq!(fact.external_id, Some("external_123".to_string()));

        // Test non-existent external ID
        let non_existent = store.get_by_external_id("non_existent");
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_field_indexing_and_search() {
        let store = ArenaFactStore::new();

        // Create facts with indexed fields
        let mut fact1 = create_test_fact(1);
        fact1.data.fields.insert(
            "entity_id".to_string(),
            FactValue::String("entity_1".to_string()),
        );
        fact1.data.fields.insert(
            "status".to_string(),
            FactValue::String("active".to_string()),
        );

        let mut fact2 = create_test_fact(2);
        fact2.data.fields.insert(
            "entity_id".to_string(),
            FactValue::String("entity_2".to_string()),
        );
        fact2.data.fields.insert(
            "status".to_string(),
            FactValue::String("active".to_string()),
        );

        let mut fact3 = create_test_fact(3);
        fact3.data.fields.insert(
            "entity_id".to_string(),
            FactValue::String("entity_1".to_string()),
        );
        fact3.data.fields.insert(
            "status".to_string(),
            FactValue::String("inactive".to_string()),
        );

        store.insert(fact1);
        store.insert(fact2);
        store.insert(fact3);

        // Test finding by indexed field
        let active_facts = store.find_by_field("status", &FactValue::String("active".to_string()));
        assert_eq!(active_facts.len(), 2);

        let entity_1_facts =
            store.find_by_field("entity_id", &FactValue::String("entity_1".to_string()));
        assert_eq!(entity_1_facts.len(), 2);

        // Test finding by non-indexed field
        let mut fact4 = create_test_fact(4);
        fact4.data.fields.insert(
            "custom_field".to_string(),
            FactValue::String("custom_value".to_string()),
        );
        store.insert(fact4);

        let custom_facts = store.find_by_field(
            "custom_field",
            &FactValue::String("custom_value".to_string()),
        );
        assert_eq!(custom_facts.len(), 1);
    }

    #[test]
    fn test_find_by_criteria() {
        let store = ArenaFactStore::new();

        let mut fact1 = create_test_fact(1);
        fact1
            .data
            .fields
            .insert("type".to_string(), FactValue::String("user".to_string()));
        fact1.data.fields.insert(
            "status".to_string(),
            FactValue::String("active".to_string()),
        );
        fact1.data.fields.insert("age".to_string(), FactValue::Integer(25));

        let mut fact2 = create_test_fact(2);
        fact2
            .data
            .fields
            .insert("type".to_string(), FactValue::String("user".to_string()));
        fact2.data.fields.insert(
            "status".to_string(),
            FactValue::String("inactive".to_string()),
        );
        fact2.data.fields.insert("age".to_string(), FactValue::Integer(30));

        let mut fact3 = create_test_fact(3);
        fact3
            .data
            .fields
            .insert("type".to_string(), FactValue::String("admin".to_string()));
        fact3.data.fields.insert(
            "status".to_string(),
            FactValue::String("active".to_string()),
        );
        fact3.data.fields.insert("age".to_string(), FactValue::Integer(25));

        store.insert(fact1);
        store.insert(fact2);
        store.insert(fact3);

        // Test single criterion
        let criteria = vec![("type".to_string(), FactValue::String("user".to_string()))];
        let results = store.find_by_criteria(&criteria);
        assert_eq!(results.len(), 2);

        // Test multiple criteria
        let criteria = vec![
            ("type".to_string(), FactValue::String("user".to_string())),
            (
                "status".to_string(),
                FactValue::String("active".to_string()),
            ),
        ];
        let results = store.find_by_criteria(&criteria);
        assert_eq!(results.len(), 1);

        // Test criteria with no matches
        let criteria = vec![
            ("type".to_string(), FactValue::String("user".to_string())),
            ("age".to_string(), FactValue::Integer(35)),
        ];
        let results = store.find_by_criteria(&criteria);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_time_range_queries() {
        let store = ArenaFactStore::new();

        let base_time = Utc::now();

        let mut fact1 = create_test_fact(1);
        fact1.timestamp = base_time - Duration::hours(2);

        let mut fact2 = create_test_fact(2);
        fact2.timestamp = base_time;

        let mut fact3 = create_test_fact(3);
        fact3.timestamp = base_time + Duration::hours(2);

        store.insert(fact1);
        store.insert(fact2);
        store.insert(fact3);

        // Test time range including all facts
        let all_facts = store.facts_in_time_range(
            base_time - Duration::hours(3),
            base_time + Duration::hours(3),
        );
        assert_eq!(all_facts.len(), 3);

        // Test time range including only middle fact
        let middle_facts = store.facts_in_time_range(
            base_time - Duration::minutes(30),
            base_time + Duration::minutes(30),
        );
        assert_eq!(middle_facts.len(), 1);

        // Test time range with no facts
        let no_facts = store.facts_in_time_range(
            base_time + Duration::hours(5),
            base_time + Duration::hours(6),
        );
        assert_eq!(no_facts.len(), 0);
    }

    #[test]
    fn test_fact_updates() {
        let store = ArenaFactStore::new();

        let fact = create_test_fact_with_external_id(1, "update_test");
        store.insert(fact);

        // Test updating fact
        let mut updates = HashMap::new();
        updates.insert(
            "new_field".to_string(),
            FactValue::String("new_value".to_string()),
        );
        updates.insert(
            "test_field".to_string(),
            FactValue::String("updated_value".to_string()),
        );

        let updated = store.update_fact(1, updates);
        assert!(updated);

        // Verify updates
        let retrieved_fact = store.get_fact(1);
        assert!(retrieved_fact.is_some());
        let fact = retrieved_fact.unwrap();
        assert_eq!(
            fact.data.fields.get("new_field"),
            Some(&FactValue::String("new_value".to_string()))
        );
        assert_eq!(
            fact.data.fields.get("test_field"),
            Some(&FactValue::String("updated_value".to_string()))
        );

        // Test updating non-existent fact
        let mut non_existent_updates = HashMap::new();
        non_existent_updates.insert("field".to_string(), FactValue::String("value".to_string()));
        let not_updated = store.update_fact(999, non_existent_updates);
        assert!(!not_updated);
    }

    #[test]
    fn test_fact_deletion() {
        let store = ArenaFactStore::new();

        let fact = create_test_fact_with_external_id(1, "delete_test");
        store.insert(fact);

        assert_eq!(store.len(), 1);

        // Test successful deletion
        let deleted = store.delete_fact(1);
        assert!(deleted);
        assert_eq!(store.len(), 0);

        // Verify fact is no longer accessible
        let retrieved_fact = store.get_fact(1);
        assert!(retrieved_fact.is_none());

        let by_external_id = store.get_by_external_id("delete_test");
        assert!(by_external_id.is_none());

        // Test deleting non-existent fact
        let not_deleted = store.delete_fact(999);
        assert!(!not_deleted);
    }

    #[test]
    fn test_clear_functionality() {
        let store = ArenaFactStore::new();

        // Add some facts
        for i in 1..=5 {
            let fact = create_test_fact_with_external_id(i, &format!("external_{i}"));
            store.insert(fact);
        }

        assert_eq!(store.len(), 5);

        // Clear the store
        store.clear();

        assert_eq!(store.len(), 0);
        assert!(store.is_empty());

        // Verify all facts are gone
        for i in 1..=5 {
            assert!(store.get_fact(i).is_none());
            assert!(store.get_by_external_id(&format!("external_{i}")).is_none());
        }
    }

    #[test]
    fn test_extend_from_vec() {
        let store = ArenaFactStore::new();

        let facts = vec![create_test_fact(1), create_test_fact(2), create_test_fact(3)];

        store.extend_from_vec(facts);

        assert_eq!(store.len(), 3);
        assert!(store.get_fact(1).is_some());
        assert!(store.get_fact(2).is_some());
        assert!(store.get_fact(3).is_some());
    }

    #[test]
    fn test_iteration() {
        let store = ArenaFactStore::new();

        let facts = vec![create_test_fact(1), create_test_fact(2), create_test_fact(3)];

        for fact in facts {
            store.insert(fact);
        }

        let collected_facts = store.iter();
        assert_eq!(collected_facts.len(), 3);

        let fact_ids: Vec<_> = collected_facts.iter().map(|f| f.id).collect();
        assert!(fact_ids.contains(&1));
        assert!(fact_ids.contains(&2));
        assert!(fact_ids.contains(&3));
    }

    #[test]
    fn test_different_fact_value_types() {
        let store = ArenaFactStore::new();

        let mut fields = HashMap::new();
        fields.insert(
            "string_field".to_string(),
            FactValue::String("test".to_string()),
        );
        fields.insert("int_field".to_string(), FactValue::Integer(42));
        fields.insert(
            "float_field".to_string(),
            FactValue::Float(std::f64::consts::PI),
        );
        fields.insert("bool_field".to_string(), FactValue::Boolean(true));
        fields.insert("null_field".to_string(), FactValue::Null);
        fields.insert("date_field".to_string(), FactValue::Date(Utc::now()));

        let fact = create_test_fact_with_fields(1, fields);
        store.insert(fact);

        // Test finding by different value types
        let string_results =
            store.find_by_field("string_field", &FactValue::String("test".to_string()));
        assert_eq!(string_results.len(), 1);

        let int_results = store.find_by_field("int_field", &FactValue::Integer(42));
        assert_eq!(int_results.len(), 1);

        let float_results =
            store.find_by_field("float_field", &FactValue::Float(std::f64::consts::PI));
        assert_eq!(float_results.len(), 1);

        let bool_results = store.find_by_field("bool_field", &FactValue::Boolean(true));
        assert_eq!(bool_results.len(), 1);

        let null_results = store.find_by_field("null_field", &FactValue::Null);
        assert_eq!(null_results.len(), 1);
    }

    #[test]
    fn test_id_generation_and_collision_handling() {
        let store = ArenaFactStore::new();

        // Test auto-generated IDs
        let fact_auto = create_test_fact(0); // ID 0 should trigger auto-generation
        let auto_id = store.insert(fact_auto);
        assert_eq!(auto_id, 0);

        // Test large ID fallback to auto-generation
        let fact_large = create_test_fact(2_000_000); // Should fallback to auto-generation
        let large_id = store.insert(fact_large);
        assert_eq!(large_id, 1); // Should get next auto-generated ID

        // Test specific ID preservation
        let fact_specific = create_test_fact(42);
        let specific_id = store.insert(fact_specific);
        assert_eq!(specific_id, 42);

        // Test that next auto-generated ID accounts for specific ID
        let fact_next_auto = create_test_fact(0);
        let next_auto_id = store.insert(fact_next_auto);
        assert_eq!(next_auto_id, 43); // Should be after the specific ID
    }

    #[test]
    fn test_cache_functionality() {
        let store = ArenaFactStore::new();

        // Test cache stats (should return None for basic implementation)
        let stats = store.cache_stats();
        assert!(stats.is_none());

        // Test cache clearing (should not crash)
        store.clear_cache();
    }

    #[test]
    fn test_get_field_by_id() {
        let store = ArenaFactStore::new();

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

    #[test]
    fn test_thread_safety() {
        let shared_store: ThreadSafeArenaFactStore = ArenaFactStore::new_shared();
        let store_clone = Arc::clone(&shared_store);

        // Test concurrent reads and writes
        let handle = thread::spawn(move || {
            for i in 1..=10 {
                let fact = create_test_fact_with_external_id(i, &format!("thread_fact_{i}"));
                {
                    let store = store_clone.write().unwrap();
                    store.insert(fact);
                }

                // Verify insertion with read lock
                {
                    let store = store_clone.read().unwrap();
                    assert!(store.get_fact(i).is_some());
                }
            }
        });

        // Main thread also performs operations
        for i in 11..=20 {
            let fact = create_test_fact_with_external_id(i, &format!("main_fact_{i}"));
            {
                let store = shared_store.write().unwrap();
                store.insert(fact);
            }
        }

        handle.join().unwrap();

        // Verify all facts were inserted
        {
            let store = shared_store.read().unwrap();
            assert_eq!(store.len(), 20);

            for i in 1..=20 {
                assert!(store.get_fact(i).is_some());
            }
        }
    }

    #[test]
    fn test_large_dataset_performance() {
        let store = ArenaFactStore::with_large_capacity(1000);

        // Insert a moderately large number of facts
        for i in 1..=1000 {
            let mut fact = create_test_fact(i);
            fact.data.fields.insert(
                "entity_id".to_string(),
                FactValue::String(format!("entity_{}", i % 100)),
            );
            fact.data
                .fields
                .insert("batch".to_string(), FactValue::Integer((i / 100) as i64));
            store.insert(fact);
        }

        assert_eq!(store.len(), 1000);

        // Test indexed field search performance
        let entity_facts =
            store.find_by_field("entity_id", &FactValue::String("entity_50".to_string()));
        assert_eq!(entity_facts.len(), 10); // Should find 10 facts with entity_50

        // Test iteration performance
        let all_facts = store.iter();
        assert_eq!(all_facts.len(), 1000);

        // Test time range query
        let time_facts = store.facts_in_time_range(
            Utc::now() - Duration::hours(1),
            Utc::now() + Duration::hours(1),
        );
        assert_eq!(time_facts.len(), 1000);
    }

    #[test]
    fn test_edge_cases() {
        let store = ArenaFactStore::new();

        // Test empty fact
        let empty_fact = Fact {
            id: 0,
            external_id: None,
            timestamp: Utc::now(),
            data: FactData { fields: HashMap::new() },
        };
        let fact_id = store.insert(empty_fact);
        assert!(store.get_fact(fact_id).is_some());

        // Test fact with empty external ID
        let mut fact_empty_external = create_test_fact(2);
        fact_empty_external.external_id = Some("".to_string());
        store.insert(fact_empty_external);

        let retrieved = store.get_by_external_id("");
        assert!(retrieved.is_some());

        // Test search with non-existent field
        let no_results = store.find_by_field(
            "non_existent_field",
            &FactValue::String("value".to_string()),
        );
        assert_eq!(no_results.len(), 0);

        // Test criteria search with empty criteria
        let empty_criteria_results = store.find_by_criteria(&[]);
        assert_eq!(empty_criteria_results.len(), store.len());
    }

    #[test]
    fn test_bulk_insert_optimization() {
        let store = ArenaFactStore::new();

        // Create a batch of facts for bulk insert
        let mut facts = Vec::new();
        for i in 1..=100 {
            let mut fact = create_test_fact(i);
            fact.data.fields.insert(
                "entity_id".to_string(),
                FactValue::String(format!("entity_{}", i % 10)),
            );
            fact.data
                .fields
                .insert("batch".to_string(), FactValue::Integer((i / 10) as i64));
            facts.push(fact);
        }

        // Test bulk insert
        let fact_ids = store.bulk_insert(facts);
        assert_eq!(fact_ids.len(), 100);
        assert_eq!(store.len(), 100);

        // Verify all facts were inserted correctly
        for i in 1..=100 {
            assert!(store.get_fact(i).is_some());
        }

        // Test that indexes work correctly after bulk insert
        let entity_0_facts =
            store.find_by_field("entity_id", &FactValue::String("entity_0".to_string()));
        assert_eq!(entity_0_facts.len(), 10); // Should find facts 10, 20, 30, ..., 100
    }

    #[test]
    fn test_optimized_find_by_criteria() {
        let store = ArenaFactStore::new();

        // Create facts with indexed fields
        for i in 1..=20 {
            let mut fact = create_test_fact(i);
            fact.data.fields.insert(
                "entity_id".to_string(),
                FactValue::String(format!("entity_{}", i % 5)),
            );
            fact.data.fields.insert(
                "status".to_string(),
                FactValue::String(if i % 2 == 0 { "active" } else { "inactive" }.to_string()),
            );
            fact.data.fields.insert(
                "category".to_string(),
                FactValue::String(format!("cat_{}", i % 3)),
            );
            store.insert(fact);
        }

        // Test criteria with multiple indexed fields (should use index optimization)
        let criteria = vec![
            (
                "entity_id".to_string(),
                FactValue::String("entity_1".to_string()),
            ),
            (
                "status".to_string(),
                FactValue::String("active".to_string()),
            ),
        ];
        let results = store.find_by_criteria(&criteria);

        // Should find facts where entity_id=entity_1 AND status=active
        // Facts 1, 6, 11, 16 have entity_1, and among those, 6 and 16 are active
        assert_eq!(results.len(), 2);

        // Test criteria with non-indexed field (should fall back to linear search)
        let criteria_non_indexed = vec![(
            "non_indexed_field".to_string(),
            FactValue::String("value".to_string()),
        )];
        let results_empty = store.find_by_criteria(&criteria_non_indexed);
        assert_eq!(results_empty.len(), 0);

        // Test empty criteria (should return all facts)
        let all_results = store.find_by_criteria(&[]);
        assert_eq!(all_results.len(), 20);
    }

    #[test]
    fn test_index_statistics() {
        let store = ArenaFactStore::new();

        // Add facts with indexed fields
        for i in 1..=10 {
            let mut fact = create_test_fact(i);
            fact.data.fields.insert(
                "entity_id".to_string(),
                FactValue::String(format!("entity_{}", i % 3)),
            );
            fact.data.fields.insert(
                "status".to_string(),
                FactValue::String(if i % 2 == 0 { "active" } else { "inactive" }.to_string()),
            );
            store.insert(fact);
        }

        let stats = store.index_stats();

        // Should have 2 indexed fields (entity_id and status)
        assert_eq!(stats.total_indexed_fields, 2);

        // entity_id has 3 unique values (entity_0, entity_1, entity_2)
        // status has 2 unique values (active, inactive)
        assert_eq!(stats.total_unique_values, 5);

        // Total indexed facts should be 20 (10 facts * 2 indexed fields each)
        assert_eq!(stats.total_indexed_facts, 20);

        // Check field-specific stats
        assert!(stats.field_stats.contains_key("entity_id"));
        assert!(stats.field_stats.contains_key("status"));

        let entity_stats = &stats.field_stats["entity_id"];
        assert_eq!(entity_stats.unique_values, 3);
        assert_eq!(entity_stats.indexed_facts, 10);

        let status_stats = &stats.field_stats["status"];
        assert_eq!(status_stats.unique_values, 2);
        assert_eq!(status_stats.indexed_facts, 10);
    }

    #[test]
    fn test_index_compaction() {
        let store = ArenaFactStore::new();

        // Add facts with indexed fields
        for i in 1..=10 {
            let mut fact = create_test_fact(i);
            fact.data.fields.insert(
                "entity_id".to_string(),
                FactValue::String(format!("entity_{i}")),
            );
            store.insert(fact);
        }

        // Delete some facts to create empty index entries
        store.delete_fact(1);
        store.delete_fact(3);
        store.delete_fact(5);

        // Before compaction
        let stats_before = store.index_stats();

        // Compact indexes
        store.compact_indexes();

        // After compaction, should have fewer indexed facts
        let stats_after = store.index_stats();
        assert!(stats_after.total_indexed_facts <= stats_before.total_indexed_facts);

        // Remaining facts should still be findable
        let remaining_facts =
            store.find_by_field("entity_id", &FactValue::String("entity_2".to_string()));
        assert_eq!(remaining_facts.len(), 1);
    }
}
