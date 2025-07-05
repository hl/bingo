//! Time Between Datetime Calculator
//!
//! Computes the duration between two ISO-8601 datetimes.
//! Inputs:
//!   * `start_datetime` / `start` / `from`
//!   * `end_datetime` / `end` / `to`
//!   * optional `unit`: `hours` (default) | `minutes` | `seconds`

use crate::{Calculator, CalculatorInputs};
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};

fn parse_datetime(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)
        .map_err(|e| anyhow!("Invalid datetime '{}': {}", value, e))?
        .with_timezone(&Utc))
}

#[derive(Debug, Default)]
pub struct TimeBetweenDatetimeCalculator;

impl Calculator for TimeBetweenDatetimeCalculator {
    fn calculate(&self, inputs: &CalculatorInputs) -> Result<String> {
        let start_raw = inputs
            .get_string("start_datetime")
            .or_else(|_| inputs.get_string("start"))
            .or_else(|_| inputs.get_string("from"))?;

        let end_raw = inputs
            .get_string("end_datetime")
            .or_else(|_| inputs.get_string("end"))
            .or_else(|_| inputs.get_string("to"))?;

        let start_dt = parse_datetime(&start_raw)?;
        let end_dt = parse_datetime(&end_raw)?;

        let duration = end_dt - start_dt;

        let unit = inputs.get_string("unit").unwrap_or_else(|_| "hours".to_string()).to_lowercase();

        let value = match unit.as_str() {
            "seconds" => duration.num_seconds() as f64,
            "minutes" => duration.num_seconds() as f64 / 60.0,
            _ => duration.num_seconds() as f64 / 3600.0, // default hours
        };

        Ok(value.to_string())
    }
}
