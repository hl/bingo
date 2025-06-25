use super::{Calculator, CalculatorInputs};
use crate::types::{CalculatorError, CalculatorFieldType, ErrorCode, FactValue, FieldSpec};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Generic calculator for computing hours between two datetime fields
/// Use cases: shift duration, overtime calculation, billing hours, project time tracking
pub struct HoursBetweenDateTimeCalculator;

impl Calculator for HoursBetweenDateTimeCalculator {
    fn calculate(&self, inputs: &CalculatorInputs) -> Result<FactValue, CalculatorError> {
        let start_field = inputs.get_string("start_field")?;
        let end_field = inputs.get_string("end_field")?;

        // Get the actual datetime values using the field names
        let start_datetime = self.parse_datetime_from_field(inputs, &start_field)?;
        let end_datetime = self.parse_datetime_from_field(inputs, &end_field)?;

        if end_datetime < start_datetime {
            return Err(CalculatorError {
                code: ErrorCode::InvalidFieldValue,
                message: "End datetime cannot be before start datetime".to_string(),
                details: Some(HashMap::from([
                    (
                        "start_datetime".to_string(),
                        FactValue::String(start_datetime.to_rfc3339()),
                    ),
                    (
                        "end_datetime".to_string(),
                        FactValue::String(end_datetime.to_rfc3339()),
                    ),
                ])),
            });
        }

        let duration = end_datetime.signed_duration_since(start_datetime);
        let hours = duration.num_seconds() as f64 / 3600.0;

        Ok(FactValue::Float(hours))
    }

    fn name(&self) -> &str {
        "hours_between_datetime"
    }

    fn required_fields(&self) -> &[FieldSpec] {
        &[
            FieldSpec {
                name: "start_field",
                field_type: CalculatorFieldType::String,
                required: true,
            },
            FieldSpec {
                name: "end_field",
                field_type: CalculatorFieldType::String,
                required: true,
            },
        ]
    }
}

impl HoursBetweenDateTimeCalculator {
    fn parse_datetime_from_field(
        &self,
        inputs: &CalculatorInputs,
        field_name: &str,
    ) -> Result<DateTime<Utc>, CalculatorError> {
        let datetime_str = inputs.get_string(field_name)?;

        DateTime::parse_from_rfc3339(&datetime_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| CalculatorError {
                code: ErrorCode::InvalidFieldValue,
                message: format!(
                    "Invalid datetime format in field '{}': {}",
                    field_name, datetime_str
                ),
                details: Some(HashMap::from([
                    (
                        "field".to_string(),
                        FactValue::String(field_name.to_string()),
                    ),
                    ("value".to_string(), FactValue::String(datetime_str)),
                    (
                        "expected_format".to_string(),
                        FactValue::String("RFC3339 (ISO8601)".to_string()),
                    ),
                ])),
            })
    }
}

/// Generic time difference calculator with configurable units
/// Use cases: break duration, meeting length, project phases, billing periods
pub struct TimeDifferenceCalculator;

impl Calculator for TimeDifferenceCalculator {
    fn calculate(&self, inputs: &CalculatorInputs) -> Result<FactValue, CalculatorError> {
        let start_field = inputs.get_string("start_field")?;
        let end_field = inputs.get_string("end_field")?;
        let unit = inputs.get_string("unit").unwrap_or("hours".to_string());

        let start_datetime = self.parse_datetime_from_field(inputs, &start_field)?;
        let end_datetime = self.parse_datetime_from_field(inputs, &end_field)?;

        if end_datetime < start_datetime {
            return Err(CalculatorError {
                code: ErrorCode::InvalidFieldValue,
                message: "End datetime cannot be before start datetime".to_string(),
                details: Some(HashMap::from([
                    (
                        "start_datetime".to_string(),
                        FactValue::String(start_datetime.to_rfc3339()),
                    ),
                    (
                        "end_datetime".to_string(),
                        FactValue::String(end_datetime.to_rfc3339()),
                    ),
                ])),
            });
        }

        let duration = end_datetime.signed_duration_since(start_datetime);

        let result = match unit.as_str() {
            "seconds" => duration.num_seconds() as f64,
            "minutes" => duration.num_minutes() as f64,
            "hours" => duration.num_seconds() as f64 / 3600.0,
            "days" => duration.num_days() as f64,
            _ => {
                return Err(CalculatorError {
                    code: ErrorCode::InvalidFieldValue,
                    message: format!("Unsupported time unit: {}", unit),
                    details: Some(HashMap::from([
                        ("unit".to_string(), FactValue::String(unit)),
                        (
                            "supported_units".to_string(),
                            FactValue::String("seconds, minutes, hours, days".to_string()),
                        ),
                    ])),
                });
            }
        };

        // Return structured result with multiple formats
        let mut result_obj = HashMap::new();
        result_obj.insert("value".to_string(), FactValue::Float(result));
        result_obj.insert("unit".to_string(), FactValue::String(unit));
        result_obj.insert(
            "hours".to_string(),
            FactValue::Float(duration.num_seconds() as f64 / 3600.0),
        );
        result_obj.insert(
            "minutes".to_string(),
            FactValue::Float(duration.num_minutes() as f64),
        );

        Ok(FactValue::Object(result_obj))
    }

