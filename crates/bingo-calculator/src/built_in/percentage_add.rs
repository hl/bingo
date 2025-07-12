//! Calculator for adding a percentage to a base amount
//!
//! This calculator adds a percentage of the base amount to the original amount.
//! For example, adding 20% to 100 results in 120 (100 + 100*0.2).

use std::collections::HashMap;

use bingo_types::FactValue;

use crate::plugin::{CalculationResult, CalculatorPlugin};

/// Calculator for percentage addition operations
///
/// # Arguments
/// * `amount` - Base amount to which percentage will be added
/// * `percentage` - Percentage to add (e.g., 0.2 for 20%)
///
/// # Returns
/// The amount plus the percentage of the amount as a FactValue::Float
#[derive(Debug, Default)]
pub struct PercentageAddCalculator;

impl CalculatorPlugin for PercentageAddCalculator {
    fn name(&self) -> &str {
        "percentage_add"
    }

    fn calculate(&self, args: &HashMap<String, &FactValue>) -> CalculationResult {
        let amount = match args.get("amount") {
            Some(FactValue::Float(f)) => *f,
            Some(FactValue::Integer(i)) => *i as f64,
            _ => return Err("Invalid argument 'amount': expected number".to_string()),
        };
        let percentage = match args.get("percentage") {
            Some(FactValue::Float(f)) => *f,
            Some(FactValue::Integer(i)) => *i as f64,
            _ => return Err("Invalid argument 'percentage': expected number".to_string()),
        };
        let result = amount + (amount * percentage);
        Ok(FactValue::Float(result))
    }
}
