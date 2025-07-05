use crate::calculator_integration::CalculatorRegistry;
use crate::fact_store::arena_store::ArenaFactStore;
use crate::lazy_aggregation::LazyAggregationManager;
use crate::memory_pools::MemoryPoolManager;
use crate::rete_nodes::{ActionResult, RuleExecutionResult};
use crate::types::{
    ActionType, AlphaNode, BetaNode, Condition, Fact, FactId, FactValue, LogicalOperator, NodeId,
    Operator, Rule, RuleId, TerminalNode,
};
use crate::types::{StreamAggregation, StreamWindowSpec};
use anyhow::Result;
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::{info, instrument};

/// Parse calculator result string into appropriate FactValue
fn parse_calculator_result(result_string: &str) -> FactValue {
    // Try parsing as different types in order of specificity

    // Boolean
    if result_string == "true" {
        return FactValue::Boolean(true);
    }
    if result_string == "false" {
        return FactValue::Boolean(false);
    }

    // Integer
    if let Ok(int_val) = result_string.parse::<i64>() {
        return FactValue::Integer(int_val);
    }

    // Float
    if let Ok(float_val) = result_string.parse::<f64>() {
        return FactValue::Float(float_val);
    }

    // JSON object or array
    if result_string.starts_with('{') || result_string.starts_with('[') {
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(result_string) {
            return convert_json_to_fact_value(json_val);
        }
    }

    // Default to string
    FactValue::String(result_string.to_string())
}

