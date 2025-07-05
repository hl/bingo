//! Percentage Add Calculator
//!
//! Adds a percentage of a base amount to the base amount.
//! Inputs (aliases):
//!   * `base_amount` / `total_amount` / `value` / `amount`
//!   * `percentage` (expressed as decimal e.g. 0.15 for 15%)
//!
use crate::{Calculator, CalculatorInputs};
use anyhow::{Result, anyhow};

#[derive(Debug, Default)]
pub struct PercentageAddCalculator;

impl Calculator for PercentageAddCalculator {
    fn calculate(&self, inputs: &CalculatorInputs) -> Result<String> {
        let amount = ["base_amount", "total_amount", "value", "amount"]
            .iter()
            .find_map(|&n| inputs.get_f64(n).ok())
            .ok_or_else(|| anyhow!("Required numeric 'base_amount' not provided"))?;

        let percentage = inputs.get_f64("percentage").or_else(|_| inputs.get_f64("percent"))?;

        let result = amount + (amount * percentage);
        Ok(result.to_string())
    }
}
