use super::{Calculator, CalculatorInputs};
use crate::types::{CalculatorError, CalculatorFieldType, ErrorCode, FactValue, FieldSpec};
use std::collections::HashMap;

/// Generic threshold checker for compliance, limits, and validations
/// Use cases:
/// - Student visa hours compliance
/// - Overtime threshold checking
/// - Budget limit validation  
/// - Performance target assessment
/// - Cost control limits
/// - Working time directive compliance
pub struct ThresholdChecker;

impl Calculator for ThresholdChecker {
    fn calculate(&self, inputs: &CalculatorInputs) -> Result<FactValue, CalculatorError> {
        let value = inputs.get_float("value")?;
        let threshold = inputs.get_float("threshold")?;
        let operator = inputs.get_string("operator").unwrap_or("LessThanOrEqual".to_string());

        if threshold < 0.0 {
            return Err(CalculatorError {
                code: ErrorCode::InvalidFieldValue,
                message: "Threshold cannot be negative".to_string(),
                details: Some(HashMap::from([(
                    "threshold".to_string(),
                    FactValue::Float(threshold),
                )])),
            });
        }

        let (passes, violation_amount) = match operator.as_str() {
            "LessThan" => (value < threshold, if value >= threshold { value - threshold } else { 0.0 }),
            "LessThanOrEqual" => (value <= threshold, if value > threshold { value - threshold } else { 0.0 }),
            "GreaterThan" => (value > threshold, if value <= threshold { threshold - value } else { 0.0 }),
            "GreaterThanOrEqual" => (value >= threshold, if value < threshold { threshold - value } else { 0.0 }),
            "Equal" => (value == threshold, (value - threshold).abs()),
            "NotEqual" => (value != threshold, 0.0),
            _ => return Err(CalculatorError {
                code: ErrorCode::InvalidFieldValue,
                message: format!("Unsupported operator: {}", operator),
                details: Some(HashMap::from([
                    ("operator".to_string(), FactValue::String(operator)),
                    ("supported_operators".to_string(), FactValue::String("LessThan, LessThanOrEqual, GreaterThan, GreaterThanOrEqual, Equal, NotEqual".to_string())),
                ])),
            }),
        };

        // Return structured result
        let mut result = HashMap::new();
        result.insert("passes".to_string(), FactValue::Boolean(passes));
        result.insert("value".to_string(), FactValue::Float(value));
        result.insert("threshold".to_string(), FactValue::Float(threshold));
        result.insert("operator".to_string(), FactValue::String(operator));
        result.insert(
            "violation_amount".to_string(),
            FactValue::Float(violation_amount),
        );
        result.insert(
            "status".to_string(),
            FactValue::String(if passes {
                "compliant".to_string()
            } else {
                "non_compliant".to_string()
            }),
        );

        Ok(FactValue::Object(result))
    }

    fn name(&self) -> &str {
        "threshold_checker"
    }

    fn required_fields(&self) -> &[FieldSpec] {
        &[
            FieldSpec { name: "value", field_type: CalculatorFieldType::Float, required: true },
            FieldSpec { name: "threshold", field_type: CalculatorFieldType::Float, required: true },
            FieldSpec {
                name: "operator",
                field_type: CalculatorFieldType::String,
                required: false,
            },
        ]
    }
}

/// Enhanced limit validator with multiple thresholds and severity levels
/// Use cases:
/// - Multi-tier overtime rates (40h normal, 48h time-and-half, 60h double-time)
/// - Budget alerts (80% warning, 95% critical, 100% breach)
/// - Performance ratings (bronze/silver/gold thresholds)
/// - Cost tier calculation
pub struct LimitValidator;

