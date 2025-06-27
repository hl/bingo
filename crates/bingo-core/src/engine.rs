use crate::fact_store::{ArenaFactStore, FactStore};
use crate::rete_network::{ReteNetwork, RuleExecutionResult};
use crate::types::*;
use anyhow::Result;
use tracing::{info, instrument};

/// Main engine for processing rules and facts - simplified for API use
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

        let fact_store = Box::new(ArenaFactStore::new());
        let rete_network = ReteNetwork::new();

        Ok(Self { rules: Vec::new(), fact_store, rete_network })
    }

    /// Create an engine with capacity hint for facts
    #[instrument]
    pub fn with_capacity(fact_count_hint: usize) -> Result<Self> {
        info!(
            fact_count_hint = fact_count_hint,
            "Creating Bingo engine with capacity hint"
        );

        let fact_store = Box::new(ArenaFactStore::with_capacity(fact_count_hint));
        let rete_network = ReteNetwork::new();

        Ok(Self { rules: Vec::new(), fact_store, rete_network })
    }

    /// Add a rule to the engine
    #[instrument(skip(self))]
    pub fn add_rule(&mut self, rule: Rule) -> Result<()> {
        info!(rule_id = rule.id, "Adding rule to engine");

        // Add to RETE network for pattern matching
        self.rete_network.add_rule(rule.clone())?;

        // Store the rule
        self.rules.push(rule);

        Ok(())
    }

    /// Add multiple rules at once
    #[instrument(skip(self, rules))]
    pub fn add_rules(&mut self, rules: Vec<Rule>) -> Result<()> {
        info!(rule_count = rules.len(), "Adding multiple rules to engine");

        for rule in rules {
            self.add_rule(rule)?;
        }

        Ok(())
    }

    /// Process facts and return rule execution results
    #[instrument(skip(self, facts))]
    pub fn process_facts(&mut self, facts: Vec<Fact>) -> Result<Vec<RuleExecutionResult>> {
        info!(fact_count = facts.len(), "Processing facts through engine");

        // Store facts in the fact store
        for fact in &facts {
            self.fact_store.insert(fact.clone());
        }

        // Process through RETE network
        let results = self.rete_network.process_facts(&facts, self.fact_store.as_ref())?;

        info!(rules_fired = results.len(), "Completed fact processing");
        Ok(results)
    }

    /// Simple API: process rules and facts, return results
    #[instrument(skip(self, rules, facts))]
    pub fn evaluate(
        &mut self,
        rules: Vec<Rule>,
        facts: Vec<Fact>,
    ) -> Result<Vec<RuleExecutionResult>> {
        info!(
            rule_count = rules.len(),
            fact_count = facts.len(),
            "Evaluating rules against facts"
        );

        // Clear previous state
        self.rules.clear();
        self.fact_store.clear();
        self.rete_network = ReteNetwork::new();

        // Add rules
        self.add_rules(rules)?;

        // Process facts
        self.process_facts(facts)
    }

    /// Get current engine statistics
    pub fn get_stats(&self) -> EngineStats {
        let rete_stats = self.rete_network.get_stats();

        EngineStats {
            rule_count: self.rules.len(),
            fact_count: self.fact_store.len(),
            node_count: rete_stats.node_count as usize,
            memory_usage_bytes: rete_stats.memory_usage_bytes as usize,
        }
    }

    /// Get the number of rules loaded
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Get the number of facts stored
    pub fn fact_count(&self) -> usize {
        self.fact_store.len()
    }

    /// Clear all rules and facts
    #[instrument(skip(self))]
    pub fn clear(&mut self) {
        info!("Clearing all rules and facts from engine");

        self.rules.clear();
        self.fact_store.clear();
        self.rete_network = ReteNetwork::new();
    }

    /// Update an existing rule
    #[instrument(skip(self))]
    pub fn update_rule(&mut self, rule: Rule) -> Result<()> {
        info!(rule_id = rule.id, "Updating rule in engine");

        // Remove the old rule if it exists
        self.remove_rule(rule.id)?;

        // Add the new rule
        self.add_rule(rule)
    }

    /// Remove a rule by ID
    #[instrument(skip(self))]
    pub fn remove_rule(&mut self, rule_id: u64) -> Result<()> {
        info!(rule_id = rule_id, "Removing rule from engine");

        // Remove from rules vector
        self.rules.retain(|rule| rule.id != rule_id);

        // For simplicity, rebuild the entire network when removing rules
        // In a more sophisticated implementation, we'd selectively remove nodes
        self.rete_network = ReteNetwork::new();
        for rule in &self.rules {
            self.rete_network.add_rule(rule.clone())?;
        }

        Ok(())
    }
}

impl Default for BingoEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default BingoEngine")
    }
}
