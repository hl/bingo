use crate::types::FactValue;
use anyhow::{Result, anyhow};
use bingo_calculator::{
    Calculator, CalculatorInputs, LimitValidateCalculator, ThresholdCheckCalculator,
    built_in::weighted_average::WeightedAverageCalculator,
};
use std::collections::HashMap;

/// Convert bingo-core FactValue to bingo-calculator FactValue
fn convert_to_calculator_fact_value(core_value: &FactValue) -> bingo_calculator::FactValue {
    match core_value {
        FactValue::String(s) => bingo_calculator::FactValue::String(s.clone()),
        FactValue::Integer(i) => bingo_calculator::FactValue::Integer(*i),
        FactValue::Float(f) => bingo_calculator::FactValue::Float(*f),
        FactValue::Boolean(b) => bingo_calculator::FactValue::Boolean(*b),
        FactValue::Array(arr) => bingo_calculator::FactValue::Array(
            arr.iter().map(convert_to_calculator_fact_value).collect(),
        ),
        FactValue::Object(obj) => bingo_calculator::FactValue::Object(
            obj.iter()
                .map(|(k, v)| (k.clone(), convert_to_calculator_fact_value(v)))
                .collect(),
        ),
        FactValue::Date(dt) => bingo_calculator::FactValue::Date(*dt),
        FactValue::Null => bingo_calculator::FactValue::Null,
    }
}

/// Registry for built-in calculators
pub struct CalculatorRegistry {
    calculators: HashMap<String, Box<dyn Calculator>>,
}

impl CalculatorRegistry {
    /// Create a new calculator registry with built-in calculators
    pub fn new() -> Self {
        let mut registry = Self { calculators: HashMap::new() };

        // Register built-in calculators
        registry.register("threshold_check", Box::new(ThresholdCheckCalculator));
        registry.register("limit_validator", Box::new(LimitValidateCalculator));
        registry.register(
            "weighted_average",
            Box::new(WeightedAverageCalculator::new()),
        );

        registry
    }

    /// Register a calculator
    pub fn register(&mut self, name: &str, calculator: Box<dyn Calculator>) {
        self.calculators.insert(name.to_string(), calculator);
    }

    /// Execute a calculator by name
    pub fn execute(
        &self,
        name: &str,
        input_mapping: &HashMap<String, String>,
        fact_fields: &HashMap<String, FactValue>,
    ) -> Result<String> {
        let calculator = self
            .calculators
            .get(name)
            .ok_or_else(|| anyhow!("Calculator '{}' not found", name))?;

        // Map inputs from fact fields to calculator inputs, converting types
        let mut calculator_inputs = HashMap::new();
        for (calc_input_name, fact_field_name) in input_mapping {
            if let Some(value) = fact_fields.get(fact_field_name) {
                let converted_value = convert_to_calculator_fact_value(value);
                calculator_inputs.insert(calc_input_name.clone(), converted_value);
            } else {
                return Err(anyhow!(
                    "Required field '{}' not found in fact",
                    fact_field_name
                ));
            }
        }

        let inputs = CalculatorInputs::new(&calculator_inputs);
        calculator.calculate(&inputs)
    }

    /// List available calculators
    pub fn list_calculators(&self) -> Vec<String> {
        self.calculators.keys().cloned().collect()
    }
}

impl Default for CalculatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}
