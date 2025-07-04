use super::{CompiledAsset, UnifiedCacheProvider, UnifiedCacheStats};
use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError};
use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisResult};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, warn};

/// Redis-backed implementation of `UnifiedCacheProvider`.
/// All Redis operations are protected with a `CircuitBreaker` so the engine keeps
/// running even if the cache is unavailable.
#[derive(Debug, Clone)]
pub struct RedisCacheProvider {
    redis_conn: MultiplexedConnection,
    ttl_sec: usize,
    circuit_breaker: CircuitBreaker,
}

impl RedisCacheProvider {
    /// Create a new provider.
    pub async fn new(redis_url: &str, ttl_min: u64) -> RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        let redis_conn = client.get_multiplexed_async_connection().await?;

        // Basic circuit-breaker config
        let breaker_cfg = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 2,
            call_timeout: Duration::from_secs(5),
        };

        Ok(Self {
            redis_conn,
            ttl_sec: (ttl_min * 60) as usize,
            circuit_breaker: CircuitBreaker::new("redis-cache".into(), breaker_cfg),
        })
    }

    fn asset_key(key: &str) -> String {
        format!("bingo:asset:{}", key)
    }
    fn etag_key(etag: &str) -> String {
        format!("bingo:etag:{}", etag)
    }
}

#[async_trait]
impl UnifiedCacheProvider for RedisCacheProvider {
    async fn get(&self, key: &str) -> Option<Arc<CompiledAsset>> {
        let redis_key = Self::asset_key(key);
        let conn = self.redis_conn.clone();

        let result = self
            .circuit_breaker
            .call(|| {
                let mut conn = conn.clone();
                let redis_key = redis_key.clone();
                async move {
                    let bytes: RedisResult<Vec<u8>> = conn.get(&redis_key).await;
                    bytes.map_err(|e| anyhow::anyhow!("Redis GET error: {}", e))
                }
            })
            .await;

        match result {
            Ok(bytes) => match bincode::deserialize(&bytes) {
                Ok(asset) => {
                    debug!(key = %key, "Cache hit for asset");
                    Some(Arc::new(asset))
                }
                Err(e) => {
                    error!("Failed to deserialize asset for key {}: {}", key, e);
                    None
                }
            },
            Err(CircuitBreakerError::CircuitOpen { .. }) => {
                warn!(%key, "Circuit breaker open – skipping Redis get");
                None
            }
            Err(e) => {
                warn!(%key, "Redis GET failed: {}", e);
                None
            }
        }
    }

    async fn set(&self, key: String, asset: Arc<CompiledAsset>) {
        let asset_key = Self::asset_key(&key);
        let etag_key = Self::etag_key(&asset.etag);
        let ttl_sec = self.ttl_sec;
        let conn = self.redis_conn.clone();

        match bincode::serialize(&*asset) {
            Ok(bytes) => {
                let res = self
                    .circuit_breaker
                    .call(|| {
                        let mut conn = conn.clone();
                        let bytes = bytes.clone();
                        let asset_key = asset_key.clone();
                        let etag_key = etag_key.clone();
                        async move {
                            let mut pipe = redis::pipe();
                            pipe.atomic()
                                .set_ex(&asset_key, bytes, ttl_sec as u64)
                                .set_ex(&etag_key, key, ttl_sec as u64);
                            pipe.query_async::<_, ()>(&mut conn)
                                .await
                                .map_err(|e| anyhow::anyhow!("Redis pipeline: {}", e))
                        }
                    })
                    .await;

                match res {
                    Ok(_) => debug!(key = %asset_key, "Cache set OK"),
                    Err(CircuitBreakerError::CircuitOpen { .. }) => {
                        warn!(%asset_key, "Circuit breaker open – skip Redis set");
                    }
                    Err(e) => error!(%asset_key, "Redis SET failed: {}", e),
                }
            }
            Err(e) => error!("Serialize asset failed: {}", e),
        }
    }

    async fn check_etag(&self, etag: &str) -> Option<String> {
        let mut conn = self.redis_conn.clone();
        let key = Self::etag_key(etag);

        match conn.get::<_, Option<String>>(key).await {
            Ok(val) => val,
            Err(e) => {
                warn!(etag = %etag, "Redis GET for ETag failed: {}", e);
                None
            }
        }
    }

    async fn get_stats(&self) -> UnifiedCacheStats {
        UnifiedCacheStats::default()
    }

    async fn remove(&self, key: &str) -> bool {
        let mut conn = self.redis_conn.clone();
        let asset_key = Self::asset_key(key);
        let etag_pattern = "bingo:etag:*"; // broad invalidation

        let mut pipe = redis::pipe();
        pipe.atomic().del(&asset_key).del(etag_pattern);

        match pipe.query_async::<_, ()>(&mut conn).await {
            Ok(_) => true,
            Err(e) => {
                error!("Redis delete pipeline failed: {}", e);
                false
            }
        }
    }

    async fn clear(&self) {
        let mut conn = self.redis_conn.clone();
        let asset_pattern = "bingo:asset:*";
        let etag_pattern = "bingo:etag:*";

        let mut pipe = redis::pipe();
        pipe.atomic().del(asset_pattern).del(etag_pattern);

        if let Err(e) = pipe.query_async::<_, ()>(&mut conn).await {
            error!("Redis clear pipeline failed: {}", e);
        }
    }
}
