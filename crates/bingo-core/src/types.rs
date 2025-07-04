use anyhow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
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

/// Represents a fact in the RETE network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub id: FactId,
    pub external_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: FactData,
}

impl Fact {
    /// Get a field value from this fact
    pub fn get_field(&self, field: &str) -> Option<&FactValue> {
        self.data.fields.get(field)
    }

    /// Convenience constructor used mainly in tests
    pub fn new(id: FactId, data: FactData) -> Self {
        Self { id, external_id: None, timestamp: chrono::Utc::now(), data }
    }
}

/// Unique identifier for facts
pub type FactId = u64;
pub type NodeId = u64;

/// The actual data content of a fact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FactData {
    pub fields: HashMap<String, FactValue>,
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
            FactValue::String(s) => serde_json::Value::String(s),
            FactValue::Integer(i) => serde_json::Value::Number(serde_json::Number::from(i)),
            FactValue::Float(f) => serde_json::Number::from_f64(f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            FactValue::Boolean(b) => serde_json::Value::Bool(b),
            FactValue::Array(arr) => {
                let vec: Vec<serde_json::Value> = arr.into_iter().map(|v| v.into()).collect();
                serde_json::Value::Array(vec)
            }
            FactValue::Object(map) => {
                let json_map = map
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<serde_json::Map<String, serde_json::Value>>();
                serde_json::Value::Object(json_map)
            }
            FactValue::Date(dt) => serde_json::Value::String(dt.to_rfc3339()),
            FactValue::Null => serde_json::Value::Null,
        }
    }
}

impl From<&FactValue> for serde_json::Value {
    fn from(value: &FactValue) -> Self {
        match value {
            FactValue::String(s) => serde_json::Value::String(s.clone()),
            FactValue::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            FactValue::Float(f) => serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            FactValue::Boolean(b) => serde_json::Value::Bool(*b),
            FactValue::Array(arr) => {
                let vec: Vec<serde_json::Value> = arr.iter().map(|v| v.into()).collect();
                serde_json::Value::Array(vec)
            }
            FactValue::Object(map) => {
                let json_map = map
                    .iter()
                    .map(|(k, v)| (k.clone(), v.into()))
                    .collect::<serde_json::Map<String, serde_json::Value>>();
                serde_json::Value::Object(json_map)
            }
            FactValue::Date(dt) => serde_json::Value::String(dt.to_rfc3339()),
            FactValue::Null => serde_json::Value::Null,
        }
    }
}

impl TryFrom<&serde_json::Value> for FactValue {
    type Error = anyhow::Error;

    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        Ok(match value {
            serde_json::Value::String(s) => FactValue::String(s.clone()),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    FactValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    FactValue::Float(f)
                } else {
                    return Err(anyhow::anyhow!("Unsupported number value: {}", n));
                }
            }
            serde_json::Value::Bool(b) => FactValue::Boolean(*b),
            serde_json::Value::Array(arr) => {
                let inner = arr.iter().map(FactValue::try_from).collect::<Result<Vec<_>, _>>()?;
                FactValue::Array(inner)
            }
            serde_json::Value::Object(map) => {
                let mut inner = HashMap::new();
                for (k, v) in map {
                    inner.insert(k.clone(), FactValue::try_from(v)?);
                }
                FactValue::Object(inner)
            }
            serde_json::Value::Null => FactValue::Null,
        })
    }
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
                dt.timestamp_nanos_opt().unwrap_or(0).hash(state);
            }
            FactValue::Null => {
                7u8.hash(state);
            }
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

impl fmt::Display for FactValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FactValue::String(s) => write!(f, "{}", s),
            FactValue::Integer(i) => write!(f, "{}", i),
            FactValue::Float(fl) => write!(f, "{}", fl),
            FactValue::Boolean(b) => write!(f, "{}", b),
            FactValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            FactValue::Object(obj) => {
                write!(f, "{{")?;
                let mut first = true;
                for (key, value) in obj {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                    first = false;
                }
                write!(f, "}}")
            }
            FactValue::Date(dt) => write!(f, "{}", dt.format("%Y-%m-%dT%H:%M:%S%.3fZ")),
            FactValue::Null => write!(f, "null"),
        }
    }
}

