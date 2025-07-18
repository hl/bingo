use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

// Re-export FactValue from bingo-types
pub use bingo_types::FactValue;

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

/// Central data structure representing a fact in the RETE network
///
/// ## Overview
///
/// Facts are the fundamental unit of data in the Bingo rules engine. They represent
/// structured information that rules can pattern match against to trigger actions.
/// Each fact has a unique internal ID, optional external identifier, timestamp,
/// and structured data fields.
///
/// ## Architecture
///
/// - **Immutable by Design**: Facts are immutable once created to ensure data consistency
/// - **High-Performance Storage**: Optimized for fast access and pattern matching
/// - **External Integration**: Support for external system identifiers
/// - **Temporal Tracking**: Automatic timestamping for event ordering
///
/// ## Usage Example
///
/// ```rust
/// use bingo_core::types::{Fact, FactData, FactValue};
/// use std::collections::HashMap;
///
/// let mut fields = HashMap::new();
/// fields.insert("employee_id".to_string(), FactValue::String("E123".to_string()));
/// fields.insert("salary".to_string(), FactValue::Float(75000.0));
/// fields.insert("department".to_string(), FactValue::String("Engineering".to_string()));
///
/// let fact_data = FactData { fields };
/// let fact = Fact::new(1, fact_data);
/// ```
///
/// ## Performance Characteristics
///
/// - **Memory Efficient**: Uses arena allocation for optimal memory usage
/// - **Fast Access**: O(1) field lookups via HashMap
/// - **Serializable**: Full JSON serialization support for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    /// Unique internal identifier for the fact (auto-generated)
    pub id: FactId,
    /// Optional external identifier for integration with external systems
    pub external_id: Option<String>,
    /// UTC timestamp when the fact was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Structured data content of the fact
    pub data: FactData,
}

impl Fact {
    /// Get a field value from this fact
    ///
    /// ## Usage
    ///
    /// This method provides fast O(1) field lookup by name.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use bingo_core::types::{Fact, FactData, FactValue};
    /// # use std::collections::HashMap;
    /// # let mut fields = HashMap::new();
    /// # fields.insert("salary".to_string(), FactValue::Float(75000.0));
    /// # let fact_data = FactData { fields };
    /// # let fact = Fact::new(1, fact_data);
    /// if let Some(salary) = fact.get_field("salary") {
    ///     if let FactValue::Float(amount) = salary {
    ///         println!("Employee salary: ${}", amount);
    ///     }
    /// }
    /// ```
    pub fn get_field(&self, field: &str) -> Option<&FactValue> {
        self.data.fields.get(field)
    }

    /// Convenience constructor for creating facts (primarily used in tests)
    ///
    /// ## Usage
    ///
    /// Creates a new fact with the specified ID and data. The timestamp is
    /// automatically set to the current UTC time, and external_id is set to None.
    ///
    /// ## Note
    ///
    /// In production, facts are typically created through the engine's fact
    /// ingestion methods which handle ID assignment automatically.
    pub fn new(id: FactId, data: FactData) -> Self {
        Self { id, external_id: None, timestamp: chrono::Utc::now(), data }
    }
}

/// Unique identifier for facts within the engine
///
/// ## Usage
///
/// Each fact is assigned a unique u64 identifier for fast lookups and references.
/// This ID is auto-generated by the engine and should not be manually assigned
/// in production code.
pub type FactId = u64;

/// Unique identifier for RETE network nodes
///
/// ## Usage
///
/// Internal identifier used by the RETE network for tracking nodes in the
/// pattern matching graph. Not typically used directly by client code.
pub type NodeId = u64;

/// Container for the structured data content of a fact
///
/// ## Overview
///
/// FactData represents the actual business data within a fact as a collection
/// of named fields. Each field can contain any supported FactValue type,
/// providing flexibility for diverse data structures.
///
/// ## Design Principles
///
/// - **Schema Flexibility**: Dynamic field structure without predefined schema
/// - **Type Safety**: Strongly typed values through FactValue enum
/// - **Performance**: HashMap-based storage for O(1) field access
/// - **Interoperability**: Full JSON serialization support
///
/// ## Usage Example
///
/// ```rust
/// use bingo_core::types::{FactData, FactValue};
/// use std::collections::HashMap;
///
/// let mut fields = HashMap::new();
/// fields.insert("name".to_string(), FactValue::String("Alice".to_string()));
/// fields.insert("age".to_string(), FactValue::Integer(30));
/// fields.insert("active".to_string(), FactValue::Boolean(true));
/// fields.insert("score".to_string(), FactValue::Float(95.5));
///
/// let fact_data = FactData { fields };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FactData {
    /// Named fields containing the fact's data
    ///
    /// Keys are field names (strings) and values are strongly-typed FactValues.
    /// The HashMap provides O(1) access time for field lookups during rule evaluation.
    pub fields: HashMap<String, FactValue>,
}

