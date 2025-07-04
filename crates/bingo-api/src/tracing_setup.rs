//! Simplified distributed tracing setup
//!
//! This module provides basic distributed tracing setup using OpenTelemetry
//! with console output and trace propagation for request correlation.

use opentelemetry::{global, trace::TracerProvider as _};
use opentelemetry_sdk::{
    resource::Resource,
    trace::{Sampler, TracerProvider},
};
use std::collections::HashMap;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Configuration for distributed tracing
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Service name for tracing
    pub service_name: String,
    /// Service version for tracing
    pub service_version: String,
    /// Environment (dev, staging, prod)
    pub environment: String,
    /// Sampling ratio (0.0-1.0)
    pub sampling_ratio: f64,
    /// Enable console exporter for development
    pub enable_console_exporter: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "bingo-api".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
            sampling_ratio: 1.0, // Sample all traces in development
            enable_console_exporter: true,
        }
    }
}

impl TracingConfig {
    /// Create configuration from environment variables
    pub fn from_environment() -> Self {
        Self {
            service_name: std::env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "bingo-api".to_string()),
            service_version: std::env::var("OTEL_SERVICE_VERSION")
                .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string()),
            environment: std::env::var("BINGO_ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),
            sampling_ratio: std::env::var("OTEL_TRACES_SAMPLER_ARG")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1.0),
            enable_console_exporter: std::env::var("OTEL_CONSOLE_EXPORTER")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}

/// Initialize distributed tracing with OpenTelemetry
pub fn init_tracing(config: TracingConfig) -> anyhow::Result<()> {
    info!(
        service_name = %config.service_name,
        service_version = %config.service_version,
        environment = %config.environment,
        sampling_ratio = config.sampling_ratio,
        "Initializing distributed tracing"
    );

    // Build resource information
    let resource = Resource::new(vec![
        opentelemetry::KeyValue::new("service.name", config.service_name.clone()),
        opentelemetry::KeyValue::new("service.version", config.service_version.clone()),
        opentelemetry::KeyValue::new("deployment.environment", config.environment.clone()),
        opentelemetry::KeyValue::new("service.instance.id", uuid::Uuid::new_v4().to_string()),
    ]);

    // Configure sampling
    let sampler = if config.sampling_ratio >= 1.0 {
        Sampler::AlwaysOn
    } else if config.sampling_ratio <= 0.0 {
        Sampler::AlwaysOff
    } else {
        Sampler::TraceIdRatioBased(config.sampling_ratio)
    };

    // Create tracer provider with console output for development
    let tracer_provider = TracerProvider::builder()
        .with_config(
            opentelemetry_sdk::trace::Config::default()
                .with_sampler(sampler)
                .with_resource(resource),
        )
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();

    // Set global tracer provider
    global::set_tracer_provider(tracer_provider.clone());

    // Initialize tracing subscriber with OpenTelemetry layer
    let telemetry_layer =
        tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("bingo-api"));

    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "bingo_api=debug,bingo_core=debug,tower_http=debug,axum=debug".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(telemetry_layer);

    subscriber.init();

    info!("Distributed tracing initialized successfully");
    Ok(())
}

/// Shutdown tracing gracefully
pub fn shutdown_tracing() {
    info!("Shutting down distributed tracing");
    global::shutdown_tracer_provider();
}

/// Middleware for adding OpenTelemetry trace context to requests
use axum::{extract::Request, middleware::Next, response::Response};
use opentelemetry::propagation::Extractor;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

struct HeaderExtractor<'a>(&'a axum::http::HeaderMap);

impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|name| name.as_str()).collect()
    }
}

/// Middleware function for OpenTelemetry trace propagation
pub async fn opentelemetry_tracing_layer(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers();

    // Extract trace context from incoming headers
    let parent_context =
        global::get_text_map_propagator(|propagator| propagator.extract(&HeaderExtractor(headers)));

    // Create a new span with trace context
    let span = tracing::info_span!(
        "http_request",
        method = %method,
        uri = %uri,
        otel.kind = "server",
        http.method = %method,
        http.url = %uri,
    );

    // Attach parent context to the span (propagate traces)
    span.set_parent(parent_context.clone());

    // Execute the request within the span context
    let response = span.in_scope(|| async move { next.run(request).await }).await;

    // Add basic trace headers to response for downstream services
    let mut response = response;

    // Simple trace ID injection for correlation
    if let Some(trace_id) = extract_trace_id_from_context(&parent_context) {
        if let Ok(header_value) = axum::http::HeaderValue::from_str(&trace_id) {
            response.headers_mut().insert("X-Trace-Id", header_value);
        }
    }

    response
}

/// Extract trace ID from context for basic correlation
fn extract_trace_id_from_context(context: &opentelemetry::Context) -> Option<String> {
    use opentelemetry::trace::TraceContextExt;
    let span_ref = context.span();
    let span_context = span_ref.span_context();
    if span_context.is_valid() {
        Some(span_context.trace_id().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_config_defaults() {
        let config = TracingConfig::default();
        assert_eq!(config.service_name, "bingo-api");
        assert_eq!(config.environment, "development");
        assert_eq!(config.sampling_ratio, 1.0);
        assert!(config.enable_console_exporter);
    }

    #[test]
    fn test_tracing_config_from_environment() {
        // Temporarily set environment variables for the test.
        // SAFETY: modifying process environment variables in a single-threaded unit
        // test is safe because no other threads exist that could observe partial
        // updates. The functions themselves are marked `unsafe` in this build
        // (e.g. when compiled with the `unsafe_io` feature); we therefore wrap
        // them accordingly.
        unsafe {
            std::env::set_var("OTEL_SERVICE_NAME", "test-service");
            std::env::set_var("BINGO_ENVIRONMENT", "test");
            std::env::set_var("OTEL_TRACES_SAMPLER_ARG", "0.5");
        }

        let config = TracingConfig::from_environment();
        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.environment, "test");
        assert_eq!(config.sampling_ratio, 0.5);

        // Clean up
        // SAFETY: restoring environment to original state (see reasoning above).
        unsafe {
            std::env::remove_var("OTEL_SERVICE_NAME");
            std::env::remove_var("BINGO_ENVIRONMENT");
            std::env::remove_var("OTEL_TRACES_SAMPLER_ARG");
        }
    }
}
