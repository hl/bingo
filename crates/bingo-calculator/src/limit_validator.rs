use std::collections::HashMap;

use bingo_types::FactValue;

use crate::plugin::{CalculationResult, CalculatorPlugin};

pub struct LimitValidateCalculator;

impl CalculatorPlugin for LimitValidateCalculator {
    fn name(&self) -> &str {
        "limit_validator"
    }

    fn calculate(&self, args: &HashMap<String, &FactValue>) -> CalculationResult {
        let value = match args.get("value") {
            Some(FactValue::Float(f)) => *f,
            Some(FactValue::Integer(i)) => *i as f64,
            _ => return Err("Invalid argument 'value': expected number".to_string()),
        };
        let min = match args.get("min") {
            Some(FactValue::Float(f)) => *f,
            Some(FactValue::Integer(i)) => *i as f64,
            _ => return Err("Invalid argument 'min': expected number".to_string()),
        };
        let max = match args.get("max") {
            Some(FactValue::Float(f)) => *f,
            Some(FactValue::Integer(i)) => *i as f64,
            _ => return Err("Invalid argument 'max': expected number".to_string()),
        };

        if value >= min && value <= max {
            Ok(FactValue::Boolean(true))
        } else {
            Ok(FactValue::Boolean(false))
        }
    }
}
