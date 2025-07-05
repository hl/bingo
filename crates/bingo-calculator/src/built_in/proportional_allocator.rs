//! Proportional Allocator Calculator
//!
//! Distributes `total_amount` proportionally based on an `individual_value`
//! relative to a `total_value`.
//!
//! result = total_amount * (individual_value / total_value)
//!
//! Returns 0 when `total_value` is 0.

use crate::{Calculator, CalculatorInputs};
use anyhow::Result;

#[derive(Debug, Default)]
pub struct ProportionalAllocatorCalculator;

impl Calculator for ProportionalAllocatorCalculator {
    fn calculate(&self, inputs: &CalculatorInputs) -> Result<String> {
        let total_amount = inputs.get_f64("total_amount").or_else(|_| inputs.get_f64("amount"))?;

        let individual_value =
            inputs.get_f64("individual_value").or_else(|_| inputs.get_f64("value"))?;

        let total_value = inputs.get_f64("total_value").or_else(|_| inputs.get_f64("aggregate"))?;

        if total_value == 0.0 {
            return Ok("0.0".to_string());
        }

        let allocation = total_amount * (individual_value / total_value);
        Ok(allocation.to_string())
    }
}
