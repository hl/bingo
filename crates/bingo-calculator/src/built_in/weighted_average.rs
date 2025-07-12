//! Calculator for computing the weighted average of a set of items
//!
//! This calculator computes the weighted average from an array of items,
//! where each item has a value and an associated weight.

use std::collections::HashMap;

use bingo_types::FactValue;

use crate::plugin::{CalculationResult, CalculatorPlugin};

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

impl CalculatorPlugin for WeightedAverageCalculator {
    fn name(&self) -> &str {
        "weighted_average"
    }

    fn calculate(&self, args: &HashMap<String, &FactValue>) -> CalculationResult {
        let items = match args.get("items") {
            Some(FactValue::Array(arr)) => arr,
            _ => return Err("missing 'items' array argument".to_string()),
        };

        if items.is_empty() {
            return Ok(FactValue::Float(0.0));
        }

        // Initialize accumulators for weighted average calculation
        let mut total_weighted_value = 0.0;
        let mut total_weight = 0.0;

        // Iterate through each item to calculate weighted sum and total weight
        for item_val in items {
            if let FactValue::Object(item) = item_val {
                // Extract numeric value with type coercion from integer to float
                let value = match item.get("value") {
                    Some(FactValue::Float(f)) => *f,
                    Some(FactValue::Integer(i)) => *i as f64,
                    _ => return Err("Each item must have a numeric 'value' field.".to_string()),
                };

                // Extract numeric weight with type coercion from integer to float
                let weight = match item.get("weight") {
                    Some(FactValue::Float(f)) => *f,
                    Some(FactValue::Integer(i)) => *i as f64,
                    _ => return Err("Each item must have a numeric 'weight' field.".to_string()),
                };

                // Accumulate weighted values: sum of (value × weight) for numerator
                total_weighted_value += value * weight;
                // Accumulate total weights for denominator
                total_weight += weight;
            } else {
                return Err("'items' must be an array of objects.".to_string());
            }
        }

        // Calculate final weighted average using the formula: Σ(value × weight) / Σ(weight)
        let weighted_average = if total_weight == 0.0 {
            // Handle edge case: avoid division by zero when all weights are zero
            0.0
        } else {
            // Standard weighted average calculation
            total_weighted_value / total_weight
        };
        Ok(FactValue::Float(weighted_average))
    }
}
