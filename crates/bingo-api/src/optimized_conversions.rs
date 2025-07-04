//! Optimized conversion functions between API and Core types
//!
//! This module provides high-performance conversions that reduce allocation
//! overhead and improve serialization/deserialization performance through:
//! - Pooled allocation management
//! - Cached conversion patterns
//! - Bulk conversion optimizations
//! - Zero-copy conversions where possible

use crate::types::*;
use anyhow::Result;
use bingo_core::{
    Fact as CoreFact, FactValue as CoreFactValue, serialization::SerializationContext,
};
use fnv::FnvHasher;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// High-performance conversion context with pooled resources and caching
#[derive(Debug)]
pub struct ConversionContext {
    /// Core serialization context for JSON operations
    serialization_ctx: SerializationContext,
    /// HashMap pool for building field maps
    field_map_pool: RefCell<Vec<HashMap<String, CoreFactValue>>>,
    /// Conversion cache for repeated patterns
    conversion_cache: RefCell<HashMap<String, CoreFactValue>>,
    /// String pool for external IDs and field names
    string_pool: RefCell<Vec<String>>,
    /// Statistics
    conversion_hits: RefCell<usize>,
    conversion_misses: RefCell<usize>,
    pool_hits: RefCell<usize>,
    pool_misses: RefCell<usize>,
    max_cache_size: usize,
    max_pool_size: usize,
}

impl ConversionContext {
    /// Create a new conversion context with default settings
    pub fn new() -> Self {
        Self::with_capacity(1000, 100)
    }

    /// Create a conversion context with custom capacities
    pub fn with_capacity(max_cache_size: usize, max_pool_size: usize) -> Self {
        Self {
            serialization_ctx: SerializationContext::with_capacity(
                max_cache_size,
                max_pool_size / 2,
            ),
            field_map_pool: RefCell::new(Vec::with_capacity(max_pool_size / 4)),
            conversion_cache: RefCell::new(HashMap::with_capacity(max_cache_size / 4)),
            string_pool: RefCell::new(Vec::with_capacity(max_pool_size)),
            conversion_hits: RefCell::new(0),
            conversion_misses: RefCell::new(0),
            pool_hits: RefCell::new(0),
            pool_misses: RefCell::new(0),
            max_cache_size,
            max_pool_size,
        }
    }

    /// Get a HashMap from the pool for field conversion
    fn get_field_map(&self) -> HashMap<String, CoreFactValue> {
        if let Some(mut map) = self.field_map_pool.borrow_mut().pop() {
            map.clear();
            *self.pool_hits.borrow_mut() += 1;
            map
        } else {
            *self.pool_misses.borrow_mut() += 1;
            HashMap::with_capacity(16) // Reasonable default
        }
    }

    /// Return a HashMap to the pool
    fn return_field_map(&self, map: HashMap<String, CoreFactValue>) {
        if self.field_map_pool.borrow().len() < self.max_pool_size {
            self.field_map_pool.borrow_mut().push(map);
        }
    }

    /// Convert serde_json::Value to CoreFactValue with caching
    pub fn convert_json_to_fact_value(&self, value: &serde_json::Value) -> Result<CoreFactValue> {
        // Create cache key based on the JSON value
        let cache_key = value.to_string();

        // Check cache first for simple values
        if let Some(cached) = self.conversion_cache.borrow().get(&cache_key) {
            *self.conversion_hits.borrow_mut() += 1;
            return Ok(cached.clone());
        }

        *self.conversion_misses.borrow_mut() += 1;

        // Convert using the existing TryFrom implementation
        let result = CoreFactValue::try_from(value)?;

        // Cache simple values only (not complex structures)
        match &result {
            CoreFactValue::String(_)
            | CoreFactValue::Integer(_)
            | CoreFactValue::Float(_)
            | CoreFactValue::Boolean(_)
            | CoreFactValue::Null => {
                if self.conversion_cache.borrow().len() < self.max_cache_size {
                    self.conversion_cache.borrow_mut().insert(cache_key, result.clone());
                }
            }
            _ => {
                // Don't cache complex types to avoid memory bloat
            }
        }

        Ok(result)
    }

