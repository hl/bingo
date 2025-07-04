//! Mathematical expression engine for payroll calculations
//!
//! This module provides a simple expression evaluator specifically designed
//! for payroll calculations like hour calculations, pay rate computations,
//! and gross pay calculations.

use super::types::{Fact, FactValue, FactData};
use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use std::collections::HashMap;
use tracing::instrument;

/// Expression evaluator for payroll calculations
pub struct PayrollExpressionEvaluator {
    /// Cache for compiled expressions
    expression_cache: HashMap<String, CompiledExpression>,
}

impl PayrollExpressionEvaluator {
    /// Create a new expression evaluator
    pub fn new() -> Self {
        Self {
            expression_cache: HashMap::new(),
        }
    }

    /// Evaluate a payroll expression against fact data
    #[instrument(skip(self, fact))]
    pub fn evaluate_expression(
        &mut self,
        expression: &str,
        fact: &Fact,
        context: Option<&HashMap<String, FactValue>>,
    ) -> Result<FactValue> {
        // Check cache first
        let compiled = if let Some(cached) = self.expression_cache.get(expression) {
            cached.clone()
        } else {
            let compiled = self.compile_expression(expression)?;
            self.expression_cache.insert(expression.to_string(), compiled.clone());
            compiled
        };

        self.execute_compiled_expression(&compiled, fact, context)
    }

    /// Compile an expression into an executable form
    fn compile_expression(&self, expression: &str) -> Result<CompiledExpression> {
        // Parse the expression and create a compiled form
        let tokens = self.tokenize(expression)?;
        let ast = self.parse_tokens(tokens)?;
        
        Ok(CompiledExpression {
            original: expression.to_string(),
            ast,
        })
    }

    /// Tokenize an expression string
    fn tokenize(&self, expression: &str) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut chars = expression.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                ' ' | '\t' | '\n' => {
                    if !current_token.is_empty() {
                        tokens.push(self.parse_token(&current_token)?);
                        current_token.clear();
                    }
                }
                '+' | '-' | '*' | '/' | '(' | ')' | ',' => {
                    if !current_token.is_empty() {
                        tokens.push(self.parse_token(&current_token)?);
                        current_token.clear();
                    }
                    tokens.push(Token::Operator(ch));
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        if !current_token.is_empty() {
            tokens.push(self.parse_token(&current_token)?);
        }

        Ok(tokens)
    }

    /// Parse a token string into a Token
    fn parse_token(&self, token_str: &str) -> Result<Token> {
        // Try to parse as number
        if let Ok(int_val) = token_str.parse::<i64>() {
            return Ok(Token::Number(int_val as f64));
        }
        
        if let Ok(float_val) = token_str.parse::<f64>() {
            return Ok(Token::Number(float_val));
        }

        // Check for built-in functions
        if token_str == "hours_between" || token_str == "minutes_between" || token_str == "minutes_to_hours" || token_str == "multiply" {
            return Ok(Token::Function(token_str.to_string()));
        }

        // Otherwise, it's a field reference
        Ok(Token::Field(token_str.to_string()))
    }

    /// Parse tokens into an AST
    fn parse_tokens(&self, tokens: Vec<Token>) -> Result<ExpressionNode> {
        let mut parser = ExpressionParser::new(tokens);
        parser.parse_expression()
    }

    /// Execute a compiled expression
    fn execute_compiled_expression(
        &self,
        compiled: &CompiledExpression,
        fact: &Fact,
        context: Option<&HashMap<String, FactValue>>,
    ) -> Result<FactValue> {
        self.evaluate_node(&compiled.ast, fact, context)
    }

    /// Evaluate an expression node
    fn evaluate_node(
        &self,
        node: &ExpressionNode,
        fact: &Fact,
        context: Option<&HashMap<String, FactValue>>,
    ) -> Result<FactValue> {
        match node {
            ExpressionNode::Number(n) => Ok(FactValue::Float(*n)),
            ExpressionNode::Field(field_name) => {
                // First check the fact fields
                if let Some(value) = fact.data.fields.get(field_name) {
                    Ok(value.clone())
                } else if let Some(ctx) = context {
                    // Then check the context
                    if let Some(value) = ctx.get(field_name) {
                        Ok(value.clone())
                    } else {
                        Err(anyhow::anyhow!("Field '{}' not found in fact or context", field_name))
                    }
                } else {
                    Err(anyhow::anyhow!("Field '{}' not found in fact", field_name))
                }
            }
            ExpressionNode::BinaryOp { left, operator, right } => {
                let left_val = self.evaluate_node(left, fact, context)?;
                let right_val = self.evaluate_node(right, fact, context)?;
                self.apply_binary_operator(&left_val, operator, &right_val)
            }
            ExpressionNode::FunctionCall { name, args } => {
                let arg_values: Result<Vec<_>, _> = args.iter()
                    .map(|arg| self.evaluate_node(arg, fact, context))
                    .collect();
                let arg_values = arg_values?;
                self.call_function(name, &arg_values)
            }
        }
    }

