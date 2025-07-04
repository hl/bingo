use crate::error::ApiResult;
use async_trait::async_trait;
use bingo_core::BingoEngine;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

pub mod in_memory_provider;
pub mod unified_cache;

#[cfg(feature = "redis-cache")]
pub mod redis_provider;

pub use unified_cache::{CompiledAsset, UnifiedCacheProvider, UnifiedCacheStats};
