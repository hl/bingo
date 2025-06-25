use bingo_core::types::{Action, ActionType, Condition, Fact, FactId, FactValue, Operator};

/// Unique identifier for nodes in the RETE network
pub type NodeId = u64;

/// Lightweight reference to a fact for token propagation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub fact_ids: Vec<FactId>,
}

impl Token {
    /// Create a new token with a single fact ID
    pub fn new(fact_id: FactId) -> Self {
        Self { fact_ids: vec![fact_id] }
    }

    /// Create a token from multiple fact IDs
    pub fn from_facts(fact_ids: impl Into<Vec<FactId>>) -> Self {
        Self { fact_ids: fact_ids.into() }
    }

    /// Join this token with another fact ID
    pub fn join(&self, fact_id: FactId) -> Self {
        let mut fact_ids = self.fact_ids.clone();
        fact_ids.push(fact_id);
        Self { fact_ids }
    }

    /// Join this token with multiple fact IDs
    pub fn join_many(&self, other_fact_ids: &[FactId]) -> Self {
        let mut fact_ids = self.fact_ids.clone();
        fact_ids.extend_from_slice(other_fact_ids);
        Self { fact_ids }
    }
}

/// Alpha nodes test single fact conditions
#[derive(Debug)]
pub struct AlphaNode {
    pub node_id: NodeId,
    pub condition: Condition,
    pub memory: Vec<FactId>,
    pub successors: Vec<NodeId>,
}

impl AlphaNode {
    pub fn new(node_id: NodeId, condition: Condition) -> Self {
        Self { node_id, condition, memory: Vec::new(), successors: Vec::new() }
    }

    /// Test if a fact matches this alpha node's condition
    pub fn test_fact(&self, fact: &Fact) -> bool {
        match &self.condition {
            Condition::Simple { field, operator, value } => {
                if let Some(fact_value) = fact.data.fields.get(field) {
                    test_condition(fact_value, operator, value)
                } else {
                    false
                }
            }
            Condition::Complex { .. } => {
                // Complex conditions should be handled by a different node type
                // For now, return false as alpha nodes are meant for simple conditions
                false
            }
            Condition::Aggregation(_) => {
                // Aggregation conditions are handled by aggregation nodes
                false
            }
            Condition::Stream(_) => {
                // Stream conditions are handled by stream processing nodes
                // Alpha nodes don't process stream conditions
                false
            }
        }
    }

    /// Process a fact and return tokens if it matches
    pub fn process_fact(&mut self, fact: &Fact) -> Vec<Token> {
        if self.test_fact(fact) {
            self.memory.push(fact.id);
            vec![Token::new(fact.id)]
        } else {
            Vec::new()
        }
    }

    /// Get all facts currently in this node's memory as tokens
    pub fn get_tokens(&self) -> Vec<Token> {
        self.memory.iter().map(|&id| Token::new(id)).collect()
    }
}

/// Beta nodes perform joins between multiple facts
#[derive(Debug)]
pub struct BetaNode {
    pub node_id: NodeId,
    pub left_memory: Vec<Token>,
    pub right_memory: Vec<Token>,
    pub join_conditions: Vec<JoinCondition>,
    pub successors: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct JoinCondition {
    pub left_field: String,
    pub right_field: String,
    pub operator: Operator,
}

impl BetaNode {
    pub fn new(node_id: NodeId, join_conditions: Vec<JoinCondition>) -> Self {
        Self {
            node_id,
            left_memory: Vec::new(),
            right_memory: Vec::new(),
            join_conditions,
            successors: Vec::new(),
        }
    }

    /// Process tokens from left input
    pub fn process_left_tokens(&mut self, tokens: Vec<Token>, facts: &[Fact]) -> Vec<Token> {
        let mut results = Vec::new();

        for token in tokens {
            self.left_memory.push(token.clone());

            // Try to join with existing right memory
            for right_token in &self.right_memory {
                if self.tokens_match(&token, right_token, facts) {
                    results.push(self.join_tokens(&token, right_token));
                }
            }
        }

        results
    }