    /// Apply a binary operator to two values
    fn apply_binary_operator(
        &self,
        left: &FactValue,
        operator: &char,
        right: &FactValue,
    ) -> Result<FactValue> {
        let left_num = self.to_number(left)?;
        let right_num = self.to_number(right)?;

        let result = match operator {
            '+' => left_num + right_num,
            '-' => left_num - right_num,
            '*' => left_num * right_num,
            '/' => {
                if right_num == 0.0 {
                    return Err(anyhow::anyhow!("Division by zero"));
                }
                left_num / right_num
            }
            _ => return Err(anyhow::anyhow!("Unknown operator: {}", operator)),
        };

        Ok(FactValue::Float(result))
    }

    /// Call a built-in function
    fn call_function(&self, name: &str, args: &[FactValue]) -> Result<FactValue> {
        match name {
            "hours_between" => {
                if args.len() != 2 {
                    return Err(anyhow::anyhow!("hours_between requires exactly 2 arguments"));
                }
                self.hours_between_function(&args[0], &args[1])
            }
            "minutes_between" => {
                if args.len() != 2 {
                    return Err(anyhow::anyhow!("minutes_between requires exactly 2 arguments"));
                }
                self.minutes_between_function(&args[0], &args[1])
            }
            "minutes_to_hours" => {
                if args.len() != 1 {
                    return Err(anyhow::anyhow!("minutes_to_hours requires exactly 1 argument"));
                }
                let minutes = self.to_number(&args[0])?;
                Ok(FactValue::Float(minutes / 60.0))
            }
            "multiply" => {
                if args.len() != 2 {
                    return Err(anyhow::anyhow!("multiply requires exactly 2 arguments"));
                }
                let left = self.to_number(&args[0])?;
                let right = self.to_number(&args[1])?;
                Ok(FactValue::Float(left * right))
            }
            _ => Err(anyhow::anyhow!("Unknown function: {}", name)),
        }
    }

    /// Calculate hours between two datetime strings
    fn hours_between_function(&self, start: &FactValue, end: &FactValue) -> Result<FactValue> {
        let start_str = match start {
            FactValue::String(s) => s,
            _ => return Err(anyhow::anyhow!("hours_between start argument must be a string")),
        };

        let end_str = match end {
            FactValue::String(s) => s,
            _ => return Err(anyhow::anyhow!("hours_between end argument must be a string")),
        };

        // Parse datetime strings (format: "YYYY-MM-DD HH:MM:SS")
        let start_dt = NaiveDateTime::parse_from_str(start_str, "%Y-%m-%d %H:%M:%S")
            .context("Failed to parse start datetime")?;
        let end_dt = NaiveDateTime::parse_from_str(end_str, "%Y-%m-%d %H:%M:%S")
            .context("Failed to parse end datetime")?;

        let duration = end_dt - start_dt;
        let hours = duration.num_seconds() as f64 / 3600.0;

        Ok(FactValue::Float(hours))
    }

    /// Calculate minutes between two datetime strings
    fn minutes_between_function(&self, start: &FactValue, end: &FactValue) -> Result<FactValue> {
        let start_str = match start {
            FactValue::String(s) => s,
            _ => return Err(anyhow::anyhow!("minutes_between start argument must be a string")),
        };

        let end_str = match end {
            FactValue::String(s) => s,
            _ => return Err(anyhow::anyhow!("minutes_between end argument must be a string")),
        };

        // Parse datetime strings (format: "YYYY-MM-DD HH:MM:SS")
        let start_dt = NaiveDateTime::parse_from_str(start_str, "%Y-%m-%d %H:%M:%S")
            .context("Failed to parse start datetime")?;
        let end_dt = NaiveDateTime::parse_from_str(end_str, "%Y-%m-%d %H:%M:%S")
            .context("Failed to parse end datetime")?;

        let duration = end_dt - start_dt;
        Ok(FactValue::Float(duration.num_minutes() as f64))
    }

