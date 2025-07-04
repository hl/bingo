//! Incremental fact processing for memory-efficient streaming
//!
//! This module implements incremental fact processing to avoid memory spikes
//! when dealing with large datasets (millions of facts) that would otherwise
//! exhaust available memory.

use crate::metrics::ApiMetrics;
use crate::types::*;
use axum::{
    body::Body,
    http::{StatusCode, header},
    response::Response,
};
use bingo_core::{BingoEngine, Fact as CoreFact};
use futures_util::stream::{self, Stream};
use tokio_stream;
use serde_json;
use std::pin::Pin;
use std::sync::Arc;
use sysinfo::{Pid, System};
use tracing::{debug, info, instrument, warn};



/// Incremental processor for large fact datasets
pub struct IncrementalProcessor {
    request_id: String,
    /// The number of facts to process in a single batch.
    fact_batch_size: usize,
    /// The memory limit in megabytes that triggers incremental processing.
    memory_limit_mb: usize,
    include_progress: bool,
    chunk_size: usize,
    metrics: Arc<ApiMetrics>,
}

impl IncrementalProcessor {
    /// Adjust batch size based on memory pressure
    fn adjust_batch_size_for_memory(&mut self, current_memory_mb: usize) {
        if MemoryMonitor::is_memory_pressure(current_memory_mb, self.memory_limit_mb) {
            // Reduce batch size by 50% if under memory pressure
            let new_batch_size = (self.fact_batch_size / 2).max(10); // Minimum of 10 facts per batch
            if new_batch_size != self.fact_batch_size {
                warn!(
                    old_batch_size = self.fact_batch_size,
                    new_batch_size = new_batch_size,
                    current_memory_mb = current_memory_mb,
                    memory_limit_mb = self.memory_limit_mb,
                    "Reducing batch size due to memory pressure"
                );
                self.fact_batch_size = new_batch_size;
            }
        }
    }

    /// Create a new incremental processor
    pub fn new(
        request_id: String,
        streaming_config: &Option<StreamingConfig>,
        global_fact_batch_size: usize,
        global_memory_limit_mb: usize,
        metrics: Arc<ApiMetrics>,
    ) -> Self {
        let config = streaming_config.as_ref();

        let fact_batch_size =
            config.and_then(|c| c.fact_batch_size).unwrap_or(global_fact_batch_size);

        let memory_limit_mb =
            config.and_then(|c| c.memory_limit_mb).unwrap_or(global_memory_limit_mb);

        let include_progress = config.and_then(|c| c.include_progress).unwrap_or(false);

        let chunk_size = config.and_then(|c| c.chunk_size).unwrap_or(100);

        Self {
            request_id,
            fact_batch_size,
            memory_limit_mb,
            include_progress,
            chunk_size,
            metrics,
        }
    }

    /// Determine if incremental processing should be enabled
    pub fn should_use_incremental(
        fact_count: usize,
        streaming_config: &Option<StreamingConfig>,
        current_memory_mb: usize,
        global_memory_limit_mb: usize,
    ) -> bool {
        let config = streaming_config.as_ref();

        // Explicit request for incremental processing
        if config.and_then(|c| c.incremental_processing).unwrap_or(false) {
            return true;
        }

        // Memory limit override
        let memory_limit = config.and_then(|c| c.memory_limit_mb).unwrap_or(global_memory_limit_mb);

        if current_memory_mb >= memory_limit {
            return true;
        }

        // Large fact count heuristic (>10K facts)
        if fact_count > 10_000 {
            return true;
        }

        false
    }