impl FactValue {
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

    /// Get the type name as a string
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

    /// Try to convert to an integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            FactValue::Integer(i) => Some(*i),
            FactValue::Float(f) => Some(*f as i64),
            FactValue::Boolean(b) => Some(if *b { 1 } else { 0 }),
            FactValue::String(s) => s.parse::<i64>().ok(),
            FactValue::Date(d) => Some(d.timestamp()),
            FactValue::Array(arr) => Some(arr.len() as i64),
            FactValue::Object(obj) => Some(obj.len() as i64),
            FactValue::Null => Some(0),
        }
    }

    /// Try to convert to a float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            FactValue::Float(f) => Some(*f),
            FactValue::Integer(i) => Some(*i as f64),
            FactValue::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            FactValue::String(s) => s.parse::<f64>().ok(),
            FactValue::Date(d) => Some(d.timestamp() as f64),
            FactValue::Array(arr) => Some(arr.len() as f64),
            FactValue::Object(obj) => Some(obj.len() as f64),
            FactValue::Null => Some(0.0),
        }
    }

    /// Try to convert to a string
    pub fn as_string(&self) -> String {
        match self {
            FactValue::String(s) => s.clone(),
            other => other.to_string(),
        }
    }

    /// Convert to string directly (alias for as_string)
    pub fn as_string_direct(&self) -> String {
        self.as_string()
    }

    /// Create array from elements
    pub fn array(elements: Vec<FactValue>) -> Self {
        FactValue::Array(elements)
    }

    /// Create object from key-value pairs
    pub fn object(fields: HashMap<String, FactValue>) -> Self {
        FactValue::Object(fields)
    }

    /// Create date from UTC timestamp
    pub fn date_from_timestamp(timestamp: i64) -> Self {
        FactValue::Date(DateTime::from_timestamp(timestamp, 0).unwrap_or_default())
    }

    /// Create date from ISO string
    pub fn date_from_iso(iso_string: &str) -> Result<Self, chrono::ParseError> {
        Ok(FactValue::Date(
            DateTime::parse_from_rfc3339(iso_string)?.with_timezone(&Utc),
        ))
    }

    /// Create null value
    pub fn null() -> Self {
        FactValue::Null
    }

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

/// Represents a rule in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: RuleId,
    pub name: String,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
}

/// Unique identifier for rules
pub type RuleId = u64;

/// A condition that must be met for a rule to fire
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Condition {
    Simple { field: String, operator: Operator, value: FactValue },
    Complex { operator: LogicalOperator, conditions: Vec<Condition> },
    Aggregation(AggregationCondition),
    Stream(StreamCondition),
}

impl PartialEq for Condition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Condition::Simple { field: f1, operator: o1, value: v1 },
                Condition::Simple { field: f2, operator: o2, value: v2 },
            ) => f1 == f2 && o1 == o2 && v1 == v2,
            (
                Condition::Complex { operator: o1, conditions: c1 },
                Condition::Complex { operator: o2, conditions: c2 },
            ) => o1 == o2 && c1 == c2,
            (Condition::Aggregation(a1), Condition::Aggregation(a2)) => {
                // For now, compare only basic fields for aggregation conditions
                a1.aggregation_type.discriminant() == a2.aggregation_type.discriminant()
                    && a1.source_field == a2.source_field
                    && a1.group_by == a2.group_by
                    && a1.alias == a2.alias
            }
            (Condition::Stream(s1), Condition::Stream(s2)) => {
                s1.window_spec == s2.window_spec
                    && s1.aggregation == s2.aggregation
                    && s1.alias == s2.alias
                // Skip filter and having conditions for simplicity in equality comparison
            }
            _ => false,
        }
    }
}

impl Eq for Condition {}

