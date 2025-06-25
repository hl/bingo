use crate::fact_store::{FactStore, FactStoreFactory};
use crate::performance_tracking::PerformanceConfig;
use crate::rete_network::ReteNetwork;
use crate::types::*;
use anyhow::{Context, Result};
use tracing::{debug, error, info, instrument, warn};

/// Main engine for processing rules and facts
pub struct BingoEngine {
    rules: Vec<Rule>,
    fact_store: Box<dyn FactStore>,
    rete_network: ReteNetwork,
}

impl BingoEngine {
    /// Create a new engine instance
    #[instrument]
    pub fn new() -> Result<Self> {
        info!("Creating new Bingo engine");

        let rete_network =
            ReteNetwork::new().context("Failed to create RETE network for new engine")?;

        let engine =
            Self { rules: Vec::new(), fact_store: FactStoreFactory::create_simple(), rete_network };

        info!(
            fact_store_type = "simple",
            "Bingo engine created successfully"
        );

        Ok(engine)
    }

    /// Create a new engine optimized for the expected number of facts
    #[instrument]
    pub fn with_capacity(fact_count_hint: usize) -> Result<Self> {
        if fact_count_hint == 0 {
            warn!("Creating engine with zero capacity hint, falling back to default");
            return Self::new();
        }

        info!(fact_count_hint, "Creating optimized Bingo engine");

        let rete_network =
            ReteNetwork::new().context("Failed to create RETE network for optimized engine")?;

        let engine = Self {
            rules: Vec::new(),
            fact_store: FactStoreFactory::create_optimized(fact_count_hint),
            rete_network,
        };

        info!(
            fact_count_hint = fact_count_hint,
            fact_store_type = "optimized",
            "Optimized Bingo engine created successfully"
        );

        Ok(engine)
    }

    /// Add a rule to the engine
    #[instrument(skip(self), fields(rule_id = %rule.id, rule_name = %rule.name))]
    pub fn add_rule(&mut self, rule: Rule) -> Result<()> {
        if rule.name.is_empty() {
            warn!(rule_id = rule.id, "Adding rule with empty name");
        }

        if rule.conditions.is_empty() {
            error!(
                rule_id = rule.id,
                rule_name = %rule.name,
                "Cannot add rule with no conditions"
            );
            return Err(anyhow::anyhow!(
                "Rule '{}' (ID: {}) must have at least one condition",
                rule.name,
                rule.id
            ))
            .context("Failed to add rule to engine");
        }

        // Check for duplicate rule ID
        if self.rules.iter().any(|r| r.id == rule.id) {
            error!(
                rule_id = rule.id,
                rule_name = %rule.name,
                "Rule with this ID already exists"
            );
            return Err(anyhow::anyhow!(
                "Rule with ID {} already exists in engine",
                rule.id
            ))
            .context("Failed to add rule to engine");
        }

        info!(
            rule_id = rule.id,
            rule_name = %rule.name,
            condition_count = rule.conditions.len(),
            action_count = rule.actions.len(),
            "Adding rule to engine"
        );

        // Add rule to RETE network for compilation
        self.rete_network.add_rule(rule.clone()).with_context(|| {
            format!(
                "Failed to compile rule '{}' (ID: {}) into RETE network",
                rule.name, rule.id
            )
        })?;

        self.rules.push(rule.clone());

        info!(
            rule_id = rule.id,
            rule_name = %rule.name,
            total_rules = self.rules.len(),
            "Rule added successfully to engine"
        );

        Ok(())
    }

