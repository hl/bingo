#![deny(warnings)]
#![allow(missing_docs)]
//! # Bingo Core - High-Performance RETE Rules Engine
//!
//! ## Overview
//!
//! Bingo Core is a high-performance business rules engine built on the RETE algorithm,
//! designed for real-time fact processing and pattern matching. It provides efficient
//! rule compilation, fast fact processing, and comprehensive monitoring capabilities.
//!
//! ## Key Features
//!
//! - **RETE Algorithm**: Optimized pattern matching with shared network nodes
//! - **High Throughput**: Process thousands of facts per second
//! - **Memory Efficient**: Arena allocation and object pooling
//! - **Thread Safe**: Concurrent access with proper synchronization
//! - **Extensible**: Plugin architecture for custom calculators
//! - **Observable**: Comprehensive metrics and debugging support
//!
//! ## Quick Start
//!
//! ```rust
//! use bingo_core::{BingoEngine, types::*};
//! use std::collections::HashMap;
//!
//! // Create engine
//! let mut engine = BingoEngine::new()?;
//!
//! // Define a rule
//! let rule = Rule {
//!     id: 1,
//!     name: "High Value Customer".to_string(),
//!     conditions: vec![
//!         Condition::Simple {
//!             field: "order_total".to_string(),
//!             operator: Operator::GreaterThan,
//!             value: FactValue::Float(1000.0),
//!         }
//!     ],
//!     actions: vec![
//!         Action {
//!             action_type: ActionType::SetField {
//!                 field: "customer_tier".to_string(),
//!                 value: FactValue::String("premium".to_string()),
//!             }
//!         }
//!     ],
//! };
//!
//! // Add rule to engine
//! engine.add_rule(rule)?;
//!
//! // Create and process facts
//! let mut fields = HashMap::new();
//! fields.insert("order_total".to_string(), FactValue::Float(1500.0));
//! fields.insert("customer_id".to_string(), FactValue::String("C123".to_string()));
//!
//! let fact = Fact::new(1, FactData { fields });
//! let results = engine.process_facts(vec![fact])?;
//!
//! println!("Rules fired: {}", results.len());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Architecture
//!
//! ### Core Components
//!
//! - **Engine**: Main orchestrator for rule processing (`BingoEngine`)
//! - **RETE Network**: Efficient pattern matching network (`rete_network`)
//! - **Fact Store**: High-performance fact storage with indexing (`fact_store`)
//! - **Calculator**: Extensible calculation framework (`bingo-calculator`)
//! - **Types**: Core data structures (`types`)
//!
//! ### Performance Features
//!
//! - **Arena Allocation**: Minimizes memory fragmentation
//! - **Object Pooling**: Reuses frequently allocated objects
//! - **Lazy Aggregation**: Deferred computation for efficiency
//! - **Memory Tracking**: Comprehensive memory usage monitoring
//!
//! ## Module Organization
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`engine`] | Main rules engine and orchestration |
//! | [`types`] | Core data structures and type definitions |
//! | [`error`] | Comprehensive error handling system |
//! | [`rete_network`] | RETE algorithm implementation |
//! | [`fact_store`] | High-performance fact storage |
//! | [`memory_pools`] | Object pooling for optimization |
//! | [`serialization`] | Fast serialization/deserialization |
//! | [`aggregation`] | Aggregation functions and windowing |
//! | [`stream_processing`] | Real-time stream processing |
//!
//! ## Performance Characteristics
//!
//! - **Rule Compilation**: O(N) where N is number of conditions
//! - **Fact Processing**: O(1) pattern matching per fact
//! - **Memory Usage**: Linear with fact count, shared rule structures
//! - **Throughput**: 10,000+ facts/second on modern hardware
//!
//! ## Use Cases
//!
//! - **Business Rules**: Complex business logic processing
//! - **Real-time Analytics**: Stream processing and alerting
//! - **Event Processing**: Complex event pattern detection
//! - **Decision Engines**: Automated decision making systems
//! - **Workflow Automation**: Process automation and orchestration
//!
//! ## Error Handling
//!
//! All public APIs return `BingoResult<T>` for comprehensive error handling:
//!
//! ```rust
//! use bingo_core::{BingoEngine, BingoResult, BingoError};
//!
//! fn process_rules() -> BingoResult<()> {
//!     let mut engine = BingoEngine::new()?;
//!     // ... engine operations
//!     Ok(())
//! }
//! ```

use tracing::{debug, instrument};

/// Aggregation functions and time-window processing
pub mod aggregation;
/// Alpha memory implementation for RETE network
pub mod alpha_memory;
/// Beta network implementation for RETE network
pub mod beta_network;
/// Caching infrastructure for performance optimisation
pub mod cache;
/// Conflict resolution strategies for rule execution ordering
pub mod conflict_resolution;

