//! Add Calculator
//!
//! Supported aliases for inputs:
//!   * First: `addend1`, `value1`, `a`, `x`
//!   * Second: `addend2`, `value2`, `b`, `y`
//!
use crate::{Calculator, CalculatorInputs};
use anyhow::{Result, anyhow};

fn extract_number(inputs: &CalculatorInputs, names: &[&str]) -> Result<f64> {
    for &n in names {
        if let Ok(v) = inputs.get_f64(n) {
            return Ok(v);
        }
    }
    Err(anyhow!(
        "Required numeric input not found; tried {:?}",
        names
    ))
}

#[derive(Debug, Default)]
pub struct AddCalculator;

impl Calculator for AddCalculator {
    fn calculate(&self, inputs: &CalculatorInputs) -> Result<String> {
        let a = extract_number(inputs, &["addend1", "value1", "a", "x"])?;
        let b = extract_number(inputs, &["addend2", "value2", "b", "y"])?;
        Ok((a + b).to_string())
    }
}
