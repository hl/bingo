use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

/// Possible values that can be stored in a fact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FactValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Floating point value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of `FactValues`
    Array(Vec<FactValue>),
    /// Object/map of string keys to `FactValues`
    Object(HashMap<String, FactValue>),
    /// UTC date/time value
    Date(DateTime<Utc>),
    /// Null value
    Null,
}

// -------------------------------------------------------------------------------------------------
// Conversions between internal `FactValue` and `serde_json::Value`.
// These allow the API layer to reuse the same data structures without the verbose
// hand-written mapping code that previously existed in `bingo-api/src/types.rs`.
// The implementation purposefully keeps the mapping logic close to the data type it
// concerns, making it considerably easier to maintain and discover.
// -------------------------------------------------------------------------------------------------

impl From<FactValue> for serde_json::Value {
    fn from(value: FactValue) -> Self {
        match value {
            FactValue::String(s) => Self::String(s),
            FactValue::Integer(i) => Self::Number(serde_json::Number::from(i)),
            FactValue::Float(f) => serde_json::Number::from_f64(f).map_or(Self::Null, Self::Number),
            FactValue::Boolean(b) => Self::Bool(b),
            FactValue::Array(arr) => {
                let vec: Vec<Self> = arr.into_iter().map(std::convert::Into::into).collect();
                Self::Array(vec)
            }
            FactValue::Object(map) => {
                let json_map = map
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<serde_json::Map<String, Self>>();
                Self::Object(json_map)
            }
            FactValue::Date(dt) => Self::String(dt.to_rfc3339()),
            FactValue::Null => Self::Null,
        }
    }
}

impl From<&FactValue> for serde_json::Value {
    fn from(value: &FactValue) -> Self {
        match value {
            FactValue::String(s) => Self::String(s.clone()),
            FactValue::Integer(i) => Self::Number(serde_json::Number::from(*i)),
            FactValue::Float(f) => {
                serde_json::Number::from_f64(*f).map_or(Self::Null, Self::Number)
            }
            FactValue::Boolean(b) => Self::Bool(*b),
            FactValue::Array(arr) => {
                let vec: Vec<Self> = arr.iter().map(std::convert::Into::into).collect();
                Self::Array(vec)
            }
            FactValue::Object(map) => {
                let json_map = map
                    .iter()
                    .map(|(k, v)| (k.clone(), v.into()))
                    .collect::<serde_json::Map<String, Self>>();
                Self::Object(json_map)
            }
            FactValue::Date(dt) => Self::String(dt.to_rfc3339()),
            FactValue::Null => Self::Null,
        }
    }
}

impl TryFrom<&serde_json::Value> for FactValue {
    type Error = anyhow::Error;

    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        Ok(match value {
            serde_json::Value::String(s) => Self::String(s.clone()),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Self::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Self::Float(f)
                } else {
                    return Err(anyhow!("Unsupported number value: {}", n));
                }
            }
            serde_json::Value::Bool(b) => Self::Boolean(*b),
            serde_json::Value::Array(arr) => {
                let inner = arr.iter().map(Self::try_from).collect::<Result<Vec<_>, _>>()?;
                Self::Array(inner)
            }
            serde_json::Value::Object(map) => {
                let mut inner = HashMap::new();
                for (k, v) in map {
                    inner.insert(k.clone(), Self::try_from(v)?);
                }
                Self::Object(inner)
            }
            serde_json::Value::Null => Self::Null,
        })
    }
}

impl std::hash::Hash for FactValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::String(s) => {
                0u8.hash(state);
                s.hash(state);
            }
            Self::Integer(i) => {
                1u8.hash(state);
                i.hash(state);
            }
            Self::Float(f) => {
                2u8.hash(state);
                f.to_bits().hash(state); // Use bits representation for consistent hashing
            }
            Self::Boolean(b) => {
                3u8.hash(state);
                b.hash(state);
            }
            Self::Array(arr) => {
                4u8.hash(state);
                arr.hash(state);
            }
            Self::Object(obj) => {
                5u8.hash(state);
                // Sort keys for consistent hashing
                let mut sorted_pairs: Vec<_> = obj.iter().collect();
                sorted_pairs.sort_by_key(|(k, _)| *k);
                for (key, value) in sorted_pairs {
                    key.hash(state);
                    value.hash(state);
                }
            }
            Self::Date(dt) => {
                6u8.hash(state);
                dt.timestamp_nanos_opt().unwrap_or(0).hash(state);
            }
            Self::Null => {
                7u8.hash(state);
            }
        }
    }
}

impl Eq for FactValue {}

impl PartialOrd for FactValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use FactValue::{Boolean, Date, Float, Integer, Null, String};
        match (self, other) {
            (String(a), String(b)) => a.partial_cmp(b),
            (Integer(a), Integer(b)) => a.partial_cmp(b),
            (Float(a), Float(b)) => a.partial_cmp(b),
            (Boolean(a), Boolean(b)) => a.partial_cmp(b),
            (Date(a), Date(b)) => a.partial_cmp(b),
            (Null, Null) => Some(std::cmp::Ordering::Equal),
            // Cross-type comparisons: convert to same type if possible
            #[allow(clippy::cast_precision_loss)]
            (Integer(a), Float(b)) => (*a as f64).partial_cmp(b),
            #[allow(clippy::cast_precision_loss)]
            (Float(a), Integer(b)) => a.partial_cmp(&(*b as f64)),
            // For incompatible types, no ordering
            _ => None,
        }
    }
}

