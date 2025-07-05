//! Simplified tracing setup for gRPC
//!
//! This module provides basic tracing setup using tracing-subscriber
//! with console output for gRPC services.

use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Configuration for tracing
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Service name for logging
    pub service_name: String,
    /// Service version for logging
    pub service_version: String,
    /// Environment (dev, staging, prod)
    pub environment: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "bingo-grpc-api".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
        }
    }
}

impl TracingConfig {
    /// Create configuration from environment variables
    pub fn from_environment() -> Self {
        Self {
            service_name: std::env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "bingo-grpc-api".to_string()),
            service_version: std::env::var("SERVICE_VERSION")
                .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string()),
            environment: std::env::var("BINGO_ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),
        }
    }
}

/// Initialize simplified tracing
pub fn init_tracing(config: TracingConfig) -> anyhow::Result<()> {
    info!("Initializing tracing for gRPC service");

    // Initialize subscriber with simple console logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bingo_api=info,info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    info!(
        service_name = %config.service_name,
        service_version = %config.service_version,
        environment = %config.environment,
        "Tracing initialized for gRPC service"
    );

    Ok(())
}

/// Shutdown tracing (no-op for simple tracing)
pub fn shutdown_tracing() {
    info!("Shutting down tracing");
}
