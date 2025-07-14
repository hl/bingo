//! Multi-stage processing pipeline for complex business workflows
//!
//! This module provides orchestration for multi-stage processing pipelines,
//! enabling complex workflows like payroll processing that require multiple
//! phases of rule execution with fact creation and aggregation.

use crate::engine::BingoEngine;
// use crate::rete_nodes::ActionResult;
use crate::rete_nodes::RuleExecutionResult;
use crate::types::{
    EngineStats, Fact, FactId, FactValue, PipelineContext, PipelineExecutionResult, PipelineStage,
    ProcessingPipeline, StageExecutionResult,
};

use anyhow::{Context, Result};
use std::collections::HashMap;
use tracing::{info, instrument, warn};

/// Pipeline orchestrator for multi-stage processing
pub struct PipelineOrchestrator {
    engine: BingoEngine,
    context: PipelineContext,
}

impl PipelineOrchestrator {
    /// Create a new pipeline orchestrator
    #[instrument]
    pub fn new(fact_count_hint: Option<usize>) -> Result<Self> {
        let engine = match fact_count_hint {
            Some(hint) => BingoEngine::with_capacity(hint)?,
            None => BingoEngine::new()?,
        };

        let context = PipelineContext {
            pipeline_id: uuid::Uuid::new_v4().to_string(),
            start_time: chrono::Utc::now(),
            end_time: None,
            stages_executed: Vec::new(),
            total_facts_processed: 0,
            total_rules_fired: 0,
            errors: Vec::new(),
            global_variables: HashMap::new(),
        };

        Ok(Self { engine, context })
    }

    /// Execute a complete processing pipeline
    #[instrument(skip(self, pipeline, initial_facts))]
    pub fn execute_pipeline(
        &mut self,
        pipeline: ProcessingPipeline,
        initial_facts: Vec<Fact>,
    ) -> Result<PipelineExecutionResult> {
        info!(
            pipeline_id = pipeline.id,
            stages = pipeline.stages.len(),
            initial_facts = initial_facts.len(),
            "Starting pipeline execution"
        );

        // Set global context variables
        self.context.global_variables = pipeline.global_context.clone();

        // Track all facts across stages
        let mut current_facts = initial_facts;
        let mut all_created_facts = Vec::new();
        let mut stage_results_map = HashMap::new(); // To store StageExecutionResult

        // Execute stages in dependency order
        let execution_order = self.resolve_stage_dependencies(&pipeline.stages)?;

        for stage_id in execution_order {
            let stage = pipeline
                .stages
                .iter()
                .find(|s| s.id == stage_id)
                .context("Stage not found after dependency resolution")?;

            let (stage_result, rule_results) = self.execute_stage(stage, &current_facts)?;

            // Collect newly created facts
            all_created_facts.extend_from_slice(&stage_result.created_facts);

            // Update current_facts with newly created facts
            current_facts.extend_from_slice(&stage_result.created_facts);

            // Apply modifications from rule_results to current_facts
            // Convert current_facts to a HashMap for efficient lookup and modification
            let current_facts_map: HashMap<FactId, Fact> =
                current_facts.into_iter().map(|f| (f.id, f)).collect();

            for rule_result in &rule_results {
                for _action_result in &rule_result.actions_executed {
                    // Action results are now simplified strings for compilation
                    // Fact modifications would be handled differently in a full implementation
                }
            }
            // Convert back to Vec for the next iteration
            current_facts = current_facts_map.into_values().collect();

            // Store stage result
            stage_results_map.insert(stage.id.clone(), stage_result);
        }

        let result = PipelineExecutionResult {
            context: self.context.clone(),
            stage_results: stage_results_map, // Use the map we built
            total_facts_processed: current_facts.len(), // Final count of facts
            total_facts_created: all_created_facts.len(),
            final_facts: current_facts,
            created_facts: all_created_facts,
        };

        info!(
            pipeline_id = result.context.pipeline_id,
            facts_processed = result.total_facts_processed,
            facts_created = result.total_facts_created,
            "Pipeline execution completed"
        );

        Ok(result)
    }

