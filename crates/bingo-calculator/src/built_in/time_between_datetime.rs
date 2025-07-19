//! Calculator for computing time differences between datetime values with workday support
//!
//! This calculator computes time differences between two datetime values and supports
//! workday-aware calculations that can split time periods around workday boundaries.
//!
//! # Parameters
//! - `start_datetime` (required): RFC3339 formatted start datetime string
//! - `finish_datetime` (required): RFC3339 formatted finish datetime string  
//! - `workday` (optional): Object with `hours` and `minutes` defining workday start time
//! - `part` (optional): "time_before" or "time_after" - which part relative to workday boundary
//! - `units` (optional): "hours" (default) or "minutes" - output unit
//!
//! # Examples
//! Simple calculation: Returns total time between start_datetime and finish_datetime
//! Workday calculation: Returns time before/after workday boundary within the time period

use std::collections::HashMap;

use chrono::{DateTime, Duration, NaiveTime, Utc};
use serde_json::Value;

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

fn parse_workday_time(value: &FactValue) -> Result<NaiveTime, String> {
    match value {
        FactValue::String(s) => {
            // Try to parse as JSON object
            let json: Value = serde_json::from_str(s).map_err(|_| {
                "Workday must be a JSON object with 'hours' and 'minutes' fields".to_string()
            })?;

            let hours = json
                .get("hours")
                .and_then(|h| h.as_u64())
                .ok_or("Workday object must have 'hours' field as number")?;

            let minutes = json
                .get("minutes")
                .and_then(|m| m.as_u64())
                .ok_or("Workday object must have 'minutes' field as number")?;

            if hours > 23 {
                return Err("Workday hours must be 0-23".to_string());
            }
            if minutes > 59 {
                return Err("Workday minutes must be 0-59".to_string());
            }

            NaiveTime::from_hms_opt(hours as u32, minutes as u32, 0)
                .ok_or("Invalid workday time".to_string())
        }
        _ => Err("Workday argument must be a JSON string".to_string()),
    }
}

fn calculate_workday_split(
    start_dt: DateTime<Utc>,
    finish_dt: DateTime<Utc>,
    workday_time: NaiveTime,
    part: &str,
) -> Result<Duration, String> {
    // Find the workday boundary within the time period
    // We need to determine which date to apply the workday time to

    let start_date = start_dt.date_naive();
    let finish_date = finish_dt.date_naive();

    // Create workday boundary datetime on the start date
    let mut workday_boundary = start_date.and_time(workday_time).and_utc();

    // If the workday boundary is before our start time on the same day,
    // move it to the next day
    if workday_boundary <= start_dt && start_date == finish_date {
        workday_boundary = start_date
            .succ_opt()
            .ok_or("Date overflow calculating workday boundary")?
            .and_time(workday_time)
            .and_utc();
    }

    // If we span multiple days, find the appropriate workday boundary
    if start_date != finish_date {
        // For multi-day periods, use the workday boundary on the day after start
        if workday_boundary <= start_dt {
            workday_boundary = start_date
                .succ_opt()
                .ok_or("Date overflow calculating workday boundary")?
                .and_time(workday_time)
                .and_utc();
        }
    }

    // Ensure workday boundary is within our time period
    if workday_boundary < start_dt {
        workday_boundary = start_dt;
    }
    if workday_boundary > finish_dt {
        workday_boundary = finish_dt;
    }

    match part {
        "time_before" => {
            // Time from start to workday boundary
            Ok(workday_boundary - start_dt)
        }
        "time_after" => {
            // Time from workday boundary to finish
            Ok(finish_dt - workday_boundary)
        }
        _ => Err(format!(
            "Invalid part '{part}': must be 'time_before' or 'time_after'"
        )),
    }
}

#[derive(Debug, Default)]
pub struct TimeBetweenDatetimeCalculator;

impl CalculatorPlugin for TimeBetweenDatetimeCalculator {
    fn name(&self) -> &str {
        "time_between_datetime"
    }

    fn calculate(&self, args: &HashMap<String, &FactValue>) -> CalculationResult {
        // Parse required parameters
        let start_dt = match args.get("start_datetime") {
            Some(value) => parse_datetime(value)?,
            None => return Err("missing 'start_datetime' argument".to_string()),
        };

        let finish_dt = match args.get("finish_datetime") {
            Some(value) => parse_datetime(value)?,
            None => return Err("missing 'finish_datetime' argument".to_string()),
        };

        // Parse optional units parameter
        let units = match args.get("units") {
            Some(FactValue::String(s)) => s.to_lowercase(),
            Some(_) => return Err("units argument must be a string".to_string()),
            None => "hours".to_string(),
        };

        // Check if workday mode is requested
        let duration = if let (Some(workday_value), Some(part_value)) =
            (args.get("workday"), args.get("part"))
        {
            // Workday mode
            let workday_time = parse_workday_time(workday_value)?;

            let part = match part_value {
                FactValue::String(s) => s.as_str(),
                _ => return Err("part argument must be a string".to_string()),
            };

            calculate_workday_split(start_dt, finish_dt, workday_time, part)?
        } else {
            // Simple mode - total time difference
            finish_dt - start_dt
        };

        // Convert duration to requested units
        let value = match units.as_str() {
            "seconds" => duration.num_seconds() as f64,
            "minutes" => duration.num_seconds() as f64 / 60.0,
            "hours" => duration.num_seconds() as f64 / 3600.0,
            _ => {
                return Err(format!(
                    "Unsupported units '{units}': must be 'seconds', 'minutes', or 'hours'"
                ));
            }
        };

        Ok(FactValue::Float(value))
    }
}
