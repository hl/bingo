pub struct ArenaFactStore {
    facts: Vec<Option<Fact>>, // Direct indexing: fact.id == Vec index
    field_indexes: std::collections::HashMap<String, std::collections::HashMap<String, Vec<FactId>>>,
    external_id_map: std::collections::HashMap<String, FactId>, // For external ID lookups
    next_id: FactId,
}
ly
impl ArenaFactStore {
    pub fn new() -> Self {
        Self {
            facts: Vec::new(),
            field_indexes: std::collections::HashMap::new(),
            external_id_map: std::collections::HashMap::new(),
            next_id: 0,
        }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            facts: Vec::with_capacity(capacity),
            field_indexes: std::collections::HashMap::with_capacity(6),
            external_id_map: std::collections::HashMap::with_capacity(capacity),
            next_id: 0,
        }
    }
    pub fn with_large_capacity(capacity: usize) -> Self {
        Self {
            facts: Vec::with_capacity(capacity),
            field_indexes: std::collections::HashMap::with_capacity(10),
            external_id_map: std::collections::HashMap::with_capacity(capacity),
            next_id: 0,
        }
    }
}