    /// Process tokens from right input
    pub fn process_right_tokens(&mut self, tokens: Vec<Token>, facts: &[Fact]) -> Vec<Token> {
        let mut results = Vec::new();

        for token in tokens {
            self.right_memory.push(token.clone());

            // Try to join with existing left memory
            for left_token in &self.left_memory {
                if self.tokens_match(left_token, &token, facts) {
                    results.push(self.join_tokens(left_token, &token));
                }
            }
        }

        results
    }

    fn tokens_match(&self, left_token: &Token, right_token: &Token, _facts: &[Fact]) -> bool {
        // For now, implement simple join - just check if we have valid facts
        // TODO: Implement proper join condition checking
        !left_token.fact_ids.is_empty() && !right_token.fact_ids.is_empty()
    }

    fn join_tokens(&self, left_token: &Token, right_token: &Token) -> Token {
        let mut fact_ids = left_token.fact_ids.clone();
        fact_ids.extend(&right_token.fact_ids);
        Token { fact_ids }
    }
}

/// Terminal nodes represent rule conclusions and execute actions
#[derive(Debug)]
pub struct TerminalNode {
    pub node_id: NodeId,
    pub rule_id: u64,
    pub actions: Vec<Action>,
    pub memory: Vec<Token>,
}

impl TerminalNode {
    pub fn new(node_id: NodeId, rule_id: u64, actions: Vec<Action>) -> Self {
        Self { node_id, rule_id, actions, memory: Vec::new() }
    }

