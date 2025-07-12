//! Calculator for adding two numeric values
//!
//! This calculator performs basic addition operations on numeric fact values.

use std::collections::HashMap;

use bingo_types::FactValue;

use crate::plugin::{CalculationResult, CalculatorPlugin};

/// Calculator for addition operations
///
/// # Arguments
/// * `a` - First numeric value to add
/// * `b` - Second numeric value to add
///
/// # Returns
/// The sum of `a` and `b` as a FactValue::Float
#[derive(Debug, Default)]
pub struct AddCalculator;

impl CalculatorPlugin for AddCalculator {
    fn name(&self) -> &str {
        "add"
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
        Ok(FactValue::Float(a + b))
    }
}
