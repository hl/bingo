# Bingo RETE Rules Engine - Comprehensive Documentation Guide

This document serves as the master index for all Bingo engine documentation, providing a structured guide to understanding, implementing, and maintaining the system.

## 📋 Documentation Overview

This documentation suite provides complete coverage of the Bingo RETE Rules Engine, from basic concepts to advanced implementation details.

## 🎯 Getting Started

### Essential Reading (In Order)
1. **[README.md](../README.md)** - Project overview, features, and quick start
2. **[Architecture Overview](../specs/architecture.md)** - System design and component relationships
3. **[Quick Start Guide](#quick-start-workflows)** - Common usage patterns and examples
4. **[API Reference](#api-documentation)** - Complete API documentation

### Quick Start Workflows

#### Basic Rule Engine Usage
```rust
use bingo_core::{BingoEngine, Rule, Condition, Action, ActionType, Fact, FactData, FactValue, Operator};
use std::collections::HashMap;

// 1. Create engine
let mut engine = BingoEngine::new()?;

// 2. Define rule
let rule = Rule {
    id: 1,
    name: "High Value Transaction".to_string(),
    conditions: vec![Condition::Simple {
        field: "amount".to_string(),
        operator: Operator::GreaterThan,
        value: FactValue::Float(10000.0),
    }],
    actions: vec![Action {
        action_type: ActionType::SetField {
            field: "risk_level".to_string(),
            value: FactValue::String("high".to_string()),
        },
    }],
};

// 3. Add rule to engine
engine.add_rule(rule)?;

// 4. Create and process facts
let mut fields = HashMap::new();
fields.insert("transaction_id".to_string(), FactValue::String("TXN001".to_string()));
fields.insert("amount".to_string(), FactValue::Float(15000.0));

let fact = Fact::new(1, FactData { fields });
let results = engine.process_facts(vec![fact])?;

// 5. Process results
for result in results {
    println!("Rule {} fired for fact {}", result.rule_id, result.fact_id);
    for action in result.actions_executed {
        println!("Action executed: {:?}", action);
    }
}
```

#### gRPC API Usage
```bash
# Start the server
cargo run --release --bin bingo

# Example gRPC call (using grpcurl)
grpcurl -plaintext localhost:50051 bingo.v1.EngineService/ProcessFacts
```

## 🏗️ Architecture Documentation

### Core Components
- **[Core Engine](../specs/architecture.md#core-engine)** - RETE network implementation and fact processing
- **[Calculator System](../specs/built-in-calculators.md)** - Pluggable business logic calculators
- **[Type System](../specs/architecture.md#type-system)** - Shared data structures and serialization
- **[API Layer](../specs/grpc-api.md)** - gRPC interface and protocol definitions

### Deep Dive Topics
- **[RETE Algorithm Implementation](../specs/rete-algorithm-implementation.md)** - Technical details of the RETE network
- **[Memory Management](../specs/architecture.md#memory-management)** - Arena allocation and optimization strategies
- **[Performance Characteristics](../specs/performance.md)** - Benchmarks and scaling analysis

## 📚 API Documentation

### Core API
- **[BingoEngine API](#engine-api-reference)** - Main engine interface
- **[Rule Definition API](#rule-api-reference)** - Rule creation and management
- **[Fact Processing API](#fact-api-reference)** - Fact ingestion and processing
- **[Calculator API](#calculator-api-reference)** - Built-in and custom calculators

### gRPC API
- **[gRPC Service Definition](../specs/grpc-api.md)** - Complete protocol buffer specifications
- **[Client Setup Guide](client-setup.md)** - Multi-language client implementation
- **[Streaming API](../specs/grpc-api.md#streaming)** - Bidirectional streaming for large datasets

## 🧮 Calculator Documentation

### Built-in Calculators
- **[Mathematical Calculators](../specs/built-in-calculators.md#mathematical)** - Add, multiply, percentage operations
- **[Aggregation Calculators](../specs/built-in-calculators.md#aggregation)** - Weighted averages, proportional allocation
- **[Validation Calculators](../specs/built-in-calculators.md#validation)** - Threshold checks, limit validation
- **[Time Calculators](../specs/built-in-calculators.md#time)** - DateTime operations and calculations

### Custom Calculator Development
- **[Calculator Plugin Guide](../specs/calculator-dsl-guide.md)** - Creating custom business logic
- **[Plugin Architecture](../specs/architecture.md#calculator-plugins)** - Integration patterns and best practices

## 🏢 Business Engine Guides

### Domain-Specific Implementations
- **[Compliance Engine](compliance-engine.md)** - Regulatory compliance and monitoring
- **[Payroll Engine](payroll-engine.md)** - Payroll processing and calculation workflows
- **[TRONC Engine](tronc-engine.md)** - Tip and gratuity distribution systems
- **[Wage Cost Estimation](wage-cost-estimation-engine.md)** - Cost modeling and estimation

### Business Rule Patterns
- **[Rule Specification Guide](../specs/rule-specification.md)** - Best practices for rule design
- **[Complex Workflow Patterns](#workflow-patterns)** - Multi-stage processing and cascading rules
- **[Error Handling Patterns](#error-handling-patterns)** - Robust error management strategies

## 🔧 Development & Operations

### Development Workflow
- **[Development Setup](#development-setup)** - Environment configuration and tooling
- **[Testing Strategy](#testing-documentation)** - Unit, integration, and performance testing
- **[Code Quality Standards](#code-quality)** - Formatting, linting, and style guidelines

### Operations & Deployment
- **[gRPC Deployment Guide](grpc-deployment-guide.md)** - Production deployment strategies
- **[Performance Testing](performance-testing.md)** - Load testing and benchmarking
- **[Performance Analysis](performance-tests.md)** - Detailed performance test results
- **[Monitoring & Observability](#monitoring-setup)** - Logging, metrics, and tracing

## 🧪 Testing Documentation

### Test Suites
- **[Unit Tests](#unit-test-coverage)** - Component-level testing
- **[Integration Tests](#integration-test-guide)** - End-to-end workflow testing
- **[Performance Tests](performance-tests.md)** - Scalability and benchmark testing
- **[Configuration Testing](testing-settings.md)** - Testing environment setup

### Test Categories
- **Engine Core Tests** - RETE network, fact store, and rule processing
- **Calculator Tests** - Built-in and custom calculator validation
- **API Tests** - gRPC interface and protocol testing
- **Workflow Tests** - Business logic and multi-stage processing

## 📊 Performance & Optimization

### Performance Documentation
- **[Performance Benchmarks](performance-tests.md)** - Comprehensive performance analysis
- **[Optimization Strategies](../specs/performance.md)** - Memory and CPU optimization techniques
- **[Scaling Guidelines](#scaling-guidelines)** - Horizontal and vertical scaling approaches

### Monitoring & Profiling
- **[Request Lifecycle](request-lifecycle.md)** - End-to-end request tracing
- **[Cache Management](cache-lifecycle.md)** - Caching strategies and lifecycle management
- **[Performance Profiling](#profiling-guide)** - Tools and techniques for performance analysis

## 🔍 Troubleshooting & Support

### Common Issues
- **[Troubleshooting Guide](#troubleshooting-common-issues)** - Solutions to frequent problems
- **[Error Reference](#error-code-reference)** - Complete error code documentation
- **[Debug Strategies](#debugging-techniques)** - Debugging tools and techniques

### Support Resources
- **[FAQ](#frequently-asked-questions)** - Common questions and answers
- **[Best Practices](#best-practices-guide)** - Proven patterns and recommendations
- **[Migration Guide](#version-migration)** - Upgrading between versions

---

## Engine API Reference

### BingoEngine Core Methods

#### `BingoEngine::new() -> BingoResult<BingoEngine>`
Creates a new instance of the rules engine with default configuration.

**Returns:**
- `Ok(BingoEngine)` - Initialized engine instance
- `Err(BingoError)` - Initialization error

**Example:**
```rust
use bingo_core::BingoEngine;

let engine = BingoEngine::new()?;
```

#### `BingoEngine::with_capacity(capacity: usize) -> BingoResult<BingoEngine>`
Creates a new engine instance with pre-allocated capacity for facts and rules.

**Parameters:**
- `capacity` - Initial capacity for internal data structures

**Example:**
```rust
let engine = BingoEngine::with_capacity(10000)?;
```

#### `add_rule(&mut self, rule: Rule) -> BingoResult<()>`
Adds a rule to the engine and compiles it into the RETE network.

**Parameters:**
- `rule` - Rule definition with conditions and actions

**Returns:**
- `Ok(())` - Rule successfully added and compiled
- `Err(BingoError)` - Rule compilation error

#### `process_facts(&mut self, facts: Vec<Fact>) -> BingoResult<Vec<RuleExecutionResult>>`
Processes facts through the RETE network and executes triggered rules.

**Parameters:**
- `facts` - Vector of facts to process

**Returns:**
- `Ok(Vec<RuleExecutionResult>)` - Results of rule executions
- `Err(BingoError)` - Processing error

## Rule API Reference

### Rule Structure
```rust
pub struct Rule {
    pub id: RuleId,
    pub name: String,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
}
```

### Condition Types
- **Simple Conditions** - Direct field comparisons
- **Complex Conditions** - Logical combinations (AND, OR, NOT)
- **Aggregation Conditions** - Multi-fact aggregations
- **Stream Conditions** - Time-windowed patterns

### Action Types
- **SetField** - Modify fact fields
- **CreateFact** - Generate new facts
- **CallCalculator** - Execute business logic
- **Formula** - Mathematical expressions
- **TriggerAlert** - External notifications

## Fact API Reference

### Fact Structure
```rust
pub struct Fact {
    pub id: FactId,
    pub external_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: FactData,
}
```

### FactValue Types
- **Integer** - 64-bit signed integers
- **Float** - 64-bit floating point numbers
- **String** - UTF-8 strings
- **Boolean** - True/false values
- **Array** - Ordered collections
- **Object** - Key-value mappings
- **DateTime** - ISO 8601 timestamps

## Calculator API Reference

### Built-in Calculators
- `add` - Addition operations
- `multiply` - Multiplication operations
- `percentage_add` - Percentage increases
- `percentage_deduct` - Percentage decreases
- `weighted_average` - Weighted calculations
- `proportional_allocator` - Proportional distribution
- `threshold_check` - Threshold validation
- `limit_validator` - Range validation
- `time_between_datetime` - Time calculations

### Custom Calculator Interface
```rust
pub trait CalculatorPlugin {
    fn name(&self) -> &str;
    fn calculate(&self, args: &HashMap<String, &FactValue>) -> CalculationResult;
}
```

## Workflow Patterns

### Multi-Stage Processing
```rust
// Stage 1: Validation
let validation_rule = Rule {
    id: 1,
    name: "Data Validation".to_string(),
    conditions: vec![/* validation conditions */],
    actions: vec![/* validation actions */],
};

// Stage 2: Processing
let processing_rule = Rule {
    id: 2,
    name: "Data Processing".to_string(),
    conditions: vec![/* processing conditions */],
    actions: vec![/* processing actions */],
};

// Stage 3: Notification
let notification_rule = Rule {
    id: 3,
    name: "Notification".to_string(),
    conditions: vec![/* notification conditions */],
    actions: vec![/* notification actions */],
};
```

### Cascading Rules
Rules can trigger other rules by creating or modifying facts that match subsequent rule conditions.

### Error Handling Patterns
```rust
// Graceful error handling with logging
Action {
    action_type: ActionType::CallCalculator {
        calculator_name: "custom_calculator".to_string(),
        input_mapping: input_map,
        output_field: "result".to_string(),
    },
}
// If calculator fails, error is logged automatically
```

---

This comprehensive guide provides complete documentation coverage for the Bingo RETE Rules Engine. Each section links to detailed documentation and provides practical examples for implementation and usage.