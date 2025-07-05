//! Multiply Calculator
//!
//! Supported aliases for the two inputs (case-sensitive):
//!   * First argument – `multiplicand`, `value1`, `a`, `x`
//!   * Second argument – `multiplier`, `value2`, `b`, `y`
//!
//! The calculator returns the product as a string so that the generic
//! `Calculator` trait signature is respected.

use crate::{Calculator, CalculatorInputs};
use anyhow::{Result, anyhow};

fn extract_number(inputs: &CalculatorInputs, candidates: &[&str]) -> Result<f64> {
    for &name in candidates {
        if let Ok(val) = inputs.get_f64(name) {
            return Ok(val);
        }
    }
    Err(anyhow!(
        "Required numeric input not found. Tried the following names: {:?}",
        candidates
    ))
}

#[derive(Debug, Default)]
pub struct MultiplyCalculator;

impl Calculator for MultiplyCalculator {
    fn calculate(&self, inputs: &CalculatorInputs) -> Result<String> {
        let a = extract_number(inputs, &["multiplicand", "value1", "a", "x"])?;
        let b = extract_number(inputs, &["multiplier", "value2", "b", "y"])?;
        Ok((a * b).to_string())
    }
}
