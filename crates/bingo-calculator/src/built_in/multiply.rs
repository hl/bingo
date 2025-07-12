//! Calculator for multiplying two numeric values
//!
//! This calculator performs basic multiplication operations on numeric fact values.

use std::collections::HashMap;

use bingo_types::FactValue;

use crate::plugin::{CalculationResult, CalculatorPlugin};

/// Calculator for multiplication operations
///
/// # Arguments
/// * `a` - First numeric value to multiply
/// * `b` - Second numeric value to multiply
///
/// # Returns
/// The product of `a` and `b` as a FactValue::Float
#[derive(Debug, Default)]
pub struct MultiplyCalculator;

impl CalculatorPlugin for MultiplyCalculator {
    fn name(&self) -> &str {
        "multiply"
    }

    fn calculate(&self, args: &HashMap<String, &FactValue>) -> CalculationResult {
        let a = match args.get("a") {
            Some(FactValue::Float(f)) => *f,
            Some(FactValue::Integer(i)) => *i as f64,
            _ => return Err("Invalid argument 'a': expected number".to_string()),
        };
        let b = match args.get("b") {
            Some(FactValue::Float(f)) => *f,
            Some(FactValue::Integer(i)) => *i as f64,
            _ => return Err("Invalid argument 'b': expected number".to_string()),
        };
        Ok(FactValue::Float(a * b))
    }
}
