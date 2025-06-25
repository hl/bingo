use bingo_core::types::{Condition, EngineStats, Fact, FactValue, Operator, Rule};
use std::collections::HashMap;
use tracing::{debug, instrument};

pub mod nodes;
pub use nodes::*;

/// The RETE network implementation
pub struct ReteNetwork {
    rules: Vec<Rule>,
    alpha_nodes: HashMap<NodeId, AlphaNode>,
    beta_nodes: HashMap<NodeId, BetaNode>,
    terminal_nodes: HashMap<NodeId, TerminalNode>,
    next_node_id: NodeId,
    fact_memory: Vec<Fact>,
}

impl ReteNetwork {
    /// Create a new RETE network
    #[instrument]
    pub fn new() -> anyhow::Result<Self> {
        debug!("Creating new RETE network");
        Ok(Self {
            rules: Vec::new(),
            alpha_nodes: HashMap::new(),
            beta_nodes: HashMap::new(),
            terminal_nodes: HashMap::new(),
            next_node_id: 1,
            fact_memory: Vec::new(),
        })
    }

    /// Add a rule to the network
    #[instrument(skip(self))]
    pub fn add_rule(&mut self, rule: Rule) -> anyhow::Result<()> {
        debug!(rule_id = rule.id, "Adding rule to RETE network");

        // Compile rule into network nodes
        self.compile_rule(&rule)?;
        self.rules.push(rule);

        Ok(())
    }

    /// Compile a rule into RETE network nodes
    fn compile_rule(&mut self, rule: &Rule) -> anyhow::Result<()> {
        if rule.conditions.is_empty() {
            return Err(anyhow::anyhow!("Rule must have at least one condition"));
        }

        let mut current_nodes = Vec::new();

        // Create alpha nodes for simple conditions
        for condition in &rule.conditions {
            match condition {
                Condition::Simple { .. } => {
                    let node_id = self.next_node_id();
                    let alpha_node = AlphaNode::new(node_id, condition.clone());
                    self.alpha_nodes.insert(node_id, alpha_node);
                    current_nodes.push(node_id);
                }
                Condition::Complex { operator, conditions } => {
                    // Expand complex condition into multiple alpha nodes
                    debug!(
                        rule_id = rule.id,
                        operator = ?operator,
                        condition_count = conditions.len(),
                        "Expanding complex condition into alpha nodes"
                    );

                    for sub_condition in conditions {
                        if let Condition::Simple { .. } = sub_condition {
                            let node_id = self.next_node_id();
                            let alpha_node = AlphaNode::new(node_id, sub_condition.clone());
                            self.alpha_nodes.insert(node_id, alpha_node);
                            current_nodes.push(node_id);
                        } else {
                            // Handle nested complex conditions recursively
                            debug!("Nested complex conditions not yet supported");
                        }
                    }
                }
                Condition::Aggregation(agg_condition) => {
                    // Create a special aggregation node for this condition
                    debug!(
                        rule_id = rule.id,
                        aggregation_type = ?agg_condition.aggregation_type,
                        field = %agg_condition.source_field,
                        "Creating aggregation node"
                    );

                    // For now, create a placeholder alpha node
                    // TODO: Implement proper aggregation node type in Phase 3
                    let node_id = self.next_node_id();
                    let placeholder_condition = Condition::Simple {
                        field: agg_condition.source_field.clone(),
                        operator: Operator::GreaterThan, // Placeholder operator
                        value: FactValue::Integer(0),    // Placeholder value
                    };
                    let alpha_node = AlphaNode::new(node_id, placeholder_condition);
                    self.alpha_nodes.insert(node_id, alpha_node);
                    current_nodes.push(node_id);
                }
                Condition::Stream(_) => {
                    // TODO: Handle stream processing conditions
                    debug!("Stream processing conditions not yet implemented in RETE network");
                }
            }
        }

        // If we have multiple conditions, create join nodes
        while current_nodes.len() > 1 {
            let left = current_nodes.remove(0);
            let right = current_nodes.remove(0);

            let node_id = self.next_node_id();
            let beta_node = BetaNode::new(node_id, Vec::new()); // TODO: Add proper join conditions

            // Link alpha nodes to beta node
            if let Some(alpha_left) = self.alpha_nodes.get_mut(&left) {
                alpha_left.successors.push(node_id);
            }
            if let Some(alpha_right) = self.alpha_nodes.get_mut(&right) {
                alpha_right.successors.push(node_id);
            }

            self.beta_nodes.insert(node_id, beta_node);
            current_nodes.insert(0, node_id);
        }

        // Create terminal node
        let terminal_id = self.next_node_id();
        let terminal_node = TerminalNode::new(terminal_id, rule.id, rule.actions.clone());

        // Link final node to terminal
        if let Some(&final_node) = current_nodes.first() {
            if let Some(alpha_node) = self.alpha_nodes.get_mut(&final_node) {
                alpha_node.successors.push(terminal_id);
            } else if let Some(beta_node) = self.beta_nodes.get_mut(&final_node) {
                beta_node.successors.push(terminal_id);
            }
        }

        self.terminal_nodes.insert(terminal_id, terminal_node);

        debug!(
            rule_id = rule.id,
            alpha_nodes = current_nodes.len(),
            "Rule compiled into network"
        );

        Ok(())
    }