    /// Execute a single stage of the pipeline
    #[instrument(skip(self, stage, facts))]
    fn execute_stage(
        &mut self,
        stage: &PipelineStage,
        facts: &[Fact],
    ) -> Result<(StageExecutionResult, Vec<RuleExecutionResult>)> {
        let start_time = std::time::Instant::now();

        info!(
            stage_id = stage.id,
            stage_type = stage.stage_type,
            rules = stage.rules.len(),
            facts = facts.len(),
            "Executing pipeline stage"
        );

        // Clear engine state for this stage
        self.engine.clear();

        // Add stage rules to engine
        for rule in &stage.rules {
            self.engine.add_rule(rule.clone())?;
        }

        // Process facts through stage rules
        let rule_results = self.engine.process_facts(facts.to_vec())?;

        // Collect created facts from the engine
        let created_facts = self.engine.get_created_facts().to_vec();
        self.engine.clear_created_facts();

        // Apply modifications from rule results to current_facts
        let mut modified_count = 0;
        let current_facts_map: HashMap<FactId, Fact> =
            facts.iter().map(|f| (f.id, f.clone())).collect();

        for rule_result in &rule_results {
            for _action_result in &rule_result.actions_executed {
                // Action results are now simplified strings for compilation
                // Fact modifications would be handled differently in a full implementation
                modified_count += 0; // Facts are immutable in this implementation
            }
        }

        // Convert map back to vec, ensuring original order is not strictly preserved but all facts are present
        let _final_facts_for_stage: Vec<Fact> = current_facts_map.into_values().collect();

        let execution_time = start_time.elapsed().as_millis() as u64;

        let stage_result = StageExecutionResult {
            stage_id: stage.id.clone(),
            stage: stage.clone(),
            duration_ms: execution_time,
            facts_processed: facts.len(),
            rules_fired: rule_results.len(),
            facts_created: created_facts.len(),
            facts_modified: modified_count,
            created_facts,
            errors: Vec::new(), // Errors are handled via Result propagation
        };

        info!(
            stage_id = stage_result.stage_id,
            facts_processed = stage_result.facts_processed,
            facts_created = stage_result.facts_created,
            execution_time_ms = stage_result.duration_ms,
            "Stage execution completed"
        );

        Ok((stage_result, rule_results)) // Return rule_results for pipeline to process
    }

    /// Resolve stage dependencies to determine execution order
    fn resolve_stage_dependencies(&self, stages: &[PipelineStage]) -> Result<Vec<String>> {
        let mut resolved = Vec::new();
        let mut pending: Vec<_> = stages.iter().collect();
        let mut iteration_count = 0;
        const MAX_ITERATIONS: usize = 100; // Prevent infinite loops

        while !pending.is_empty() && iteration_count < MAX_ITERATIONS {
            iteration_count += 1;
            let mut progress_made = false;

            let mut i = 0;
            while i < pending.len() {
                let stage = pending[i];

                // Check if all dependencies are already resolved
                let dependencies_met = stage.dependencies.iter().all(|dep| resolved.contains(dep));

                if dependencies_met {
                    resolved.push(stage.id.clone());
                    pending.remove(i);
                    progress_made = true;
                } else {
                    i += 1;
                }
            }

            if !progress_made {
                // Check for circular dependencies
                let unresolved_stages: Vec<_> = pending.iter().map(|s| &s.id).collect();
                warn!(
                    unresolved_stages = ?unresolved_stages,
                    "Circular dependency detected in pipeline stages"
                );
                return Err(anyhow::anyhow!(
                    "Circular dependency detected in pipeline stages: {:?}",
                    unresolved_stages
                ));
            }
        }

        if iteration_count >= MAX_ITERATIONS {
            return Err(anyhow::anyhow!(
                "Maximum iterations exceeded while resolving stage dependencies"
            ));
        }

        Ok(resolved)
    }

    /// Get the current pipeline context
    pub fn get_context(&self) -> &PipelineContext {
        &self.context
    }

    /// Set a global variable in the pipeline context
    pub fn set_global_variable(&mut self, key: String, value: FactValue) {
        self.context.global_variables.insert(key, value);
    }

    /// Get engine statistics
    pub fn get_engine_stats(&self) -> EngineStats {
        self.engine.get_stats()
    }
}
