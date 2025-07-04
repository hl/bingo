//! Streaming response functionality for large result sets
//!
//! This module implements chunked JSON and NDJSON streaming to handle
//! calculator-heavy scenarios that generate massive ActionResult counts
//! without triggering memory exhaustion.

use crate::types::{
    ApiRuleExecutionResult, EvaluateResponse, ResponseFormat, StreamingConfig, StreamingMetadata,
};
use axum::{
    body::Body,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use futures_util::{
    StreamExt,
    stream::{self, Stream},
};
use serde_json;
use std::pin::Pin;
use tracing::{debug, info, instrument};

/// Default thresholds for triggering streaming mode
const DEFAULT_RESULT_THRESHOLD: usize = 1000;
const DEFAULT_CHUNK_SIZE: usize = 100;

/// Streaming response builder
pub struct StreamingResponseBuilder {
    /// Unique request identifier
    pub request_id: String,
    /// Rule execution results to stream
    pub results: Vec<ApiRuleExecutionResult>,
    /// Number of rules that were processed
    pub rules_processed: usize,
    /// Number of facts that were processed
    pub facts_processed: usize,
    /// Total processing time in milliseconds
    pub processing_time_ms: u64,
    /// Engine statistics after processing
    pub stats: crate::types::EngineStats,
}

impl StreamingResponseBuilder {
    /// Determine if streaming should be used based on result count and configuration
    pub fn should_stream(
        result_count: usize,
        response_format: &Option<ResponseFormat>,
        streaming_config: &Option<StreamingConfig>,
        memory_safety_threshold: usize,
    ) -> bool {
        // Memory safety override: always stream if result count is dangerously high
        if result_count >= memory_safety_threshold {
            return true;
        }

        match response_format {
            Some(ResponseFormat::Stream) => true,
            Some(ResponseFormat::Standard) => false,
            Some(ResponseFormat::Auto) | None => {
                // Auto-detect based on result count threshold
                let threshold = streaming_config
                    .as_ref()
                    .and_then(|c| c.result_threshold)
                    .unwrap_or(DEFAULT_RESULT_THRESHOLD);

                result_count >= threshold
            }
        }
    }

    /// Build standard JSON response (non-streaming)
    pub fn build_standard_response(self) -> EvaluateResponse {
        let rules_fired = self.results.len();

        EvaluateResponse {
            request_id: self.request_id,
            results: Some(self.results),
            streaming: None,
            rules_processed: self.rules_processed,
            facts_processed: self.facts_processed,
            rules_fired,
            processing_time_ms: self.processing_time_ms,
            stats: self.stats,
        }
    }

    /// Build streaming NDJSON response for large result sets
    #[instrument(skip(self))]
    pub fn build_streaming_response(self, streaming_config: Option<StreamingConfig>) -> Response {
        let chunk_size = streaming_config
            .as_ref()
            .and_then(|c| c.chunk_size)
            .unwrap_or(DEFAULT_CHUNK_SIZE);

        let include_progress =
            streaming_config.as_ref().and_then(|c| c.include_progress).unwrap_or(false);

        let total_results = self.results.len();
        let estimated_chunks = total_results.div_ceil(chunk_size);

        info!(
            request_id = %self.request_id,
            total_results = total_results,
            chunk_size = chunk_size,
            estimated_chunks = estimated_chunks,
            "🚀 STREAMING: Building NDJSON streaming response"
        );

        // Create metadata header
        let metadata = StreamingMetadata {
            format: "ndjson".to_string(),
            estimated_chunks,
            chunk_size,
            consumption_hint: "Read newline-delimited JSON from response body".to_string(),
        };

        // First, send the response header with metadata
        let header_response = EvaluateResponse {
            request_id: self.request_id.clone(),
            results: None, // No results in header - they're in the stream
            streaming: Some(metadata),
            rules_processed: self.rules_processed,
            facts_processed: self.facts_processed,
            rules_fired: total_results,
            processing_time_ms: self.processing_time_ms,
            stats: self.stats,
        };

        // Create the streaming body
        let stream =
            create_ndjson_stream(header_response, self.results, chunk_size, include_progress);

        // Build response with appropriate headers
        match Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/x-ndjson")
            .header("X-Streaming-Format", "ndjson")
            .header("X-Total-Results", total_results.to_string())
            .header("X-Chunk-Size", chunk_size.to_string())
            .body(Body::from_stream(stream))
        {
            Ok(resp) => resp,
            Err(e) => {
                tracing::error!("Failed to build streaming response: {}", e);
                Response::new(Body::from("Failed to build streaming response"))
            }
        }
    }
}

/// Create NDJSON stream for rule execution results
fn create_ndjson_stream(
    header: EvaluateResponse,
    results: Vec<ApiRuleExecutionResult>,
    chunk_size: usize,
    include_progress: bool,
) -> Pin<Box<dyn Stream<Item = Result<String, std::io::Error>> + Send>> {
    let total_results = results.len();

    // Create chunks of results
    let chunks: Vec<Vec<ApiRuleExecutionResult>> =
        results.chunks(chunk_size).map(|chunk| chunk.to_vec()).collect();

    let stream = stream::iter(chunks).enumerate().map(move |(chunk_index, chunk)| {
        let chunk_start = chunk_index * chunk_size;
        let chunk_len = chunk.len();
        let chunk_end = (chunk_start + chunk_len).min(total_results);

        // ✅ OPTIMIZED: Pre-allocate items with estimated capacity
        let estimated_capacity = if chunk_index == 0 {
            chunk_len + 2
        } else {
            chunk_len + 1
        };
        let mut items = Vec::with_capacity(estimated_capacity);

        if chunk_index == 0 {
            // Send header as first NDJSON line
            match serde_json::to_string(&header) {
                Ok(header_json) => items.push(header_json),
                Err(e) => {
                    debug!("Failed to serialize header: {}", e);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to serialize header: {}", e),
                    ));
                }
            }
        }

        // Add progress update if requested
        if include_progress && chunk_index > 0 {
            let progress = serde_json::json!({
                "type": "progress",
                "chunk": chunk_index,
                "processed": chunk_end,
                "total": total_results,
                "percentage": (chunk_end as f64 / total_results as f64 * 100.0).round()
            });

            match serde_json::to_string(&progress) {
                Ok(progress_json) => items.push(progress_json),
                Err(e) => {
                    debug!("Failed to serialize progress: {}", e);
                }
            }
        }

        // Serialize results in this chunk
        for result in chunk {
            match serde_json::to_string(&result) {
                Ok(result_json) => items.push(result_json),
                Err(e) => {
                    debug!("Failed to serialize result: {}", e);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to serialize result: {}", e),
                    ));
                }
            }
        }

        debug!(
            chunk_index = chunk_index,
            chunk_size = chunk_len,
            range = format!("{}-{}", chunk_start, chunk_end),
            "📦 STREAMING: Serialized chunk"
        );

        // Join with newlines for NDJSON format
        Ok(items.join("\n") + "\n")
    });

    Box::pin(stream)
}

