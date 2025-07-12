//! Calculator for deducting a percentage from a base amount
//!
//! This calculator subtracts a percentage of the base amount from the original amount.
//! For example, deducting 20% from 100 results in 80 (100 - 100*0.2).

use std::collections::HashMap;

use bingo_types::FactValue;

use crate::plugin::{CalculationResult, CalculatorPlugin};

/// Calculator for percentage deduction operations
///
/// # Arguments
/// * `amount` - Base amount from which percentage will be deducted
/// * `percentage` - Percentage to deduct (e.g., 0.2 for 20%)
///
/// # Returns
/// The amount minus the percentage of the amount as a FactValue::Float
#[derive(Debug, Default)]
pub struct PercentageDeductCalculator;

impl CalculatorPlugin for PercentageDeductCalculator {
    fn name(&self) -> &str {
        "percentage_deduct"
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
        let result = amount - (amount * percentage);
        Ok(FactValue::Float(result))
    }
}