impl fmt::Display for FactValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "{s}"),
            Self::Integer(i) => write!(f, "{i}"),
            Self::Float(fl) => write!(f, "{fl}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Self::Object(obj) => {
                write!(f, "{{")?;
                let mut first = true;
                for (key, value) in obj {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{key}: {value}")?;
                    first = false;
                }
                write!(f, "}}")
            }
            Self::Date(dt) => write!(f, "{}", dt.format("%Y-%m-%dT%H:%M:%S%.3fZ")),
            Self::Null => write!(f, "null"),
        }
    }
}

impl FactValue {
    /// Check if two `FactValues` are compatible for comparison
    #[must_use]
    pub const fn is_compatible_with(&self, other: &Self) -> bool {
        use FactValue::{Array, Boolean, Date, Float, Integer, Null, Object, String};
        matches!(
            (self, other),
            (String(_), String(_))
                | (Integer(_) | Float(_), Integer(_) | Float(_))
                | (Boolean(_), Boolean(_))
                | (Date(_), Date(_))
                | (Array(_), Array(_))
                | (Object(_), Object(_))
                | (Null, _)
                | (_, Null)
        )
    }

    /// Convert to a normalised comparison value
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn to_comparable(&self) -> Option<f64> {
        match self {
            Self::Integer(i) => Some(*i as f64),
            Self::Float(f) => Some(*f),
            Self::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            Self::Date(dt) => Some(dt.timestamp() as f64),
            Self::Array(arr) => Some(arr.len() as f64), // Length for comparison
            Self::Object(obj) => Some(obj.len() as f64), // Length for comparison
            Self::String(_) | Self::Null => None,
        }
    }

    /// Check if this value is "truthy" for conditional logic
    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Boolean(b) => *b,
            Self::Integer(i) => *i != 0,
            Self::Float(f) => *f != 0.0,
            Self::String(s) => !s.is_empty(),
            Self::Array(arr) => !arr.is_empty(),
            Self::Object(obj) => !obj.is_empty(),
            Self::Date(_) => true, // Dates are always truthy
            Self::Null => false,
        }
    }

    /// Get the type name as a string
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "string",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::Boolean(_) => "boolean",
            Self::Array(_) => "array",
            Self::Object(_) => "object",
            Self::Date(_) => "date",
            Self::Null => "null",
        }
    }

    /// Try to convert to an integer
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            #[allow(clippy::cast_possible_truncation)]
            Self::Float(f) => Some(*f as i64),
            Self::Boolean(b) => Some(i64::from(*b)),
            Self::String(s) => s.parse::<i64>().ok(),
            Self::Date(d) => Some(d.timestamp()),
            Self::Array(arr) => Some(arr.len() as i64),
            Self::Object(obj) => Some(obj.len() as i64),
            Self::Null => Some(0),
        }
    }

    /// Try to convert to a float
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Integer(i) => Some(*i as f64),
            Self::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            Self::String(s) => s.parse::<f64>().ok(),
            Self::Date(d) => Some(d.timestamp() as f64),
            Self::Array(arr) => Some(arr.len() as f64),
            Self::Object(obj) => Some(obj.len() as f64),
            Self::Null => Some(0.0),
        }
    }

    /// Try to convert to a string
    #[must_use]
    pub fn as_string(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            other => other.to_string(),
        }
    }

    /// Convert to string directly (alias for `as_string`)
    #[must_use]
    pub fn as_string_direct(&self) -> String {
        self.as_string()
    }

    /// Create array from elements
    #[must_use]
    pub const fn array(elements: Vec<Self>) -> Self {
        Self::Array(elements)
    }

    /// Create object from key-value pairs
    #[must_use]
    pub const fn object(fields: HashMap<String, Self>) -> Self {
        Self::Object(fields)
    }

    /// Create date from UTC timestamp
    #[must_use]
    pub fn date_from_timestamp(timestamp: i64) -> Self {
        Self::Date(DateTime::from_timestamp(timestamp, 0).unwrap_or_default())
    }

    /// Create date from ISO string
    ///
    /// # Errors
    ///
    /// Returns a `chrono::ParseError` if the ISO string cannot be parsed.
    pub fn date_from_iso(iso_string: &str) -> Result<Self, chrono::ParseError> {
        Ok(Self::Date(
            DateTime::parse_from_rfc3339(iso_string)?.with_timezone(&Utc),
        ))
    }

    /// Create null value
    #[must_use]
    pub const fn null() -> Self {
        Self::Null
    }

    /// Convenience accessor returning an `f64` representation if this value is numeric.
    /// Returns `None` when the variant is not `Integer` or `Float`.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub const fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Integer(i) => Some(*i as f64),
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }
}
