# Bingo RETE Rules Engine - Complete API Reference

This document provides comprehensive API documentation for all public interfaces in the Bingo RETE Rules Engine, including the advanced RETE algorithm implementation with rule optimization, parallel processing, conflict resolution, and dependency analysis using Kahn's topological sorting algorithm.

## 📋 Table of Contents

- [Core Engine API](#core-engine-api)
- [Advanced RETE Features](#advanced-rete-features)
- [Rule Management API](#rule-management-api)
- [Fact Processing API](#fact-processing-api)
- [Calculator API](#calculator-api)
- [Type System API](#type-system-api)
- [Error Handling API](#error-handling-api)
- [Performance Monitoring API](#performance-monitoring-api)
- [gRPC API](#grpc-api)

---

## Core Engine API

### BingoEngine

The main engine class that implements a complete RETE algorithm with advanced optimizations including rule reordering, parallel processing, conflict resolution, and dependency analysis using Kahn's topological sorting algorithm.

#### 🧠 RETE Algorithm Features
- **Alpha Memory Network**: Hash-indexed single-condition fact matching with O(1) lookups
- **Beta Memory Network**: Token-based multi-condition processing with efficient join operations  
- **Rule Optimization**: Automatic condition reordering based on selectivity and cost analysis
- **Conflict Resolution**: Multiple strategies (Priority, Salience, Specificity, Lexicographic) with tie-breaking
- **Dependency Analysis**: Kahn's algorithm for optimal rule execution order
- **Parallel Processing**: Multi-threaded RETE with work-stealing queues
- **Incremental Processing**: O(Δfacts) complexity - only processes new/changed facts

#### Constructors

##### `BingoEngine::new() -> BingoResult<BingoEngine>`

Creates a new rules engine instance with default configuration.

**Returns:**
- `Ok(BingoEngine)` - Successfully initialized engine
- `Err(BingoError)` - Initialization failure

**Example:**
```rust
use bingo_core::BingoEngine;

let mut engine = BingoEngine::new()?;
```

**Memory Usage:** Approximately 1MB for initial allocation

##### `BingoEngine::with_capacity(capacity: usize) -> BingoResult<BingoEngine>`

Creates a new engine with pre-allocated capacity for optimal performance.

**Parameters:**
- `capacity: usize` - Expected number of facts/rules for capacity planning

**Performance Notes:**
- Pre-allocation reduces memory reallocations during runtime
- Recommended for known workload sizes > 1000 facts

**Example:**
```rust
// For processing 10,000 facts efficiently
let mut engine = BingoEngine::with_capacity(10_000)?;
```

#### Rule Management

##### `add_rule(&mut self, rule: Rule) -> BingoResult<()>`

Compiles and adds a rule to the RETE network.

**Parameters:**
- `rule: Rule` - Rule definition with conditions and actions

**Returns:**
- `Ok(())` - Rule successfully compiled and added
- `Err(BingoError::Rule)` - Rule compilation error
- `Err(BingoError::ReteNetwork)` - Network integration error

**Performance:** O(1) for simple rules, O(n) for complex conditions where n = condition count

**Example:**
```rust
use bingo_core::types::{Rule, Condition, Action, ActionType, Operator, FactValue};

let rule = Rule {
    id: 1,
    name: "High Value Alert".to_string(),
    conditions: vec![
        Condition::Simple {
            field: "amount".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Float(10000.0),
        }
    ],
    actions: vec![
        Action {
            action_type: ActionType::SetField {
                field: "alert_level".to_string(),
                value: FactValue::String("high".to_string()),
            },
        }
    ],
};

engine.add_rule(rule)?;
```

##### `add_rules(&mut self, rules: Vec<Rule>) -> BingoResult<()>`

Batch operation for adding multiple rules efficiently.

**Parameters:**
- `rules: Vec<Rule>` - Vector of rules to add

**Performance Benefits:**
- Optimized network compilation for multiple rules
- Reduced memory fragmentation
- Better cache locality

**Example:**
```rust
let rules = vec![rule1, rule2, rule3];
engine.add_rules(rules)?;
```

##### `update_rule(&mut self, rule: Rule) -> BingoResult<()>`

Updates an existing rule by ID, recompiling the affected network paths.

**Parameters:**
- `rule: Rule` - Updated rule definition (must have existing ID)

**Performance:** Selective recompilation affects only changed network paths

##### `remove_rule(&mut self, rule_id: RuleId) -> BingoResult<bool>`

Removes a rule from the engine and cleans up network nodes.

**Parameters:**
- `rule_id: RuleId` - Unique identifier of the rule to remove

**Returns:**
- `Ok(true)` - Rule successfully removed
- `Ok(false)` - Rule not found
- `Err(BingoError)` - Removal error

##### `rule_count(&self) -> usize`

Returns the current number of rules in the engine.

**Performance:** O(1) constant time operation

#### Fact Processing

##### `process_facts(&mut self, facts: Vec<Fact>) -> BingoResult<Vec<RuleExecutionResult>>`

Processes facts through the RETE network and executes triggered rules.

**Parameters:**
- `facts: Vec<Fact>` - Facts to process through the rules engine

**Returns:**
- `Ok(Vec<RuleExecutionResult>)` - Results of all rule executions
- `Err(BingoError)` - Processing error

**Performance Characteristics:**
- **Time Complexity:** O(Δfacts) - RETE algorithm only processes new/changed facts
- **Memory Usage:** Efficient alpha/beta memory networks with arena allocation
- **Throughput:** Up to 1.9M facts/sec for optimized workloads, 462K facts/sec for alpha memory processing
- **Rule Independence:** Performance scales independently of non-matching rule count
- **Optimization:** Automatic condition reordering and parallel processing

**Example:**
```rust
use bingo_core::types::{Fact, FactData, FactValue};
use std::collections::HashMap;

// Create facts
let mut fields = HashMap::new();
fields.insert("transaction_id".to_string(), FactValue::String("TXN001".to_string()));
fields.insert("amount".to_string(), FactValue::Float(15000.0));
fields.insert("currency".to_string(), FactValue::String("USD".to_string()));

let fact = Fact::new(1, FactData { fields });

// Process through engine
let results = engine.process_facts(vec![fact])?;

// Handle results
for result in results {
    println!("Rule {} fired for fact {}", result.rule_id, result.fact_id);
    for action in result.actions_executed {
        match action {
            ActionResult::FieldSet { fact_id, field, value } => {
                println!("Set {}={:?} on fact {}", field, value, fact_id);
            }
            ActionResult::CalculatorResult { calculator, result, output_field, .. } => {
                println!("Calculator {} computed {} -> {}", calculator, output_field, result);
            }
            _ => {}
        }
    }
}
```

##### `evaluate(&mut self, facts: Vec<Fact>) -> BingoResult<Vec<EvaluationResult>>`

Evaluates facts against rules without executing actions (dry-run mode).

**Use Cases:**
- Rule testing and validation
- Impact analysis before rule deployment
- Debugging rule logic

**Performance:** ~30% faster than `process_facts` since actions are not executed

#### Statistics and Monitoring

##### `get_stats(&self) -> EngineStats`

Retrieves comprehensive engine statistics for monitoring and optimization.

**Returns:** `EngineStats` struct containing:
- `rule_count: usize` - Number of active rules
- `fact_count: usize` - Number of stored facts
- `node_count: usize` - RETE network node count
- `memory_usage_bytes: usize` - Approximate memory usage

**Example:**
```rust
let stats = engine.get_stats();
println!("Engine Stats:");
println!("  Rules: {}", stats.rule_count);
println!("  Facts: {}", stats.fact_count);
println!("  Network Nodes: {}", stats.node_count);
println!("  Memory Usage: {} MB", stats.memory_usage_bytes / 1024 / 1024);
```

##### `clear(&mut self)`

Clears all facts and resets the engine state while preserving rules.

**Use Cases:**
- Batch processing cleanup
- Memory management between processing cycles
- Testing isolation

**Performance:** O(1) operation using arena allocation reset

#### Engine Configuration

##### `with_performance_config(config: PerformanceConfig) -> BingoResult<BingoEngine>`

Creates an engine with custom performance tuning.

**Configuration Options:**
- `initial_capacity: usize` - Pre-allocated memory
- `max_memory_mb: usize` - Memory usage limit
- `enable_profiling: bool` - Performance monitoring
- `cache_size: usize` - Result cache size

**Example:**
```rust
use bingo_core::PerformanceConfig;

let config = PerformanceConfig {
    initial_capacity: 50_000,
    max_memory_mb: 1024,
    enable_profiling: true,
    cache_size: 10_000,
};

let engine = BingoEngine::with_performance_config(config)?;
```

---

## Advanced RETE Features

The Bingo engine implements advanced RETE optimizations beyond the standard algorithm:

### Rule Optimization API

#### `add_rule_optimized(&mut self, rule: Rule) -> BingoResult<OptimizationResult>`

Adds a rule with automatic optimization including condition reordering and selectivity analysis.

**Parameters:**
- `rule: Rule` - Rule definition to be optimized and added

**Returns:**
- `OptimizationResult` containing:
  - `optimized_rule: Rule` - Rule with reordered conditions
  - `estimated_improvement: f64` - Expected performance improvement percentage
  - `strategies_applied: Vec<String>` - List of optimization strategies used
  - `analysis: OptimizationAnalysis` - Detailed analysis including condition selectivity

**Example:**
```rust
let optimization_result = engine.add_rule_optimized(rule)?;
println!("Estimated improvement: {:.1}%", optimization_result.estimated_improvement);
```

### Conflict Resolution API

#### `configure_conflict_resolution(&mut self, config: ConflictResolutionConfig)`

Configures the conflict resolution strategy for rule execution order.

**Available Strategies:**
- **Priority**: Rules execute based on priority values (higher first)
- **Salience**: Rules execute based on salience values (higher first)  
- **Specificity**: Rules with more conditions execute first
- **Lexicographic**: Rules execute in alphabetical order by name

**Example:**
```rust
use bingo_core::conflict_resolution::{ConflictResolutionConfig, ConflictResolutionStrategy};

let config = ConflictResolutionConfig {
    primary_strategy: ConflictResolutionStrategy::Priority,
    tie_breaker: Some(ConflictResolutionStrategy::Salience),
    enable_logging: true,
    max_conflict_set_size: 1000,
};

engine.configure_conflict_resolution(config);
```

#### `register_rule_priority(&mut self, rule_id: RuleId, priority: i32, salience: i32) -> BingoResult<()>`

Sets priority and salience values for conflict resolution.

**Parameters:**
- `rule_id: RuleId` - Rule identifier
- `priority: i32` - Priority value (higher executes first)
- `salience: i32` - Salience value (higher executes first for tie-breaking)

### Dependency Analysis API

#### `analyze_rule_dependencies(&self) -> BingoResult<DependencyAnalysis>`

Analyzes rule dependencies and provides execution order using Kahn's topological sorting algorithm.

**Returns:**
- `DependencyAnalysis` containing:
  - `execution_order: Vec<RuleId>` - Optimal rule execution order
  - `dependency_graph: HashMap<RuleId, Vec<RuleId>>` - Rule dependency relationships
  - `circular_dependencies: Vec<Vec<RuleId>>` - Any detected circular dependencies

**Example:**
```rust
let analysis = engine.analyze_rule_dependencies()?;
println!("Optimal execution order: {:?}", analysis.execution_order);
```

### Parallel Processing API

#### `configure_parallel_rete(&mut self, config: ParallelReteConfig)`

Configures parallel RETE processing with work-stealing queues.

**Configuration Options:**
- `worker_count: usize` - Number of worker threads
- `parallel_threshold: usize` - Minimum facts for parallel processing
- `fact_chunk_size: usize` - Fact processing batch size
- `enable_work_stealing: bool` - Enable work-stealing between threads

**Example:**
```rust
use bingo_core::parallel_rete::ParallelReteConfig;

let config = ParallelReteConfig {
    worker_count: 4,
    parallel_threshold: 100,
    fact_chunk_size: 25,
    enable_work_stealing: true,
    enable_parallel_alpha: true,
    enable_parallel_beta: true,
    enable_parallel_execution: true,
    work_queue_capacity: 1000,
};

engine.configure_parallel_rete(config);
```

#### `process_facts_advanced_parallel(&mut self, facts: Vec<Fact>, config: &ParallelReteConfig) -> BingoResult<Vec<RuleExecutionResult>>`

Processes facts using advanced parallel RETE processing.

**Performance Benefits:**
- **Multi-threaded Processing**: Utilizes multiple CPU cores
- **Work-stealing Queues**: Balanced load distribution across workers
- **Alpha/Beta Parallelization**: Parallel processing in both alpha and beta networks
- **Configurable Thresholds**: Automatic fallback to sequential processing for small datasets

---

## Rule Management API

### Rule Structure

#### Rule Definition
```rust
pub struct Rule {
    pub id: RuleId,                    // Unique identifier
    pub name: String,                  // Human-readable name
    pub conditions: Vec<Condition>,    // Rule conditions
    pub actions: Vec<Action>,          // Actions to execute
}
```

#### Condition Types

##### Simple Conditions
Direct field comparisons against values.

```rust
Condition::Simple {
    field: String,           // Field name to test
    operator: Operator,      // Comparison operator
    value: FactValue,        // Value to compare against
}
```

**Supported Operators:**
- `Equal` - Exact match
- `NotEqual` - Non-match
- `GreaterThan` - Numeric comparison
- `LessThan` - Numeric comparison
- `GreaterThanOrEqual` - Numeric comparison
- `LessThanOrEqual` - Numeric comparison
- `Contains` - String/array containment

**Example:**
```rust
Condition::Simple {
    field: "age".to_string(),
    operator: Operator::GreaterThanOrEqual,
    value: FactValue::Integer(18),
}
```

##### Complex Conditions
Logical combinations of other conditions.

```rust
Condition::Complex {
    operator: LogicalOperator,    // AND, OR, NOT
    conditions: Vec<Condition>,   // Sub-conditions
}
```

**Example:**
```rust
Condition::Complex {
    operator: LogicalOperator::And,
    conditions: vec![
        Condition::Simple {
            field: "department".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("Engineering".to_string()),
        },
        Condition::Simple {
            field: "salary".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Float(80000.0),
        },
    ],
}
```

##### Aggregation Conditions
Patterns across multiple facts with aggregation functions.

```rust
Condition::Aggregation(AggregationCondition {
    aggregation_type: AggregationType,   // SUM, COUNT, AVERAGE, etc.
    source_field: String,                // Field to aggregate
    group_by: Vec<String>,               // Grouping fields
    having: Option<Box<Condition>>,      // Post-aggregation filter
    alias: String,                       // Result field name
    window: Option<AggregationWindow>,   // Time/count window
})
```

**Aggregation Types:**
- `Sum` - Sum of numeric values
- `Count` - Count of records
- `Average` - Arithmetic mean
- `Min` - Minimum value
- `Max` - Maximum value
- `StandardDeviation` - Statistical deviation
- `Percentile(f64)` - Percentile calculation

**Example:**
```rust
Condition::Aggregation(AggregationCondition {
    aggregation_type: AggregationType::Sum,
    source_field: "hours_worked".to_string(),
    group_by: vec!["employee_id".to_string()],
    having: Some(Box::new(Condition::Simple {
        field: "total_hours".to_string(),
        operator: Operator::GreaterThan,
        value: FactValue::Float(40.0),
    })),
    alias: "total_hours".to_string(),
    window: Some(AggregationWindow::Time { duration_ms: 604800000 }), // 1 week
})
```

##### Stream Conditions
Time-windowed pattern matching for temporal data.

```rust
Condition::Stream(StreamCondition {
    window_spec: StreamWindowSpec,        // Window definition
    aggregation: StreamAggregation,       // Aggregation function
    filter: Option<Box<Condition>>,       // Pre-aggregation filter
    having: Option<Box<Condition>>,       // Post-aggregation filter
    alias: String,                        // Result field name
})
```

**Window Types:**
- `Tumbling { duration_ms: u64 }` - Non-overlapping time windows
- `Sliding { size_ms: u64, advance_ms: u64 }` - Overlapping time windows
- `Session { gap_timeout_ms: u64 }` - Activity-based windows
- `CountTumbling { count: usize }` - Count-based windows
- `CountSliding { size: usize, advance: usize }` - Sliding count windows

#### Action Types

##### SetField Action
Modifies a field value on the triggering fact.

```rust
ActionType::SetField {
    field: String,        // Field name to set
    value: FactValue,     // Value to assign
}
```

##### CreateFact Action
Generates a new fact with specified data.

```rust
ActionType::CreateFact {
    data: FactData,       // Complete fact data
}
```

##### CallCalculator Action
Invokes a business logic calculator.

```rust
ActionType::CallCalculator {
    calculator_name: String,                    // Calculator identifier
    input_mapping: HashMap<String, String>,     // Input field mapping
    output_field: String,                       // Result field name
}
```

##### Formula Action
Evaluates mathematical expressions.

```rust
ActionType::Formula {
    expression: String,   // Mathematical expression
    output_field: String, // Result field name
}
```

**Supported Operations:**
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Functions: `min()`, `max()`, `abs()`, `round()`
- Constants: `PI`, `E`
- Field references: Direct field names

**Example:**
```rust
ActionType::Formula {
    expression: "base_salary * 1.1 + bonus".to_string(),
    output_field: "total_compensation".to_string(),
}
```

##### TriggerAlert Action
Sends notifications to external systems.

```rust
ActionType::TriggerAlert {
    alert_type: String,                           // Alert category
    message: String,                              // Alert message
    severity: AlertSeverity,                      // Priority level
    metadata: HashMap<String, FactValue>,         // Additional data
}
```

**Alert Severity Levels:**
- `Low` - Informational alerts
- `Medium` - Warning conditions
- `High` - Error conditions
- `Critical` - Emergency conditions

---

## Fact Processing API

### Fact Structure

#### Fact Definition
```rust
pub struct Fact {
    pub id: FactId,                                      // Unique identifier
    pub external_id: Option<String>,                     // External system ID
    pub timestamp: chrono::DateTime<chrono::Utc>,        // Creation time
    pub data: FactData,                                  // Structured data
}
```

#### FactData Structure
```rust
pub struct FactData {
    pub fields: HashMap<String, FactValue>,   // Named fields
}
```

#### FactValue Types

##### Primitive Types
```rust
pub enum FactValue {
    Integer(i64),           // 64-bit signed integer
    Float(f64),            // 64-bit floating point
    String(String),        // UTF-8 string
    Boolean(bool),         // True/false value
    DateTime(chrono::DateTime<chrono::Utc>),  // ISO 8601 datetime
    // ... complex types below
}
```

##### Collection Types
```rust
pub enum FactValue {
    // ... primitive types above
    Array(Vec<FactValue>),                        // Ordered collection
    Object(HashMap<String, FactValue>),           // Key-value mapping
}
```

**Array Examples:**
```rust
// Numeric array
FactValue::Array(vec![
    FactValue::Integer(1),
    FactValue::Integer(2),
    FactValue::Integer(3),
])

// Mixed type array
FactValue::Array(vec![
    FactValue::String("item1".to_string()),
    FactValue::Float(99.99),
    FactValue::Boolean(true),
])
```

**Object Examples:**
```rust
// Nested object structure
FactValue::Object({
    let mut obj = HashMap::new();
    obj.insert("name".to_string(), FactValue::String("John".to_string()));
    obj.insert("age".to_string(), FactValue::Integer(30));
    obj.insert("active".to_string(), FactValue::Boolean(true));
    obj
})
```

#### Fact Construction

##### Basic Fact Creation
```rust
use bingo_core::types::{Fact, FactData, FactValue};
use std::collections::HashMap;

let mut fields = HashMap::new();
fields.insert("user_id".to_string(), FactValue::String("USER123".to_string()));
fields.insert("login_time".to_string(), FactValue::DateTime(chrono::Utc::now()));
fields.insert("session_duration".to_string(), FactValue::Integer(3600));

let fact = Fact::new(1, FactData { fields });
```

##### Fact with External ID
```rust
let mut fact = Fact::new(1, fact_data);
fact.external_id = Some("EXT_ID_12345".to_string());
```

#### Fact Utility Methods

##### Field Access
```rust
// Get field value
if let Some(user_id) = fact.get_field("user_id") {
    if let FactValue::String(id) = user_id {
        println!("User ID: {}", id);
    }
}
```

##### Type Conversion Helpers
```rust
impl FactValue {
    pub fn as_integer(&self) -> Option<i64> { /* ... */ }
    pub fn as_float(&self) -> Option<f64> { /* ... */ }
    pub fn as_string(&self) -> String { /* ... */ }
    pub fn as_boolean(&self) -> Option<bool> { /* ... */ }
    pub fn as_datetime(&self) -> Option<chrono::DateTime<chrono::Utc>> { /* ... */ }
}
```

---

## Calculator API

### Built-in Calculator Reference

#### Mathematical Calculators

##### Add Calculator
Performs addition of two numeric values.

**Name:** `"add"`

**Inputs:**
- `a: Number` - First operand
- `b: Number` - Second operand

**Output:** Sum of `a` and `b`

**Example:**
```rust
ActionType::CallCalculator {
    calculator_name: "add".to_string(),
    input_mapping: {
        let mut map = HashMap::new();
        map.insert("a".to_string(), "base_amount".to_string());
        map.insert("b".to_string(), "additional_amount".to_string());
        map
    },
    output_field: "total_amount".to_string(),
}
```

##### Multiply Calculator
Performs multiplication of two numeric values.

**Name:** `"multiply"`

**Inputs:**
- `a: Number` - Multiplicand
- `b: Number` - Multiplier

**Example:**
```rust
// Calculate total cost: quantity * unit_price
ActionType::CallCalculator {
    calculator_name: "multiply".to_string(),
    input_mapping: {
        let mut map = HashMap::new();
        map.insert("a".to_string(), "quantity".to_string());
        map.insert("b".to_string(), "unit_price".to_string());
        map
    },
    output_field: "total_cost".to_string(),
}
```

##### Percentage Add Calculator
Increases a value by a specified percentage.

**Name:** `"percentage_add"`

**Inputs:**
- `base: Number` - Base value
- `percentage: Number` - Percentage to add (e.g., 15 for 15%)

**Formula:** `base * (1 + percentage/100)`

##### Percentage Deduct Calculator
Decreases a value by a specified percentage.

**Name:** `"percentage_deduct"`

**Inputs:**
- `base: Number` - Base value
- `percentage: Number` - Percentage to deduct

**Formula:** `base * (1 - percentage/100)`

#### Aggregation Calculators

##### Weighted Average Calculator
Computes weighted average from an array of items.

**Name:** `"weighted_average"`

**Inputs:**
- `items: Array<Object>` - Array of objects with `value` and `weight` fields

**Example Input:**
```rust
FactValue::Array(vec![
    FactValue::Object({
        let mut item = HashMap::new();
        item.insert("value".to_string(), FactValue::Float(100.0));
        item.insert("weight".to_string(), FactValue::Float(2.0));
        item
    }),
    FactValue::Object({
        let mut item = HashMap::new();
        item.insert("value".to_string(), FactValue::Float(200.0));
        item.insert("weight".to_string(), FactValue::Float(3.0));
        item
    }),
])
```

**Formula:** `Σ(value_i * weight_i) / Σ(weight_i)`

##### Proportional Allocator Calculator
Distributes a total amount proportionally based on ratios.

**Name:** `"proportional_allocator"`

**Inputs:**
- `total: Number` - Total amount to distribute
- `ratios: Array<Number>` - Proportional ratios
- `target_index: Integer` - Index of the target allocation

#### Validation Calculators

##### Threshold Check Calculator
Checks if a value exceeds a threshold.

**Name:** `"threshold_check"`

**Inputs:**
- `value: Number` - Value to check
- `threshold: Number` - Threshold value

**Output:** `"true"` if value > threshold, `"false"` otherwise

##### Limit Validator Calculator
Validates that a value falls within specified bounds.

**Name:** `"limit_validator"`

**Inputs:**
- `value: Number` - Value to validate
- `min: Number` - Minimum allowed value
- `max: Number` - Maximum allowed value

**Output:** `"true"` if min ≤ value ≤ max, `"false"` otherwise

#### Time Calculators

##### Time Between DateTime Calculator
Calculates time difference between two datetime values.

**Name:** `"time_between_datetime"`

**Inputs:**
- `start_time: DateTime` - Start timestamp
- `end_time: DateTime` - End timestamp
- `unit: String` - Time unit ("seconds", "minutes", "hours", "days")

**Output:** Numeric difference in specified units

### Custom Calculator Development

#### Calculator Plugin Interface
```rust
pub trait CalculatorPlugin {
    /// Returns the unique name identifier for this calculator
    fn name(&self) -> &str;
    
    /// Performs the calculation with provided arguments
    fn calculate(&self, args: &HashMap<String, &FactValue>) -> CalculationResult;
}

pub type CalculationResult = Result<FactValue, String>;
```

#### Example Custom Calculator
```rust
use bingo_calculator::plugin::{CalculatorPlugin, CalculationResult};
use bingo_types::FactValue;
use std::collections::HashMap;

pub struct CompoundInterestCalculator;

impl CalculatorPlugin for CompoundInterestCalculator {
    fn name(&self) -> &str {
        "compound_interest"
    }
    
    fn calculate(&self, args: &HashMap<String, &FactValue>) -> CalculationResult {
        let principal = match args.get("principal") {
            Some(FactValue::Float(p)) => *p,
            Some(FactValue::Integer(p)) => *p as f64,
            _ => return Err("Invalid argument 'principal': expected number".to_string()),
        };
        
        let rate = match args.get("rate") {
            Some(FactValue::Float(r)) => *r,
            Some(FactValue::Integer(r)) => *r as f64,
            _ => return Err("Invalid argument 'rate': expected number".to_string()),
        };
        
        let time = match args.get("time") {
            Some(FactValue::Float(t)) => *t,
            Some(FactValue::Integer(t)) => *t as f64,
            _ => return Err("Invalid argument 'time': expected number".to_string()),
        };
        
        let compound_frequency = match args.get("compound_frequency") {
            Some(FactValue::Float(n)) => *n,
            Some(FactValue::Integer(n)) => *n as f64,
            _ => 1.0, // Default to annual compounding
        };
        
        // Formula: A = P(1 + r/n)^(nt)
        let amount = principal * (1.0 + rate / compound_frequency).powf(compound_frequency * time);
        
        Ok(FactValue::Float(amount))
    }
}
```

#### Calculator Registration
```rust
use bingo_calculator::Calculator;

let mut calculator = Calculator::new();
calculator.register_plugin(Box::new(CompoundInterestCalculator));
```

---

## Type System API

### Core Types

#### Identifiers
```rust
pub type FactId = u64;      // Unique fact identifier
pub type RuleId = u64;      // Unique rule identifier
pub type NodeId = u64;      // RETE network node identifier
```

#### Result Types
```rust
pub type BingoResult<T> = Result<T, BingoError>;
pub type CalculationResult = Result<FactValue, String>;
```

### Serialization Support

All core types implement Serde serialization for JSON/binary encoding:

```rust
use serde_json;

// Serialize fact to JSON
let fact_json = serde_json::to_string(&fact)?;

// Deserialize from JSON
let fact: Fact = serde_json::from_str(&fact_json)?;
```

### Type Conversion Utilities

#### FactValue Conversions
```rust
impl FactValue {
    pub fn from_json(value: serde_json::Value) -> Self { /* ... */ }
    pub fn to_json(&self) -> serde_json::Value { /* ... */ }
    
    pub fn type_name(&self) -> &'static str { /* ... */ }
    pub fn is_numeric(&self) -> bool { /* ... */ }
    pub fn is_collection(&self) -> bool { /* ... */ }
}
```

#### Validation Helpers
```rust
impl FactValue {
    pub fn validate_type(&self, expected: &str) -> Result<(), String> { /* ... */ }
    pub fn coerce_to_number(&self) -> Option<f64> { /* ... */ }
    pub fn coerce_to_string(&self) -> String { /* ... */ }
}
```

---

## Error Handling API

### BingoError Enumeration

#### Error Categories
```rust
pub enum BingoError {
    /// Rule compilation and validation errors
    Rule {
        message: String,
        rule_id: Option<u64>,
        rule_name: Option<String>,
        details: Option<String>,
    },
    
    /// RETE network processing errors
    ReteNetwork {
        message: String,
        node_id: Option<u64>,
        operation: String,
        details: Option<String>,
    },
    
    /// Fact processing and validation errors
    Fact {
        message: String,
        fact_id: Option<u64>,
        field_name: Option<String>,
        details: Option<String>,
    },
    
    /// Calculator execution errors
    Calculator {
        message: String,
        calculator_name: String,
        input_details: Option<String>,
        details: Option<String>,
    },
    
    /// Action execution errors
    Action {
        message: String,
        action_type: String,
        rule_id: Option<u64>,
        details: Option<String>,
    },
    
    /// Memory and resource management errors
    Memory {
        message: String,
        operation: String,
        current_usage: Option<usize>,
        limit: Option<usize>,
    },
    
    /// Engine configuration and initialization errors
    Configuration {
        message: String,
        parameter: Option<String>,
        expected_type: Option<String>,
        details: Option<String>,
    },
    
    /// Performance monitoring and profiling errors
    Performance {
        message: String,
        operation: String,
        duration_ms: Option<u64>,
        details: Option<String>,
    },
    
    /// External system integration errors
    External {
        message: String,
        system: String,
        error_code: Option<String>,
        details: Option<String>,
    },
    
    /// Data serialization and format errors
    Serialization {
        message: String,
        format: String,
        details: Option<String>,
    },
    
    /// Validation and constraint errors
    Validation {
        message: String,
        field: Option<String>,
        constraint: Option<String>,
        details: Option<String>,
    },
    
    /// General internal errors
    Internal {
        message: String,
        operation: String,
        details: Option<String>,
    },
}
```

#### Error Severity Levels
```rust
pub enum ErrorSeverity {
    Low,        // Warnings and informational
    Medium,     // Recoverable errors
    High,       // Serious errors requiring attention
    Critical,   // System-threatening errors
}
```

#### Error Context Enhancement
```rust
pub trait ResultExt<T> {
    fn with_context<F>(self, f: F) -> BingoResult<T>
    where
        F: FnOnce() -> String;
        
    fn with_rule_context(self, rule_id: u64, rule_name: &str) -> BingoResult<T>;
    fn with_fact_context(self, fact_id: u64) -> BingoResult<T>;
    fn with_calculator_context(self, calculator_name: &str) -> BingoResult<T>;
}
```

**Usage Example:**
```rust
use bingo_core::error::ResultExt;

fn process_rule(rule: &Rule) -> BingoResult<()> {
    validate_rule_conditions(&rule.conditions)
        .with_rule_context(rule.id, &rule.name)?;
    
    compile_rule_to_network(rule)
        .with_context(|| format!("Failed to compile rule '{}'", rule.name))?;
    
    Ok(())
}
```

---

## Performance Monitoring API

### EngineProfiler

#### Real-time Performance Monitoring
```rust
pub struct EngineProfiler {
    // Internal implementation details
}

impl EngineProfiler {
    pub fn new() -> Self { /* ... */ }
    pub fn with_thresholds(thresholds: PerformanceThresholds) -> Self { /* ... */ }
    
    /// Start timing an operation
    pub fn start_operation(&mut self, operation: &str) { /* ... */ }
    
    /// End timing and record duration
    pub fn end_operation(&mut self, operation: &str) -> Option<Duration> { /* ... */ }
    
    /// Time a closure and return its result
    pub fn time_operation<T, F>(&mut self, operation: &str, f: F) -> T
    where
        F: FnOnce() -> T { /* ... */ }
    
    /// Get comprehensive performance report
    pub fn get_performance_report(&self) -> PerformanceReport { /* ... */ }
    
    /// Check for performance threshold violations
    pub fn check_alerts(&self) -> Vec<PerformanceAlert> { /* ... */ }
}
```

#### Performance Thresholds
```rust
pub struct PerformanceThresholds {
    pub rule_compilation_ms: u64,      // Max rule compilation time
    pub fact_processing_ms: u64,       // Max fact processing time
    pub calculator_execution_ms: u64,  // Max calculator execution time
    pub memory_usage_mb: usize,        // Max memory usage
    pub throughput_facts_per_sec: f64, // Min throughput requirement
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            rule_compilation_ms: 100,
            fact_processing_ms: 50,
            calculator_execution_ms: 10,
            memory_usage_mb: 1024,
            throughput_facts_per_sec: 1000.0,
        }
    }
}
```

#### Performance Reports
```rust
pub struct PerformanceReport {
    pub operation_metrics: HashMap<String, OperationMetrics>,
    pub bottleneck_analysis: BottleneckAnalysis,
    pub memory_usage: MemoryUsage,
    pub throughput_stats: ThroughputStats,
    pub alert_summary: AlertSummary,
}

pub struct OperationMetrics {
    pub total_calls: u64,
    pub total_time: Duration,
    pub average_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub p95_time: Duration,
    pub p99_time: Duration,
}

pub struct BottleneckAnalysis {
    pub slowest_operations: Vec<(String, Duration)>,
    pub high_frequency_operations: Vec<(String, u64)>,
    pub recommendations: Vec<String>,
}
```

#### Usage Example
```rust
use bingo_core::profiler::{EngineProfiler, PerformanceThresholds};

let mut profiler = EngineProfiler::with_thresholds(PerformanceThresholds {
    rule_compilation_ms: 50,
    fact_processing_ms: 25,
    calculator_execution_ms: 5,
    memory_usage_mb: 512,
    throughput_facts_per_sec: 2000.0,
});

// Time an operation
let result = profiler.time_operation("rule_compilation", || {
    engine.add_rule(complex_rule)
});

// Get performance report
let report = profiler.get_performance_report();
println!("Average rule compilation time: {:?}", 
         report.operation_metrics["rule_compilation"].average_time);

// Check for alerts
let alerts = profiler.check_alerts();
for alert in alerts {
    match alert.severity {
        AlertSeverity::Critical => eprintln!("CRITICAL: {}", alert.message),
        AlertSeverity::High => println!("WARNING: {}", alert.message),
        _ => {}
    }
}
```

---

## gRPC API

### Service Definition

The Bingo engine exposes a comprehensive gRPC API defined in Protocol Buffers. The complete service definition is available in [specs/grpc-api.md](../specs/grpc-api.md).

#### Core Services
```protobuf
service RulesEngineService {
    // Rule management
    rpc AddRule(AddRuleRequest) returns (AddRuleResponse);
    rpc UpdateRule(UpdateRuleRequest) returns (UpdateRuleResponse);
    rpc RemoveRule(RemoveRuleRequest) returns (RemoveRuleResponse);
    rpc ListRules(ListRulesRequest) returns (ListRulesResponse);
    
    // Fact processing
    rpc ProcessFacts(ProcessFactsRequest) returns (ProcessFactsResponse);
    rpc ProcessFactsStream(stream ProcessFactsStreamRequest) returns (stream ProcessFactsStreamResponse);
    
    // Engine management
    rpc GetEngineStats(GetEngineStatsRequest) returns (GetEngineStatsResponse);
    rpc ClearEngine(ClearEngineRequest) returns (ClearEngineResponse);
    
    // Health and diagnostics
    rpc Health(HealthRequest) returns (HealthResponse);
}
```

#### Streaming API
```protobuf
// Bidirectional streaming for large datasets
rpc ProcessFactsStream(stream ProcessFactsStreamRequest) returns (stream ProcessFactsStreamResponse);
```

**Benefits:**
- Memory-efficient processing of large fact sets
- Real-time result streaming
- Flow control and backpressure handling
- Connection multiplexing

#### Error Handling
```protobuf
message ErrorDetails {
    string error_code = 1;
    string message = 2;
    string category = 3;
    ErrorSeverity severity = 4;
    map<string, string> context = 5;
}

enum ErrorSeverity {
    ERROR_SEVERITY_UNSPECIFIED = 0;
    ERROR_SEVERITY_LOW = 1;
    ERROR_SEVERITY_MEDIUM = 2;
    ERROR_SEVERITY_HIGH = 3;
    ERROR_SEVERITY_CRITICAL = 4;
}
```

### Client Examples

#### Rust Client
```rust
use tonic::transport::Channel;
use bingo_api::generated::rules_engine_service_client::RulesEngineServiceClient;
use bingo_api::generated::{ProcessFactsRequest, Fact, FactValue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://127.0.0.1:50051").connect().await?;
    let mut client = RulesEngineServiceClient::new(channel);
    
    let request = tonic::Request::new(ProcessFactsRequest {
        facts: vec![
            Fact {
                id: 1,
                external_id: Some("TXN001".to_string()),
                timestamp: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
                data: Some(FactData {
                    fields: {
                        let mut fields = std::collections::HashMap::new();
                        fields.insert("amount".to_string(), FactValue {
                            value: Some(fact_value::Value::FloatValue(15000.0)),
                        });
                        fields
                    },
                }),
            }
        ],
    });
    
    let response = client.process_facts(request).await?;
    println!("Response: {:?}", response.into_inner());
    
    Ok(())
}
```

#### Python Client
```python
import grpc
from bingo_api import rules_engine_pb2_grpc, rules_engine_pb2
from google.protobuf.timestamp_pb2 import Timestamp
import time

def main():
    channel = grpc.insecure_channel('localhost:50051')
    client = rules_engine_pb2_grpc.RulesEngineServiceStub(channel)
    
    # Create timestamp
    timestamp = Timestamp()
    timestamp.FromDatetime(datetime.utcnow())
    
    # Create fact
    fact = rules_engine_pb2.Fact(
        id=1,
        external_id="TXN001",
        timestamp=timestamp,
        data=rules_engine_pb2.FactData(
            fields={
                "amount": rules_engine_pb2.FactValue(float_value=15000.0),
                "currency": rules_engine_pb2.FactValue(string_value="USD")
            }
        )
    )
    
    # Process facts
    request = rules_engine_pb2.ProcessFactsRequest(facts=[fact])
    response = client.ProcessFacts(request)
    
    print(f"Processed {len(response.results)} rule executions")

if __name__ == "__main__":
    main()
```

---

This comprehensive API reference provides complete documentation for all public interfaces in the Bingo RETE Rules Engine. Each section includes detailed parameter descriptions, usage examples, and performance characteristics to enable effective implementation and optimization.