/// Business rule definition for pattern matching and action execution
///
/// ## Overview
///
/// Rules are the core logical constructs in the Bingo rules engine. Each rule
/// defines a pattern of conditions that must be satisfied and a set of actions
/// to execute when those conditions are met. Rules are compiled into an efficient
/// RETE network for high-performance pattern matching.
///
/// ## Rule Structure
///
/// - **ID**: Unique identifier for the rule
/// - **Name**: Human-readable name for debugging and management
/// - **Conditions**: Logical patterns that facts must match
/// - **Actions**: Operations to perform when conditions are satisfied
///
/// ## Rule Execution Model
///
/// When facts are processed through the engine:
/// 1. **Pattern Matching**: Conditions are evaluated against incoming facts
/// 2. **Conflict Resolution**: Multiple matching rules are prioritized
/// 3. **Action Execution**: Actions are performed for matched rules
/// 4. **Fact Creation**: New facts may be created as side effects
///
/// ## Usage Example
///
/// ```rust
/// use bingo_core::types::{Rule, Condition, Action, ActionType, Operator, FactValue};
///
/// let rule = Rule {
///     id: 1,
///     name: "High Salary Alert".to_string(),
///     conditions: vec![
///         Condition::Simple {
///             field: "salary".to_string(),
///             operator: Operator::GreaterThan,
///             value: FactValue::Float(100000.0),
///         }
///     ],
///     actions: vec![
///         Action {
///             action_type: ActionType::Log {
///                 message: "High salary detected".to_string(),
///             }
///         }
///     ],
/// };
/// ```
///
/// ## Performance Characteristics
///
/// - **Compilation**: Rules are compiled once into RETE network nodes
/// - **Execution**: O(1) pattern matching through pre-compiled network
/// - **Memory**: Shared network nodes for common condition patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique identifier for this rule
    pub id: RuleId,
    /// Human-readable name for debugging and management
    pub name: String,
    /// Logical conditions that must be satisfied for rule to fire
    pub conditions: Vec<Condition>,
    /// Actions to execute when conditions are met
    pub actions: Vec<Action>,
}

/// Unique identifier for rules within the engine
///
/// ## Usage
///
/// Each rule is assigned a unique u64 identifier for fast lookups, references,
/// and conflict resolution. This ID is typically assigned by the client code
/// or rule management system.
pub type RuleId = u64;

/// Condition types for rule pattern matching
///
/// ## Overview
///
/// Conditions define the patterns that facts must match for a rule to fire.
/// The Bingo engine supports simple field comparisons, complex logical combinations,
/// aggregations across multiple facts, and stream processing patterns.
///
/// ## Condition Types
///
/// - **Simple**: Direct field comparison (field op value)
/// - **Complex**: Logical combinations of other conditions (AND, OR, NOT)
/// - **Aggregation**: Patterns across multiple facts (SUM, COUNT, etc.)
/// - **Stream**: Time-windowed patterns for real-time processing
///
/// ## Usage Examples
///
/// ### Simple Condition
/// ```rust
/// # use bingo_core::types::{Condition, Operator, FactValue};
/// let condition = Condition::Simple {
///     field: "age".to_string(),
///     operator: Operator::GreaterThan,
///     value: FactValue::Integer(18),
/// };
/// ```
///
/// ### Complex Condition
/// ```rust
/// # use bingo_core::types::{Condition, Operator, FactValue, LogicalOperator};
/// let condition = Condition::Complex {
///     operator: LogicalOperator::And,
///     conditions: vec![
///         Condition::Simple {
///             field: "department".to_string(),
///             operator: Operator::Equal,
///             value: FactValue::String("Engineering".to_string()),
///         },
///         Condition::Simple {
///             field: "salary".to_string(),
///             operator: Operator::GreaterThan,
///             value: FactValue::Float(75000.0),
///         },
///     ],
/// };
/// ```
///
/// ## Performance Notes
///
/// - Simple conditions compile to alpha nodes for O(1) evaluation
/// - Complex conditions create beta nodes with optimized join algorithms
/// - Aggregation conditions use lazy evaluation for efficiency
/// - Stream conditions leverage time-window indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Condition {
    /// Simple field comparison condition
    Simple {
        /// Field name to test
        field: String,
        /// Comparison operator
        operator: Operator,
        /// Value to compare against
        value: FactValue,
    },
    /// Complex logical combination of conditions
    Complex {
        /// Logical operator (AND, OR, NOT)
        operator: LogicalOperator,
        /// Sub-conditions to combine
        conditions: Vec<Condition>,
    },
    /// AND combination of conditions
    And {
        /// Conditions that must all be true
        conditions: Vec<Condition>,
    },
    /// OR combination of conditions
    Or {
        /// Conditions where at least one must be true
        conditions: Vec<Condition>,
    },
    /// Aggregation-based condition across multiple facts
    Aggregation(AggregationCondition),
    /// Stream processing condition with time windows
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
            (Condition::And { conditions: c1 }, Condition::And { conditions: c2 }) => c1 == c2,
            (Condition::Or { conditions: c1 }, Condition::Or { conditions: c2 }) => c1 == c2,
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
            Condition::And { conditions } => {
                4u8.hash(state);
                conditions.hash(state);
            }
            Condition::Or { conditions } => {
                5u8.hash(state);
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
    StartsWith,
    EndsWith,
}