impl std::hash::Hash for Condition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Condition::Simple { field, operator, value } => {
                0u8.hash(state);
                field.hash(state);
                operator.hash(state);
                value.hash(state);
            }
            Condition::Complex { operator, conditions } => {
                1u8.hash(state);
                operator.hash(state);
                conditions.hash(state);
            }
            Condition::Aggregation(agg) => {
                2u8.hash(state);
                agg.aggregation_type.discriminant().hash(state);
                agg.source_field.hash(state);
                agg.group_by.hash(state);
                agg.alias.hash(state);
                // Skip optional and complex fields for simplicity
            }
            Condition::Stream(stream) => {
                3u8.hash(state);
                stream.window_spec.hash(state);
                stream.aggregation.hash(state);
                stream.alias.hash(state);
                // Skip optional filter and having for simplicity
            }
        }
    }
}

/// Aggregation-based condition for multi-fact rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationCondition {
    pub aggregation_type: AggregationType,
    pub source_field: String,
    pub group_by: Vec<String>,
    pub having: Option<Box<Condition>>,
    pub alias: String,
    pub window: Option<AggregationWindow>,
}

/// Stream processing condition for temporal pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamCondition {
    /// Window specification for the stream condition
    pub window_spec: StreamWindowSpec,
    /// Aggregation function to apply over the window
    pub aggregation: StreamAggregation,
    /// Filter condition for facts entering the window
    pub filter: Option<Box<Condition>>,
    /// Condition to evaluate against the aggregated result
    pub having: Option<Box<Condition>>,
    /// Alias for the result
    pub alias: String,
}

/// Window specification for stream processing conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StreamWindowSpec {
    /// Tumbling time window
    Tumbling { duration_ms: u64 },
    /// Sliding time window
    Sliding { size_ms: u64, advance_ms: u64 },
    /// Session window with gap timeout
    Session { gap_timeout_ms: u64 },
    /// Count-based tumbling window
    CountTumbling { count: usize },
    /// Count-based sliding window
    CountSliding { size: usize, advance: usize },
}

/// Aggregation functions for stream processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StreamAggregation {
    /// Count facts in window
    Count,
    /// Sum field values
    Sum { field: String },
    /// Average field values
    Average { field: String },
    /// Minimum field value
    Min { field: String },
    /// Maximum field value
    Max { field: String },
    /// Count distinct values
    Distinct { field: String },
    /// First value in window
    First { field: String },
    /// Last value in window
    Last { field: String },
    /// Rate of events (count per time unit)
    Rate { time_unit_ms: u64 },
    /// Custom aggregation using calculator expression
    Custom { expression: String },
}

/// Operators for rule conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Operator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
}

/// Logical operators for complex conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// Actions to take when a rule fires
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub action_type: ActionType,
}

/// Types of actions that can be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Log {
        message: String,
    },
    SetField {
        field: String,
        value: FactValue,
    },
    CreateFact {
        data: FactData,
    },

    // Calculator-generated actions (Phase 3+)
    CallCalculator {
        calculator_name: String,
        input_mapping: HashMap<String, String>,
        output_field: String,
    },

    TriggerAlert {
        alert_type: String,
        message: String,
        severity: AlertSeverity,
        metadata: HashMap<String, FactValue>,
    },

    // Formula-based calculations using simple expression DSL
    Formula {
        expression: String,
        output_field: String,
    },

    // Advanced fact manipulation actions
    UpdateFact {
        fact_id_field: String,               // Field containing the fact ID to update
        updates: HashMap<String, FactValue>, // Fields to update
    },

    DeleteFact {
        fact_id_field: String, // Field containing the fact ID to delete
    },

    IncrementField {
        field: String,
        increment: FactValue, // Amount to increment (Integer or Float)
    },

    AppendToArray {
        field: String,
        value: FactValue, // Value to append to the array
    },

    SendNotification {
        recipient: String,
        subject: String,
        message: String,
        notification_type: NotificationType,
        metadata: HashMap<String, FactValue>,
    },
}