    fn name(&self) -> &str {
        "time_difference"
    }

    fn required_fields(&self) -> &[FieldSpec] {
        &[
            FieldSpec {
                name: "start_field",
                field_type: CalculatorFieldType::String,
                required: true,
            },
            FieldSpec {
                name: "end_field",
                field_type: CalculatorFieldType::String,
                required: true,
            },
            FieldSpec { name: "unit", field_type: CalculatorFieldType::String, required: false },
        ]
    }
}

impl TimeDifferenceCalculator {
    fn parse_datetime_from_field(
        &self,
        inputs: &CalculatorInputs,
        field_name: &str,
    ) -> Result<DateTime<Utc>, CalculatorError> {
        let datetime_str = inputs.get_string(field_name)?;

        DateTime::parse_from_rfc3339(&datetime_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| CalculatorError {
                code: ErrorCode::InvalidFieldValue,
                message: format!(
                    "Invalid datetime format in field '{}': {}",
                    field_name, datetime_str
                ),
                details: Some(HashMap::from([
                    (
                        "field".to_string(),
                        FactValue::String(field_name.to_string()),
                    ),
                    ("value".to_string(), FactValue::String(datetime_str)),
                    (
                        "expected_format".to_string(),
                        FactValue::String("RFC3339 (ISO8601)".to_string()),
                    ),
                ])),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculators::CalculatorInputs;
    use std::collections::HashMap;

    #[test]
    fn test_hours_between_datetime() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "start_field".to_string(),
            FactValue::String("start_datetime".to_string()),
        );
        inputs.insert(
            "end_field".to_string(),
            FactValue::String("finish_datetime".to_string()),
        );
        inputs.insert(
            "start_datetime".to_string(),
            FactValue::String("2024-06-17T09:00:00Z".to_string()),
        );
        inputs.insert(
            "finish_datetime".to_string(),
            FactValue::String("2024-06-17T17:00:00Z".to_string()),
        );

        let calc_inputs = CalculatorInputs::new(&inputs);
        let calculator = HoursBetweenDateTimeCalculator;

        let result = calculator.calculate(&calc_inputs).unwrap();

        match result {
            FactValue::Float(hours) => {
                assert_eq!(hours, 8.0);
            }
            _ => panic!("Expected float result"),
        }
    }

    #[test]
    fn test_time_difference_with_units() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "start_field".to_string(),
            FactValue::String("start_datetime".to_string()),
        );
        inputs.insert(
            "end_field".to_string(),
            FactValue::String("finish_datetime".to_string()),
        );
        inputs.insert("unit".to_string(), FactValue::String("minutes".to_string()));
        inputs.insert(
            "start_datetime".to_string(),
            FactValue::String("2024-06-17T09:00:00Z".to_string()),
        );
        inputs.insert(
            "finish_datetime".to_string(),
            FactValue::String("2024-06-17T10:30:00Z".to_string()),
        );

        let calc_inputs = CalculatorInputs::new(&inputs);
        let calculator = TimeDifferenceCalculator;

        let result = calculator.calculate(&calc_inputs).unwrap();

        match result {
            FactValue::Object(obj) => {
                assert_eq!(obj.get("value"), Some(&FactValue::Float(90.0))); // 90 minutes
                assert_eq!(
                    obj.get("unit"),
                    Some(&FactValue::String("minutes".to_string()))
                );
                assert_eq!(obj.get("hours"), Some(&FactValue::Float(1.5))); // 1.5 hours
            }
            _ => panic!("Expected object result"),
        }
    }
}
