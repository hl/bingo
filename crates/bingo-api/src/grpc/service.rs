use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};

use crate::AppState;
use crate::generated::processing_control::ControlType;
use crate::generated::rules_engine_service_server::RulesEngineService;
use crate::generated::*;
use crate::grpc::conversions::{from_proto_fact, from_proto_rule, to_proto_result};
use bingo_core::{BingoEngine, Rule as CoreRule};

pub struct RulesEngineServiceImpl {
    app_state: Arc<AppState>,
}

impl RulesEngineServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

#[tonic::async_trait]
impl RulesEngineService for RulesEngineServiceImpl {
    // Phase 1: Compile and validate rules
    async fn compile_rules(
        &self,
        request: Request<CompileRulesRequest>,
    ) -> Result<Response<CompileRulesResponse>, Status> {
        let req = request.into_inner();
        let session_id = if req.session_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            req.session_id
        };

        tracing::info!(
            session_id = %session_id,
            rules_count = req.rules.len(),
            "Compiling rules for session"
        );

        let start_time = std::time::Instant::now();

        // Convert proto rules to core rules
        let core_rules: Vec<CoreRule> = req
            .rules
            .into_iter()
            .map(from_proto_rule)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Status::invalid_argument(format!("Invalid rule: {e}")))?;

        // Get or create engine for this session
        let engine = self.app_state.get_or_create_engine(&session_id);

        // Add rules to the engine
        for rule in core_rules.iter() {
            engine
                .add_rule(rule.clone())
                .map_err(|e| Status::invalid_argument(format!("Rule compilation failed: {e}")))?;
        }

        let compilation_time = start_time.elapsed();
        let stats = engine.get_stats();

        tracing::info!(
            session_id = %session_id,
            rules_compiled = core_rules.len(),
            network_nodes = stats.node_count,
            compilation_time_ms = compilation_time.as_millis(),
            "Rules compiled and validated successfully"
        );

        Ok(Response::new(CompileRulesResponse {
            session_id,
            success: true,
            error_message: String::new(),
            rules_compiled: core_rules.len() as i32,
            network_nodes_created: stats.node_count as i32,
            compilation_time_ms: compilation_time.as_millis() as i64,
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
        }))
    }

    type ProcessFactsStreamStream =
        Pin<Box<dyn Stream<Item = Result<RuleExecutionResult, Status>> + Send>>;

    // Phase 2: Stream facts through rules (simplified implementation)
    async fn process_facts_stream(
        &self,
        request: Request<Streaming<ProcessFactsStreamRequest>>,
    ) -> Result<Response<Self::ProcessFactsStreamStream>, Status> {
        let mut request_stream = request.into_inner();

        let stream = async_stream::stream! {
            let mut session_id = String::new();

            while let Some(request) = request_stream.next().await {
                let request = match request {
                    Ok(req) => req,
                    Err(e) => {
                        yield Err(Status::internal(format!("Stream error: {e}")));
                        return;
                    }
                };

                match request.request {
                    Some(process_facts_stream_request::Request::SessionId(sid)) => {
                        session_id = sid;
                        tracing::info!(session_id = %session_id, "Session initialized for fact streaming");
                    }
                    Some(process_facts_stream_request::Request::FactBatch(fact)) => {
                        if session_id.is_empty() {
                            yield Err(Status::failed_precondition("No session initialized"));
                            return;
                        }

                        let core_fact = match from_proto_fact(fact) {
                            Ok(f) => f,
                            Err(e) => {
                                yield Err(Status::invalid_argument(format!("Invalid fact: {e}")));
                                continue;
                            }
                        };

                        // Create a simple dummy result for demonstration
                        let result = RuleExecutionResult {
                            rule_id: "demo_rule".to_string(),
                            rule_name: "Demo Rule".to_string(),
                            matched_fact: Some(crate::grpc::conversions::to_proto_fact(&core_fact)),
                            action_results: vec![],
                            execution_time_ns: 1000,
                            metadata: HashMap::new(),
                        };

                        yield Ok(result);
                    }
                    Some(process_facts_stream_request::Request::Control(control)) => {
                        // Handle control messages
                        match control.r#type() {
                            ControlType::Pause => {
                                tracing::info!(session_id = %session_id, "Processing paused");
                            }
                            ControlType::Resume => {
                                tracing::info!(session_id = %session_id, "Processing resumed");
                            }
                            ControlType::Stop => {
                                tracing::info!(session_id = %session_id, "Processing stopped by client");
                                return;
                            }
                            ControlType::Flush => {
                                tracing::info!(session_id = %session_id, "Flush requested");
                            }
                        }
                    }
                    None => {
                        yield Err(Status::invalid_argument("Empty request"));
                        return;
                    }
                }
            }
        };

        Ok(Response::new(Box::pin(stream)))
    }

    type ProcessWithRulesStreamStream =
        Pin<Box<dyn Stream<Item = Result<ProcessingResponse, Status>> + Send>>;

    // Alternative: Single-call with rules validation before fact streaming
    async fn process_with_rules_stream(
        &self,
        request: Request<ProcessWithRulesRequest>,
    ) -> Result<Response<Self::ProcessWithRulesStreamStream>, Status> {
        let req = request.into_inner();
        let request_id = req.request_id.clone();

        tracing::info!(
            request_id = %request_id,
            rules_count = req.rules.len(),
            facts_count = req.facts.len(),
            validate_only = req.validate_rules_only,
            "Starting single-call processing with rules validation"
        );

        // Clone data for use in the stream
        let rules = req.rules.clone();
        let facts = req.facts.clone();
        let validate_only = req.validate_rules_only;
        let options = req.options.clone();

        // For now, create a simple working version that doesn't use streaming engine processing
        // This avoids the thread safety issues with BingoEngine while we establish the gRPC foundation
        let stream = async_stream::stream! {
            let start_time = std::time::Instant::now();

            // Phase 1: Convert and validate rules
            let core_rules: Vec<CoreRule> = match rules
                .into_iter()
                .map(from_proto_rule)
                .collect::<Result<Vec<_>, _>>() {
                Ok(rules) => rules,
                Err(e) => {
                    yield Err(Status::invalid_argument(format!("Invalid rule: {e}")));
                    return;
                }
            };

            // For compilation validation, use BingoEngine
            let rules_count = core_rules.len();
            let engine = BingoEngine::new().map_err(|e| Status::internal(format!("Failed to create engine: {e}")))?;
            let start = std::time::Instant::now();

            // Add rules to the engine
            for rule in core_rules.iter() {
                if let Err(e) = engine.add_rule(rule.clone()) {
                    yield Err(Status::invalid_argument(format!("Rule compilation failed: {e}")));
                    return;
                }
            }

            let stats = engine.get_stats();
            let compilation_time = start.elapsed();
            let node_count = stats.node_count;

            // Yield compilation result first
            yield Ok(ProcessingResponse {
                response: Some(processing_response::Response::RulesCompiled(
                    CompileRulesResponse {
                        session_id: request_id.clone(),
                        success: true,
                        error_message: String::new(),
                        rules_compiled: rules_count as i32,
                        network_nodes_created: node_count as i32,
                        compilation_time_ms: compilation_time.as_millis() as i64,
                        engine_version: env!("CARGO_PKG_VERSION").to_string(),
                    }
                ))
            });

            // If validation only, stop here
            if validate_only {
                return;
            }

            // Phase 2: Process facts - simplified for demonstration
            let core_facts: Vec<bingo_core::Fact> = match facts
                .into_iter()
                .map(from_proto_fact)
                .collect::<Result<Vec<_>, _>>() {
                Ok(facts) => facts,
                Err(e) => {
                    yield Err(Status::invalid_argument(format!("Invalid fact: {e}")));
                    return;
                }
            };

            // Process facts through the engine
            let total_facts = core_facts.len();
            let mut total_results = 0;

            // Process facts in the engine
            if !validate_only {
                match engine.process_facts(core_facts) {
                    Ok(results) => {
                        total_results = results.len();

                        yield Ok(ProcessingResponse {
                            response: Some(processing_response::Response::StatusUpdate(
                                ProcessingStatus {
                                    request_id: request_id.clone(),
                                    facts_processed: total_facts as i32,
                                    rules_executed: rules_count as i32,
                                    results_generated: total_results as i32,
                                    processing_time_ms: start_time.elapsed().as_millis() as i64,
                                    completed: false,
                                    error_message: String::new(),
                                }
                            ))
                        });
                    }
                    Err(e) => {
                        yield Err(Status::internal(format!("Fact processing failed: {e}")));
                        return;
                    }
                }
            } else {
                yield Ok(ProcessingResponse {
                    response: Some(processing_response::Response::StatusUpdate(
                        ProcessingStatus {
                            request_id: request_id.clone(),
                            facts_processed: 0, // Validation only
                            rules_executed: rules_count as i32,
                            results_generated: 0,
                            processing_time_ms: start_time.elapsed().as_millis() as i64,
                            completed: false,
                            error_message: String::new(),
                        }
                    ))
                });
            }

            // Final completion message
            yield Ok(ProcessingResponse {
                response: Some(processing_response::Response::Completion(
                    ProcessingComplete {
                        request_id,
                        total_facts_processed: if validate_only { 0 } else { total_facts as i32 },
                        total_results_generated: total_results as i32,
                        total_processing_time_ms: start_time.elapsed().as_millis() as i64,
                        success: true,
                        error_message: String::new(),
                    }
                ))
            });
        };

        Ok(Response::new(Box::pin(stream)))
    }

    type ProcessFactsBatchStream =
        Pin<Box<dyn Stream<Item = Result<ProcessingStatus, Status>> + Send>>;

    async fn process_facts_batch(
        &self,
        _request: Request<ProcessFactsRequest>,
    ) -> Result<Response<Self::ProcessFactsBatchStream>, Status> {
        // Implementation similar to above but returns status updates instead of individual results
        Err(Status::unimplemented(
            "ProcessFactsBatch not yet implemented",
        ))
    }

    type EvaluateRulesetStreamStream =
        Pin<Box<dyn Stream<Item = Result<RuleExecutionResult, Status>> + Send>>;

    async fn evaluate_ruleset_stream(
        &self,
        _request: Request<EvaluateRulesetRequest>,
    ) -> Result<Response<Self::EvaluateRulesetStreamStream>, Status> {
        // Implementation using cached rulesets
        Err(Status::unimplemented(
            "EvaluateRulesetStream not yet implemented",
        ))
    }

    async fn register_ruleset(
        &self,
        _request: Request<RegisterRulesetRequest>,
    ) -> Result<Response<RegisterRulesetResponse>, Status> {
        // Implementation for ruleset caching
        Err(Status::unimplemented("RegisterRuleset not yet implemented"))
    }

    async fn health_check(
        &self,
        _request: Request<()>,
    ) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.app_state.elapsed().as_secs() as i64,
        }))
    }
}