/// Alert severity levels for stream processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Notification types for SendNotification action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NotificationType {
    Email,
    Sms,
    Push,
    Webhook,
    Slack,
    Teams,
    InApp,
}

/// Aggregation types supported by the engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Count,
    Average,
    Min,
    Max,
    StandardDeviation,
    Percentile(f64),
}

impl AggregationType {
    /// Get a discriminant for hashing purposes
    pub fn discriminant(&self) -> u8 {
        match self {
            AggregationType::Sum => 0,
            AggregationType::Count => 1,
            AggregationType::Average => 2,
            AggregationType::Min => 3,
            AggregationType::Max => 4,
            AggregationType::StandardDeviation => 5,
            AggregationType::Percentile(_) => 6,
        }
    }
}

/// Window types for aggregations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationWindow {
    Sliding { size: usize },
    Tumbling { size: usize },
    Session { timeout_ms: u64 },
    Time { duration_ms: u64 },
}

/// Engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    pub rule_count: usize,
    pub fact_count: usize,
    pub node_count: usize,
    pub memory_usage_bytes: usize,
}

// ============================================================================
// Calculator DSL Types (Phase 3+)
// ============================================================================

/// Calculator abstraction for business-friendly rule authoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calculator {
    pub id: String,
    pub description: String,
    pub calculator_type: CalculatorType,
    pub conditions: Vec<Condition>,
    pub metadata: CalculatorMetadata,
}

/// Business-friendly calculator operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CalculatorType {
    ApplyPercentage {
        source_field: String,
        target_field: String,
        percentage: f64,
    },
    ApplyFlat {
        target_field: String,
        amount: f64,
        currency: Option<String>,
    },
    ApplyFormula {
        target_field: String,
        formula: String,
        dependencies: Vec<String>,
    },
    TieredRate {
        source_field: String,
        target_field: String,
        tiers: Vec<RateTier>,
    },
    ConditionalRate {
        source_field: String,
        target_field: String,
        rate_table: HashMap<String, f64>,
    },
    AccumulateValue {
        source_field: String,
        target_field: String,
        group_by: Vec<String>,
        operation: AggregationType,
        reset_condition: Option<Condition>,
    },
}

/// Rate tier for tiered calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateTier {
    pub threshold: f64,
    pub rate: f64,
}

/// Calculator metadata for tracking and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatorMetadata {
    pub author: Option<String>,
    pub created_at: Option<String>,
    pub tags: Vec<String>,
    pub version: Option<String>,
}

/// Field type information for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    Currency { precision: u8, currency_code: String },
    Percentage { min: f64, max: f64, display_as_decimal: bool },
    Integer { min: i64, max: i64 },
    Decimal { precision: u8, scale: u8 },
    Text { max_length: usize, pattern: Option<String> },
    Date { format: String },
    Boolean,
    Enum { values: Vec<String> },
}

/// Compilation result for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledCalculator {
    pub calculator_id: String,
    pub generated_rules: Vec<Rule>,
    pub source_mapping: SourceMapping,
    pub compilation_time_ms: u64,
    pub warnings: Vec<String>,
}

/// Source mapping for debugging calculator execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapping {
    pub calculator_id: String,
    pub rete_node_ids: Vec<u64>, // NodeId references
    pub source_location: Option<SourceLocation>,
    pub generated_action_ids: Vec<String>,
}

/// Source location for error reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

/// RETE network node types (simplified for BSSN)
#[derive(Debug, Clone)]
pub struct AlphaNode {
    pub id: NodeId,
    pub condition: Condition,
}

impl AlphaNode {
    pub fn new(id: NodeId, condition: Condition) -> Self {
        Self { id, condition }
    }
}

#[derive(Debug, Clone)]
pub struct BetaNode {
    pub id: NodeId,
    pub rule_ids: Vec<RuleId>,
}

#[derive(Debug, Clone)]
pub struct TerminalNode {
    pub id: NodeId,
    pub rule_id: RuleId,
    pub actions: Vec<Action>,
}

