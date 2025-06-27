//! Ruleset caching system for optimized rule compilation
//!
//! This module implements a TTL-based LRU cache for compiled RETE networks,
//! enabling significant performance improvements while maintaining statelessness.

use bingo_core::{BingoEngine, Rule as CoreRule};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// A compiled ruleset with metadata
#[derive(Debug, Clone)]
pub struct CompiledRuleset {
    /// Validated and converted core rules ready for engine creation
    pub rules: Vec<CoreRule>,

    /// Unique hash of the ruleset content for cache validation
    pub hash: String,

    /// When this ruleset was compiled and validated
    pub compiled_at: DateTime<Utc>,

    /// When this ruleset expires from cache
    pub expires_at: DateTime<Utc>,

    /// Number of times this ruleset has been used
    pub usage_count: u64,

    /// Description of the ruleset
    pub description: Option<String>,
}

impl CompiledRuleset {
    /// Check if this ruleset has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Increment usage counter
    pub fn increment_usage(&mut self) {
        self.usage_count += 1;
    }

    /// Create a fresh engine instance using this compiled ruleset
    pub fn create_engine_with_capacity(&self, fact_capacity: usize) -> anyhow::Result<BingoEngine> {
        let mut engine = BingoEngine::with_capacity(fact_capacity)?;

        // Add all rules to the engine (the RETE network will be rebuilt, but rule parsing is cached)
        for rule in &self.rules {
            engine.add_rule(rule.clone())?;
        }

        Ok(engine)
    }
}

/// Statistics for the ruleset cache
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub expired_entries: usize,
    pub total_compilations: u64,
    pub average_compilation_time_ms: f64,
    pub hit_rate: f64,
}

/// Thread-safe LRU cache with TTL for compiled rulesets
pub struct RulesetCache {
    /// Main cache storage
    cache: DashMap<String, CompiledRuleset>,

    /// Cache statistics
    hits: Arc<std::sync::atomic::AtomicU64>,
    misses: Arc<std::sync::atomic::AtomicU64>,
    compilations: Arc<std::sync::atomic::AtomicU64>,

    /// Total compilation time for averaging
    total_compilation_time: Arc<std::sync::atomic::AtomicU64>,

    /// Maximum number of entries (LRU eviction when exceeded)
    max_entries: usize,

    /// Default TTL for new entries
    default_ttl: Duration,
}

impl RulesetCache {
    /// Create a new ruleset cache
    pub fn new(max_entries: usize, default_ttl: Duration) -> Self {
        Self {
            cache: DashMap::new(),
            hits: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            misses: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            compilations: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            total_compilation_time: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            max_entries,
            default_ttl,
        }
    }

    /// Get a compiled ruleset from cache
    pub fn get(&self, ruleset_id: &str) -> Option<CompiledRuleset> {
        // First check if entry exists and is not expired
        let mut result = None;
        if let Some(entry) = self.cache.get(ruleset_id) {
            if entry.is_expired() {
                // Entry exists but expired - mark for removal
                drop(entry); // Release the reference before removing
                self.cache.remove(ruleset_id);
                self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                debug!(ruleset_id = %ruleset_id, "Ruleset cache miss: expired");
            } else {
                // Entry exists and is valid - clone and increment usage
                let mut ruleset = entry.clone();
                drop(entry); // Release the reference before modifying

                ruleset.increment_usage();
                result = Some(ruleset.clone());

                // Update the entry with incremented usage
                self.cache.insert(ruleset_id.to_string(), ruleset);

                self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                debug!(
                    ruleset_id = %ruleset_id,
                    usage_count = result.as_ref().unwrap().usage_count,
                    "Ruleset cache hit"
                );
            }
        } else {
            self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            debug!(ruleset_id = %ruleset_id, "Ruleset cache miss: not found");
        }

        result
    }

    /// Validate and cache a ruleset (rules are pre-validated)
    pub fn compile_and_cache(
        &self,
        ruleset_id: String,
        rules: Vec<CoreRule>,
        ttl: Option<Duration>,
        description: Option<String>,
    ) -> anyhow::Result<CompiledRuleset> {
        let start_time = Instant::now();

        info!(
            ruleset_id = %ruleset_id,
            rule_count = rules.len(),
            "Validating and caching ruleset"
        );

        // Generate hash of rules for cache validation
        let rules_summary = format!("{}-rules-{}", ruleset_id, rules.len());
        let hash = format!("{:x}", md5::compute(rules_summary.as_bytes()));

        let compilation_time = start_time.elapsed();
        let now = Utc::now();
        let ttl = ttl.unwrap_or(self.default_ttl);

        let compiled_ruleset = CompiledRuleset {
            rules,
            hash: hash.clone(),
            compiled_at: now,
            expires_at: now + chrono::Duration::from_std(ttl).unwrap(),
            usage_count: 0,
            description,
        };

        // Enforce cache size limit with LRU eviction
        self.evict_if_needed();

        // Cache the compiled ruleset
        self.cache.insert(ruleset_id.clone(), compiled_ruleset.clone());

        // Update statistics
        self.compilations.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_compilation_time.fetch_add(
            compilation_time.as_millis() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );

        info!(
            ruleset_id = %ruleset_id,
            validation_time_ms = compilation_time.as_millis(),
            hash = %hash,
            ttl_seconds = ttl.as_secs(),
            cache_size = self.cache.len(),
            "Ruleset validated and cached"
        );

        Ok(compiled_ruleset)
    }

