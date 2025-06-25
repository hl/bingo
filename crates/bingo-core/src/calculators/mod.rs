//! Built-in calculator framework for compliance and payroll processing

use crate::types::{CalculatorError, ErrorCode, FactValue, FieldSpec};
use std::collections::HashMap;
use std::sync::LazyLock;

pub mod hours_calculator;
pub mod threshold_checker;

/// Type-safe calculator trait for business logic execution
pub trait Calculator: Send + Sync {
    fn calculate(&self, inputs: &CalculatorInputs) -> Result<FactValue, CalculatorError>;
    fn name(&self) -> &str;
    fn required_fields(&self) -> &[FieldSpec];
}

/// Type-safe wrapper for calculator inputs with validation
pub struct CalculatorInputs<'a> {
    bound_variables: &'a HashMap<String, FactValue>,
}

impl<'a> CalculatorInputs<'a> {
    pub fn new(bound_variables: &'a HashMap<String, FactValue>) -> Self {
        Self { bound_variables }
    }

    pub fn get_integer(&self, field: &str) -> Result<i64, CalculatorError> {
        match self.bound_variables.get(field) {
            Some(FactValue::Integer(i)) => Ok(*i),
            Some(other) => Err(CalculatorError {
                code: ErrorCode::InvalidFieldType,
                message: format!(
                    "Field '{}' expected integer, got {}",
                    field,
                    other.type_name()
                ),
                details: Some(HashMap::from([
                    ("field".to_string(), FactValue::String(field.to_string())),
                    (
                        "expected".to_string(),
                        FactValue::String("integer".to_string()),
                    ),
                    ("actual".to_string(), other.clone()),
                ])),
            }),
            None => Err(CalculatorError {
                code: ErrorCode::MissingRequiredField,
                message: format!("Required field '{}' is missing", field),
                details: Some(HashMap::from([(
                    "field".to_string(),
                    FactValue::String(field.to_string()),
                )])),
            }),
        }
    }

    pub fn get_float(&self, field: &str) -> Result<f64, CalculatorError> {
        match self.bound_variables.get(field) {
            Some(FactValue::Float(f)) => Ok(*f),
            Some(FactValue::Integer(i)) => Ok(*i as f64),
            Some(other) => Err(CalculatorError {
                code: ErrorCode::InvalidFieldType,
                message: format!(
                    "Field '{}' expected float, got {}",
                    field,
                    other.type_name()
                ),
                details: Some(HashMap::from([
                    ("field".to_string(), FactValue::String(field.to_string())),
                    (
                        "expected".to_string(),
                        FactValue::String("float".to_string()),
                    ),
                    ("actual".to_string(), other.clone()),
                ])),
            }),
            None => Err(CalculatorError {
                code: ErrorCode::MissingRequiredField,
                message: format!("Required field '{}' is missing", field),
                details: Some(HashMap::from([(
                    "field".to_string(),
                    FactValue::String(field.to_string()),
                )])),
            }),
        }
    }

    pub fn get_optional_float(&self, field: &str) -> Result<Option<f64>, CalculatorError> {
        match self.bound_variables.get(field) {
            Some(FactValue::Float(f)) => Ok(Some(*f)),
            Some(FactValue::Integer(i)) => Ok(Some(*i as f64)),
            Some(other) => Err(CalculatorError {
                code: ErrorCode::InvalidFieldType,
                message: format!(
                    "Field '{}' expected float, got {}",
                    field,
                    other.type_name()
                ),
                details: Some(HashMap::from([
                    ("field".to_string(), FactValue::String(field.to_string())),
                    ("actual".to_string(), other.clone()),
                ])),
            }),
            None => Ok(None),
        }
    }

    pub fn get_string(&self, field: &str) -> Result<String, CalculatorError> {
        match self.bound_variables.get(field) {
            Some(FactValue::String(s)) => Ok(s.clone()),
            Some(other) => Err(CalculatorError {
                code: ErrorCode::InvalidFieldType,
                message: format!(
                    "Field '{}' expected string, got {}",
                    field,
                    other.type_name()
                ),
                details: Some(HashMap::from([
                    ("field".to_string(), FactValue::String(field.to_string())),
                    (
                        "expected".to_string(),
                        FactValue::String("string".to_string()),
                    ),
                    ("actual".to_string(), other.clone()),
                ])),
            }),
            None => Err(CalculatorError {
                code: ErrorCode::MissingRequiredField,
                message: format!("Required field '{}' is missing", field),
                details: Some(HashMap::from([(
                    "field".to_string(),
                    FactValue::String(field.to_string()),
                )])),
            }),
        }
    }

    pub fn get_boolean(&self, field: &str) -> Result<bool, CalculatorError> {
        match self.bound_variables.get(field) {
            Some(FactValue::Boolean(b)) => Ok(*b),
            Some(other) => Err(CalculatorError {
                code: ErrorCode::InvalidFieldType,
                message: format!(
                    "Field '{}' expected boolean, got {}",
                    field,
                    other.type_name()
                ),
                details: Some(HashMap::from([
                    ("field".to_string(), FactValue::String(field.to_string())),
                    (
                        "expected".to_string(),
                        FactValue::String("boolean".to_string()),
                    ),
                    ("actual".to_string(), other.clone()),
                ])),
            }),
            None => Err(CalculatorError {
                code: ErrorCode::MissingRequiredField,
                message: format!("Required field '{}' is missing", field),
                details: Some(HashMap::from([(
                    "field".to_string(),
                    FactValue::String(field.to_string()),
                )])),
            }),
        }
    }
}

/// Global calculator registry - populated at startup
static CALCULATORS: LazyLock<HashMap<String, Box<dyn Calculator>>> = LazyLock::new(|| {
    let mut map: HashMap<String, Box<dyn Calculator>> = HashMap::new();

    // Time and hour calculations
    map.insert(
        "hours_between_datetime".to_string(),
        Box::new(hours_calculator::HoursBetweenDateTimeCalculator),
    );
    map.insert(
        "time_difference".to_string(),
        Box::new(hours_calculator::TimeDifferenceCalculator),
    );

    // Threshold and limit checking (reusable for compliance, payroll, cost limits)
    map.insert(
        "threshold_checker".to_string(),
        Box::new(threshold_checker::ThresholdChecker),
    );
    map.insert(
        "limit_validator".to_string(),
        Box::new(threshold_checker::LimitValidator),
    );

    map
});

pub fn get_calculator(name: &str) -> Option<&'static dyn Calculator> {
    CALCULATORS.get(name).map(|v| &**v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FactValue;

    #[test]
    fn test_calculator_inputs() {
        let mut data = HashMap::new();
        data.insert("test_int".to_string(), FactValue::Integer(42));
        data.insert(
            "test_float".to_string(),
            FactValue::Float(std::f64::consts::PI),
        );
        data.insert(
            "test_string".to_string(),
            FactValue::String("hello".to_string()),
        );
        data.insert("test_bool".to_string(), FactValue::Boolean(true));

        let inputs = CalculatorInputs::new(&data);

        assert_eq!(inputs.get_integer("test_int").unwrap(), 42);
        assert_eq!(
            inputs.get_float("test_float").unwrap(),
            std::f64::consts::PI
        );
        assert_eq!(inputs.get_string("test_string").unwrap(), "hello");
        assert!(inputs.get_boolean("test_bool").unwrap());

        // Test type coercion
        assert_eq!(inputs.get_float("test_int").unwrap(), 42.0);

        // Test missing field
        assert!(inputs.get_integer("missing").is_err());
    }
}