impl TerminalNode {
    pub fn new(id: NodeId, rule_id: RuleId, actions: Vec<Action>) -> Self {
        Self { id, rule_id, actions }
    }
}

// ============================================================================
// Missing Types for Compilation
// ============================================================================

/// Token for parsing/debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub value: String,
    pub position: usize,
}

/// Pipeline execution context
#[derive(Debug, Clone)]
pub struct PipelineContext {
    pub pipeline_id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub stages_executed: Vec<String>,
    pub total_facts_processed: usize,
    pub total_rules_fired: usize,
    pub errors: Vec<String>,
    pub global_variables: HashMap<String, FactValue>,
}

/// Processing pipeline definition
#[derive(Debug, Clone)]
pub struct ProcessingPipeline {
    pub id: String,
    pub name: String,
    pub stages: Vec<PipelineStage>,
    pub global_context: HashMap<String, FactValue>,
}

/// Single stage in processing pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub id: String,
    pub stage_type: String,
    pub rules: Vec<Rule>,
    pub dependencies: Vec<String>,
}

/// Result of pipeline execution
#[derive(Debug, Clone)]
pub struct PipelineExecutionResult {
    pub context: PipelineContext,
    pub stage_results: HashMap<String, StageExecutionResult>,
    pub total_facts_processed: usize,
    pub total_facts_created: usize,
    pub final_facts: Vec<Fact>,
    pub created_facts: Vec<Fact>,
}

/// Result of single stage execution
#[derive(Debug, Clone)]
pub struct StageExecutionResult {
    pub stage_id: String,
    pub stage: PipelineStage,
    pub duration_ms: u64,
    pub facts_processed: usize,
    pub rules_fired: usize,
    pub facts_created: usize,
    pub facts_modified: usize,
    pub created_facts: Vec<Fact>,
    pub errors: Vec<String>,
}

/// Calculator inputs
#[derive(Debug, Clone)]
pub struct CalculatorInputs {
    pub fields: HashMap<String, FactValue>,
}

/// Calculator hash map pool for optimization
#[derive(Debug, Clone)]
pub struct CalculatorHashMapPool {
    pub pool: Vec<HashMap<String, FactValue>>,
    hits: usize,
    misses: usize,
}

impl Default for CalculatorHashMapPool {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorHashMapPool {
    pub fn new() -> Self {
        Self { pool: Vec::new(), hits: 0, misses: 0 }
    }

    pub fn get(&mut self) -> HashMap<String, FactValue> {
        if let Some(mut map) = self.pool.pop() {
            map.clear(); // Clear the map for reuse
            self.hits += 1;
            map
        } else {
            self.misses += 1;
            HashMap::new()
        }
    }

    pub fn return_map(&mut self, map: HashMap<String, FactValue>) {
        self.pool.push(map);
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        let pool_size = self.pool.len();
        (self.hits, self.misses, pool_size)
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// Calculator result cache
#[derive(Debug, Clone)]
pub struct CalculatorResultCache {
    pub cache: HashMap<String, FactValue>,
    hits: usize,
    misses: usize,
}

impl CalculatorResultCache {
    pub fn new(_capacity: usize) -> Self {
        Self { cache: HashMap::new(), hits: 0, misses: 0 }
    }

    pub fn get(&mut self, calculator: &str, inputs: &CalculatorInputs) -> Option<String> {
        let key = format!("{}:{:?}", calculator, inputs.fields);
        if let Some(value) = self.cache.get(&key) {
            self.hits += 1;
            Some(value.as_string())
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn put(&mut self, calculator: &str, inputs: &CalculatorInputs, result: String) {
        let key = format!("{}:{:?}", calculator, inputs.fields);
        self.cache.insert(key, FactValue::String(result));
    }

    pub fn stats(&self) -> (usize, usize, usize, f64) {
        let size = self.cache.len();
        let total = self.hits + self.misses;
        let hit_rate = if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        };
        (self.hits, self.misses, size, hit_rate)
    }
}