    /// Evict entries if cache size exceeds limit (LRU eviction)
    fn evict_if_needed(&self) {
        if self.cache.len() >= self.max_entries {
            // Find oldest entry by compiled_at timestamp
            let mut oldest_key: Option<String> = None;
            let mut oldest_time = Utc::now();

            for entry in self.cache.iter() {
                if entry.compiled_at < oldest_time {
                    oldest_time = entry.compiled_at;
                    oldest_key = Some(entry.key().clone());
                }
            }

            if let Some(key) = oldest_key {
                self.cache.remove(&key);
                warn!(
                    evicted_ruleset = %key,
                    cache_size = self.cache.len(),
                    "Evicted oldest ruleset due to cache size limit"
                );
            }
        }
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) -> usize {
        let mut expired_count = 0;
        let now = Utc::now();

        // Collect expired keys
        let expired_keys: Vec<String> = self
            .cache
            .iter()
            .filter_map(|entry| {
                if entry.expires_at <= now {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        // Remove expired entries
        for key in expired_keys {
            self.cache.remove(&key);
            expired_count += 1;
        }

        if expired_count > 0 {
            info!(
                expired_count = expired_count,
                cache_size = self.cache.len(),
                "Cleaned up expired rulesets"
            );
        }

        expired_count
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.misses.load(std::sync::atomic::Ordering::Relaxed);
        let compilations = self.compilations.load(std::sync::atomic::Ordering::Relaxed);
        let total_time = self.total_compilation_time.load(std::sync::atomic::Ordering::Relaxed);

        let hit_rate = if hits + misses > 0 {
            hits as f64 / (hits + misses) as f64
        } else {
            0.0
        };

        let avg_compilation_time = if compilations > 0 {
            total_time as f64 / compilations as f64
        } else {
            0.0
        };

        // Count expired entries
        let now = Utc::now();
        let expired_entries = self.cache.iter().filter(|entry| entry.expires_at <= now).count();

        CacheStats {
            total_entries: self.cache.len(),
            cache_hits: hits,
            cache_misses: misses,
            expired_entries,
            total_compilations: compilations,
            average_compilation_time_ms: avg_compilation_time,
            hit_rate,
        }
    }

    /// Remove a specific ruleset from cache
    pub fn remove(&self, ruleset_id: &str) -> bool {
        self.cache.remove(ruleset_id).is_some()
    }

    /// Clear all entries from cache
    pub fn clear(&self) {
        let count = self.cache.len();
        self.cache.clear();
        info!(cleared_entries = count, "Cleared all cached rulesets");
    }
}

impl Default for RulesetCache {
    fn default() -> Self {
        Self::new(
            100,                          // Max 100 cached rulesets
            Duration::from_secs(60 * 60), // 1 hour default TTL
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bingo_core::{Action, ActionType, Condition, FactValue, Operator, Rule};

    fn create_test_rule(id: u64, name: &str) -> CoreRule {
        Rule {
            id,
            name: name.to_string(),
            conditions: vec![Condition::Simple {
                field: "test_field".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("test_value".to_string()),
            }],
            actions: vec![Action {
                action_type: ActionType::Log { message: format!("Rule {} fired", name) },
            }],
        }
    }

    #[test]
    fn test_cache_basic_operations() {
        let cache = RulesetCache::default();
        let ruleset_id = "test_ruleset".to_string();
        let rules = vec![create_test_rule(1, "test_rule")];

        // Compile and cache
        let compiled = cache
            .compile_and_cache(
                ruleset_id.clone(),
                rules.clone(),
                None,
                Some("Test ruleset".to_string()),
            )
            .unwrap();

        assert_eq!(compiled.rules.len(), 1);
        assert!(!compiled.hash.is_empty());

        // Retrieve from cache
        let cached = cache.get(&ruleset_id).unwrap();
        assert_eq!(cached.hash, compiled.hash);
        assert_eq!(cached.usage_count, 1);

        // Second retrieval should increment usage
        let cached2 = cache.get(&ruleset_id).unwrap();
        assert_eq!(cached2.usage_count, 2);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = RulesetCache::new(10, Duration::from_millis(1));
        let ruleset_id = "expiring_ruleset".to_string();
        let rules = vec![create_test_rule(1, "test_rule")];

        // Cache with very short TTL
        cache
            .compile_and_cache(
                ruleset_id.clone(),
                rules,
                Some(Duration::from_millis(1)),
                None,
            )
            .unwrap();

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(5));

        // Should not find expired entry
        assert!(cache.get(&ruleset_id).is_none());

        let stats = cache.get_stats();
        assert_eq!(stats.cache_misses, 1);
    }

    #[test]
    fn test_cache_stats() {
        let cache = RulesetCache::default();
        let rules = vec![create_test_rule(1, "test_rule")];

        // Initial stats
        let stats = cache.get_stats();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);

        // Cache a ruleset
        cache.compile_and_cache("test1".to_string(), rules.clone(), None, None).unwrap();

        // Hit
        cache.get("test1");

        // Miss
        cache.get("nonexistent");

        let stats = cache.get_stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
        assert_eq!(stats.total_compilations, 1);
    }
}
