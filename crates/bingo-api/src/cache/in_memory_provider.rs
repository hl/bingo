use super::{CompiledAsset, UnifiedCacheProvider, UnifiedCacheStats};
use async_trait::async_trait;
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

/// An in-memory cache for compiled assets, backed by `moka`.
#[derive(Clone, Debug)]
pub struct InMemoryCacheProvider {
    cache: Cache<String, Arc<CompiledAsset>>,
    etag_to_key: Cache<String, String>,
}

impl InMemoryCacheProvider {
    pub fn new(max_capacity: u64, time_to_live_minutes: u64) -> Self {
        let ttl = Duration::from_secs(time_to_live_minutes * 60);
        Self {
            cache: Cache::builder().max_capacity(max_capacity).time_to_live(ttl).build(),
            etag_to_key: Cache::builder()
                .max_capacity(max_capacity * 2) // ETag cache can be larger
                .time_to_live(ttl)
                .build(),
        }
    }
}

#[async_trait]
impl UnifiedCacheProvider for InMemoryCacheProvider {
    async fn get(&self, key: &str) -> Option<Arc<CompiledAsset>> {
        self.cache.get(key).await
    }

    async fn set(&self, key: String, asset: Arc<CompiledAsset>) {
        self.etag_to_key.insert(asset.etag.clone(), key.clone()).await;
        self.cache.insert(key, asset).await;
    }

    async fn check_etag(&self, etag: &str) -> Option<String> {
        self.etag_to_key.get(etag).await
    }

    async fn get_stats(&self) -> UnifiedCacheStats {
        // In a real implementation, this would track more detailed stats.
        UnifiedCacheStats::default()
    }

    async fn remove(&self, key: &str) -> bool {
        self.cache.invalidate(key).await;
        self.etag_to_key.invalidate_all(); // Remove .await - this is synchronous
        true
    }

    async fn clear(&self) {
        self.cache.invalidate_all();
        self.etag_to_key.invalidate_all();
    }
}
