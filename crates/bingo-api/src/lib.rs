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

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::anyhow;
use bingo_core::BingoEngine;
use chrono::{DateTime, Utc};
use tracing::{info, warn};

// Only keep what we need for gRPC
pub mod grpc;
pub mod tracing_setup;

// Enhanced error handling modules
pub mod error_cli;
pub mod grpc_error_handler;

// Generated protocol buffer code
pub mod generated {
    tonic::include_proto!("rules_engine.v1");
}

/// Application state for gRPC service with shared engine management
#[derive(Debug)]
pub struct AppState {
    pub start_time: DateTime<Utc>,
    /// Shared engine instances by session ID
    pub engines: RwLock<HashMap<String, Arc<BingoEngine>>>,
    /// Default engine for stateless operations
    pub default_engine: Arc<BingoEngine>,
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        info!("Initializing gRPC application state with thread-safe engines");

        let default_engine = Arc::new(
            BingoEngine::new().map_err(|e| anyhow!("Failed to create default engine: {}", e))?,
        );

        Ok(Self { start_time: Utc::now(), engines: RwLock::new(HashMap::new()), default_engine })
    }

    pub fn elapsed(&self) -> Duration {
        (Utc::now() - self.start_time).to_std().unwrap_or_default()
    }

    /// Get or create an engine for a session
    pub fn get_or_create_engine(&self, session_id: &str) -> Arc<BingoEngine> {
        // First try to get existing engine with read lock
        {
            let engines = self.engines.read().unwrap();
            if let Some(engine) = engines.get(session_id) {
                return engine.clone();
            }
        }

        // Create new engine with write lock
        let mut engines = self.engines.write().unwrap();
        // Double-check in case another thread created it
        if let Some(engine) = engines.get(session_id) {
            return engine.clone();
        }

        info!("Creating new engine for session: {}", session_id);
        let engine =
            Arc::new(BingoEngine::new().unwrap_or_else(|e| {
                panic!("Failed to create engine for session {session_id}: {e}")
            }));
        engines.insert(session_id.to_string(), engine.clone());
        engine
    }

    /// Get the default engine for stateless operations
    pub fn get_default_engine(&self) -> Arc<BingoEngine> {
        self.default_engine.clone()
    }

    /// Remove an engine for a session (cleanup)
    pub fn remove_engine(&self, session_id: &str) -> Option<Arc<BingoEngine>> {
        info!("Removing engine for session: {}", session_id);
        self.engines.write().unwrap().remove(session_id)
    }

    /// Get the number of active engine sessions
    pub fn active_sessions(&self) -> usize {
        self.engines.read().unwrap().len()
    }
}