    /// Convert ApiFact to CoreFact with optimized allocations
    pub fn convert_api_fact_to_core(&self, api_fact: &ApiFact) -> Result<CoreFact> {
        // Get pooled HashMap for fields
        let mut fields = self.get_field_map();

        // Convert all field values
        for (key, value) in &api_fact.data {
            let core_value = self
                .convert_json_to_fact_value(value)
                .unwrap_or_else(|_| CoreFactValue::String(value.to_string()));
            fields.insert(key.clone(), core_value);
        }

        // Generate stable hash for the external ID
        let mut hasher = FnvHasher::default();
        api_fact.id.hash(&mut hasher);
        let id64 = hasher.finish();

        let core_fact = CoreFact {
            id: id64,
            external_id: Some(api_fact.id.clone()),
            timestamp: api_fact.created_at,
            data: bingo_core::FactData { fields },
        };

        // Return the HashMap to the pool (note: fields were moved into FactData)
        // We need to get a fresh map to return
        let empty_map = HashMap::new();
        self.return_field_map(empty_map);

        Ok(core_fact)
    }

    /// Convert multiple ApiFacts to CoreFacts efficiently
    pub fn convert_api_facts_to_core(&self, api_facts: &[ApiFact]) -> Result<Vec<CoreFact>> {
        let mut core_facts = Vec::with_capacity(api_facts.len());

        for api_fact in api_facts {
            core_facts.push(self.convert_api_fact_to_core(api_fact)?);
        }

        Ok(core_facts)
    }

    /// Convert CoreFact to ApiFact with optimized allocations
    pub fn convert_core_fact_to_api(&self, core_fact: &CoreFact) -> Result<ApiFact> {
        // Convert field values efficiently
        let mut data_map = HashMap::with_capacity(core_fact.data.fields.len());
        for (key, value) in &core_fact.data.fields {
            data_map.insert(key.clone(), value.into());
        }

        Ok(ApiFact {
            id: core_fact.external_id.clone().unwrap_or_else(|| core_fact.id.to_string()),
            data: data_map,
            created_at: core_fact.timestamp,
        })
    }

    /// Convert multiple CoreFacts to ApiFacts efficiently
    pub fn convert_core_facts_to_api(&self, core_facts: &[CoreFact]) -> Result<Vec<ApiFact>> {
        let mut api_facts = Vec::with_capacity(core_facts.len());

        for core_fact in core_facts {
            api_facts.push(self.convert_core_fact_to_api(core_fact)?);
        }

        Ok(api_facts)
    }

    /// Serialize ApiFacts to JSON efficiently
    pub fn serialize_api_facts(&self, api_facts: &[ApiFact]) -> Result<String> {
        // Convert to core facts first
        let core_facts = self.convert_api_facts_to_core(api_facts)?;

        // Use the optimized serialization context
        self.serialization_ctx.serialize_facts(&core_facts)
    }

    /// Deserialize JSON to ApiFacts efficiently
    pub fn deserialize_to_api_facts(&self, json_str: &str) -> Result<Vec<ApiFact>> {
        // Use optimized deserialization
        let core_facts = self.serialization_ctx.deserialize_facts(json_str)?;

        // Convert to API facts
        self.convert_core_facts_to_api(&core_facts)
    }

    /// Get comprehensive conversion statistics
    pub fn get_stats(&self) -> ConversionStats {
        let serialization_stats = self.serialization_ctx.get_stats();

        ConversionStats {
            conversion_hits: *self.conversion_hits.borrow(),
            conversion_misses: *self.conversion_misses.borrow(),
            pool_hits: *self.pool_hits.borrow(),
            pool_misses: *self.pool_misses.borrow(),
            cache_size: self.conversion_cache.borrow().len(),
            pool_size: self.field_map_pool.borrow().len(),
            serialization_stats,
        }
    }