    /// Convert a FactValue to a number
    fn to_number(&self, value: &FactValue) -> Result<f64> {
        match value {
            FactValue::Integer(i) => Ok(*i as f64),
            FactValue::Float(f) => Ok(*f),
            FactValue::String(s) => {
                s.parse::<f64>()
                    .context(format!("Could not parse '{}' as number", s))
            }
            _ => Err(anyhow::anyhow!("Cannot convert {:?} to number", value)),
        }
    }

    /// Clear the expression cache
    pub fn clear_cache(&mut self) {
        self.expression_cache.clear();
    }
}

impl Default for PayrollExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Compiled expression for efficient re-execution
#[derive(Debug, Clone)]
struct CompiledExpression {
    original: String,
    ast: ExpressionNode,
}

/// Token in an expression
#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Field(String),
    Operator(char),
    Function(String),
    LeftParen,
    RightParen,
}

/// Expression AST node
#[derive(Debug, Clone)]
enum ExpressionNode {
    Number(f64),
    Field(String),
    BinaryOp {
        left: Box<ExpressionNode>,
        operator: char,
        right: Box<ExpressionNode>,
    },
    FunctionCall {
        name: String,
        args: Vec<ExpressionNode>,
    },
}

/// Simple recursive descent parser for expressions
struct ExpressionParser {
    tokens: Vec<Token>,
    position: usize,
}

