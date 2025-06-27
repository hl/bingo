use crate::calculators::get_calculator;
use crate::fact_store::FactStore;
use crate::rete_nodes::*;
use crate::types::{ActionType, Condition, Fact, FactId, Operator, Rule, RuleId};
use anyhow::{Context, Result};
use std::collections::HashMap;
use tracing::{info, instrument};

/// RETE network for rule processing - simplified for essential functionality
#[derive(Debug)]
pub struct ReteNetwork {
    /// Alpha nodes for fact pattern matching
    alpha_nodes: HashMap<String, AlphaNode>,
    /// Beta nodes for join operations
    beta_nodes: HashMap<NodeId, BetaNode>,
    /// Terminal nodes for rule actions
    terminal_nodes: HashMap<RuleId, TerminalNode>,
    /// Rules in the network
    rules: HashMap<RuleId, Rule>,
    /// Next available node ID
    next_node_id: NodeId,
}

impl ReteNetwork {
    /// Create a new RETE network
    #[instrument]
    pub fn new() -> Self {
        info!("Creating new RETE network");
        Self {
            alpha_nodes: HashMap::new(),
            beta_nodes: HashMap::new(),
            terminal_nodes: HashMap::new(),
            rules: HashMap::new(),
            next_node_id: 1,
        }
    }

    /// Add a rule to the network
    #[instrument(skip(self))]
    pub fn add_rule(&mut self, rule: Rule) -> Result<()> {
        let rule_id = rule.id;
        info!(rule_id = rule_id, "Adding rule to RETE network");

        // Create alpha nodes for conditions
        for condition in &rule.conditions {
            self.create_alpha_node_for_condition(condition)?;
        }

        // Create terminal node for actions
        let node_id = self.next_node_id;
        self.next_node_id += 1;
        let terminal_node = TerminalNode::new(node_id, rule_id, rule.actions.clone());
        self.terminal_nodes.insert(rule_id, terminal_node);

        // Store the rule
        self.rules.insert(rule_id, rule);

        Ok(())
    }