    /// Update an existing rule in the engine
    #[instrument(skip(self), fields(rule_id = %updated_rule.id, rule_name = %updated_rule.name))]
    pub fn update_rule(&mut self, updated_rule: Rule) -> Result<()> {
        info!(rule_id = %updated_rule.id, rule_name = %updated_rule.name, "Updating rule in engine");

        // Find and replace the rule in memory
        if let Some(existing_rule_index) = self.rules.iter().position(|r| r.id == updated_rule.id) {
            // Remove old rule from RETE network
            self.rete_network.remove_rule(updated_rule.id)?;

            // Update rule in memory
            self.rules[existing_rule_index] = updated_rule.clone();

            // Add updated rule to RETE network
            self.rete_network.add_rule(updated_rule.clone())?;

            info!(rule_id = %updated_rule.id, "Rule updated successfully in engine");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Rule with ID {} not found in engine",
                updated_rule.id
            ))
        }
    }

    /// Remove a rule from the engine
    #[instrument(skip(self), fields(rule_id = %rule_id))]
    pub fn remove_rule(&mut self, rule_id: u64) -> Result<()> {
        info!(rule_id = %rule_id, "Removing rule from engine");

        // Find and remove the rule from memory
        if let Some(existing_rule_index) = self.rules.iter().position(|r| r.id == rule_id) {
            // Remove from RETE network
            self.rete_network.remove_rule(rule_id)?;

            // Remove from memory
            let removed_rule = self.rules.remove(existing_rule_index);

            info!(rule_id = %rule_id, rule_name = %removed_rule.name, "Rule removed successfully from engine");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Rule with ID {} not found in engine",
                rule_id
            ))
        }
    }

    /// Process a batch of facts through the rule engine
    #[instrument(skip(self, facts), fields(fact_count = facts.len()))]
    pub fn process_facts(&mut self, facts: Vec<Fact>) -> Result<Vec<Fact>> {
        let fact_count = facts.len();
        let start_time = std::time::Instant::now();

        if facts.is_empty() {
            debug!("No facts provided for processing, returning empty result");
            return Ok(Vec::new());
        }

        if self.rules.is_empty() {
            warn!(
                fact_count = fact_count,
                "Processing facts but no rules are defined - no results will be produced"
            );
            return Ok(Vec::new());
        }

        // Validate facts before processing
        let mut invalid_facts = 0;
        for fact in &facts {
            if fact.data.fields.is_empty() {
                invalid_facts += 1;
            }
        }

        if invalid_facts > 0 {
            warn!(
                invalid_facts = invalid_facts,
                total_facts = fact_count,
                "Some facts have no fields and may not match any rules"
            );
        }

        info!(
            fact_count = fact_count,
            rule_count = self.rules.len(),
            "Processing facts through engine"
        );

        // Start performance tracking session
        let _session_id = self.rete_network.start_performance_session(fact_count);

        let result = if fact_count > 10_000 {
            self.process_facts_parallel(facts)
                .context("Failed to process large fact batch in parallel")
        } else {
            self.process_facts_sequential(facts)
                .context("Failed to process fact batch sequentially")
        };

        // Finish performance tracking session
        if let Some(session_summary) = self.rete_network.performance_tracker_mut().finish_session()
        {
            debug!(
                session_id = %session_summary.session_id,
                session_duration_ms = session_summary.total_duration.as_millis(),
                facts_processed = session_summary.facts_processed,
                rules_fired = session_summary.rules_fired,
                throughput = session_summary.average_throughput,
                "Performance tracking session completed"
            );
        }

        let processing_time = start_time.elapsed();

        match &result {
            Ok(output_facts) => {
                info!(
                    input_fact_count = fact_count,
                    output_fact_count = output_facts.len(),
                    processing_time_ms = processing_time.as_millis(),
                    rule_count = self.rules.len(),
                    "Fact processing completed successfully"
                );
            }
            Err(error) => {
                error!(
                    input_fact_count = fact_count,
                    processing_time_ms = processing_time.as_millis(),
                    error = %error,
                    "Fact processing failed"
                );
            }
        }

        result
    }

    /// Process facts sequentially (optimal for smaller batches)
    fn process_facts_sequential(&mut self, facts: Vec<Fact>) -> Result<Vec<Fact>> {
        let fact_count = facts.len();

        // Store facts in fact store
        for fact in &facts {
            self.fact_store.insert(fact.clone());
        }

        // Process facts through RETE network
        let results = self.rete_network.process_facts(facts)?;

        info!(
            facts_processed = fact_count,
            results_generated = results.len(),
            mode = "sequential",
            "Facts processed through RETE network"
        );
        Ok(results)
    }

    /// Process facts in parallel for large batches (requires parallel feature)
    fn process_facts_parallel(&mut self, facts: Vec<Fact>) -> Result<Vec<Fact>> {
        #[cfg(feature = "parallel")]
        {
            let fact_count = facts.len();
            let chunk_size = (fact_count / rayon::current_num_threads()).max(1000);

            info!(
                fact_count,
                chunk_size,
                threads = rayon::current_num_threads(),
                "Processing facts in parallel"
            );

            // Store facts in parallel chunks
            let chunks: Vec<_> = facts.chunks(chunk_size).collect();

            for chunk in chunks {
                for fact in chunk {
                    self.fact_store.insert(fact.clone());
                }
            }

            // Process through RETE network (currently sequential, could be parallelized in future)
            let results = self.rete_network.process_facts(facts)?;

            info!(
                facts_processed = fact_count,
                results_generated = results.len(),
                mode = "parallel",
                "Facts processed through RETE network"
            );
            Ok(results)
        }

        #[cfg(not(feature = "parallel"))]
        {
            // Fall back to sequential processing
            info!("Parallel feature not enabled, falling back to sequential processing");
            self.process_facts_sequential(facts)
        }
    }

    /// Get engine statistics
    #[instrument(skip(self))]
    pub fn get_stats(&self) -> EngineStats {
        let rete_stats = self.rete_network.get_stats();
        EngineStats {
            rule_count: self.rules.len(),
            fact_count: self.fact_store.len(),
            node_count: rete_stats.node_count,
            memory_usage_bytes: std::mem::size_of_val(&self.rules)
                + std::mem::size_of_val(&*self.fact_store)
                + rete_stats.memory_usage_bytes,
        }
    }

    /// Get performance summary for all rules
    pub fn get_performance_summary(&self) -> crate::performance_tracking::PerformanceSummary {
        self.rete_network.get_performance_summary()
    }

    /// Get performance profile for a specific rule
    pub fn get_rule_performance_profile(
        &self,
        rule_id: u64,
    ) -> Option<&crate::performance_tracking::RuleExecutionProfile> {
        self.rete_network.performance_tracker().get_rule_profile(rule_id)
    }

    /// Identify performance bottlenecks
    pub fn identify_performance_bottlenecks(
        &self,
    ) -> Vec<crate::performance_tracking::PerformanceBottleneck> {
        self.rete_network.identify_performance_bottlenecks()
    }

    /// Configure performance tracking
    pub fn configure_performance_tracking(&mut self, config: PerformanceConfig) {
        self.rete_network.configure_performance_tracking(config);
    }

    /// Get global performance metrics
    pub fn get_global_performance_metrics(
        &self,
    ) -> &crate::performance_tracking::GlobalPerformanceMetrics {
        self.rete_network.performance_tracker().get_global_metrics()
    }
}
