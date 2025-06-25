//! Calculator DSL for dynamic rule expressions
//!
//! This module implements a domain-specific language for expressing calculations
//! and logic within rule actions. The DSL is designed to be:
//! - Safe: No arbitrary code execution, sandboxed evaluation
//! - Fast: Compiled expressions with optimized evaluation
//! - Simple: Intuitive syntax for business users
//! - Extensible: Plugin system for custom functions

pub mod ast;
pub mod evaluator;
pub mod functions;
pub mod parser;

use crate::types::{Fact, FactData, FactValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Calculator expression that can be evaluated against fact context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatorExpression {
    /// Original expression string
    pub source: String,
    /// Compiled abstract syntax tree
    pub ast: ast::Expression,
    /// Variables referenced in the expression
    pub variables: Vec<String>,
}

/// Context for evaluating calculator expressions
#[derive(Debug, Clone)]
pub struct EvaluationContext<'a> {
    /// Primary fact being processed
    pub current_fact: &'a Fact,
    /// Additional facts available in context
    pub facts: &'a [Fact],
    /// Global variables and constants
    pub globals: HashMap<String, FactValue>,
}

impl<'a> EvaluationContext<'a> {
    /// Create an empty evaluation context for testing
    pub fn empty() -> EvaluationContext<'static> {
        use std::sync::LazyLock;

        static EMPTY_FACT: LazyLock<Fact> =
            LazyLock::new(|| Fact { id: 0, data: FactData { fields: HashMap::new() } });
        static EMPTY_FACTS: &[Fact] = &[];

        EvaluationContext {
            current_fact: &*EMPTY_FACT,
            facts: EMPTY_FACTS,
            globals: HashMap::new(),
        }
    }
}

/// Result of evaluating a calculator expression
#[derive(Debug, Clone, PartialEq)]
pub enum CalculatorResult {
    /// Single value result
    Value(FactValue),
    /// Multiple field updates
    FieldUpdates(HashMap<String, FactValue>),
    /// New fact to be created
    NewFact(FactData),
}

/// Main calculator engine for parsing and evaluating expressions
#[derive(Debug, Default)]
pub struct Calculator {
    /// Built-in function registry
    functions: functions::FunctionRegistry,
    /// Compiled expressions cache
    expression_cache: HashMap<String, CalculatorExpression>,
}

impl Calculator {
    /// Create a new calculator instance
    pub fn new() -> Self {
        Self {
            functions: functions::FunctionRegistry::with_builtins(),
            expression_cache: HashMap::new(),
        }
    }

    /// Parse and compile an expression
    pub fn compile(&mut self, expression: &str) -> Result<CalculatorExpression> {
        // Check cache first
        if let Some(cached) = self.expression_cache.get(expression) {
            return Ok(cached.clone());
        }

        // Parse the expression
        let ast = parser::parse_expression(expression)?;

        // Extract variables
        let variables = ast::extract_variables(&ast);

        let compiled = CalculatorExpression { source: expression.to_string(), ast, variables };

        // Cache the compiled expression
        self.expression_cache.insert(expression.to_string(), compiled.clone());

        Ok(compiled)
    }

    /// Evaluate an expression in the given context
    pub fn evaluate(
        &self,
        expression: &CalculatorExpression,
        context: &EvaluationContext,
    ) -> Result<CalculatorResult> {
        evaluator::evaluate_expression(&expression.ast, context, &self.functions)
    }

    /// Convenience method to compile and evaluate in one step
    pub fn eval(
        &mut self,
        expression: &str,
        context: &EvaluationContext,
    ) -> Result<CalculatorResult> {
        let compiled = self.compile(expression)?;
        self.evaluate(&compiled, context)
    }

    /// Register a custom function
    pub fn register_function<F>(&mut self, name: &str, func: F)
    where
        F: functions::CalculatorFunction + 'static,
    {
        self.functions.register(name, Box::new(func));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FactData, FactValue};
    use std::collections::HashMap;

    fn create_test_fact(id: u64, fields: HashMap<String, FactValue>) -> Fact {
        Fact { id, data: FactData { fields } }
    }

    #[test]
    fn test_calculator_basic_functionality() {
        let mut calculator = Calculator::new();

        // Create test fact
        let mut fields = HashMap::new();
        fields.insert("amount".to_string(), FactValue::Float(100.0));
        fields.insert("rate".to_string(), FactValue::Float(0.15));

        let fact = create_test_fact(1, fields);
        let context =
            EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

        // Test simple arithmetic
        let result = calculator.eval("amount * rate", &context).unwrap();

        match result {
            CalculatorResult::Value(FactValue::Float(value)) => {
                assert!((value - 15.0).abs() < f64::EPSILON);
            }
            _ => panic!("Expected float result"),
        }
    }

    #[test]
    fn test_expression_compilation() {
        let mut calculator = Calculator::new();

        let expression = calculator.compile("amount + tax").unwrap();
        assert_eq!(expression.source, "amount + tax");
        assert_eq!(expression.variables, vec!["amount", "tax"]);
    }
}
