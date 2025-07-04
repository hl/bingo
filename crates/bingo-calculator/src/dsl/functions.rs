//! Built-in functions for calculator DSL
//!
//! This module provides a registry of mathematical, logical, and utility functions
//! that can be called from calculator expressions.

use crate::dsl::EvaluationContext;
use crate::types::FactValue;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// Trait for functions that can be called from calculator expressions
pub trait CalculatorFunction: Send + Sync {
    /// Call the function with the given arguments
    fn call(&self, args: &[FactValue]) -> Result<FactValue>;

    /// Get the expected number of arguments (None for variadic)
    fn arity(&self) -> Option<usize>;

    /// Get a description of this function
    fn description(&self) -> &'static str;
}

/// Trait for context-aware functions that need access to evaluation context
pub trait ContextAwareFunction: Send + Sync {
    /// Call the function with arguments and context
    fn call_with_context(
        &self,
        args: &[FactValue],
        context: &EvaluationContext,
    ) -> Result<FactValue>;

    /// Get the expected number of arguments (None for variadic)
    fn arity(&self) -> Option<usize>;

    /// Get a description of this function
    fn description(&self) -> &'static str;
}

/// Registry for calculator functions
#[derive(Default)]
pub struct FunctionRegistry {
    functions: HashMap<String, Box<dyn CalculatorFunction>>,
    context_functions: HashMap<String, Box<dyn ContextAwareFunction>>,
}

impl std::fmt::Debug for FunctionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionRegistry")
            .field("functions", &self.functions.keys().collect::<Vec<_>>())
            .field(
                "context_functions",
                &self.context_functions.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl FunctionRegistry {
    /// Create a new empty function registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a function registry with built-in functions
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();

        // Register basic math functions
        registry.register("max", Box::new(MaxFunction));
        registry.register("min", Box::new(MinFunction));
        registry.register("round", Box::new(RoundFunction));
        registry.register("abs", Box::new(AbsFunction));

        registry
    }

    /// Register a new function
    pub fn register(&mut self, name: &str, function: Box<dyn CalculatorFunction>) {
        self.functions.insert(name.to_string(), function);
    }

    /// Register a context-aware function
    pub fn register_context(&mut self, name: &str, function: Box<dyn ContextAwareFunction>) {
        self.context_functions.insert(name.to_string(), function);
    }

    /// Call a function with context
    pub fn call_with_context(
        &self,
        name: &str,
        args: &[FactValue],
        context: &EvaluationContext,
    ) -> Result<FactValue> {
        // Check context-aware functions first
        if let Some(func) = self.context_functions.get(name) {
            return func.call_with_context(args, context);
        }

        // Fall back to regular functions
        if let Some(func) = self.functions.get(name) {
            return func.call(args);
        }

        Err(anyhow!("Unknown function: {}", name))
    }
}

// Basic built-in functions

struct MaxFunction;

impl CalculatorFunction for MaxFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        if args.is_empty() {
            return Err(anyhow!("max() requires at least one argument"));
        }

        let mut max_val = &args[0];
        for arg in &args[1..] {
            match (max_val, arg) {
                (FactValue::Integer(a), FactValue::Integer(b)) => {
                    if b > a {
                        max_val = arg;
                    }
                }
                (FactValue::Float(a), FactValue::Float(b)) => {
                    if b > a {
                        max_val = arg;
                    }
                }
                (FactValue::Integer(a), FactValue::Float(b)) => {
                    if b > &(*a as f64) {
                        max_val = arg;
                    }
                }
                (FactValue::Float(a), FactValue::Integer(b)) => {
                    if (*b as f64) > *a {
                        max_val = arg;
                    }
                }
                _ => return Err(anyhow!("max() only supports numeric values")),
            }
        }

        Ok(max_val.clone())
    }

    fn arity(&self) -> Option<usize> {
        None // variadic
    }

    fn description(&self) -> &'static str {
        "Returns the maximum value from the arguments"
    }
}

struct MinFunction;

impl CalculatorFunction for MinFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        if args.is_empty() {
            return Err(anyhow!("min() requires at least one argument"));
        }

        let mut min_val = &args[0];
        for arg in &args[1..] {
            match (min_val, arg) {
                (FactValue::Integer(a), FactValue::Integer(b)) => {
                    if b < a {
                        min_val = arg;
                    }
                }
                (FactValue::Float(a), FactValue::Float(b)) => {
                    if b < a {
                        min_val = arg;
                    }
                }
                (FactValue::Integer(a), FactValue::Float(b)) => {
                    if b < &(*a as f64) {
                        min_val = arg;
                    }
                }
                (FactValue::Float(a), FactValue::Integer(b)) => {
                    if (*b as f64) < *a {
                        min_val = arg;
                    }
                }
                _ => return Err(anyhow!("min() only supports numeric values")),
            }
        }

        Ok(min_val.clone())
    }

    fn arity(&self) -> Option<usize> {
        None // variadic
    }

    fn description(&self) -> &'static str {
        "Returns the minimum value from the arguments"
    }
}

struct RoundFunction;

impl CalculatorFunction for RoundFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match args {
            [FactValue::Float(n)] => Ok(FactValue::Float(n.round())),
            [FactValue::Integer(n)] => Ok(FactValue::Integer(*n)),
            [FactValue::Float(n), FactValue::Integer(precision)] => {
                let factor = 10.0_f64.powi(*precision as i32);
                Ok(FactValue::Float((n * factor).round() / factor))
            }
            _ => Err(anyhow!("round() expects a number and optional precision")),
        }
    }

    fn arity(&self) -> Option<usize> {
        None // 1 or 2 arguments
    }

    fn description(&self) -> &'static str {
        "Rounds a number to the nearest integer or specified precision"
    }
}

struct AbsFunction;

impl CalculatorFunction for AbsFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match args {
            [FactValue::Integer(n)] => Ok(FactValue::Integer(n.abs())),
            [FactValue::Float(n)] => Ok(FactValue::Float(n.abs())),
            _ => Err(anyhow!("abs() expects exactly one numeric argument")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }

    fn description(&self) -> &'static str {
        "Returns the absolute value of a number"
    }
}
