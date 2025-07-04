//! A calculator for computing the weighted average of a set of items.

use crate::FactValue;
use crate::{Calculator, CalculatorInputs};
use anyhow::{Result, anyhow};

/// Calculates the weighted average for a list of items.
///
/// # Expected Inputs
/// - `items`: An array of objects. Each object must contain:
///   - `value`: A numeric field representing the value.
///   - `weight`: A numeric field representing the weight.
///
/// # Output
/// A string representation of the calculated weighted average (float).
/// Returns "0.0" if the total weight is zero.
#[derive(Debug, Default)]
pub struct WeightedAverageCalculator;

impl WeightedAverageCalculator {
    pub fn new() -> Self {
        Self
    }
}

impl Calculator for WeightedAverageCalculator {
    fn calculate(&self, inputs: &CalculatorInputs) -> Result<String> {
        let items = inputs.get_array("items")?;

        if items.is_empty() {
            return Ok("0.0".to_string());
        }

        let mut total_weighted_value = 0.0;
        let mut total_weight = 0.0;

        for item_val in items {
            if let FactValue::Object(item) = item_val {
                // Safely extract value and weight as f64
                let value = item
                    .get("value")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| anyhow!("Each item must have a numeric 'value' field."))?;

                let weight = item
                    .get("weight")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| anyhow!("Each item must have a numeric 'weight' field."))?;

                total_weighted_value += value * weight;
                total_weight += weight;
            } else {
                return Err(anyhow!("'items' must be an array of objects."));
            }
        }

        let weighted_average = if total_weight == 0.0 {
            0.0
        } else {
            total_weighted_value / total_weight
        };
        Ok(weighted_average.to_string())
    }
}
