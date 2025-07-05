#![deny(warnings)]
#![allow(
    missing_docs,
    unused_imports,
    unused_variables,
    dead_code,
    unused_assignments,
    unused_mut,
    unreachable_patterns,
    clippy::enum_variant_names
)]
//! Bingo Rules Engine gRPC API
//!
//! This module provides a gRPC streaming API for the Bingo RETE rules engine
//! with efficient memory usage and real-time processing.

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

// Only keep what we need for gRPC
pub mod grpc;
pub mod tracing_setup;

// Generated protocol buffer code
pub mod generated {
    tonic::include_proto!("rules_engine.v1");
}

/// Minimal application state for gRPC service
#[derive(Debug, Clone)]
pub struct AppState {
    pub start_time: DateTime<Utc>,
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        info!("Initializing gRPC application state");

        Ok(Self { start_time: Utc::now() })
    }

    pub fn elapsed(&self) -> Duration {
        (Utc::now() - self.start_time).to_std().unwrap_or_default()
    }
}