/// Convert serde_json::Value to FactValue
fn convert_json_to_fact_value(json_val: serde_json::Value) -> FactValue {
    match json_val {
        serde_json::Value::Null => FactValue::Null,
        serde_json::Value::Bool(b) => FactValue::Boolean(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                FactValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                FactValue::Float(f)
            } else {
                FactValue::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => FactValue::String(s),
        serde_json::Value::Array(arr) => {
            let fact_array = arr.into_iter().map(convert_json_to_fact_value).collect();
            FactValue::Array(fact_array)
        }
        serde_json::Value::Object(obj) => {
            let fact_object =
                obj.into_iter().map(|(k, v)| (k, convert_json_to_fact_value(v))).collect();
            FactValue::Object(fact_object)
        }
    }
}

/// Simple expression evaluator for formula actions
/// Supports basic arithmetic and field references
fn evaluate_formula_expression(
    expression: &str,
    fact_fields: &HashMap<String, FactValue>,
) -> Result<FactValue> {
    // Very simple implementation for BSSN - handle basic cases
    let expr = expression.trim();

    // Handle field reference (e.g., "amount")
    if let Some(value) = fact_fields.get(expr) {
        return Ok(value.clone());
    }

    // Handle simple arithmetic (e.g., "amount * 1.2", "price + tax")
    if let Some((left, op, right)) = parse_simple_binary_expression(expr) {
        let left_val = evaluate_operand(&left, fact_fields)?;
        let right_val = evaluate_operand(&right, fact_fields)?;

        return evaluate_binary_operation(&left_val, &op, &right_val);
    }

    // Handle literal values
    if let Ok(int_val) = expr.parse::<i64>() {
        return Ok(FactValue::Integer(int_val));
    }

    if let Ok(float_val) = expr.parse::<f64>() {
        return Ok(FactValue::Float(float_val));
    }

    if expr == "true" {
        return Ok(FactValue::Boolean(true));
    }

    if expr == "false" {
        return Ok(FactValue::Boolean(false));
    }

    // Handle string literals (quoted)
    if expr.starts_with('"') && expr.ends_with('"') && expr.len() >= 2 {
        return Ok(FactValue::String(expr[1..expr.len() - 1].to_string()));
    }

    Err(anyhow::anyhow!(
        "Unable to evaluate expression: {}",
        expression
    ))
}

/// Parse simple binary expressions like "a + b", "amount * 1.2"
fn parse_simple_binary_expression(expr: &str) -> Option<(String, String, String)> {
    let operators = vec![" + ", " - ", " * ", " / ", " % "];

    for op in operators {
        if let Some(pos) = expr.find(op) {
            let left = expr[..pos].trim().to_string();
            let right = expr[pos + op.len()..].trim().to_string();
            let operator = op.trim().to_string();
            return Some((left, operator, right));
        }
    }

    None
}

/// Evaluate an operand (field reference or literal)
fn evaluate_operand(operand: &str, fact_fields: &HashMap<String, FactValue>) -> Result<FactValue> {
    // Field reference
    if let Some(value) = fact_fields.get(operand) {
        return Ok(value.clone());
    }

    // Literal values
    if let Ok(int_val) = operand.parse::<i64>() {
        return Ok(FactValue::Integer(int_val));
    }

    if let Ok(float_val) = operand.parse::<f64>() {
        return Ok(FactValue::Float(float_val));
    }

    // String literal
    if operand.starts_with('"') && operand.ends_with('"') && operand.len() >= 2 {
        return Ok(FactValue::String(operand[1..operand.len() - 1].to_string()));
    }

    Err(anyhow::anyhow!("Unable to evaluate operand: {}", operand))
}

/// Evaluate binary operations between FactValues
fn evaluate_binary_operation(left: &FactValue, op: &str, right: &FactValue) -> Result<FactValue> {
    use FactValue::*;

    match (left, right) {
        (Integer(a), Integer(b)) => match op {
            "+" => Ok(Integer(a + b)),
            "-" => Ok(Integer(a - b)),
            "*" => Ok(Integer(a * b)),
            "/" => {
                if *b == 0 {
                    Err(anyhow::anyhow!("Division by zero"))
                } else {
                    Ok(Float(*a as f64 / *b as f64))
                }
            }
            "%" => {
                if *b == 0 {
                    Err(anyhow::anyhow!("Modulo by zero"))
                } else {
                    Ok(Integer(a % b))
                }
            }
            _ => Err(anyhow::anyhow!("Unsupported operator: {}", op)),
        },
        (Float(_), Float(_)) | (Integer(_), Float(_)) | (Float(_), Integer(_)) => {
            let a_val = match left {
                Integer(i) => *i as f64,
                Float(f) => *f,
                _ => unreachable!(),
            };
            let b_val = match right {
                Integer(i) => *i as f64,
                Float(f) => *f,
                _ => unreachable!(),
            };

            match op {
                "+" => Ok(Float(a_val + b_val)),
                "-" => Ok(Float(a_val - b_val)),
                "*" => Ok(Float(a_val * b_val)),
                "/" => {
                    if b_val == 0.0 {
                        Err(anyhow::anyhow!("Division by zero"))
                    } else {
                        Ok(Float(a_val / b_val))
                    }
                }
                "%" => {
                    if b_val == 0.0 {
                        Err(anyhow::anyhow!("Modulo by zero"))
                    } else {
                        Ok(Float(a_val % b_val))
                    }
                }
                _ => Err(anyhow::anyhow!("Unsupported operator: {}", op)),
            }
        }
        (String(a), String(b)) => match op {
            "+" => Ok(String(format!("{a}{b}"))),
            _ => Err(anyhow::anyhow!("Unsupported string operator: {}", op)),
        },
        _ => Err(anyhow::anyhow!(
            "Incompatible types for operation: {:?} {} {:?}",
            left,
            op,
            right
        )),
    }
}

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
    /// Next available fact ID for created facts
    next_fact_id: FactId,

    /// Simple partial-match cache: (rule_id, fact_id) pairs already evaluated.
    /// Size is capped to keep memory bounded.
    partial_match_cache: HashSet<(RuleId, FactId)>,
    /// Facts created during rule execution
    created_facts: Vec<Fact>,
    /// Memory pool manager for efficient allocation
    memory_pools: MemoryPoolManager,
    /// Lazy aggregation manager for performance optimization
    lazy_aggregation_manager: LazyAggregationManager,
}

