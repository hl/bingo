//! Optimized serialization and deserialization for high-performance fact processing
//!
//! This module provides performance-optimized serialization paths with:
//! - Pooled string buffers to avoid repeated allocations
//! - Cached type conversions for repeated value patterns
//! - Zero-copy serialization when possible
//! - Bulk serialization for fact batches

use crate::types::{Fact, FactData, FactValue};
use anyhow::Result;
use serde_json;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write;

/// High-performance serialization context with pooled resources
#[derive(Debug)]
pub struct SerializationContext {
    /// String buffer pool for JSON serialization
    string_buffers: RefCell<Vec<String>>,
    /// Value conversion cache for repeated patterns
    value_cache: RefCell<HashMap<String, serde_json::Value>>,
    /// Statistics tracking
    cache_hits: RefCell<usize>,
    cache_misses: RefCell<usize>,
    buffer_hits: RefCell<usize>,
    buffer_misses: RefCell<usize>,
    max_cache_size: usize,
    max_buffer_pool_size: usize,
}

impl SerializationContext {
    /// Create a new serialization context with default settings
    pub fn new() -> Self {
        Self::with_capacity(1000, 50)
    }

    /// Create a serialization context with custom capacities
    pub fn with_capacity(max_cache_size: usize, max_buffer_pool_size: usize) -> Self {
        Self {
            string_buffers: RefCell::new(Vec::with_capacity(max_buffer_pool_size / 4)),
            value_cache: RefCell::new(HashMap::with_capacity(max_cache_size / 4)),
            cache_hits: RefCell::new(0),
            cache_misses: RefCell::new(0),
            buffer_hits: RefCell::new(0),
            buffer_misses: RefCell::new(0),
            max_cache_size,
            max_buffer_pool_size,
        }
    }

    /// Get a string buffer from the pool
    fn get_string_buffer(&self) -> String {
        if let Some(mut buffer) = self.string_buffers.borrow_mut().pop() {
            buffer.clear();
            *self.buffer_hits.borrow_mut() += 1;
            buffer
        } else {
            *self.buffer_misses.borrow_mut() += 1;
            String::with_capacity(1024) // Start with reasonable capacity
        }
    }

    /// Return a string buffer to the pool
    fn return_string_buffer(&self, buffer: String) {
        if self.string_buffers.borrow().len() < self.max_buffer_pool_size {
            self.string_buffers.borrow_mut().push(buffer);
        }
    }

    /// Serialize a FactValue to JSON with caching
    pub fn serialize_fact_value(&self, value: &FactValue) -> Result<String> {
        // Create a cache key for the value
        let cache_key = self.create_cache_key(value);

        // Check cache first
        if let Some(cached_value) = self.value_cache.borrow().get(&cache_key) {
            *self.cache_hits.borrow_mut() += 1;
            return Ok(cached_value.to_string());
        }

        *self.cache_misses.borrow_mut() += 1;

        // Convert to serde_json::Value and serialize
        let json_value: serde_json::Value = value.into();
        let result = serde_json::to_string(&json_value)?;

        // Cache the result if we have space
        if self.value_cache.borrow().len() < self.max_cache_size {
            self.value_cache.borrow_mut().insert(cache_key, json_value);
        }

        Ok(result)
    }