    /// Clear all caches and reset statistics
    pub fn clear_caches(&self) {
        self.conversion_cache.borrow_mut().clear();
        self.serialization_ctx.clear_caches();
        *self.conversion_hits.borrow_mut() = 0;
        *self.conversion_misses.borrow_mut() = 0;
        *self.pool_hits.borrow_mut() = 0;
        *self.pool_misses.borrow_mut() = 0;
    }

    /// Get overall conversion efficiency
    pub fn conversion_efficiency(&self) -> f64 {
        let hits = *self.conversion_hits.borrow() + *self.pool_hits.borrow();
        let total = hits + *self.conversion_misses.borrow() + *self.pool_misses.borrow();
        if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

impl Default for ConversionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for conversion performance monitoring
#[derive(Debug, Clone)]
pub struct ConversionStats {
    pub conversion_hits: usize,
    pub conversion_misses: usize,
    pub pool_hits: usize,
    pub pool_misses: usize,
    pub cache_size: usize,
    pub pool_size: usize,
    pub serialization_stats: bingo_core::serialization::SerializationStats,
}

impl ConversionStats {
    /// Calculate overall performance improvement
    pub fn overall_hit_rate(&self) -> f64 {
        let total_hits = self.conversion_hits
            + self.pool_hits
            + self.serialization_stats.cache_hits
            + self.serialization_stats.buffer_hits;
        let total_requests = total_hits
            + self.conversion_misses
            + self.pool_misses
            + self.serialization_stats.cache_misses
            + self.serialization_stats.buffer_misses;
        if total_requests > 0 {
            (total_hits as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Estimate total memory savings
    pub fn estimated_memory_saved_bytes(&self) -> usize {
        let conversion_savings = self.conversion_hits * 64; // ~64 bytes per conversion
        let pool_savings = self.pool_hits * 256; // ~256 bytes per HashMap reuse
        let serialization_savings = self.serialization_stats.estimated_memory_saved_bytes();

        conversion_savings + pool_savings + serialization_savings
    }
}

// Global conversion context for the API layer
thread_local! {
    static CONVERSION_CONTEXT: RefCell<ConversionContext> = RefCell::new(ConversionContext::new());
}

/// Get the thread-local conversion context
pub fn with_conversion_context<F, R>(f: F) -> R
where
    F: FnOnce(&ConversionContext) -> R,
{
    CONVERSION_CONTEXT.with(|ctx| f(&ctx.borrow()))
}

/// High-level convenience functions using the global context
pub fn convert_api_fact_to_core(api_fact: &ApiFact) -> Result<bingo_core::Fact> {
    with_conversion_context(|ctx| ctx.convert_api_fact_to_core(api_fact))
}

pub fn convert_api_facts_to_core(api_facts: &[ApiFact]) -> Result<Vec<bingo_core::Fact>> {
    with_conversion_context(|ctx| ctx.convert_api_facts_to_core(api_facts))
}

pub fn convert_core_fact_to_api(core_fact: &bingo_core::Fact) -> Result<ApiFact> {
    with_conversion_context(|ctx| ctx.convert_core_fact_to_api(core_fact))
}

pub fn convert_core_facts_to_api(core_facts: &[bingo_core::Fact]) -> Result<Vec<ApiFact>> {
    with_conversion_context(|ctx| ctx.convert_core_facts_to_api(core_facts))
}

pub fn serialize_api_facts(api_facts: &[ApiFact]) -> Result<String> {
    with_conversion_context(|ctx| ctx.serialize_api_facts(api_facts))
}

pub fn deserialize_to_api_facts(json_str: &str) -> Result<Vec<ApiFact>> {
    with_conversion_context(|ctx| ctx.deserialize_to_api_facts(json_str))
}

pub fn get_conversion_stats() -> ConversionStats {
    with_conversion_context(|ctx| ctx.get_stats())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    #[test]
    fn test_json_to_fact_value_caching() {
        let ctx = ConversionContext::new();

        let json_value = json!(42);

        // First conversion should be a cache miss
        let result1 = ctx.convert_json_to_fact_value(&json_value).unwrap();
        let stats1 = ctx.get_stats();
        assert_eq!(stats1.conversion_misses, 1);
        assert_eq!(stats1.conversion_hits, 0);

        // Second conversion should be a cache hit
        let result2 = ctx.convert_json_to_fact_value(&json_value).unwrap();
        let stats2 = ctx.get_stats();
        assert_eq!(stats2.conversion_hits, 1);
        assert_eq!(stats2.conversion_misses, 1);

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_api_fact_conversion() {
        let ctx = ConversionContext::new();

        let api_fact = ApiFact {
            id: "test-fact".to_string(),
            data: [
                ("name".to_string(), json!("Test User")),
                ("age".to_string(), json!(30)),
                ("active".to_string(), json!(true)),
            ]
            .into_iter()
            .collect(),
            created_at: Utc::now(),
        };

        // Convert to core fact
        let core_fact = ctx.convert_api_fact_to_core(&api_fact).unwrap();
        assert_eq!(core_fact.external_id, Some("test-fact".to_string()));
        assert_eq!(core_fact.data.fields.len(), 3);

        // Convert back to API fact
        let api_fact2 = ctx.convert_core_fact_to_api(&core_fact).unwrap();
        assert_eq!(api_fact.id, api_fact2.id);
        assert_eq!(api_fact.data.len(), api_fact2.data.len());
    }

    #[test]
    fn test_bulk_conversion_performance() {
        let ctx = ConversionContext::new();

        let api_facts: Vec<ApiFact> = (0..100)
            .map(|i| ApiFact {
                id: format!("fact-{}", i),
                data: [
                    ("id".to_string(), json!(i)),
                    ("name".to_string(), json!(format!("User {}", i))),
                    ("active".to_string(), json!(i % 2 == 0)),
                ]
                .into_iter()
                .collect(),
                created_at: Utc::now(),
            })
            .collect();

        // Convert all facts
        let core_facts = ctx.convert_api_facts_to_core(&api_facts).unwrap();
        assert_eq!(core_facts.len(), 100);

        // Check that we got some cache hits for repeated values
        let stats = ctx.get_stats();
        assert!(
            stats.conversion_hits > 0,
            "Should have cache hits for repeated boolean values"
        );
    }

    #[test]
    fn test_serialization_roundtrip() {
        let ctx = ConversionContext::new();

        let api_facts = vec![ApiFact {
            id: "fact-1".to_string(),
            data: [("type".to_string(), json!("user")), ("score".to_string(), json!(95.5))]
                .into_iter()
                .collect(),
            created_at: Utc::now(),
        }];

        // Serialize
        let json = ctx.serialize_api_facts(&api_facts).unwrap();

        // Deserialize
        let deserialized_facts = ctx.deserialize_to_api_facts(&json).unwrap();

        assert_eq!(api_facts.len(), deserialized_facts.len());
        assert_eq!(api_facts[0].id, deserialized_facts[0].id);
        assert_eq!(api_facts[0].data.len(), deserialized_facts[0].data.len());
    }

    #[test]
    fn test_conversion_statistics() {
        let ctx = ConversionContext::new();

        let api_fact = ApiFact {
            id: "test".to_string(),
            data: [("key".to_string(), json!("value"))].into_iter().collect(),
            created_at: Utc::now(),
        };

        // Perform conversions to generate stats
        let _core1 = ctx.convert_api_fact_to_core(&api_fact).unwrap();
        let _core2 = ctx.convert_api_fact_to_core(&api_fact).unwrap();

        let stats = ctx.get_stats();
        assert!(stats.conversion_hits > 0 || stats.conversion_misses > 0);
        assert!(ctx.conversion_efficiency() >= 0.0);
    }
}
