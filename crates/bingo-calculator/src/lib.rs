#![deny(warnings)]
//! The calculator ecosystem for the Bingo Rules Engine.
//!
//! This crate provides the `Calculator` trait and the `CalculatorInputs` struct
//! for creating custom, high-performance business logic that can be invoked
//! from rule actions.

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Possible values that can be stored in a fact (simplified version for calculators)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FactValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<FactValue>),
    Object(HashMap<String, FactValue>),
    Date(DateTime<Utc>),
    Null,
}

impl FactValue {
    /// Convenience accessor returning an `f64` representation if this value is numeric.
    /// Returns `None` when the variant is not `Integer` or `Float`.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            FactValue::Integer(i) => Some(*i as f64),
            FactValue::Float(f) => Some(*f),
            _ => None,
        }
    }
}

pub mod built_in;
pub mod limit_validator;
pub mod threshold_check;

/// A trait for all calculators.
/// Calculators are stateless and thread-safe.
pub trait Calculator: Send + Sync {
    /// Calculates a result based on the provided inputs.
    /// The result is returned as a string to support complex objects via JSON.
    fn calculate(&self, inputs: &CalculatorInputs) -> Result<String>;
}

/// Provides a safe interface for calculators to access input variables.
#[derive(Debug)]
pub struct CalculatorInputs<'a> {
    variables: &'a HashMap<String, FactValue>,
}

impl<'a> CalculatorInputs<'a> {
    /// Creates a new `CalculatorInputs`.
    pub fn new(variables: &'a HashMap<String, FactValue>) -> Self {
        Self { variables }
    }

    /// Gets an array value from the inputs.
    pub fn get_array(&self, name: &str) -> Result<&'a Vec<FactValue>> {
        match self.variables.get(name) {
            Some(FactValue::Array(arr)) => Ok(arr),
            Some(_) => Err(anyhow!(
                "Input '{}' was found, but it is not an array.",
                name
            )),
            None => Err(anyhow!("Required input array '{}' was not found.", name)),
        }
    }

    /// Gets a string value from the inputs.
    pub fn get_string(&self, name: &str) -> Result<String> {
        match self.variables.get(name) {
            Some(FactValue::String(s)) => Ok(s.clone()),
            Some(_) => Err(anyhow!(
                "Input '{}' was found, but it is not a string.",
                name
            )),
            None => Err(anyhow!("Required input string '{}' was not found.", name)),
        }
    }

    /// Gets a floating-point number value from the inputs.
    pub fn get_f64(&self, name: &str) -> Result<f64> {
        match self.variables.get(name) {
            Some(FactValue::Float(f)) => Ok(*f),
            Some(FactValue::Integer(i)) => Ok(*i as f64),
            Some(_) => Err(anyhow!(
                "Input '{}' was found, but it is not a number.",
                name
            )),
            None => Err(anyhow!("Required input number '{}' was not found.", name)),
        }
    }

    /// Gets a boolean value from the inputs.
    pub fn get_bool(&self, name: &str) -> Result<bool> {
        match self.variables.get(name) {
            Some(FactValue::Boolean(b)) => Ok(*b),
            Some(_) => Err(anyhow!(
                "Input '{}' was found, but it is not a boolean.",
                name
            )),
            None => Err(anyhow!("Required input boolean '{}' was not found.", name)),
        }
    }
}

// Re-export calculator implementations
pub use limit_validator::LimitValidateCalculator;
pub use threshold_check::ThresholdCheckCalculator;