impl ReteNetwork {
    /// Create a new RETE network
    #[instrument]
    pub fn new() -> Self {
        info!("Creating new RETE network");
        let memory_pools = MemoryPoolManager::new();
        #[allow(clippy::arc_with_non_send_sync)]
        let lazy_aggregation_manager =
            LazyAggregationManager::new(std::sync::Arc::new(memory_pools.clone()));

        Self {
            alpha_nodes: HashMap::new(),
            beta_nodes: HashMap::new(),
            terminal_nodes: HashMap::new(),
            rules: HashMap::new(),
            next_node_id: 1,
            next_fact_id: 1_000_000, // Start created facts at a high ID to avoid conflicts
            partial_match_cache: HashSet::with_capacity(1024),
            created_facts: Vec::new(),
            memory_pools,
            lazy_aggregation_manager,
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
    #[instrument(skip(self, fact_store, calculator_registry))]
    pub fn process_facts(
        &mut self,
        facts: &[Fact],
        fact_store: &mut ArenaFactStore,
        calculator_registry: &CalculatorRegistry,
    ) -> Result<Vec<RuleExecutionResult>> {
        let mut results = self.memory_pools.rule_execution_results.get();

        // For each fact, check which rules match
        for fact in facts {
            let mut matching_rules = self.memory_pools.rule_id_vecs.get();
            matching_rules = self.find_matching_rules_pooled(fact, fact_store, matching_rules)?;

            for rule_id in &matching_rules {
                // Skip if we have recently executed this rule for this fact
                if self.partial_match_cache.contains(&(*rule_id, fact.id)) {
                    continue;
                }

                if let Some(rule) = self.rules.get(rule_id).cloned() {
                    let result = self.execute_rule(&rule, fact, fact_store, calculator_registry)?;
                    results.push(result);

                    // Update cache with basic eviction (keep <=10_000 entries)
                    if self.partial_match_cache.len() > 10_000 {
                        // rudimentary eviction: clear half
                        let remove_count = self.partial_match_cache.len() / 2;
                        for key in self
                            .partial_match_cache
                            .iter()
                            .take(remove_count)
                            .cloned()
                            .collect::<Vec<_>>()
                        {
                            self.partial_match_cache.remove(&key);
                        }
                    }
                    self.partial_match_cache.insert((*rule_id, fact.id));
                }
            }

            // Return rule ID vector to pool
            self.memory_pools.rule_id_vecs.return_vec(matching_rules);
        }

        Ok(results)
    }

    // Removed unused `find_matching_rules` helper to eliminate dead code.

    /// Find rules that match a given fact using pooled vector
    fn find_matching_rules_pooled(
        &self,
        fact: &Fact,
        fact_store: &ArenaFactStore,
        mut matching_rules: Vec<RuleId>,
    ) -> Result<Vec<RuleId>> {
        for (rule_id, rule) in &self.rules {
            if self.fact_matches_rule(fact, rule, fact_store)? {
                matching_rules.push(*rule_id);
            }
        }

        Ok(matching_rules)
    }

    /// Check if a fact matches all conditions of a rule
    fn fact_matches_rule(
        &self,
        fact: &Fact,
        rule: &Rule,
        fact_store: &ArenaFactStore,
    ) -> Result<bool> {
        if rule.conditions.is_empty() {
            return Ok(false);
        }

        // Check first condition against current fact
        if !self.fact_matches_condition(fact, &rule.conditions[0], fact_store)? {
            return Ok(false);
        }

        // Remaining conditions can be satisfied by ANY fact in store (including current)
        for cond in rule.conditions.iter().skip(1) {
            // fast path: current fact matches
            if self.fact_matches_condition(fact, cond, fact_store)? {
                continue;
            }

            // Fast path using field index when condition is a simple equality
            let mut satisfied = false;
            if let Condition::Simple { field, operator: Operator::Equal, value } = cond {
                // Use indexed lookup if possible
                let candidates = fact_store.find_by_field(field, value);
                if !candidates.is_empty() {
                    satisfied = true;
                }
            } else {
                // Fallback to linear scan across all facts
                for idx in 0..fact_store.len() as u64 {
                    if let Some(_other) = fact_store.get_fact(idx) {
                        if self.fact_matches_condition(_other, cond, fact_store)? {
                            satisfied = true;
                            break;
                        }
                    }
                }
            }

            if !satisfied {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if a fact matches a specific condition
    fn fact_matches_condition(
        &self,
        fact: &Fact,
        condition: &Condition,
        fact_store: &ArenaFactStore,
    ) -> Result<bool> {
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
            Condition::Complex { operator, conditions } => {
                if conditions.is_empty() {
                    return Ok(false);
                }

                let mut evaluations = Vec::with_capacity(conditions.len());
                for sub in conditions {
                    evaluations.push(self.fact_matches_condition(fact, sub, fact_store)?);
                }

                let result = match operator {
                    LogicalOperator::And => evaluations.iter().all(|v| *v),
                    LogicalOperator::Or => evaluations.iter().any(|v| *v),
                    LogicalOperator::Not => !evaluations[0],
                };
                Ok(result)
            }
            Condition::Aggregation(agg) => {
                // Register the aggregation with the lazy aggregation manager so that statistics
                // are updated even when we choose the simplified eager evaluation path below.
                use std::sync::Arc;
                let _ = self.lazy_aggregation_manager.get_or_create_aggregation(
                    agg.clone(),
                    fact.clone(),
                    Arc::new(ArenaFactStore::new()),
                );
                // Use eager evaluation for aggregation conditions
                use crate::types::{AggregationType::*, FactValue};

                // Determine group key based on group_by fields of CURRENT fact
                let group_match = |candidate: &Fact| {
                    for gb_field in &agg.group_by {
                        let current_val = fact.data.fields.get(gb_field);
                        if current_val != candidate.data.fields.get(gb_field) {
                            return false;
                        }
                    }
                    true
                };

                // Helper to push numeric value from a fact
                let collect_val = |f: &Fact, vec: &mut Vec<f64>| {
                    if let Some(val) = f.data.fields.get(&agg.source_field) {
                        if let Some(n) = val.as_f64() {
                            vec.push(n);
                        }
                    }
                };

                // Determine candidate facts (windowed or all)
                let candidates: Vec<&Fact> = if let Some(window) = &agg.window {
                    match window {
                        crate::types::AggregationWindow::Time { duration_ms } => {
                            let start = fact.timestamp
                                - chrono::Duration::milliseconds(*duration_ms as i64);
                            let end = fact.timestamp;
                            fact_store.facts_in_time_range(start, end)
                        }
                        crate::types::AggregationWindow::Sliding { size } => {
                            // Get last `size` facts (by timestamp) in same group.
                            // Simple implementation: iterate, collect candidates, sort, take last size.
                            let mut all: Vec<&Fact> = fact_store.iter().collect();
                            all.sort_by_key(|f| f.timestamp);
                            if *size >= all.len() {
                                all
                            } else {
                                all.split_off(all.len() - size)
                            }
                        }
                        crate::types::AggregationWindow::Tumbling { size } => {
                            // Determine window index based on trigger fact position in sorted list.
                            let mut all: Vec<&Fact> = fact_store.iter().collect();
                            all.sort_by_key(|f| f.timestamp);
                            if all.is_empty() {
                                vec![]
                            } else {
                                let idx = all.iter().position(|f| f.id == fact.id).unwrap_or(0);
                                let window_start = (idx / size) * size;
                                all.into_iter().skip(window_start).take(*size).collect()
                            }
                        }
                        crate::types::AggregationWindow::Session { timeout_ms } => {
                            // Use session window semantics for aggregation
                            self.find_session_window_facts(fact, *timeout_ms, fact_store)
                        }
                    }
                } else {
                    fact_store.iter().collect()
                };

                let mut nums = self.memory_pools.numeric_vecs.get();
                for f in candidates {
                    if !group_match(f) {
                        continue;
                    }
                    collect_val(f, &mut nums);
                }

                if nums.is_empty() {
                    return Ok(false);
                }

                let aggregate_value = match agg.aggregation_type {
                    Count => FactValue::Integer(nums.len() as i64),
                    Sum => FactValue::Float(nums.iter().sum()),
                    Average => FactValue::Float(nums.iter().sum::<f64>() / nums.len() as f64),
                    Min => {
                        let min_val = nums.iter().cloned().fold(f64::INFINITY, f64::min);
                        FactValue::Float(min_val)
                    }
                    Max => {
                        let max_val = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                        FactValue::Float(max_val)
                    }
                    StandardDeviation => {
                        // population standard deviation
                        let mean = nums.iter().sum::<f64>() / nums.len() as f64;
                        let variance = nums.iter().map(|v| (*v - mean).powi(2)).sum::<f64>()
                            / nums.len() as f64;
                        FactValue::Float(variance.sqrt())
                    }
                    Percentile(p) => {
                        let mut sorted = nums.clone();
                        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        let rank_f = (p / 100.0) * (sorted.len() as f64 - 1.0);
                        let lower = rank_f.floor() as usize;
                        let upper = rank_f.ceil() as usize;
                        let interp = if upper == lower {
                            sorted[lower]
                        } else {
                            let w = rank_f - lower as f64;
                            sorted[lower] * (1.0 - w) + sorted[upper] * w
                        };
                        FactValue::Float(interp)
                    }
                };

                // If there is a HAVING clause, evaluate it against a synthetic fact
                let result = if let Some(having_cond) = &agg.having {
                    let mut map = self.memory_pools.fact_field_maps.get();
                    map.insert(agg.alias.clone(), aggregate_value.clone());
                    let synthetic_fact = Fact {
                        id: 0,
                        external_id: None,
                        timestamp: chrono::Utc::now(),
                        data: crate::types::FactData { fields: map },
                    };
                    let result =
                        self.fact_matches_condition(&synthetic_fact, having_cond, fact_store);
                    self.memory_pools.fact_field_maps.return_map(synthetic_fact.data.fields);
                    result
                } else {
                    // Without HAVING, simple truthy check (non-zero / non-empty)
                    Ok(match &aggregate_value {
                        FactValue::Integer(i) => *i != 0,
                        FactValue::Float(f) => *f != 0.0,
                        _ => true,
                    })
                };

                // Return numeric vector to pool
                self.memory_pools.numeric_vecs.return_vec(nums);
                result
            }
            Condition::Stream(stream) => {
                // Only Count aggregation with optional filter and time windows supported.

                // Build candidate list based on window spec
                let mut candidates: Vec<&Fact> = match &stream.window_spec {
                    StreamWindowSpec::Tumbling { duration_ms } => {
                        let start =
                            fact.timestamp - chrono::Duration::milliseconds(*duration_ms as i64);
                        fact_store.facts_in_time_range(start, fact.timestamp)
                    }
                    StreamWindowSpec::Sliding { size_ms, .. } => {
                        let start =
                            fact.timestamp - chrono::Duration::milliseconds(*size_ms as i64);
                        fact_store.facts_in_time_range(start, fact.timestamp)
                    }
                    StreamWindowSpec::Session { gap_timeout_ms } => {
                        // For session windows, we need to find all facts that form a continuous session
                        // with gaps no larger than gap_timeout_ms ending with the current fact
                        self.find_session_window_facts(fact, *gap_timeout_ms, fact_store)
                    }
                    _ => fact_store.iter().collect(),
                };

                // Apply filter if present
                if let Some(filter_cond) = &stream.filter {
                    candidates.retain(|f| {
                        self.fact_matches_condition(f, filter_cond, fact_store).unwrap_or(false)
                    });
                }

                // Currently support only Count aggregation
                let aggregate_value = match &stream.aggregation {
                    StreamAggregation::Count => {
                        crate::types::FactValue::Integer(candidates.len() as i64)
                    }
                    _ => return Ok(false),
                };

                // HAVING clause processing
                if let Some(having_cond) = &stream.having {
                    let mut map = self.memory_pools.fact_field_maps.get();
                    map.insert(stream.alias.clone(), aggregate_value.clone());
                    let synthetic = Fact {
                        id: 0,
                        external_id: None,
                        timestamp: chrono::Utc::now(),
                        data: crate::types::FactData { fields: map },
                    };
                    let result = self.fact_matches_condition(&synthetic, having_cond, fact_store);
                    self.memory_pools.fact_field_maps.return_map(synthetic.data.fields);
                    return result;
                }

                Ok(match aggregate_value {
                    crate::types::FactValue::Integer(i) => i > 0,
                    _ => true,
                })
            }
        }
    }

    /// Generate a new fact ID
    fn generate_fact_id(&mut self) -> FactId {
        let id = self.next_fact_id;
        self.next_fact_id += 1;
        id
    }

    /// Get and clear created facts
    pub fn take_created_facts(&mut self) -> Vec<Fact> {
        std::mem::take(&mut self.created_facts)
    }

    /// Execute a rule with the given fact
    fn execute_rule(
        &mut self,
        rule: &Rule,
        fact: &Fact,
        fact_store: &mut ArenaFactStore,
        calculator_registry: &CalculatorRegistry,
    ) -> Result<RuleExecutionResult> {
        let mut action_results = Vec::new();

        for action in &rule.actions {
            let result = match &action.action_type {
                ActionType::SetField { field, value } => ActionResult::FieldSet {
                    fact_id: fact.id,
                    field: field.clone(),
                    value: value.clone(),
                },
                ActionType::CallCalculator { calculator_name, input_mapping, output_field } => {
                    // Execute calculator using the registry
                    match calculator_registry.execute(
                        calculator_name,
                        input_mapping,
                        &fact.data.fields,
                    ) {
                        Ok(result_string) => {
                            // Parse the result string and convert to appropriate FactValue
                            let result_value = parse_calculator_result(&result_string);

                            info!(
                                rule_id = rule.id,
                                calculator = calculator_name,
                                output_field = output_field,
                                result = %result_string,
                                "Calculator executed successfully"
                            );

                            ActionResult::CalculatorResult {
                                calculator: calculator_name.clone(),
                                result: result_string,
                                output_field: output_field.clone(),
                                parsed_value: result_value,
                            }
                        }
                        Err(e) => {
                            info!(
                                rule_id = rule.id,
                                calculator = calculator_name,
                                error = %e,
                                "Calculator execution failed"
                            );
                            ActionResult::Logged {
                                message: format!("Calculator '{calculator_name}' failed: {e}"),
                            }
                        }
                    }
                }
                ActionType::Log { message } => {
                    info!(rule_id = rule.id, message = message, "Rule action: Log");
                    ActionResult::Logged { message: message.clone() }
                }
                ActionType::CreateFact { data } => {
                    // Actually create a new fact
                    let new_fact_id = self.generate_fact_id();
                    let new_fact = Fact {
                        id: new_fact_id,
                        external_id: None,
                        timestamp: chrono::Utc::now(),
                        data: data.clone(),
                    };

                    // Store the created fact for potential processing in subsequent stages
                    self.created_facts.push(new_fact.clone());

                    info!(
                        rule_id = rule.id,
                        new_fact_id = new_fact_id,
                        "Rule action: CreateFact - fact created"
                    );

                    ActionResult::FactCreated { fact_id: new_fact_id, fact_data: data.clone() }
                }
                ActionType::TriggerAlert { alert_type, message, severity: _, metadata: _ } => {
                    info!(
                        rule_id = rule.id,
                        alert_type = alert_type,
                        "Rule action: TriggerAlert"
                    );
                    ActionResult::Logged { message: format!("Alert [{alert_type}]: {message}") }
                }
                ActionType::Formula { expression, output_field } => {
                    // Evaluate the formula expression
                    match evaluate_formula_expression(expression, &fact.data.fields) {
                        Ok(result_value) => {
                            info!(
                                rule_id = rule.id,
                                expression = expression,
                                output_field = output_field,
                                result = ?result_value,
                                "Formula evaluated successfully"
                            );

                            ActionResult::CalculatorResult {
                                calculator: "formula".to_string(),
                                result: result_value.to_string(),
                                output_field: output_field.clone(),
                                parsed_value: result_value,
                            }
                        }
                        Err(e) => {
                            info!(
                                rule_id = rule.id,
                                expression = expression,
                                error = %e,
                                "Formula evaluation failed"
                            );
                            ActionResult::Logged {
                                message: format!("Formula '{expression}' failed: {e}"),
                            }
                        }
                    }
                }
                ActionType::UpdateFact { fact_id_field, updates } => {
                    // Get the fact ID from the current fact's field
                    if let Some(fact_id_value) = fact.data.fields.get(fact_id_field) {
                        if let Some(target_fact_id) = fact_id_value.as_integer() {
                            info!(
                                rule_id = rule.id,
                                target_fact_id = target_fact_id,
                                fact_id_field = fact_id_field,
                                "Rule action: UpdateFact"
                            );

                            // Actually update the fact in the fact store
                            if fact_store.update_fact(target_fact_id as u64, updates.clone()) {
                                let updated_fields: Vec<String> = updates.keys().cloned().collect();
                                info!(
                                    rule_id = rule.id,
                                    target_fact_id = target_fact_id,
                                    updated_fields = ?updated_fields,
                                    "Successfully updated fact"
                                );
                                ActionResult::FactUpdated {
                                    fact_id: target_fact_id as u64,
                                    updated_fields,
                                }
                            } else {
                                ActionResult::Logged {
                                    message: format!(
                                        "UpdateFact failed: fact with ID {target_fact_id} not found"
                                    ),
                                }
                            }
                        } else {
                            ActionResult::Logged {
                                message: format!(
                                    "UpdateFact failed: field '{fact_id_field}' is not an integer"
                                ),
                            }
                        }
                    } else {
                        ActionResult::Logged {
                            message: format!(
                                "UpdateFact failed: field '{fact_id_field}' not found"
                            ),
                        }
                    }
                }
                ActionType::DeleteFact { fact_id_field } => {
                    // Get the fact ID from the current fact's field
                    if let Some(fact_id_value) = fact.data.fields.get(fact_id_field) {
                        if let Some(target_fact_id) = fact_id_value.as_integer() {
                            info!(
                                rule_id = rule.id,
                                target_fact_id = target_fact_id,
                                fact_id_field = fact_id_field,
                                "Rule action: DeleteFact"
                            );

                            // Actually delete the fact from the fact store
                            if fact_store.delete_fact(target_fact_id as u64) {
                                info!(
                                    rule_id = rule.id,
                                    target_fact_id = target_fact_id,
                                    "Successfully deleted fact"
                                );
                                ActionResult::FactDeleted { fact_id: target_fact_id as u64 }
                            } else {
                                ActionResult::Logged {
                                    message: format!(
                                        "DeleteFact failed: fact with ID {target_fact_id} not found"
                                    ),
                                }
                            }
                        } else {
                            ActionResult::Logged {
                                message: format!(
                                    "DeleteFact failed: field '{fact_id_field}' is not an integer"
                                ),
                            }
                        }
                    } else {
                        ActionResult::Logged {
                            message: format!(
                                "DeleteFact failed: field '{fact_id_field}' not found"
                            ),
                        }
                    }
                }
                ActionType::IncrementField { field, increment } => {
                    // Get the current value of the field
                    if let Some(current_value) = fact.data.fields.get(field) {
                        match (current_value, increment) {
                            (FactValue::Integer(current), FactValue::Integer(inc)) => {
                                let new_value = FactValue::Integer(current + inc);
                                info!(
                                    rule_id = rule.id,
                                    field = field,
                                    old_value = current,
                                    increment = inc,
                                    new_value = current + inc,
                                    "Rule action: IncrementField (Integer)"
                                );
                                ActionResult::FieldIncremented {
                                    fact_id: fact.id,
                                    field: field.clone(),
                                    old_value: current_value.clone(),
                                    new_value,
                                }
                            }
                            (FactValue::Float(current), FactValue::Float(inc)) => {
                                let new_value = FactValue::Float(current + inc);
                                info!(
                                    rule_id = rule.id,
                                    field = field,
                                    old_value = current,
                                    increment = inc,
                                    new_value = current + inc,
                                    "Rule action: IncrementField (Float)"
                                );
                                ActionResult::FieldIncremented {
                                    fact_id: fact.id,
                                    field: field.clone(),
                                    old_value: current_value.clone(),
                                    new_value,
                                }
                            }
                            (FactValue::Integer(current), FactValue::Float(inc)) => {
                                let new_value = FactValue::Float(*current as f64 + inc);
                                info!(
                                    rule_id = rule.id,
                                    field = field,
                                    old_value = current,
                                    increment = inc,
                                    new_value = *current as f64 + inc,
                                    "Rule action: IncrementField (Mixed)"
                                );
                                ActionResult::FieldIncremented {
                                    fact_id: fact.id,
                                    field: field.clone(),
                                    old_value: current_value.clone(),
                                    new_value,
                                }
                            }
                            (FactValue::Float(current), FactValue::Integer(inc)) => {
                                let new_value = FactValue::Float(current + *inc as f64);
                                info!(
                                    rule_id = rule.id,
                                    field = field,
                                    old_value = current,
                                    increment = inc,
                                    new_value = current + *inc as f64,
                                    "Rule action: IncrementField (Mixed)"
                                );
                                ActionResult::FieldIncremented {
                                    fact_id: fact.id,
                                    field: field.clone(),
                                    old_value: current_value.clone(),
                                    new_value,
                                }
                            }
                            _ => ActionResult::Logged {
                                message: format!(
                                    "IncrementField failed: incompatible types for field '{field}'"
                                ),
                            },
                        }
                    } else {
                        // Field doesn't exist, treat as starting from 0
                        info!(
                            rule_id = rule.id,
                            field = field,
                            increment = ?increment,
                            "Rule action: IncrementField (new field)"
                        );
                        ActionResult::FieldIncremented {
                            fact_id: fact.id,
                            field: field.clone(),
                            old_value: FactValue::Integer(0),
                            new_value: increment.clone(),
                        }
                    }
                }
                ActionType::AppendToArray { field, value } => {
                    // Get the current array value or create a new one
                    if let Some(current_value) = fact.data.fields.get(field) {
                        if let FactValue::Array(mut current_array) = current_value.clone() {
                            current_array.push(value.clone());
                            let new_length = current_array.len();
                            info!(
                                rule_id = rule.id,
                                field = field,
                                appended_value = ?value,
                                new_length = new_length,
                                "Rule action: AppendToArray"
                            );
                            ActionResult::ArrayAppended {
                                fact_id: fact.id,
                                field: field.clone(),
                                appended_value: value.clone(),
                                new_length,
                            }
                        } else {
                            ActionResult::Logged {
                                message: format!(
                                    "AppendToArray failed: field '{field}' is not an array"
                                ),
                            }
                        }
                    } else {
                        // Field doesn't exist, create new array with the value
                        info!(
                            rule_id = rule.id,
                            field = field,
                            appended_value = ?value,
                            "Rule action: AppendToArray (new array)"
                        );
                        ActionResult::ArrayAppended {
                            fact_id: fact.id,
                            field: field.clone(),
                            appended_value: value.clone(),
                            new_length: 1,
                        }
                    }
                }
                ActionType::SendNotification {
                    recipient,
                    subject,
                    message: _,
                    notification_type,
                    metadata: _,
                } => {
                    info!(
                        rule_id = rule.id,
                        recipient = recipient,
                        subject = subject,
                        notification_type = ?notification_type,
                        "Rule action: SendNotification"
                    );
                    ActionResult::NotificationSent {
                        recipient: recipient.clone(),
                        notification_type: notification_type.clone(),
                        subject: subject.clone(),
                    }
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
            let key = format!("{field}_{operator:?}_{value:?}");

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

    /// Remove a rule from the network
    pub fn remove_rule(&mut self, rule_id: RuleId) -> Result<()> {
        // Remove from rules map
        self.rules.remove(&rule_id);

        // Remove terminal node
        self.terminal_nodes.remove(&rule_id);

        // Note: Alpha/beta node cleanup could be implemented for memory optimization
        // but is not required for correctness in this stateless architecture
        Ok(())
    }

    /// Get created facts
    /// In the stateless architecture, facts are processed immediately
    pub fn get_created_facts(&self) -> &[crate::types::Fact] {
        // Facts are processed immediately in this stateless implementation
        &[]
    }

    /// Clear created facts
    /// In the stateless architecture, this is a no-op
    pub fn clear_created_facts(&mut self) {
        // No-op in stateless implementation - facts are processed immediately
    }

    /// Get action result pool statistics (simplified)
    pub fn get_action_result_pool_stats(&self) -> (usize, usize) {
        // Return (pool_size, active_items) - simplified implementation
        (0, 0)
    }

    /// Get comprehensive memory pool statistics
    pub fn get_memory_pool_stats(&self) -> crate::memory_pools::MemoryPoolStats {
        self.memory_pools.get_comprehensive_stats()
    }

    /// Get memory pool efficiency percentage
    pub fn get_memory_pool_efficiency(&self) -> f64 {
        self.memory_pools.overall_efficiency()
    }

    /// Get lazy aggregation statistics
    pub fn get_lazy_aggregation_stats(
        &self,
    ) -> crate::lazy_aggregation::LazyAggregationManagerStats {
        self.lazy_aggregation_manager.get_stats()
    }

    /// Invalidate all lazy aggregation caches (when fact store changes)
    pub fn invalidate_lazy_aggregation_caches(&self) {
        self.lazy_aggregation_manager.invalidate_all_caches();
    }

    /// Clean up inactive lazy aggregations to free memory
    pub fn cleanup_lazy_aggregations(&self) {
        self.lazy_aggregation_manager.cleanup_inactive_aggregations();
    }

    /// Find all facts that belong to the same session window as the given fact
    /// using session window semantics with gap timeout
    fn find_session_window_facts<'a>(
        &self,
        current_fact: &'a Fact,
        gap_timeout_ms: u64,
        fact_store: &'a ArenaFactStore,
    ) -> Vec<&'a Fact> {
        let mut session_facts = Vec::new();
        let gap_duration = chrono::Duration::milliseconds(gap_timeout_ms as i64);

        // Start by including the current fact
        session_facts.push(current_fact);

        // Collect all facts and sort by timestamp for session analysis
        let mut all_facts: Vec<&Fact> = fact_store.iter().collect();
        all_facts.sort_by_key(|f| f.timestamp);

        // Find the session boundaries around the current fact
        let current_timestamp = current_fact.timestamp;

        // Look backwards from current fact to find session start
        let mut session_start = current_timestamp;
        for fact in all_facts.iter().rev() {
            if fact.timestamp <= current_timestamp {
                // Check if this fact can extend the session backwards
                if current_timestamp - fact.timestamp <= gap_duration {
                    session_start = fact.timestamp;
                    if fact.id != current_fact.id {
                        session_facts.push(fact);
                    }
                } else {
                    // Gap too large, session starts after this fact
                    break;
                }
            }
        }

        // Look forwards from current fact to find session end
        let mut session_end = current_timestamp;
        let mut last_fact_time = current_timestamp;

        for fact in all_facts.iter() {
            if fact.timestamp >= current_timestamp && fact.id != current_fact.id {
                // Check if this fact extends the session forwards
                if fact.timestamp - last_fact_time <= gap_duration {
                    session_end = fact.timestamp;
                    last_fact_time = fact.timestamp;
                    session_facts.push(fact);
                } else {
                    // Gap too large, session ends before this fact
                    break;
                }
            }
        }

        // Now collect all facts within the session boundaries that form a continuous session
        let mut final_session_facts = Vec::new();
        let session_facts_sorted: Vec<&Fact> = session_facts
            .into_iter()
            .filter(|f| f.timestamp >= session_start && f.timestamp <= session_end)
            .collect();

        // Verify session continuity - ensure no gaps larger than timeout
        if !session_facts_sorted.is_empty() {
            let mut sorted_facts = session_facts_sorted;
            sorted_facts.sort_by_key(|f| f.timestamp);

            final_session_facts.push(sorted_facts[0]);
            let mut last_time = sorted_facts[0].timestamp;

            for fact in sorted_facts.iter().skip(1) {
                if fact.timestamp - last_time <= gap_duration {
                    final_session_facts.push(fact);
                    last_time = fact.timestamp;
                } else {
                    // Gap too large, start new session
                    break;
                }
            }
        }

        final_session_facts
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
