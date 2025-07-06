# Bingo Rules Engine - Language Implementation Specification

This document provides comprehensive specifications for implementing the Bingo Rules Engine in another programming language. It covers all data structures, algorithms, and behavioural requirements needed to create a compatible implementation.

## Overview

The Bingo Rules Engine is a high-performance, memory-optimized rules engine based on the RETE algorithm for pattern matching. It processes facts against rules to trigger actions, supporting complex conditions, aggregations, streaming windows, and business calculator integrations.

## Core Data Structures

### Facts and Values

#### FactValue Enum
```typescript
enum FactValue {
  String(string),
  Integer(number), // i64 equivalent
  Float(number),   // f64 equivalent
  Boolean(boolean),
  Array(FactValue[]),
  Object(Record<string, FactValue>),
  Date(Date),      // ISO 8601 UTC timestamps
  Null
}
```

**Type Conversion Requirements:**
- Implement `as_integer()`, `as_float()`, `as_string()` methods
- Support cross-type comparisons (Integer ↔ Float)
- Implement `is_truthy()` for conditional logic
- Hash implementation must be consistent across types

#### Fact Structure
```typescript
interface Fact {
  id: number,                    // Unique fact identifier (u64)
  external_id?: string,          // Optional external reference
  timestamp: Date,               // UTC timestamp
  data: FactData
}

interface FactData {
  fields: Record<string, FactValue>
}
```

### Rules and Conditions

#### Rule Structure
```typescript
interface Rule {
  id: number,                    // Unique rule identifier (u64)
  name: string,
  conditions: Condition[],
  actions: Action[]
}
```

#### Condition Types
```typescript
type Condition = 
  | SimpleCondition
  | ComplexCondition
  | AggregationCondition
  | StreamCondition

interface SimpleCondition {
  field: string,
  operator: Operator,
  value: FactValue
}

interface ComplexCondition {
  operator: LogicalOperator,
  conditions: Condition[]
}

interface AggregationCondition {
  aggregation_type: AggregationType,
  source_field: string,
  group_by: string[],
  having?: Condition,
  alias: string,
  window?: AggregationWindow
}

interface StreamCondition {
  window_spec: StreamWindowSpec,
  aggregation: StreamAggregation,
  filter?: Condition,
  having?: Condition,
  alias: string
}
```

#### Operators
```typescript
enum Operator {
  Equal,
  NotEqual,
  GreaterThan,
  LessThan,
  GreaterThanOrEqual,
  LessThanOrEqual,
  Contains
}

enum LogicalOperator {
  And,
  Or,
  Not
}
```

### Actions

#### Action Types
```typescript
interface Action {
  action_type: ActionType
}

type ActionType = 
  | LogAction
  | SetFieldAction
  | CreateFactAction
  | CallCalculatorAction
  | TriggerAlertAction
  | FormulaAction
  | UpdateFactAction
  | DeleteFactAction
  | IncrementFieldAction
  | AppendToArrayAction
  | SendNotificationAction

interface LogAction {
  type: "Log",
  message: string
}

interface SetFieldAction {
  type: "SetField",
  field: string,
  value: FactValue
}

interface CreateFactAction {
  type: "CreateFact",
  data: FactData
}

interface CallCalculatorAction {
  type: "CallCalculator",
  calculator_name: string,
  input_mapping: Record<string, string>,
  output_field: string
}

interface FormulaAction {
  type: "Formula",
  expression: string,
  output_field: string
}
```

## RETE Network Implementation

### Node Types

#### Alpha Nodes
- **Purpose**: Test single fact conditions
- **Implementation**: Hash fact fields and apply operator tests
- **Optimisation**: Use field indexing for fast lookups

```typescript
interface AlphaNode {
  id: number,
  condition: Condition,
  matches(fact: Fact): boolean,
  propagate(fact: Fact): void
}
```

#### Beta Nodes  
- **Purpose**: Join multiple facts and test cross-fact conditions
- **Implementation**: Maintain working memory of fact combinations
- **Optimisation**: Use hash joins where possible

```typescript
interface BetaNode {
  id: number,
  rule_ids: number[],
  left_memory: Fact[],
  right_memory: Fact[],
  join(left_fact: Fact, right_fact: Fact): boolean,
  propagate(fact_combination: Fact[]): void
}
```

#### Terminal Nodes
- **Purpose**: Execute rule actions when all conditions match
- **Implementation**: Receive complete fact combinations and trigger actions

```typescript
interface TerminalNode {
  id: number,
  rule_id: number,
  actions: Action[],
  execute(facts: Fact[]): RuleExecutionResult
}
```

### Network Construction

1. **Rule Analysis**: Parse conditions to identify fact patterns
2. **Alpha Network**: Create alpha nodes for single-fact tests  
3. **Beta Network**: Build join network for multi-fact patterns
4. **Action Nodes**: Attach terminal nodes for rule execution

### Fact Processing Algorithm

```typescript
function process_fact(fact: Fact, network: ReteNetwork): RuleExecutionResult[] {
  let results: RuleExecutionResult[] = []
  
  // Phase 1: Alpha network propagation
  for (const alpha_node of network.alpha_nodes) {
    if (alpha_node.matches(fact)) {
      alpha_node.propagate(fact)
    }
  }
  
  // Phase 2: Beta network joins
  for (const beta_node of network.beta_nodes) {
    const new_combinations = beta_node.try_join(fact)
    for (const combination of new_combinations) {
      beta_node.propagate(combination)
    }
  }
  
  // Phase 3: Terminal node execution
  for (const terminal_node of network.activated_terminals) {
    const result = terminal_node.execute()
    results.push(result)
  }
  
  return results
}
```

