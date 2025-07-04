use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// Built-in Calculator Error Handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatorError {
    pub code: ErrorCode,
    pub message: String,
    pub details: Option<HashMap<String, FactValue>>,
}

impl std::error::Error for CalculatorError {}

impl fmt::Display for CalculatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Calculator error: {}", self.message)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
    MissingRequiredField,
    InvalidFieldType,
    InvalidFieldValue,
    CalculationOverflow,
    BusinessRuleViolation,
    ConfigurationError,
}

#[derive(Debug, Clone)]
pub struct FieldSpec {
    pub name: &'static str,
    pub field_type: CalculatorFieldType,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub enum CalculatorFieldType {
    Integer,
    Float,
    String,
    Boolean,
    DateTime,
}

/// Possible values that can be stored in a fact
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

impl std::hash::Hash for FactValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            FactValue::String(s) => {
                0u8.hash(state);
                s.hash(state);
            }
            FactValue::Integer(i) => {
                1u8.hash(state);
                i.hash(state);
            }
            FactValue::Float(f) => {
                2u8.hash(state);
                f.to_bits().hash(state); // Use bits representation for consistent hashing
            }
            FactValue::Boolean(b) => {
                3u8.hash(state);
                b.hash(state);
            }
            FactValue::Array(arr) => {
                4u8.hash(state);
                arr.hash(state);
            }
            FactValue::Object(obj) => {
                5u8.hash(state);
                // Sort keys for consistent hashing
                let mut sorted_pairs: Vec<_> = obj.iter().collect();
                sorted_pairs.sort_by_key(|(k, _)| *k);
                for (key, value) in sorted_pairs {
                    key.hash(state);
                    value.hash(state);
                }
            }
            FactValue::Date(dt) => {
                6u8.hash(state);
                dt.timestamp().hash(state);
            }
            FactValue::Null => {
                7u8.hash(state);
            }
        }
    }
}

impl fmt::Display for FactValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FactValue::String(s) => write!(f, "\"{}\"", s),
            FactValue::Integer(i) => write!(f, "{}", i),
            FactValue::Float(fl) => write!(f, "{}", fl),
            FactValue::Boolean(b) => write!(f, "{}", b),
            FactValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            FactValue::Object(obj) => {
                let pairs: Vec<String> =
                    obj.iter().map(|(k, v)| format!("\"{}\": {}", k, v)).collect();
                write!(f, "{{{}}}", pairs.join(", "))
            }
            FactValue::Date(dt) => write!(f, "\"{}\"", dt.to_rfc3339()),
            FactValue::Null => write!(f, "null"),
        }
    }
}

impl From<String> for FactValue {
    fn from(value: String) -> Self {
        FactValue::String(value)
    }
}

impl From<&str> for FactValue {
    fn from(value: &str) -> Self {
        FactValue::String(value.to_string())
    }
}

impl From<i64> for FactValue {
    fn from(value: i64) -> Self {
        FactValue::Integer(value)
    }
}

impl From<f64> for FactValue {
    fn from(value: f64) -> Self {
        FactValue::Float(value)
    }
}

impl From<bool> for FactValue {
    fn from(value: bool) -> Self {
        FactValue::Boolean(value)
    }
}

impl From<DateTime<Utc>> for FactValue {
    fn from(value: DateTime<Utc>) -> Self {
        FactValue::Date(value)
    }
}

impl FactValue {
    pub fn as_string(&self) -> Option<&String> {
        match self {
            FactValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            FactValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            FactValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            FactValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_date(&self) -> Option<&DateTime<Utc>> {
        match self {
            FactValue::Date(dt) => Some(dt),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<FactValue>> {
        match self {
            FactValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, FactValue>> {
        match self {
            FactValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, FactValue::Null)
    }

    /// Convert to number if possible (integer or float)
    pub fn as_number(&self) -> Option<f64> {
        match self {
            FactValue::Integer(i) => Some(*i as f64),
            FactValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Type checking utility
    pub fn type_name(&self) -> &'static str {
        match self {
            FactValue::String(_) => "string",
            FactValue::Integer(_) => "integer",
            FactValue::Float(_) => "float",
            FactValue::Boolean(_) => "boolean",
            FactValue::Array(_) => "array",
            FactValue::Object(_) => "object",
            FactValue::Date(_) => "date",
            FactValue::Null => "null",
        }
    }

    /// Check if this value is "truthy" for conditional logic
    pub fn is_truthy(&self) -> bool {
        match self {
            FactValue::Boolean(b) => *b,
            FactValue::Integer(i) => *i != 0,
            FactValue::Float(f) => *f != 0.0,
            FactValue::String(s) => !s.is_empty(),
            FactValue::Array(arr) => !arr.is_empty(),
            FactValue::Object(obj) => !obj.is_empty(),
            FactValue::Date(_) => true, // Dates are always truthy
            FactValue::Null => false,
        }
    }

    /// Convert to a normalized comparison value
    pub fn to_comparable(&self) -> Option<f64> {
        match self {
            FactValue::Integer(i) => Some(*i as f64),
            FactValue::Float(f) => Some(*f),
            FactValue::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            FactValue::Date(dt) => Some(dt.timestamp() as f64),
            FactValue::Array(arr) => Some(arr.len() as f64), // Length for comparison
            FactValue::Object(obj) => Some(obj.len() as f64), // Length for comparison
            FactValue::String(_) | FactValue::Null => None,
        }
    }

    /// Try to convert to a string (returns String directly, not Option)
    pub fn as_string_direct(&self) -> String {
        match self {
            FactValue::String(s) => s.clone(),
            other => other.to_string(),
        }
    }

    /// Create date from ISO string
    pub fn date_from_iso(iso_string: &str) -> Result<Self, chrono::ParseError> {
        use chrono::{DateTime, Utc};
        Ok(FactValue::Date(
            DateTime::parse_from_rfc3339(iso_string)?.with_timezone(&Utc),
        ))
    }

    /// Check if two FactValues are compatible for comparison
    pub const fn is_compatible_with(&self, other: &Self) -> bool {
        use FactValue::*;
        match (self, other) {
            (String(_), String(_))
            | (Integer(_), Integer(_))
            | (Float(_), Float(_))
            | (Boolean(_), Boolean(_))
            | (Date(_), Date(_))
            | (Array(_), Array(_))
            | (Object(_), Object(_))
            | (Null, Null) => true,
            (Integer(_), Float(_)) | (Float(_), Integer(_)) => true,
            // Null is compatible with everything for equality checks
            (Null, _) | (_, Null) => true,
            _ => false,
        }
    }
}

impl Eq for FactValue {}

impl PartialOrd for FactValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use FactValue::*;
        match (self, other) {
            (String(a), String(b)) => a.partial_cmp(b),
            (Integer(a), Integer(b)) => a.partial_cmp(b),
            (Float(a), Float(b)) => a.partial_cmp(b),
            (Boolean(a), Boolean(b)) => a.partial_cmp(b),
            (Date(a), Date(b)) => a.partial_cmp(b),
            (Null, Null) => Some(std::cmp::Ordering::Equal),
            // Cross-type comparisons: convert to same type if possible
            (Integer(a), Float(b)) => (*a as f64).partial_cmp(b),
            (Float(a), Integer(b)) => a.partial_cmp(&(*b as f64)),
            // For incompatible types, no ordering
            _ => None,
        }
    }
}