impl Calculator for LimitValidator {
    fn calculate(&self, inputs: &CalculatorInputs) -> Result<FactValue, CalculatorError> {
        let value = inputs.get_float("value")?;
        let warning_threshold = inputs.get_optional_float("warning_threshold")?;
        let critical_threshold = inputs.get_optional_float("critical_threshold")?;
        let max_threshold = inputs.get_optional_float("max_threshold")?;

        if value < 0.0 {
            return Err(CalculatorError {
                code: ErrorCode::InvalidFieldValue,
                message: "Value cannot be negative".to_string(),
                details: Some(HashMap::from([(
                    "value".to_string(),
                    FactValue::Float(value),
                )])),
            });
        }

        let mut result = HashMap::new();
        result.insert("value".to_string(), FactValue::Float(value));

        // Determine severity level and compliance status
        let (severity, status, exceeded_threshold) = if let Some(max_limit) = max_threshold {
            if value > max_limit {
                (
                    "breach".to_string(),
                    "non_compliant".to_string(),
                    Some(max_limit),
                )
            } else if let Some(critical_limit) = critical_threshold {
                if value > critical_limit {
                    (
                        "critical".to_string(),
                        "at_risk".to_string(),
                        Some(critical_limit),
                    )
                } else if let Some(warning_limit) = warning_threshold {
                    if value > warning_limit {
                        (
                            "warning".to_string(),
                            "caution".to_string(),
                            Some(warning_limit),
                        )
                    } else {
                        ("normal".to_string(), "compliant".to_string(), None)
                    }
                } else {
                    ("normal".to_string(), "compliant".to_string(), None)
                }
            } else if let Some(warning_limit) = warning_threshold {
                if value > warning_limit {
                    (
                        "warning".to_string(),
                        "caution".to_string(),
                        Some(warning_limit),
                    )
                } else {
                    ("normal".to_string(), "compliant".to_string(), None)
                }
            } else {
                ("normal".to_string(), "compliant".to_string(), None)
            }
        } else if let Some(critical_limit) = critical_threshold {
            if value > critical_limit {
                (
                    "critical".to_string(),
                    "at_risk".to_string(),
                    Some(critical_limit),
                )
            } else if let Some(warning_limit) = warning_threshold {
                if value > warning_limit {
                    (
                        "warning".to_string(),
                        "caution".to_string(),
                        Some(warning_limit),
                    )
                } else {
                    ("normal".to_string(), "compliant".to_string(), None)
                }
            } else {
                ("normal".to_string(), "compliant".to_string(), None)
            }
        } else if let Some(warning_limit) = warning_threshold {
            if value > warning_limit {
                (
                    "warning".to_string(),
                    "caution".to_string(),
                    Some(warning_limit),
                )
            } else {
                ("normal".to_string(), "compliant".to_string(), None)
            }
        } else {
            ("normal".to_string(), "compliant".to_string(), None)
        };

        result.insert("severity".to_string(), FactValue::String(severity));
        result.insert("status".to_string(), FactValue::String(status));

        if let Some(threshold) = exceeded_threshold {
            result.insert(
                "exceeded_threshold".to_string(),
                FactValue::Float(threshold),
            );
            result.insert(
                "excess_amount".to_string(),
                FactValue::Float(value - threshold),
            );
        }

        // Add threshold information
        if let Some(warning) = warning_threshold {
            result.insert("warning_threshold".to_string(), FactValue::Float(warning));
        }
        if let Some(critical) = critical_threshold {
            result.insert("critical_threshold".to_string(), FactValue::Float(critical));
        }
        if let Some(max_limit) = max_threshold {
            result.insert("max_threshold".to_string(), FactValue::Float(max_limit));
        }

        // Calculate utilization percentage if max threshold is provided
        if let Some(max_limit) = max_threshold {
            let utilization = (value / max_limit) * 100.0;
            result.insert(
                "utilization_percent".to_string(),
                FactValue::Float(utilization),
            );
        }

        Ok(FactValue::Object(result))
    }

    fn name(&self) -> &str {
        "limit_validator"
    }

    fn required_fields(&self) -> &[FieldSpec] {
        &[
            FieldSpec { name: "value", field_type: CalculatorFieldType::Float, required: true },
            FieldSpec {
                name: "warning_threshold",
                field_type: CalculatorFieldType::Float,
                required: false,
            },
            FieldSpec {
                name: "critical_threshold",
                field_type: CalculatorFieldType::Float,
                required: false,
            },
            FieldSpec {
                name: "max_threshold",
                field_type: CalculatorFieldType::Float,
                required: false,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculators::CalculatorInputs;
    use std::collections::HashMap;

    #[test]
    fn test_threshold_checker_compliant() {
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), FactValue::Float(15.0));
        inputs.insert("threshold".to_string(), FactValue::Float(20.0));
        inputs.insert(
            "operator".to_string(),
            FactValue::String("LessThanOrEqual".to_string()),
        );

        let calc_inputs = CalculatorInputs::new(&inputs);
        let calculator = ThresholdChecker;

        let result = calculator.calculate(&calc_inputs).unwrap();

        match result {
            FactValue::Object(obj) => {
                assert_eq!(obj.get("passes"), Some(&FactValue::Boolean(true)));
                assert_eq!(
                    obj.get("status"),
                    Some(&FactValue::String("compliant".to_string()))
                );
                assert_eq!(obj.get("violation_amount"), Some(&FactValue::Float(0.0)));
            }
            _ => panic!("Expected object result"),
        }
    }

    #[test]
    fn test_threshold_checker_non_compliant() {
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), FactValue::Float(25.0));
        inputs.insert("threshold".to_string(), FactValue::Float(20.0));
        inputs.insert(
            "operator".to_string(),
            FactValue::String("LessThanOrEqual".to_string()),
        );

        let calc_inputs = CalculatorInputs::new(&inputs);
        let calculator = ThresholdChecker;

        let result = calculator.calculate(&calc_inputs).unwrap();

        match result {
            FactValue::Object(obj) => {
                assert_eq!(obj.get("passes"), Some(&FactValue::Boolean(false)));
                assert_eq!(
                    obj.get("status"),
                    Some(&FactValue::String("non_compliant".to_string()))
                );
                assert_eq!(obj.get("violation_amount"), Some(&FactValue::Float(5.0)));
            }
            _ => panic!("Expected object result"),
        }
    }

    #[test]
    fn test_limit_validator_multi_tier() {
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), FactValue::Float(85.0));
        inputs.insert("warning_threshold".to_string(), FactValue::Float(80.0));
        inputs.insert("critical_threshold".to_string(), FactValue::Float(95.0));
        inputs.insert("max_threshold".to_string(), FactValue::Float(100.0));

        let calc_inputs = CalculatorInputs::new(&inputs);
        let calculator = LimitValidator;

        let result = calculator.calculate(&calc_inputs).unwrap();

        match result {
            FactValue::Object(obj) => {
                assert_eq!(
                    obj.get("severity"),
                    Some(&FactValue::String("warning".to_string()))
                );
                assert_eq!(
                    obj.get("status"),
                    Some(&FactValue::String("caution".to_string()))
                );
                assert_eq!(
                    obj.get("utilization_percent"),
                    Some(&FactValue::Float(85.0))
                );
                assert_eq!(obj.get("exceeded_threshold"), Some(&FactValue::Float(80.0)));
                assert_eq!(obj.get("excess_amount"), Some(&FactValue::Float(5.0)));
            }
            _ => panic!("Expected object result"),
        }
    }
}
