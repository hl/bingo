use crate::{Calculator, CalculatorInputs};
use anyhow::Result;

pub struct ThresholdCheckCalculator;

impl Calculator for ThresholdCheckCalculator {
    fn calculate(&self, inputs: &CalculatorInputs) -> Result<String> {
        let value = inputs.get_f64("value")?;
        let threshold = inputs.get_f64("threshold")?;

        if value > threshold {
            Ok("true".to_string())
        } else {
            Ok("false".to_string())
        }
    }
}
