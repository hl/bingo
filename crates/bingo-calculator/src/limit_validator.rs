use crate::{Calculator, CalculatorInputs};
use anyhow::Result;

pub struct LimitValidateCalculator;

impl Calculator for LimitValidateCalculator {
    fn calculate(&self, inputs: &CalculatorInputs) -> Result<String> {
        let value = inputs.get_f64("value")?;
        let min = inputs.get_f64("min")?;
        let max = inputs.get_f64("max")?;

        if value >= min && value <= max {
            Ok("true".to_string())
        } else {
            Ok("false".to_string())
        }
    }
}
