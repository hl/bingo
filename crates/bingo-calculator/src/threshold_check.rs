use std::collections::HashMap;

use bingo_types::FactValue;

use crate::plugin::{CalculationResult, CalculatorPlugin};

pub struct ThresholdCheckCalculator;

impl CalculatorPlugin for ThresholdCheckCalculator {
    fn name(&self) -> &str {
        "threshold_check"
    }

    fn calculate(&self, args: &HashMap<String, &FactValue>) -> CalculationResult {
        let value = match args.get("value") {
            Some(FactValue::Float(f)) => *f,
            Some(FactValue::Integer(i)) => *i as f64,
            _ => return Err("Invalid argument 'value': expected number".to_string()),
        };
        let threshold = match args.get("threshold") {
            Some(FactValue::Float(f)) => *f,
            Some(FactValue::Integer(i)) => *i as f64,
            _ => return Err("Invalid argument 'threshold': expected number".to_string()),
        };

        if value > threshold {
            Ok(FactValue::Boolean(true))
        } else {
            Ok(FactValue::Boolean(false))
        }
    }
}
