//! Calculator for proportional allocation of amounts
//!
//! This calculator allocates a total amount proportionally based on individual values
//! relative to the sum of all values.

use std::collections::HashMap;

use bingo_types::FactValue;

use crate::plugin::{CalculationResult, CalculatorPlugin};

#[derive(Debug, Default)]
pub struct ProportionalAllocatorCalculator;

impl CalculatorPlugin for ProportionalAllocatorCalculator {
    fn name(&self) -> &str {
        "proportional_allocator"
    }

    fn calculate(&self, args: &HashMap<String, &FactValue>) -> CalculationResult {
        let total_amount = match args.get("total_amount") {
            Some(FactValue::Float(f)) => *f,
            Some(FactValue::Integer(i)) => *i as f64,
            _ => return Err("Invalid argument 'total_amount': expected number".to_string()),
        };
        let individual_value = match args.get("individual_value") {
            Some(FactValue::Float(f)) => *f,
            Some(FactValue::Integer(i)) => *i as f64,
            _ => return Err("Invalid argument 'individual_value': expected number".to_string()),
        };
        let total_value = match args.get("total_value") {
            Some(FactValue::Float(f)) => *f,
            Some(FactValue::Integer(i)) => *i as f64,
            _ => return Err("Invalid argument 'total_value': expected number".to_string()),
        };

        // Prevent division by zero in proportional allocation
        if total_value == 0.0 {
            return Ok(FactValue::Float(0.0));
        }

        // Calculate proportional allocation using the formula:
        // allocation = total_amount × (individual_value / total_value)
        // This distributes the total amount proportionally based on individual contribution
        let allocation = total_amount * (individual_value / total_value);
        Ok(FactValue::Float(allocation))
    }
}