/// Logical operators for complex conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// Executable action to perform when a rule fires
///
/// ## Overview
///
/// Actions define what operations should be performed when a rule's conditions
/// are satisfied. Actions can modify facts, create new facts, trigger external
/// systems, perform calculations, and more.
///
/// ## Action Execution Model
///
/// - **Atomic**: All actions in a rule execute as a unit
/// - **Ordered**: Actions execute in the order defined in the rule
/// - **Side Effects**: Actions may create new facts that trigger other rules
/// - **Reversible**: Some actions support rollback for transaction-like behavior
///
/// ## Usage Example
///
/// ```rust
/// use bingo_core::types::{Action, ActionType, FactValue};
///
/// let action = Action {
///     action_type: ActionType::SetField {
///         field: "processed".to_string(),
///         value: FactValue::Boolean(true),
///     }
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// The specific type of action to perform
    pub action_type: ActionType,
}

/// Comprehensive set of action types for rule execution
///
/// ## Overview
///
/// ActionType defines all the operations that can be performed when rules fire.
/// Actions range from simple logging and field updates to complex calculations,
/// fact manipulation, and external system integration.
///
/// ## Action Categories
///
/// - **Data Manipulation**: SetField, UpdateFact, DeleteFact, CreateFact
/// - **Calculations**: Formula, CallCalculator, IncrementField
/// - **External Integration**: TriggerAlert, SendNotification
/// - **Debugging**: Log
/// - **Collections**: AppendToArray
///
/// ## Usage Examples
///
/// ### Basic Field Update
/// ```rust
/// # use bingo_core::types::{ActionType, FactValue};
/// ActionType::SetField {
///     field: "status".to_string(),
///     value: FactValue::String("processed".to_string()),
/// };
/// ```
///
/// ### Formula Calculation
/// ```rust
/// # use bingo_core::types::ActionType;
/// ActionType::Formula {
///     expression: "salary * 1.1".to_string(),
///     output_field: "new_salary".to_string(),
/// };
/// ```
///
/// ### Fact Creation
/// ```rust
/// # use bingo_core::types::{ActionType, FactData, FactValue};
/// # use std::collections::HashMap;
/// let mut fields = HashMap::new();
/// fields.insert("type".to_string(), FactValue::String("alert".to_string()));
/// fields.insert("level".to_string(), FactValue::String("high".to_string()));
///
/// ActionType::CreateFact {
///     data: FactData { fields },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    /// Log a message for debugging and audit trails
    Log {
        /// Message to log
        message: String,
    },
    /// Set a field value on the triggering fact
    SetField {
        /// Field name to set
        field: String,
        /// Value to assign to the field
        value: FactValue,
    },
    /// Create a new fact with specified data
    CreateFact {
        /// Data for the new fact
        data: FactData,
    },

    /// Call an external calculator for complex business logic
    CallCalculator {
        /// Name of the calculator to invoke
        calculator_name: String,
        /// Mapping from calculator inputs to fact fields
        input_mapping: HashMap<String, String>,
        /// Field to store the calculator result
        output_field: String,
    },

    /// Trigger an alert in external systems
    TriggerAlert {
        /// Type/category of alert
        alert_type: String,
        /// Alert message
        message: String,
        /// Severity level for prioritization
        severity: AlertSeverity,
        /// Additional metadata
        metadata: HashMap<String, FactValue>,
    },

    /// Execute a formula expression and store the result
    Formula {
        /// Expression to evaluate (supports basic arithmetic)
        expression: String,
        /// Field to store the calculated result
        output_field: String,
    },

    /// Update an existing fact by ID
    UpdateFact {
        /// Field containing the ID of the fact to update
        fact_id_field: String,
        /// Map of field names to new values
        updates: HashMap<String, FactValue>,
    },

    /// Delete an existing fact by ID
    DeleteFact {
        /// Field containing the ID of the fact to delete
        fact_id_field: String,
    },

    /// Increment a numeric field by a specified amount
    IncrementField {
        /// Field to increment
        field: String,
        /// Amount to add (Integer or Float)
        increment: FactValue,
    },

    /// Append a value to an array field
    AppendToArray {
        /// Array field to modify
        field: String,
        /// Value to append
        value: FactValue,
    },

    /// Send a notification to external systems
    SendNotification {
        /// Notification recipient
        recipient: String,
        /// Message subject line
        subject: String,
        /// Message body
        message: String,
        /// Delivery method
        notification_type: NotificationType,
        /// Additional metadata
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

/// Comprehensive performance and resource statistics for the engine
///
/// ## Overview
///
/// EngineStats provides detailed metrics about the current state and performance
/// characteristics of the Bingo rules engine. These statistics are essential for
/// monitoring, capacity planning, and performance optimization.
///
/// ## Metrics Categories
///
/// - **Capacity**: Current rule and fact counts
/// - **Architecture**: RETE network complexity (node count)
/// - **Resource Usage**: Memory consumption tracking
/// - **Performance**: Processing efficiency indicators
///
/// ## Usage Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use bingo_core::BingoEngine;
///
/// let engine = BingoEngine::new()?;
/// let stats = engine.get_stats();
///
/// println!("Rules loaded: {}", stats.rule_count);
/// println!("Facts stored: {}", stats.fact_count);
/// println!("Network nodes: {}", stats.node_count);
/// println!("Memory usage: {} bytes", stats.memory_usage_bytes);
/// # Ok(())
/// # }
/// ```
///
/// ## Monitoring Integration
///
/// These statistics can be integrated with monitoring systems like Prometheus,
/// CloudWatch, or custom dashboards for production observability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    /// Number of active rules currently loaded in the engine
    pub rule_count: usize,
    /// Number of facts currently stored in the fact store
    pub fact_count: usize,
    /// Number of nodes in the compiled RETE network
    pub node_count: usize,
    /// Approximate memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Number of aggregations created (for lazy aggregation stats)
    pub aggregations_created: usize,
    /// Number of aggregations reused (for lazy aggregation stats)
    pub aggregations_reused: usize,
    /// Number of cache invalidations performed
    pub cache_invalidations: usize,
    /// Rule execution result pool information
    pub rule_execution_result_pool: PoolStats,
    /// Rule ID vector pool information
    pub rule_id_vec_pool: PoolStats,
    /// Cache hits for serialization
    pub cache_hits: usize,
    /// Cache misses for serialization
    pub cache_misses: usize,
    /// Total facts processed by alpha memory
    pub total_facts_processed: usize,
    /// Total matches found by alpha memory
    pub total_matches_found: usize,
    /// Buffer hits for serialization
    pub buffer_hits: usize,
    /// Buffer misses for serialization
    pub buffer_misses: usize,
    /// Cache size
    pub cache_size: usize,
    /// Buffer pool size
    pub buffer_pool_size: usize,
    /// Total full computations for lazy aggregation
    pub total_full_computations: usize,
    /// Total early terminations for lazy aggregation
    pub total_early_terminations: usize,
    /// Fact ID vector pool information
    pub fact_id_vec_pool: PoolStats,
    /// Fact field map pool information
    pub fact_field_map_pool: PoolStats,
    /// Numeric vector pool information
    pub numeric_vec_pool: PoolStats,
}

/// Pool statistics for object pools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    /// Pool hits
    pub hits: usize,
    /// Pool misses
    pub misses: usize,
    /// Pool size
    pub pool_size: usize,
    /// Objects currently allocated
    pub allocated: usize,
}

impl PoolStats {
    /// Calculate hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
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
    /// Rules that use this alpha node condition
    pub rule_ids: Vec<RuleId>,
}

impl AlphaNode {
    pub fn new(id: NodeId, condition: Condition) -> Self {
        Self { id, condition, rule_ids: Vec::new() }
    }

    /// Add a rule that uses this alpha node
    pub fn add_rule(&mut self, rule_id: RuleId) {
        if !self.rule_ids.contains(&rule_id) {
            self.rule_ids.push(rule_id);
        }
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