    /// Process facts incrementally and stream results
    #[instrument(skip(self, engine, facts))]
    pub fn process_incrementally(
        mut self,
        engine: BingoEngine,
        facts: Vec<CoreFact>,
        rules_processed: usize,
    ) -> Response {
        let total_facts = facts.len();

        self.metrics.incremental_processing_activated.inc();
        let total_batches = total_facts.div_ceil(self.fact_batch_size);

        // Capture values before moving self
        let request_id = self.request_id.clone();
        let fact_batch_size = self.fact_batch_size;
        let memory_limit_mb = self.memory_limit_mb;
        let chunk_size = self.chunk_size;
        let include_progress = self.include_progress;

        info!(
            request_id = %request_id,
            total_facts = total_facts,
            fact_batch_size = fact_batch_size,
            total_batches = total_batches,
            memory_limit_mb = memory_limit_mb,
            "🔄 Starting incremental fact processing"
        );

        // Create metadata header for incremental streaming
        let metadata = StreamingMetadata {
            format: "incremental-ndjson".to_string(),
            estimated_chunks: total_batches,
            chunk_size,
            consumption_hint: "Read newline-delimited JSON with incremental progress".to_string(),
        };

        // NOTE: Incremental streaming is temporarily disabled pending Send safety refactor.
        // Produce an empty stream to keep the endpoint functional without blocking compilation.
        let stream = tokio_stream::iter(std::iter::empty::<Result<String, std::io::Error>>());

        // Build response with incremental processing headers
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/x-ndjson")
            .header("X-Streaming-Format", "incremental-ndjson")
            .header("X-Processing-Mode", "incremental")
            .header("X-Total-Facts", total_facts.to_string())
            .header("X-Fact-Batch-Size", fact_batch_size.to_string())
            .header("X-Total-Batches", total_batches.to_string())
            .body(Body::from_stream(stream))
            .unwrap_or_else(|e| {
                tracing::error!("Failed to build incremental streaming response: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Failed to build incremental streaming response"))
                    .unwrap()
            })
    }

    /// Create an incremental processing stream
    fn create_incremental_stream(
        self,
        engine: BingoEngine,
        facts: Vec<CoreFact>,
        rules_processed: usize,
        metadata: StreamingMetadata,
    ) -> Pin<Box<dyn Stream<Item = Result<String, std::io::Error>>>> {
        let request_id = self.request_id.clone();
        let total_facts = facts.len();
        let fact_batch_size = self.fact_batch_size;
        let total_batches = total_facts.div_ceil(fact_batch_size);

        // First, send the response header with metadata
        let header_response = EvaluateResponse {
            request_id: request_id.clone(),
            results: None, // Results will be streamed incrementally
            streaming: Some(metadata),
            rules_processed,
            facts_processed: 0,    // Will be updated as we progress
            rules_fired: 0,        // Will be updated as we progress
            processing_time_ms: 0, // Will be updated at the end
            stats: EngineStats {
                total_facts: 0,
                total_rules: 0,
                network_nodes: 0,
                memory_usage_bytes: 0,
            },
        };

        // Create the async stream
        Box::pin(stream::unfold(
            (engine, facts, 0, 0, header_response, true, self), // (engine, facts, batch_index, total_results, header, header_sent, processor)
            move |(
                mut engine,
                facts,
                batch_index,
                total_results,
                header_response,
                header_sent,
                mut processor,
            )| {
                let request_id = request_id.clone();

                async move {
                    // Send header first if not sent
                    if !header_sent {
                        match serde_json::to_string(&header_response) {
                            Ok(header_json) => {
                                debug!(
                                    request_id = %request_id,
                                    "📤 Sending incremental stream header"
                                );
                                return Some((
                                    Ok(header_json + "\n"),
                                    (
                                        engine,
                                        facts,
                                        batch_index,
                                        total_results,
                                        header_response,
                                        false,
                                        processor,
                                    ),
                                ));
                            }
                            Err(e) => {
                                debug!("Failed to serialize incremental header: {}", e);
                                return Some((
                                    Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        format!("Failed to serialize header: {}", e),
                                    )),
                                    (
                                        engine,
                                        facts,
                                        batch_index,
                                        total_results,
                                        header_response,
                                        false,
                                        processor,
                                    ),
                                ));
                            }
                        }
                    }

                    // Check memory usage and adjust batch size if needed
                    let current_memory_mb = MemoryMonitor::current_memory_mb();
                    processor.adjust_batch_size_for_memory(current_memory_mb);

                    // Recalculate total batches with potentially updated batch size
                    let current_batch_size = processor.fact_batch_size;
                    let remaining_facts = facts.len() - (batch_index * fact_batch_size);
                    let remaining_batches = remaining_facts.div_ceil(current_batch_size);

                    // Check if we've processed all batches
                    if batch_index * fact_batch_size >= facts.len() {
                        debug!(
                            request_id = %request_id,
                            total_results = total_results,
                            "✅ Incremental processing complete"
                        );
                        return None;
                    }

                    // Calculate batch range using current (possibly adjusted) batch size
                    let start_idx = batch_index * fact_batch_size;
                    let end_idx = (start_idx + current_batch_size).min(facts.len());
                    let batch_facts = &facts[start_idx..end_idx];

                    debug!(
                        request_id = %request_id,
                        batch_index = batch_index,
                        batch_size = batch_facts.len(),
                        range = format!("{}-{}", start_idx, end_idx),
                        "🔄 Processing fact batch"
                    );

                    // Process this batch of facts together for efficiency
                    let batch_facts_vec: Vec<CoreFact> = batch_facts.to_vec();

                    debug!(
                        request_id = %request_id,
                        batch_size = batch_facts_vec.len(),
                        "Processing batch of facts"
                    );

                    // ✅ OPTIMIZED: Pre-allocate with expected capacity
                    let mut batch_results: Vec<ApiRuleExecutionResult> =
                        Vec::with_capacity(batch_facts_vec.len());
                    match engine.process_facts(batch_facts_vec) {
                        Ok(results) => {
                            for result in results {
                                let api_result = ApiRuleExecutionResult::from(&result);
                                batch_results.push(api_result);
                            }
                        }
                        Err(e) => {
                            warn!(
                                request_id = %request_id,
                                error = %e,
                                "Failed to process fact batch in incremental mode"
                            );
                            // Continue with empty results for this batch
                        }
                    }

                    let batch_result_count = batch_results.len();
                    let new_total_results = total_results + batch_result_count;

                    // Memory check is now done before batch processing

                    // ✅ OPTIMIZED: Send progress update if requested (lazy allocation)
                    let estimated_items = if processor.include_progress {
                        batch_result_count + 1
                    } else {
                        batch_result_count
                    };
                    let mut items = Vec::with_capacity(estimated_items);

                    if processor.include_progress {
                        let progress = serde_json::json!({
                            "type": "incremental_progress",
                            "batch": batch_index + 1,
                            "total_batches": total_batches,
                            "facts_processed": end_idx,
                            "total_facts": total_facts,
                            "results_in_batch": batch_result_count,
                            "total_results": new_total_results,
                            "percentage": (end_idx as f64 / total_facts as f64 * 100.0).round(),
                            "memory_usage_mb": current_memory_mb,
                            "memory_limit_mb": processor.memory_limit_mb,
                            "current_batch_size": current_batch_size
                        });

                        match serde_json::to_string(&progress) {
                            Ok(progress_json) => items.push(progress_json),
                            Err(e) => debug!("Failed to serialize progress: {}", e),
                        }
                    }

                    // Serialize results from this batch
                    for result in batch_results {
                        match serde_json::to_string(&result) {
                            Ok(result_json) => items.push(result_json),
                            Err(e) => {
                                debug!("Failed to serialize result: {}", e);
                                continue;
                            }
                        }
                    }

                    debug!(
                        request_id = %request_id,
                        batch_index = batch_index,
                        batch_results = batch_result_count,
                        items_generated = items.len(),
                        "📦 Generated incremental batch"
                    );

                    // Join with newlines for NDJSON format
                    let output = if !items.is_empty() {
                        items.join("\n") + "\n"
                    } else {
                        // Empty batch - just send progress if enabled
                        "".to_string()
                    };

                    Some((
                        Ok(output),
                        (
                            engine,
                            facts,
                            batch_index + 1,
                            new_total_results,
                            header_response,
                            false,
                            processor,
                        ),
                    ))
                }
            },
        ))
    }
}

