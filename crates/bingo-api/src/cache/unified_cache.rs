use crate::error::{ApiError, ApiResult};
use async_trait::async_trait;
use bingo_core::{BingoEngine, Rule as CoreRule};
use chrono::{DateTime, Utc};

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

fn default_usage_count() -> Arc<AtomicUsize> {
    Arc::new(AtomicUsize::new(0))
}

/// A unified compiled asset (ruleset or engine template) with metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompiledAsset {
    /// Unique identifier for the asset (ruleset_id or hash of ad-hoc rules).
    pub id: String,
    /// Unique hash of the asset content for cache validation (ETag).
    pub etag: String,
    /// Validated and converted core rules ready for engine creation.
    pub rules: Vec<CoreRule>,
    /// Number of rules in this asset.
    pub rule_count: usize,
    /// Optional description of the asset.
    pub description: Option<String>,
    /// When this asset was compiled and validated.
    pub compiled_at: DateTime<Utc>,
    /// When this asset expires from cache.
    pub expires_at: DateTime<Utc>,
    /// Number of times this asset has been used.
    #[serde(skip, default = "default_usage_count")]
    pub usage_count: Arc<AtomicUsize>,
}

impl CompiledAsset {
    /// Check if this asset has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Increment usage counter.
    pub fn increment_usage(&self) {
        self.usage_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Create a fresh engine instance using this compiled asset.
    pub fn create_engine_with_capacity(&self, fact_capacity: usize) -> ApiResult<BingoEngine> {
        let mut engine = BingoEngine::with_capacity(fact_capacity)
            .map_err(|e| ApiError::internal(e.to_string()))?;

        // Add all rules to the engine (the RETE network will be rebuilt, but rule parsing is cached)
        for rule in &self.rules {
            engine.add_rule(rule.clone()).map_err(|e| ApiError::internal(e.to_string()))?;
        }

        Ok(engine)
    }

    /// Get cache headers for HTTP response.
    pub fn get_cache_headers(&self) -> Vec<(String, String)> {
        let headers = vec![
            ("ETag".to_string(), self.etag.clone()),
            (
                "Cache-Control".to_string(),
                "public, max-age=300".to_string(),
            ),
        ];
        headers
    }
}

/// Statistics for the unified cache.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct UnifiedCacheStats {
    pub total_entries: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hit_rate: f64,
}

/// A trait for a unified cache provider.
#[async_trait]
pub trait UnifiedCacheProvider: Send + Sync + std::fmt::Debug {
    async fn get(&self, key: &str) -> Option<Arc<CompiledAsset>>;
    async fn set(&self, key: String, asset: Arc<CompiledAsset>);
    async fn check_etag(&self, etag: &str) -> Option<String>;
    async fn get_stats(&self) -> UnifiedCacheStats;
    async fn remove(&self, key: &str) -> bool;
    async fn clear(&self);
}