impl ExpressionParser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    fn parse_expression(&mut self) -> Result<ExpressionNode> {
        self.parse_addition()
    }

    fn parse_addition(&mut self) -> Result<ExpressionNode> {
        let mut left = self.parse_multiplication()?;

        while self.position < self.tokens.len() {
            let op = if let Token::Operator(op) = &self.tokens[self.position] {
                if *op == '+' || *op == '-' {
                    *op
                } else {
                    break;
                }
            } else {
                break;
            };
            
            self.position += 1;
            let right = self.parse_multiplication()?;
            left = ExpressionNode::BinaryOp {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<ExpressionNode> {
        let mut left = self.parse_primary()?;

        while self.position < self.tokens.len() {
            let op = if let Token::Operator(op) = &self.tokens[self.position] {
                if *op == '*' || *op == '/' {
                    *op
                } else {
                    break;
                }
            } else {
                break;
            };
            
            self.position += 1;
            let right = self.parse_primary()?;
            left = ExpressionNode::BinaryOp {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<ExpressionNode> {
        if self.position >= self.tokens.len() {
            return Err(anyhow::anyhow!("Unexpected end of expression"));
        }

        match &self.tokens[self.position] {
            Token::Number(n) => {
                self.position += 1;
                Ok(ExpressionNode::Number(*n))
            }
            Token::Field(name) => {
                self.position += 1;
                Ok(ExpressionNode::Field(name.clone()))
            }
            Token::Function(name) => {
                let func_name = name.clone();
                self.position += 1;
                
                // Expect opening parenthesis
                if self.position >= self.tokens.len() || !matches!(self.tokens[self.position], Token::Operator('(')) {
                    return Err(anyhow::anyhow!("Expected '(' after function name"));
                }
                self.position += 1;

                // Parse arguments
                let mut args = Vec::new();
                while self.position < self.tokens.len() && !matches!(self.tokens[self.position], Token::Operator(')')) {
                    args.push(self.parse_expression()?);
                    
                    // Skip comma if present
                    if self.position < self.tokens.len() && matches!(self.tokens[self.position], Token::Operator(',')) {
                        self.position += 1;
                    }
                }

                // Expect closing parenthesis
                if self.position >= self.tokens.len() || !matches!(self.tokens[self.position], Token::Operator(')')) {
                    return Err(anyhow::anyhow!("Expected ')' after function arguments"));
                }
                self.position += 1;

                Ok(ExpressionNode::FunctionCall {
                    name: func_name,
                    args,
                })
            }
            Token::Operator('(') => {
                self.position += 1;
                let expr = self.parse_expression()?;
                
                if self.position >= self.tokens.len() || !matches!(self.tokens[self.position], Token::Operator(')')) {
                    return Err(anyhow::anyhow!("Expected ')'"));
                }
                self.position += 1;
                
                Ok(expr)
            }
            _ => Err(anyhow::anyhow!("Unexpected token: {:?}", self.tokens[self.position])),
        }
    }
}

/// Helper functions for common payroll calculations
pub struct PayrollCalculations;

impl PayrollCalculations {
    /// Calculate payable hours: (finish_datetime - start_datetime) - break_minutes
    pub fn calculate_payable_hours(
        start_datetime: &str,
        finish_datetime: &str,
        break_minutes: f64,
    ) -> Result<f64> {
        let start_dt = NaiveDateTime::parse_from_str(start_datetime, "%Y-%m-%d %H:%M:%S")
            .context("Failed to parse start datetime")?;
        let finish_dt = NaiveDateTime::parse_from_str(finish_datetime, "%Y-%m-%d %H:%M:%S")
            .context("Failed to parse finish datetime")?;

        let duration = finish_dt - start_dt;
        let total_minutes = duration.num_minutes() as f64;
        let payable_minutes = total_minutes - break_minutes;
        
        Ok(payable_minutes / 60.0)
    }

    /// Calculate pay rate based on pay code and multipliers
    pub fn calculate_pay_rate(
        base_rate: f64,
        pay_code: &str,
        holiday_multiplier: Option<f64>,
    ) -> f64 {
        match pay_code {
            "base_pay" => base_rate,
            "holiday" => {
                base_rate * holiday_multiplier.unwrap_or(1.5)
            }
            "overtime" => base_rate, // Overtime uses base rate, just different classification
            _ => base_rate, // Default to base rate
        }
    }

    /// Calculate gross pay: hours * pay_rate
    pub fn calculate_gross_pay(hours: f64, pay_rate: f64) -> f64 {
        hours * pay_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_fact() -> Fact {
        let mut fields = HashMap::new();
        fields.insert("start_datetime".to_string(), 
                     FactValue::String("2025-01-01 08:00:00".to_string()));
        fields.insert("finish_datetime".to_string(), 
                     FactValue::String("2025-01-01 18:00:00".to_string()));
        fields.insert("break_minutes".to_string(), FactValue::Integer(60));
        fields.insert("base_rate".to_string(), FactValue::Float(20.0));
        fields.insert("multiplier".to_string(), FactValue::Float(1.5));

        Fact {
            id: 1,
            external_id: None,
            data: FactData { fields },
        }
    }

    #[test]
    fn test_simple_arithmetic() {
        let mut evaluator = PayrollExpressionEvaluator::new();
        let fact = create_test_fact();

        let result = evaluator.evaluate_expression("10 + 5", &fact, None).unwrap();
        assert_eq!(result, FactValue::Float(15.0));

        let result = evaluator.evaluate_expression("20 * 1.5", &fact, None).unwrap();
        assert_eq!(result, FactValue::Float(30.0));
    }

    #[test]
    fn test_field_references() {
        let mut evaluator = PayrollExpressionEvaluator::new();
        let fact = create_test_fact();

        let result = evaluator.evaluate_expression("base_rate * multiplier", &fact, None).unwrap();
        assert_eq!(result, FactValue::Float(30.0));
    }

    #[test]
    fn test_hours_between_function() {
        let mut evaluator = PayrollExpressionEvaluator::new();
        let fact = create_test_fact();

        let result = evaluator.evaluate_expression(
            "hours_between(start_datetime, finish_datetime)", 
            &fact, 
            None
        ).unwrap();
        
        assert_eq!(result, FactValue::Float(10.0)); // 18:00 - 08:00 = 10 hours
    }

    #[test]
    fn test_minutes_between_function() {
        let mut evaluator = PayrollExpressionEvaluator::new();
        let fact = create_test_fact();

        let result = evaluator.evaluate_expression(
            "minutes_between(start_datetime, finish_datetime)",
            &fact,
            None
        ).unwrap();
        assert_eq!(result, FactValue::Float(600.0)); // 10 hours * 60 minutes
    }

    #[test]
    fn test_payroll_calculations() {
        // Test payable hours calculation
        let hours = PayrollCalculations::calculate_payable_hours(
            "2025-01-01 08:00:00",
            "2025-01-01 18:00:00",
            60.0
        ).unwrap();
        assert_eq!(hours, 9.0); // 10 hours - 1 hour break

        // Test pay rate calculation
        let rate = PayrollCalculations::calculate_pay_rate(20.0, "holiday", Some(1.5));
        assert_eq!(rate, 30.0);

        // Test gross pay calculation
        let gross = PayrollCalculations::calculate_gross_pay(9.0, 30.0);
        assert_eq!(gross, 270.0);
    }
}