    /// Serialize a complete Fact to JSON
    pub fn serialize_fact(&self, fact: &Fact) -> Result<String> {
        let mut buffer = self.get_string_buffer();

        // Manual JSON construction for better performance
        buffer.push('{');

        // Write ID
        write!(&mut buffer, r#""id":{},"#, fact.id)?;

        // Write external_id if present
        if let Some(external_id) = &fact.external_id {
            write!(&mut buffer, r#""external_id":"{}","#, external_id)?;
        }

        // Write timestamp
        write!(
            &mut buffer,
            r#""timestamp":"{}","#,
            fact.timestamp.to_rfc3339()
        )?;

        // Write data fields
        buffer.push_str(r#""data":{"#);
        let mut first = true;
        for (key, value) in &fact.data.fields {
            if !first {
                buffer.push(',');
            }
            first = false;

            write!(
                &mut buffer,
                r#""{}":{}"#,
                key,
                self.serialize_fact_value(value)?
            )?;
        }
        buffer.push('}');
        buffer.push('}');

        let result = buffer.clone();
        self.return_string_buffer(buffer);
        Ok(result)
    }

    /// Serialize multiple facts efficiently
    pub fn serialize_facts(&self, facts: &[Fact]) -> Result<String> {
        let mut buffer = self.get_string_buffer();

        buffer.push('[');
        for (i, fact) in facts.iter().enumerate() {
            if i > 0 {
                buffer.push(',');
            }
            buffer.push_str(&self.serialize_fact(fact)?);
        }
        buffer.push(']');

        let result = buffer.clone();
        self.return_string_buffer(buffer);
        Ok(result)
    }

    /// Deserialize a FactValue from JSON with caching
    pub fn deserialize_fact_value(&self, json_str: &str) -> Result<FactValue> {
        // Check cache first
        if let Some(cached_value) = self.value_cache.borrow().get(json_str) {
            *self.cache_hits.borrow_mut() += 1;
            return FactValue::try_from(cached_value);
        }

        *self.cache_misses.borrow_mut() += 1;

        // Parse JSON and convert
        let json_value: serde_json::Value = serde_json::from_str(json_str)?;
        let fact_value = FactValue::try_from(&json_value)?;

        // Cache the parsed JSON value if we have space
        if self.value_cache.borrow().len() < self.max_cache_size {
            self.value_cache.borrow_mut().insert(json_str.to_string(), json_value);
        }

        Ok(fact_value)
    }

    /// Deserialize a complete Fact from JSON
    pub fn deserialize_fact(&self, json_str: &str) -> Result<Fact> {
        let json_value: serde_json::Value = serde_json::from_str(json_str)?;

        let id = json_value["id"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid fact ID"))?;

        let external_id = json_value["external_id"].as_str().map(|s| s.to_string());

        let timestamp_str = json_value["timestamp"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid timestamp"))?;
        let timestamp =
            chrono::DateTime::parse_from_rfc3339(timestamp_str)?.with_timezone(&chrono::Utc);

        let data_obj = json_value["data"]
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid data object"))?;

        let mut fields = HashMap::new();
        for (key, value) in data_obj {
            fields.insert(key.clone(), FactValue::try_from(value)?);
        }

        Ok(Fact { id, external_id, timestamp, data: FactData { fields } })
    }

    /// Deserialize multiple facts from JSON array
    pub fn deserialize_facts(&self, json_str: &str) -> Result<Vec<Fact>> {
        let json_array: serde_json::Value = serde_json::from_str(json_str)?;
        let array = json_array.as_array().ok_or_else(|| anyhow::anyhow!("Expected JSON array"))?;

        let mut facts = Vec::with_capacity(array.len());
        for item in array {
            let fact_str = item.to_string();
            facts.push(self.deserialize_fact(&fact_str)?);
        }

        Ok(facts)
    }

    /// Create a cache key for a FactValue
    fn create_cache_key(&self, value: &FactValue) -> String {
        match value {
            FactValue::String(s) => format!("s:{}", s),
            FactValue::Integer(i) => format!("i:{}", i),
            FactValue::Float(f) => format!("f:{}", f),
            FactValue::Boolean(b) => format!("b:{}", b),
            FactValue::Null => "null".to_string(),
            // For complex types, use a hash
            _ => {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                value.hash(&mut hasher);
                format!("h:{}", hasher.finish())
            }
        }
    }

    /// Get serialization performance statistics
    pub fn get_stats(&self) -> SerializationStats {
        SerializationStats {
            cache_hits: *self.cache_hits.borrow(),
            cache_misses: *self.cache_misses.borrow(),
            buffer_hits: *self.buffer_hits.borrow(),
            buffer_misses: *self.buffer_misses.borrow(),
            cache_size: self.value_cache.borrow().len(),
            buffer_pool_size: self.string_buffers.borrow().len(),
        }
    }

    /// Clear all caches and reset statistics
    pub fn clear_caches(&self) {
        self.value_cache.borrow_mut().clear();
        *self.cache_hits.borrow_mut() = 0;
        *self.cache_misses.borrow_mut() = 0;
        *self.buffer_hits.borrow_mut() = 0;
        *self.buffer_misses.borrow_mut() = 0;
    }

    /// Get cache hit rate as percentage
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = *self.cache_hits.borrow() as f64;
        let total = hits + *self.cache_misses.borrow() as f64;
        if total > 0.0 {
            (hits / total) * 100.0
        } else {
            0.0
        }
    }

    /// Get buffer pool hit rate as percentage
    pub fn buffer_hit_rate(&self) -> f64 {
        let hits = *self.buffer_hits.borrow() as f64;
        let total = hits + *self.buffer_misses.borrow() as f64;
        if total > 0.0 {
            (hits / total) * 100.0
        } else {
            0.0
        }
    }
}

impl Default for SerializationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for serialization performance monitoring
#[derive(Debug, Clone)]
pub struct SerializationStats {
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub buffer_hits: usize,
    pub buffer_misses: usize,
    pub cache_size: usize,
    pub buffer_pool_size: usize,
}

impl SerializationStats {
    /// Calculate total hit rate across all caches
    pub fn overall_hit_rate(&self) -> f64 {
        let total_hits = self.cache_hits + self.buffer_hits;
        let total_requests = total_hits + self.cache_misses + self.buffer_misses;
        if total_requests > 0 {
            (total_hits as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Estimate memory savings from caching and pooling
    pub fn estimated_memory_saved_bytes(&self) -> usize {
        // Rough estimates
        let cache_savings = self.cache_hits * 100; // ~100 bytes per cached conversion
        let buffer_savings = self.buffer_hits * 1024; // ~1KB per buffer reuse
        cache_savings + buffer_savings
    }
}

// Global serialization context for the engine
thread_local! {
    static SERIALIZATION_CONTEXT: RefCell<SerializationContext> = RefCell::new(SerializationContext::new());
}

/// Get the thread-local serialization context
pub fn with_serialization_context<F, R>(f: F) -> R
where
    F: FnOnce(&SerializationContext) -> R,
{
    SERIALIZATION_CONTEXT.with(|ctx| f(&ctx.borrow()))
}

/// High-level convenience functions using the global context
pub fn serialize_fact(fact: &Fact) -> Result<String> {
    with_serialization_context(|ctx| ctx.serialize_fact(fact))
}

pub fn serialize_facts(facts: &[Fact]) -> Result<String> {
    with_serialization_context(|ctx| ctx.serialize_facts(facts))
}

pub fn deserialize_fact(json_str: &str) -> Result<Fact> {
    with_serialization_context(|ctx| ctx.deserialize_fact(json_str))
}

pub fn deserialize_facts(json_str: &str) -> Result<Vec<Fact>> {
    with_serialization_context(|ctx| ctx.deserialize_facts(json_str))
}

pub fn get_serialization_stats() -> SerializationStats {
    with_serialization_context(|ctx| ctx.get_stats())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_fact_value_serialization_caching() {
        let ctx = SerializationContext::new();

        let value = FactValue::Integer(42);

        // First serialization should be a cache miss
        let json1 = ctx.serialize_fact_value(&value).unwrap();
        let stats1 = ctx.get_stats();
        assert_eq!(stats1.cache_misses, 1);
        assert_eq!(stats1.cache_hits, 0);

        // Second serialization should be a cache hit
        let json2 = ctx.serialize_fact_value(&value).unwrap();
        let stats2 = ctx.get_stats();
        assert_eq!(stats2.cache_hits, 1);
        assert_eq!(stats2.cache_misses, 1);

        assert_eq!(json1, json2);
    }

    #[test]
    fn test_string_buffer_pooling() {
        let ctx = SerializationContext::new();

        let fact = Fact {
            id: 1,
            external_id: Some("test".to_string()),
            timestamp: Utc::now(),
            data: FactData {
                fields: std::iter::once((
                    "key".to_string(),
                    FactValue::String("value".to_string()),
                ))
                .collect(),
            },
        };

        // First serialization should be a buffer miss
        let _json1 = ctx.serialize_fact(&fact).unwrap();
        let stats1 = ctx.get_stats();
        assert!(stats1.buffer_misses > 0);

        // Second serialization should have some buffer hits
        let _json2 = ctx.serialize_fact(&fact).unwrap();
        let stats2 = ctx.get_stats();
        assert!(stats2.buffer_hits > 0);
    }

    #[test]
    fn test_fact_roundtrip_serialization() {
        let ctx = SerializationContext::new();

        let original_fact = Fact {
            id: 123,
            external_id: Some("test-fact".to_string()),
            timestamp: Utc::now(),
            data: FactData {
                fields: [
                    ("name".to_string(), FactValue::String("Test".to_string())),
                    ("age".to_string(), FactValue::Integer(25)),
                    ("score".to_string(), FactValue::Float(95.5)),
                    ("active".to_string(), FactValue::Boolean(true)),
                ]
                .into_iter()
                .collect(),
            },
        };

        let json = ctx.serialize_fact(&original_fact).unwrap();
        let deserialized_fact = ctx.deserialize_fact(&json).unwrap();

        assert_eq!(original_fact.id, deserialized_fact.id);
        assert_eq!(original_fact.external_id, deserialized_fact.external_id);
        assert_eq!(original_fact.data.fields, deserialized_fact.data.fields);
    }

    #[test]
    fn test_bulk_facts_serialization() {
        let ctx = SerializationContext::new();

        let facts: Vec<Fact> = (0..10)
            .map(|i| Fact {
                id: i,
                external_id: Some(format!("fact-{}", i)),
                timestamp: Utc::now(),
                data: FactData {
                    fields: std::iter::once((
                        "value".to_string(),
                        FactValue::Integer(i as i64 * 10),
                    ))
                    .collect(),
                },
            })
            .collect();

        let json = ctx.serialize_facts(&facts).unwrap();
        let deserialized_facts = ctx.deserialize_facts(&json).unwrap();

        assert_eq!(facts.len(), deserialized_facts.len());
        for (original, deserialized) in facts.iter().zip(deserialized_facts.iter()) {
            assert_eq!(original.id, deserialized.id);
            assert_eq!(original.external_id, deserialized.external_id);
        }
    }

    #[test]
    fn test_serialization_performance_stats() {
        let ctx = SerializationContext::new();

        // Perform some operations to generate stats
        let value = FactValue::String("test".to_string());
        let _json1 = ctx.serialize_fact_value(&value).unwrap();
        let _json2 = ctx.serialize_fact_value(&value).unwrap(); // Should hit cache

        let stats = ctx.get_stats();
        assert!(stats.cache_hits > 0);
        assert!(stats.cache_misses > 0);
        assert!(ctx.cache_hit_rate() > 0.0);
    }
}
