//! Percentage Deduct Calculator
//!
//! Deducts a percentage from a total amount.
//! Inputs (aliases):
//!   * `total_amount` / `base_amount` / `value` / `amount`
//!   * `percentage` (decimal e.g., 0.05 for 5%)
//!
use crate::{Calculator, CalculatorInputs};
use anyhow::{Result, anyhow};

#[derive(Debug, Default)]
pub struct PercentageDeductCalculator;

impl Calculator for PercentageDeductCalculator {
    fn calculate(&self, inputs: &CalculatorInputs) -> Result<String> {
        let amount = ["total_amount", "base_amount", "value", "amount"]
            .iter()
            .find_map(|&n| inputs.get_f64(n).ok())
            .ok_or_else(|| anyhow!("Required numeric 'total_amount' not provided"))?;

        let percentage = inputs.get_f64("percentage").or_else(|_| inputs.get_f64("percent"))?;

        let result = amount - (amount * percentage);
        Ok(result.to_string())
    }
}