/// Response wrapper that handles both streaming and standard responses
pub enum ApiResponse {
    /// Standard JSON response
    Standard(EvaluateResponse),
    /// Standard JSON response with custom headers
    StandardWithHeaders(EvaluateResponse, Vec<(String, String)>),
    /// Streaming NDJSON response
    Streaming(Response),
    /// HTTP 304 Not Modified response
    NotModified,
    /// Streaming NDJSON response with custom headers
    StreamingWithHeaders(Response, Vec<(String, String)>),
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        match self {
            ApiResponse::Standard(response) => {
                // Standard JSON response
                match serde_json::to_string(&response) {
                    Ok(json) => match Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(json))
                    {
                        Ok(resp) => resp,
                        Err(_) => Response::new(Body::from("Failed to serialize response")),
                    },
                    Err(e) => {
                        tracing::error!("Failed to serialize standard response: {}", e);
                        Response::new(Body::from("Failed to serialize response"))
                    }
                }
            }
            ApiResponse::StandardWithHeaders(response, headers) => {
                // Standard JSON response with custom headers
                match serde_json::to_string(&response) {
                    Ok(json) => {
                        let mut builder = Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "application/json");

                        // Add custom headers
                        for (name, value) in headers {
                            builder = builder.header(&name, &value);
                        }

                        match builder.body(Body::from(json)) {
                            Ok(resp) => resp,
                            Err(_) => Response::new(Body::from("Failed to serialize response")),
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to serialize standard response with headers: {}",
                            e
                        );
                        Response::new(Body::from("Failed to serialize response"))
                    }
                }
            }
            ApiResponse::NotModified => (StatusCode::NOT_MODIFIED, "").into_response(),
            ApiResponse::Streaming(response) => response,
            ApiResponse::StreamingWithHeaders(mut response, headers) => {
                // Add headers to existing streaming response
                let response_headers = response.headers_mut();
                for (name, value) in headers {
                    if let Ok(header_name) = header::HeaderName::from_bytes(name.as_bytes()) {
                        if let Ok(header_value) = header::HeaderValue::from_str(&value) {
                            response_headers.insert(header_name, header_value);
                        }
                    }
                }
                response
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ApiActionResult, EngineStats};

    fn create_test_result(rule_id: &str, fact_id: &str) -> ApiRuleExecutionResult {
        ApiRuleExecutionResult {
            rule_id: rule_id.to_string(),
            fact_id: fact_id.to_string(),
            actions_executed: vec![ApiActionResult::Logged {
                message: format!("Rule {} fired for fact {}", rule_id, fact_id),
            }],
        }
    }

    #[test]
    fn test_should_stream_thresholds() {
        // Auto mode with default threshold
        assert!(!StreamingResponseBuilder::should_stream(
            500,
            &Some(ResponseFormat::Auto),
            &None,
            10000
        ));
        assert!(StreamingResponseBuilder::should_stream(
            1500,
            &Some(ResponseFormat::Auto),
            &None,
            10000
        ));

        // Force standard
        assert!(!StreamingResponseBuilder::should_stream(
            5000,
            &Some(ResponseFormat::Standard),
            &None,
            10000
        ));

        // Force streaming
        assert!(StreamingResponseBuilder::should_stream(
            10,
            &Some(ResponseFormat::Stream),
            &None,
            10000
        ));

        // Memory safety override
        assert!(StreamingResponseBuilder::should_stream(
            15000,
            &Some(ResponseFormat::Standard),
            &None,
            10000
        ));
    }

    #[test]
    fn test_standard_response_build() {
        let results =
            vec![create_test_result("rule_1", "fact_1"), create_test_result("rule_2", "fact_2")];

        let builder = StreamingResponseBuilder {
            request_id: "test_123".to_string(),
            results,
            rules_processed: 2,
            facts_processed: 2,
            processing_time_ms: 50,
            stats: EngineStats {
                total_facts: 2,
                total_rules: 2,
                network_nodes: 5,
                memory_usage_bytes: 1024,
            },
        };

        let response = builder.build_standard_response();

        assert_eq!(response.request_id, "test_123");
        assert!(response.results.is_some());
        assert_eq!(response.results.unwrap().len(), 2);
        assert!(response.streaming.is_none());
        assert_eq!(response.rules_fired, 2);
    }
}
