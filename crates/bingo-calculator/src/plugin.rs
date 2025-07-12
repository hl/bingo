use bingo_types::FactValue;
use std::collections::HashMap;

pub type CalculationResult = Result<FactValue, String>;

/// Calculator inputs
#[derive(Debug, Clone)]
pub struct CalculatorInputs {
    /// Input fields for the calculator
    pub fields: HashMap<String, FactValue>,
}

/// A trait for calculator plugins.
pub trait CalculatorPlugin: Send + Sync {
    /// The name of the calculator.
    fn name(&self) -> &str;

    /// Performs the calculation.
    fn calculate(&self, args: &HashMap<String, &FactValue>) -> CalculationResult;
}