    /// Process facts through the network
    #[instrument(skip(self, facts))]
    pub fn process_facts(&mut self, facts: Vec<Fact>) -> anyhow::Result<Vec<Fact>> {
        debug!(
            fact_count = facts.len(),
            "Processing facts through RETE network"
        );

        // Store facts in memory
        self.fact_memory.extend(facts.clone());

        let mut results = Vec::new();

        // Process each fact through alpha network
        for fact in &facts {
            let mut alpha_tokens: HashMap<NodeId, Vec<Token>> = HashMap::new();

            // Test fact against all alpha nodes
            for (node_id, alpha_node) in &mut self.alpha_nodes {
                let tokens = alpha_node.process_fact(fact);
                if !tokens.is_empty() {
                    alpha_tokens.insert(*node_id, tokens);
                }
            }

            // Propagate tokens through beta network
            let mut beta_tokens: HashMap<NodeId, Vec<Token>> = HashMap::new();

            for (alpha_id, tokens) in alpha_tokens {
                // Find beta nodes that should receive these tokens
                if let Some(alpha_node) = self.alpha_nodes.get(&alpha_id) {
                    for &successor_id in &alpha_node.successors {
                        if let Some(beta_node) = self.beta_nodes.get_mut(&successor_id) {
                            // For simplicity, treat all tokens as left input
                            let new_tokens =
                                beta_node.process_left_tokens(tokens.clone(), &self.fact_memory);
                            if !new_tokens.is_empty() {
                                beta_tokens.entry(successor_id).or_default().extend(new_tokens);
                            }
                        } else if let Some(terminal_node) =
                            self.terminal_nodes.get_mut(&successor_id)
                        {
                            // Direct alpha to terminal connection
                            let mut fact_memory = self.fact_memory.clone();
                            let terminal_results =
                                terminal_node.process_tokens(tokens.clone(), &mut fact_memory)?;
                            results.extend(terminal_results);
                        }
                    }
                }
            }

            // Process beta node outputs to terminals
            for (beta_id, tokens) in beta_tokens {
                if let Some(beta_node) = self.beta_nodes.get(&beta_id) {
                    for &successor_id in &beta_node.successors {
                        if let Some(terminal_node) = self.terminal_nodes.get_mut(&successor_id) {
                            let mut fact_memory = self.fact_memory.clone();
                            let terminal_results =
                                terminal_node.process_tokens(tokens.clone(), &mut fact_memory)?;
                            results.extend(terminal_results);
                        }
                    }
                }
            }
        }

        debug!(
            facts_processed = facts.len(),
            results_generated = results.len(),
            "Fact processing completed"
        );

        Ok(results)
    }

    /// Get network statistics
    #[instrument(skip(self))]
    pub fn get_stats(&self) -> EngineStats {
        EngineStats {
            rule_count: self.rules.len(),
            fact_count: self.fact_memory.len(),
            node_count: self.alpha_nodes.len() + self.beta_nodes.len() + self.terminal_nodes.len(),
            memory_usage_bytes: std::mem::size_of_val(&self.fact_memory)
                + self.alpha_nodes.len() * std::mem::size_of::<AlphaNode>()
                + self.beta_nodes.len() * std::mem::size_of::<BetaNode>()
                + self.terminal_nodes.len() * std::mem::size_of::<TerminalNode>(),
        }
    }

    /// Allocate a new node ID
    fn next_node_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bingo_core::types::{
        Action, ActionType, Condition, FactData, FactValue, LogicalOperator, Operator, Rule,
    };
    use std::collections::HashMap;

    #[test]
    fn test_complex_condition_rete_network() {
        let mut network = ReteNetwork::new().unwrap();

        // Create a rule with complex condition
        let rule = Rule {
            id: 1,
            name: "Complex Condition Test".to_string(),
            conditions: vec![Condition::Complex {
                operator: LogicalOperator::And,
                conditions: vec![
                    Condition::Simple {
                        field: "age".to_string(),
                        operator: Operator::GreaterThan,
                        value: FactValue::Integer(18),
                    },
                    Condition::Simple {
                        field: "department".to_string(),
                        operator: Operator::Equal,
                        value: FactValue::String("Engineering".to_string()),
                    },
                ],
            }],
            actions: vec![Action {
                action_type: ActionType::Log { message: "Complex condition matched".to_string() },
            }],
        };

        // Add rule to network
        network.add_rule(rule).unwrap();

        // Verify that alpha nodes were created for sub-conditions
        assert!(
            network.alpha_nodes.len() >= 2,
            "Should have created alpha nodes for sub-conditions"
        );

        // Create test fact
        let mut fields = HashMap::new();
        fields.insert("age".to_string(), FactValue::Integer(25));
        fields.insert(
            "department".to_string(),
            FactValue::String("Engineering".to_string()),
        );

        let fact = Fact { id: 1, data: FactData { fields } };

        // Process fact
        let _results = network.process_facts(vec![fact]).unwrap();

        // Verify network statistics
        let stats = network.get_stats();
        assert_eq!(stats.rule_count, 1);
        assert!(stats.node_count >= 2, "Should have multiple nodes");
    }

    #[test]
    fn test_aggregation_condition_rete_network() {
        use bingo_core::types::{AggregationCondition, AggregationType};

        let mut network = ReteNetwork::new().unwrap();

        // Create a rule with aggregation condition
        let rule = Rule {
            id: 2,
            name: "Aggregation Condition Test".to_string(),
            conditions: vec![Condition::Aggregation(AggregationCondition {
                aggregation_type: AggregationType::Sum,
                source_field: "salary".to_string(),
                group_by: vec!["department".to_string()],
                having: None,
                alias: "total_salary".to_string(),
                window: None,
            })],
            actions: vec![Action {
                action_type: ActionType::Log {
                    message: "Aggregation condition processed".to_string(),
                },
            }],
        };

        // Add rule to network
        network.add_rule(rule).unwrap();

        // Verify that a placeholder alpha node was created
        assert!(
            network.alpha_nodes.len() >= 1,
            "Should have created at least one alpha node for aggregation"
        );

        let stats = network.get_stats();
        assert_eq!(stats.rule_count, 1);
        assert!(stats.node_count >= 1, "Should have at least one node");
    }
}