    /// Process tokens and execute actions
    pub fn process_tokens(
        &mut self,
        tokens: Vec<Token>,
        facts: &mut Vec<Fact>,
    ) -> anyhow::Result<Vec<Fact>> {
        let mut results = Vec::new();

        for token in tokens {
            self.memory.push(token.clone());

            // Execute actions for this token
            for action in &self.actions {
                match &action.action_type {
                    ActionType::Log { message } => {
                        tracing::info!(rule_id = self.rule_id, message = %message, "Rule fired");
                    }
                    ActionType::SetField { field, value } => {
                        // Find the primary fact in the token and modify it
                        if let Some(&fact_id) = token.fact_ids.first() {
                            if let Some(fact) = facts.iter_mut().find(|f| f.id == fact_id) {
                                fact.data.fields.insert(field.clone(), value.clone());
                                results.push(fact.clone());
                            }
                        }
                    }
                    ActionType::CreateFact { data } => {
                        let new_fact = Fact {
                            id: facts.len() as u64 + 1000, // Simple ID generation
                            data: data.clone(),
                        };
                        results.push(new_fact);
                    }
                    ActionType::Formula { target_field, expression, source_calculator } => {
                        // TODO: Implement formula evaluation (Phase 3)
                        tracing::warn!(
                            rule_id = self.rule_id,
                            target_field = %target_field,
                            formula = %expression,
                            calculator = ?source_calculator,
                            "Formula action not yet implemented"
                        );
                    }
                    ActionType::ConditionalSet { target_field, conditions, source_calculator } => {
                        // TODO: Implement conditional set logic (Phase 3)
                        tracing::warn!(
                            rule_id = self.rule_id,
                            target_field = %target_field,
                            condition_count = conditions.len(),
                            calculator = ?source_calculator,
                            "ConditionalSet action not yet implemented"
                        );
                    }
                    ActionType::EmitWindow { window_name, fields } => {
                        // TODO: Implement window emission for stream processing
                        tracing::info!(
                            rule_id = self.rule_id,
                            window_name = %window_name,
                            field_count = fields.len(),
                            "EmitWindow action not yet implemented"
                        );
                    }
                    ActionType::TriggerAlert { alert_type, message, severity, metadata } => {
                        // TODO: Implement alert triggering for stream processing
                        tracing::warn!(
                            rule_id = self.rule_id,
                            alert_type = %alert_type,
                            message = %message,
                            severity = ?severity,
                            metadata_count = metadata.len(),
                            "Alert triggered: {}", message
                        );
                    }
                    ActionType::CallCalculator { input_mapping, output_field, calculator_name } => {
                        tracing::warn!(
                            rule_id = self.rule_id,
                            calculator_name = %calculator_name,
                            output_field = %output_field,
                            "CallCalculator action not yet implemented in RETE nodes"
                        );
                    }
                }
            }
        }

        Ok(results)
    }
}

/// Test a condition against a fact value using modern pattern matching
fn test_condition(fact_value: &FactValue, operator: &Operator, expected_value: &FactValue) -> bool {
    use {FactValue::*, Operator::*};

    match (fact_value, expected_value, operator) {
        // Integer comparisons
        (Integer(a), Integer(b), op) => match op {
            Equal => a == b,
            NotEqual => a != b,
            GreaterThan => a > b,
            LessThan => a < b,
            GreaterThanOrEqual => a >= b,
            LessThanOrEqual => a <= b,
            Contains => false, // Not applicable for integers
        },

        // Float comparisons with epsilon handling
        (Float(a), Float(b), op) => match op {
            Equal => (a - b).abs() < f64::EPSILON,
            NotEqual => (a - b).abs() >= f64::EPSILON,
            GreaterThan => a > b,
            LessThan => a < b,
            GreaterThanOrEqual => a >= b,
            LessThanOrEqual => a <= b,
            Contains => false, // Not applicable for floats
        },

        // Cross-numeric comparisons (Integer vs Float)
        (Integer(a), Float(_b), _op) => {
            let a_float = *a as f64;
            test_condition(&Float(a_float), operator, expected_value)
        }
        (Float(_a), Integer(b), _op) => {
            let b_float = *b as f64;
            test_condition(fact_value, operator, &Float(b_float))
        }

        // String comparisons
        (String(a), String(b), op) => match op {
            Equal => a == b,
            NotEqual => a != b,
            GreaterThan => a > b,
            LessThan => a < b,
            GreaterThanOrEqual => a >= b,
            LessThanOrEqual => a <= b,
            Contains => a.contains(b),
        },

        // Boolean comparisons
        (Boolean(a), Boolean(b), op) => match op {
            Equal => a == b,
            NotEqual => a != b,
            _ => false, // Other operators not applicable for booleans
        },

        // Type mismatch - return false
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bingo_core::types::{Condition, FactData, FactValue, Operator};
    use std::collections::HashMap;

    #[test]
    fn test_alpha_node_simple_condition() {
        let condition = Condition::Simple {
            field: "age".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(18),
        };

        let mut alpha_node = AlphaNode::new(1, condition);

        let mut fields = HashMap::new();
        fields.insert("age".to_string(), FactValue::Integer(25));

        let fact = Fact { id: 1, data: FactData { fields } };

        let tokens = alpha_node.process_fact(&fact);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].fact_ids, vec![1]);
        assert_eq!(alpha_node.memory.len(), 1);
    }

    #[test]
    fn test_condition_matching() {
        use {FactValue::*, Operator::*};

        // Integer comparisons
        assert!(test_condition(&Integer(25), &GreaterThan, &Integer(18)));
        assert!(!test_condition(&Integer(15), &GreaterThan, &Integer(18)));

        // String operations
        assert!(test_condition(
            &String("hello world".to_string()),
            &Contains,
            &String("world".to_string())
        ));

        // Cross-type numeric comparisons (new in 2024)
        assert!(test_condition(&Integer(25), &GreaterThan, &Float(24.5)));
        assert!(test_condition(&Float(25.5), &GreaterThan, &Integer(25)));

        // Boolean operations
        assert!(test_condition(&Boolean(true), &Equal, &Boolean(true)));
        assert!(!test_condition(&Boolean(true), &Equal, &Boolean(false)));
    }
}