/// Debug visualisation and tracing utilities
pub mod debugging;
/// Core rules engine and RETE network management
pub mod engine;
/// Enhanced monitoring system for comprehensive observability
pub mod enhanced_monitoring;
/// Comprehensive error handling for core engine operations
pub mod error;
/// Enhanced error diagnostics and debugging tools
pub mod error_diagnostics;
/// Error testing and validation framework
pub mod error_testing;
/// Fact storage and retrieval with indexing support
pub mod fact_store;
/// Fast lookup optimisations for rule pattern matching
pub mod fast_lookup;
/// Field-based indexing for efficient fact queries
pub mod field_indexing;
/// Lazy evaluation for complex aggregations
pub mod lazy_aggregation;
/// Memory management for RETE network nodes
pub mod memory;
/// Memory pooling for frequently allocated objects
pub mod memory_pools;
/// Parallel processing for improved throughput
pub mod parallel;
/// Advanced parallel RETE processing for multi-core systems
pub mod parallel_rete;
/// Performance testing configuration and environment detection
pub mod performance_config;
/// Processing pipeline for staged rule execution
pub mod pipeline;
/// Production readiness validation and configuration
pub mod production_readiness;
/// Advanced performance profiling and monitoring
pub mod profiler;
/// RETE network construction and execution
pub mod rete_network;
/// Individual RETE node implementations
pub mod rete_nodes;
/// Rule dependency analysis and optimization
pub mod rule_dependency;
/// Advanced rule optimization for RETE network performance
pub mod rule_optimizer;
/// Rule visualisation and debugging support
pub mod rule_visualization;
/// High-performance serialization and deserialization
pub mod serialization;
/// Stream processing for real-time rule evaluation
pub mod stream_processing;
/// Performance testing utilities
pub mod test_utils;

/// Test module for verifying Send + Sync bounds on core components
pub mod send_sync_test;
/// Integration tests for threading safety in parallel RETE
pub mod threading_integration_test;
/// Core types and functionality for the Bingo RETE rules engine
pub mod types;
/// Unified statistics collection across engine components
pub mod unified_statistics;

// Re-export critical types for API layer
pub use engine::BingoEngine;
pub use error::{BingoError, BingoResult, ErrorContext, ErrorSeverity, ResultExt};
pub use error_diagnostics::{
    DiagnosticsConfig, ErrorDiagnostic, ErrorDiagnosticsManager, ErrorSuggestion,
    ErrorToDiagnostic, InteractiveDebugSession, ResultDiagnosticExt,
};
pub use error_testing::{ErrorTestConfig, ErrorTestSuite, ErrorTestSummary, run_error_tests};
pub use rete_nodes::{ActionResult, RuleExecutionResult};
pub use serialization::{
    SerializationContext, SerializationStats, deserialize_fact, deserialize_facts,
    get_serialization_stats, serialize_fact, serialize_facts,
};
pub use types::{
    Action, ActionType, Condition, Fact, FactData, FactValue, LogicalOperator, Operator, Rule,
};

// Additional re-exports required by benchmarks and external crates
pub use conflict_resolution::{
    ConflictResolutionConfig, ConflictResolutionManager, ConflictResolutionStats,
    ConflictResolutionStrategy, RuleExecution,
};
pub use enhanced_monitoring::{
    BusinessMetrics, EnhancedMonitoring, MonitoringConfig, MonitoringReport, MonitoringSummary,
    PerformanceMetrics, ResourceMetrics,
};
pub use fact_store::arena_store::ArenaFactStore;
pub use memory::MemoryTracker;
pub use parallel::{ParallelAggregationEngine, ParallelAggregator, ParallelConfig};
pub use parallel_rete::{
    ParallelReteConfig, ParallelReteProcessor, ParallelReteStats, WorkItem, WorkQueue,
};
pub use production_readiness::{
    CheckResult, CheckSeverity, CheckStatus, PerformanceConfig, ProductionConfig,
    ProductionReadinessValidator, ReadinessReport, ReadinessSummary, ResourceConfig,
    SecurityConfig, ServiceConfig, check_production_readiness, load_config_from_env,
};
pub use profiler::{EngineProfiler, PerformanceReport, PerformanceThresholds};
pub use rule_dependency::{
    CircularDependency, CircularDependencySeverity, DependencyAnalysisConfig,
    DependencyAnalysisStats, DependencyType, ExecutionCluster, RuleDependency,
    RuleDependencyAnalyzer,
};
pub use rule_optimizer::{
    OptimizationAnalysis, OptimizationMetrics, OptimizationResult, OptimizationStrategy,
    OptimizerConfig, RuleOptimizer, optimize_rule_batch,
};

/// Initialize the core engine components
#[instrument]
pub fn init() -> BingoResult<()> {
    debug!("Initializing Bingo core engine");
    Ok(())
}