## Aggregation and Streaming

### Aggregation Types
```typescript
enum AggregationType {
  Sum,
  Count,
  Average,
  Min,
  Max,
  StandardDeviation,
  Percentile(number) // percentile value 0-100
}
```

### Stream Windows
```typescript
type StreamWindowSpec = 
  | TumblingWindow
  | SlidingWindow  
  | SessionWindow
  | CountTumblingWindow
  | CountSlidingWindow

interface TumblingWindow {
  type: "Tumbling",
  duration_ms: number
}

interface SlidingWindow {
  type: "Sliding", 
  size_ms: number,
  advance_ms: number
}

interface SessionWindow {
  type: "Session",
  gap_timeout_ms: number
}
```

### Streaming Implementation Requirements

1. **Window Management**: Maintain fact collections per window
2. **Time-based Triggers**: Process windows on time boundaries
3. **Incremental Updates**: Efficiently update aggregations
4. **Memory Cleanup**: Remove expired facts from windows

## Calculator Integration

### Calculator Interface
```typescript
interface Calculator {
  id: string,
  description: string,
  calculator_type: CalculatorType,
  conditions: Condition[],
  metadata: CalculatorMetadata
}

interface CalculatorInputs {
  fields: Record<string, FactValue>
}

interface CalculatorResult {
  success: boolean,
  result?: FactValue,
  error?: CalculatorError
}
```

### Built-in Calculator Types
- **ApplyPercentage**: Apply percentage to source field
- **ApplyFlat**: Add flat amount to target field  
- **ApplyFormula**: Evaluate mathematical expression
- **TieredRate**: Apply tiered rate calculations
- **ConditionalRate**: Apply conditional rate tables
- **AccumulateValue**: Accumulate values with grouping

## Performance Requirements

### Memory Optimisation
- **Fact Store**: Use arena allocation for facts
- **Working Memory**: Pool reusable data structures
- **Node Memory**: Minimise per-node memory overhead
- **Hash Maps**: Use object pools for temporary calculations

### Execution Performance
- **Index Usage**: Leverage field indices for fact lookups
- **Lazy Evaluation**: Defer expensive computations
- **Batch Processing**: Process multiple facts efficiently
- **Parallel Execution**: Support concurrent rule evaluation where safe

### Benchmarks (Reference Implementation)
- **Simple Rules**: >1M facts/second with <100 rules
- **Complex Rules**: >100K facts/second with aggregations
- **Memory Usage**: <1KB per active fact in working memory
- **Startup Time**: <100ms for 1000 rules compilation

## Error Handling

### Error Types
```typescript
interface CalculatorError {
  code: ErrorCode,
  message: string,
  details?: Record<string, FactValue>
}

enum ErrorCode {
  MissingRequiredField,
  InvalidFieldType,
  InvalidFieldValue,
  CalculationOverflow,
  BusinessRuleViolation,
  ConfigurationError
}
```

### Error Propagation
- Errors in conditions should prevent rule firing
- Errors in actions should be captured but not halt processing
- Provide detailed error context for debugging

## Serialisation and API Compatibility

### JSON Serialisation Requirements
- All data structures must serialise to/from JSON
- Date fields must use ISO 8601 format with UTC timezone
- Numeric precision must be preserved for financial calculations
- Enum values should use string representations

### gRPC Protobuf Compatibility
The implementation should support gRPC protocol buffers as defined in:
- `rules_engine.v1.proto` service definitions
- Streaming fact processing
- Rule compilation and session management
- Health check endpoints

## Testing Requirements

### Unit Tests
- Test all operators against all value type combinations  
- Verify aggregation calculations with known datasets
- Test streaming window behaviour with time-series data
- Validate calculator execution with edge cases

### Integration Tests  
- End-to-end rule processing with complex scenarios
- Performance tests with large fact volumes
- Memory leak detection during long-running processing
- Concurrent access patterns and thread safety

### Compatibility Tests
- JSON round-trip serialisation for all data types
- Cross-language fact processing with reference implementation
- gRPC API compatibility with existing clients

## Implementation Guidelines

### Memory Management
- Use arena/region allocation for facts where possible
- Pool frequently allocated objects (hashmaps, vectors)
- Implement efficient cleanup of expired data
- Monitor memory usage and implement limits

### Thread Safety
- Engine instances should be thread-safe for concurrent fact processing
- Rule compilation can be single-threaded
- Shared state should use appropriate synchronisation primitives

### Extensibility
- Calculator interface should allow custom implementations
- Action types should be extensible through interfaces
- Network node types should support custom implementations

### Debugging Support  
- Provide rule execution tracing capabilities
- Support fact visualisation and rule matching explanation
- Include performance profiling hooks
- Generate detailed error messages with context

## Reference Implementation

The canonical Rust implementation provides:
- `bingo-core` crate with RETE network and fact processing
- `bingo-calculator` crate with business calculator implementations  
- `bingo-api` crate with gRPC streaming service
- Comprehensive test suite with performance benchmarks

Use this specification alongside the Rust codebase to ensure behavioural compatibility and performance characteristics in your target language implementation.