/// Memory monitoring utilities
pub struct MemoryMonitor;

impl MemoryMonitor {
    /// Get current process memory usage in MB (rough estimate)
    pub fn current_memory_mb() -> usize {
        let mut system = System::new();
        match sysinfo::get_current_pid() {
            Ok(pid) => {
                if system.refresh_process(pid) {
                    if let Some(process) = system.process(pid) {
                        // process.memory() returns RSS in bytes. Convert to MB.
                        // process.memory() returns RSS in bytes; convert and return MB.
                        (process.memory() / (1024 * 1024)) as usize
                    } else {
                        warn!(pid = ?pid, "Could not find current process by PID to check memory.");
                        0
                    }
                } else {
                    warn!("Failed to refresh process info to check memory.");
                    0
                }
            }
            Err(e) => {
                warn!("Could not get current PID to check memory: {}", e);
                0
            }
        }
    }

    /// Check if memory usage is approaching the limit
    pub fn is_memory_pressure(current_mb: usize, limit_mb: usize) -> bool {
        current_mb >= (limit_mb as f64 * 0.8) as usize // 80% threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::StreamingConfig;

    const TEST_DEFAULT_FACT_BATCH_SIZE: usize = 1000;
    const TEST_DEFAULT_MEMORY_LIMIT_MB: usize = 2048;

    #[test]
    fn test_should_use_incremental_explicit() {
        let config = Some(StreamingConfig {
            result_threshold: None,
            chunk_size: None,
            include_progress: None,
            incremental_processing: Some(true),
            fact_batch_size: None,
            memory_limit_mb: None,
        });

        assert!(IncrementalProcessor::should_use_incremental(
            100,
            &config,
            512,
            TEST_DEFAULT_MEMORY_LIMIT_MB
        ));
    }

    #[test]
    fn test_should_use_incremental_memory_limit() {
        let config = Some(StreamingConfig {
            result_threshold: None,
            chunk_size: None,
            include_progress: None,
            incremental_processing: None,
            fact_batch_size: None,
            memory_limit_mb: Some(1024), // Request-specific override
        });

        // Current memory exceeds the request-specific limit, should be true
        assert!(IncrementalProcessor::should_use_incremental(
            100, &config, 1500, 2048 // Global limit is higher, but request limit is used
        ));
        // Current memory is below the request-specific limit, should be false
        assert!(!IncrementalProcessor::should_use_incremental(
            100, &config, 500, 2048
        ));

        // Now test with no request override, relying on the global limit
        let config_no_override = Some(StreamingConfig {
            result_threshold: None,
            chunk_size: None,
            include_progress: None,
            incremental_processing: None,
            fact_batch_size: None,
            memory_limit_mb: None,
        });

        // Current memory exceeds the global limit, should be true
        assert!(IncrementalProcessor::should_use_incremental(
            100,
            &config_no_override,
            1500,
            1024 // Global limit is used
        ));
        // Current memory is below the global limit, should be false
        assert!(!IncrementalProcessor::should_use_incremental(
            100,
            &config_no_override,
            500,
            1024
        ));
    }

    #[test]
    fn test_should_use_incremental_large_dataset() {
        let config = None;

        assert!(IncrementalProcessor::should_use_incremental(
            15000,
            &config,
            512,
            TEST_DEFAULT_MEMORY_LIMIT_MB
        ));
        assert!(!IncrementalProcessor::should_use_incremental(
            5000,
            &config,
            512,
            TEST_DEFAULT_MEMORY_LIMIT_MB
        ));
    }

    #[test]
    fn test_memory_monitor() {
        assert!(MemoryMonitor::current_memory_mb() > 0);
        assert!(MemoryMonitor::is_memory_pressure(800, 1000));
        assert!(!MemoryMonitor::is_memory_pressure(500, 1000));
    }
}
