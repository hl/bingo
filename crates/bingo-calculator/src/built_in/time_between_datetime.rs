//! Calculator for computing time differences between datetime values
//!
//! This calculator computes the difference between two datetime values and returns
//! the result in the specified unit (seconds, minutes, hours, days).

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use bingo_types::FactValue;

use crate::plugin::{CalculationResult, CalculatorPlugin};

fn parse_datetime(value: &FactValue) -> Result<DateTime<Utc>, String> {
    match value {
        FactValue::String(s) => Ok(DateTime::parse_from_rfc3339(s)
            .map_err(|e| format!("Invalid datetime '{s}': {e}"))?
            .with_timezone(&Utc)),
        _ => Err("Invalid datetime argument: expected string".to_string()),
    }
}

#[derive(Debug, Default)]
pub struct TimeBetweenDatetimeCalculator;

impl CalculatorPlugin for TimeBetweenDatetimeCalculator {
    fn name(&self) -> &str {
        "time_between_datetime"
    }

    fn calculate(&self, args: &HashMap<String, &FactValue>) -> CalculationResult {
        let start_dt = match args.get("start") {
            Some(value) => parse_datetime(value)?,
            None => return Err("missing 'start' argument".to_string()),
        };

        let end_dt = match args.get("end") {
            Some(value) => parse_datetime(value)?,
            None => return Err("missing 'end' argument".to_string()),
        };

        let unit = match args.get("unit") {
            Some(FactValue::String(s)) => s.to_lowercase(),
            Some(_) => return Err("unit argument must be a string".to_string()),
            None => "hours".to_string(),
        };

        let duration = end_dt - start_dt;

        let value = match unit.as_str() {
            "seconds" => duration.num_seconds() as f64,
            "minutes" => duration.num_seconds() as f64 / 60.0,
            _ => duration.num_seconds() as f64 / 3600.0, // default hours
        };

        Ok(FactValue::Float(value))
    }
}