    /// Process facts through the network and execute matching rules
    #[instrument(skip(self, fact_store))]
    pub fn process_facts(
        &mut self,
        facts: &[Fact],
        fact_store: &dyn FactStore,
    ) -> Result<Vec<RuleExecutionResult>> {
        let mut results = Vec::new();

        // For each fact, check which rules match
        for fact in facts {
            let matching_rules = self.find_matching_rules(fact)?;

            for rule_id in matching_rules {
                if let Some(rule) = self.rules.get(&rule_id) {
                    let result = self.execute_rule(rule, fact, fact_store)?;
                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    /// Find rules that match a given fact
    fn find_matching_rules(&self, fact: &Fact) -> Result<Vec<RuleId>> {
        let mut matching_rules = Vec::new();

        for (rule_id, rule) in &self.rules {
            if self.fact_matches_rule(fact, rule)? {
                matching_rules.push(*rule_id);
            }
        }

        Ok(matching_rules)
    }

    /// Check if a fact matches all conditions of a rule
    fn fact_matches_rule(&self, fact: &Fact, rule: &Rule) -> Result<bool> {
        for condition in &rule.conditions {
            if !self.fact_matches_condition(fact, condition)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Check if a fact matches a specific condition
    fn fact_matches_condition(&self, fact: &Fact, condition: &Condition) -> Result<bool> {
        match condition {
            Condition::Simple { field, operator, value } => {
                let fact_value = fact.data.fields.get(field);

                match fact_value {
                    Some(fact_val) => {
                        match operator {
                            Operator::Equal => Ok(fact_val == value),
                            Operator::NotEqual => Ok(fact_val != value),
                            Operator::GreaterThan => Ok(fact_val > value),
                            Operator::LessThan => Ok(fact_val < value),
                            Operator::GreaterThanOrEqual => Ok(fact_val >= value),
                            Operator::LessThanOrEqual => Ok(fact_val <= value),
                            Operator::Contains => {
                                // Basic contains implementation for strings
                                match (fact_val, value) {
                                    (
                                        crate::types::FactValue::String(fact_str),
                                        crate::types::FactValue::String(pattern),
                                    ) => Ok(fact_str.contains(pattern)),
                                    _ => Ok(false),
                                }
                            }
                        }
                    }
                    None => Ok(false), // Field doesn't exist
                }
            }
            Condition::Complex { operator: _, conditions: _ } => {
                // For now, complex conditions are not supported in this simplified version
                Ok(false)
            }
            Condition::Aggregation(_) => {
                // Aggregation conditions not supported in simplified version
                Ok(false)
            }
            Condition::Stream(_) => {
                // Stream conditions not supported in simplified version
                Ok(false)
            }
        }
    }

    /// Execute a rule with the given fact
    fn execute_rule(
        &self,
        rule: &Rule,
        fact: &Fact,
        _fact_store: &dyn FactStore,
    ) -> Result<RuleExecutionResult> {
        let mut action_results = Vec::new();

        for action in &rule.actions {
            let result = match &action.action_type {
                ActionType::SetField { field, value } => {
                    ActionResult::FieldSet { field: field.clone(), value: value.clone() }
                }
                ActionType::CallCalculator { calculator_name, input_mapping, output_field: _ } => {
                    // Get the predefined calculator
                    let calculator = get_calculator(calculator_name)
                        .with_context(|| format!("Calculator '{}' not found", calculator_name))?;

                    // Map inputs from fact fields to calculator inputs
                    let mut bound_variables = HashMap::new();
                    for (calc_field, fact_field) in input_mapping {
                        if let Some(value) = fact.data.fields.get(fact_field) {
                            bound_variables.insert(calc_field.clone(), value.clone());
                        }
                    }

                    // Create calculator inputs with the bound variables
                    let calc_inputs = crate::calculators::CalculatorInputs::new(&bound_variables);

                    // Execute the calculator
                    let calc_result = calculator.calculate(&calc_inputs)?;

                    ActionResult::CalculatorResult {
                        calculator: calculator_name.clone(),
                        result: format!("{:?}", calc_result), // Convert result to string for simplicity
                    }
                }
                ActionType::Log { message } => {
                    info!(rule_id = rule.id, message = message, "Rule action: Log");
                    ActionResult::Logged { message: message.clone() }
                }
                ActionType::CreateFact { data } => {
                    // For now, just log that we would create a fact
                    info!(rule_id = rule.id, "Rule action: CreateFact");
                    ActionResult::Logged {
                        message: format!("Would create fact with data: {:?}", data),
                    }
                }
                // Phase 3+ features - not implemented in simplified version
                ActionType::Formula { target_field, expression, source_calculator: _ } => {
                    info!(rule_id = rule.id, "Rule action: Formula (not implemented)");
                    ActionResult::Logged {
                        message: format!("Formula: {} = {}", target_field, expression),
                    }
                }
                ActionType::ConditionalSet {
                    target_field,
                    conditions: _,
                    source_calculator: _,
                } => {
                    info!(
                        rule_id = rule.id,
                        "Rule action: ConditionalSet (not implemented)"
                    );
                    ActionResult::Logged { message: format!("ConditionalSet: {}", target_field) }
                }
                ActionType::EmitWindow { window_name, fields: _ } => {
                    info!(
                        rule_id = rule.id,
                        "Rule action: EmitWindow (not implemented)"
                    );
                    ActionResult::Logged { message: format!("EmitWindow: {}", window_name) }
                }
                ActionType::TriggerAlert { alert_type, message, severity: _, metadata: _ } => {
                    info!(
                        rule_id = rule.id,
                        alert_type = alert_type,
                        "Rule action: TriggerAlert"
                    );
                    ActionResult::Logged { message: format!("Alert [{}]: {}", alert_type, message) }
                }
            };
            action_results.push(result);
        }

        Ok(RuleExecutionResult {
            rule_id: rule.id,
            fact_id: fact.id,
            actions_executed: action_results,
        })
    }

    /// Create an alpha node for a condition (simplified)
    fn create_alpha_node_for_condition(&mut self, condition: &Condition) -> Result<()> {
        if let Condition::Simple { field, operator, value } = condition {
            let key = format!("{}_{:?}_{:?}", field, operator, value);

            if !self.alpha_nodes.contains_key(&key) {
                let node_id = self.next_node_id;
                self.next_node_id += 1;
                let alpha_node = AlphaNode::new(node_id, condition.clone());
                self.alpha_nodes.insert(key, alpha_node);
            }
        }
        // For complex conditions, we'd need more sophisticated handling
        // but for now, we'll skip them in this simplified version

        Ok(())
    }

    /// Get statistics about the network
    pub fn get_stats(&self) -> NetworkStats {
        NetworkStats {
            node_count: (self.alpha_nodes.len() + self.beta_nodes.len() + self.terminal_nodes.len())
                as u64,
            memory_usage_bytes: 1024, // Simplified estimate
        }
    }
}

/// Simple statistics for the RETE network
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub node_count: u64,
    pub memory_usage_bytes: u64,
}

impl Default for ReteNetwork {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of executing a rule
#[derive(Debug, Clone)]
pub struct RuleExecutionResult {
    pub rule_id: RuleId,
    pub fact_id: FactId,
    pub actions_executed: Vec<ActionResult>,
}

/// Result of executing an action
#[derive(Debug, Clone)]
pub enum ActionResult {
    FieldSet { field: String, value: crate::types::FactValue },
    CalculatorResult { calculator: String, result: String },
    Logged { message: String },
}
