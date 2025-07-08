#![deny(warnings)]
#![allow(missing_docs)]
//! Core functionality for the Bingo RETE rules engine.
//!
//! This crate provides the foundational components for building high-performance
//! business rules engines using the RETE algorithm with optimizations for
//! fact-based pattern matching and rule execution.

use tracing::{debug, instrument};

/// Aggregation functions and time-window processing
pub mod aggregation;
/// Caching infrastructure for performance optimisation
pub mod cache;
/// Calculator integration for rule actions
pub mod calculator_integration;

/// Debug visualisation and tracing utilities
pub mod debugging;
/// Core rules engine and RETE network management
pub mod engine;
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
/// Performance testing configuration and environment detection
pub mod performance_config;
/// Processing pipeline for staged rule execution
pub mod pipeline;
/// RETE network construction and execution
pub mod rete_network;
/// Individual RETE node implementations
pub mod rete_nodes;
/// Rule visualisation and debugging support
pub mod rule_visualization;
/// High-performance serialization and deserialization
pub mod serialization;
/// Stream processing for real-time rule evaluation
pub mod stream_processing;
/// Performance testing utilities
pub mod test_utils;

/// Core types and functionality for the Bingo RETE rules engine
pub mod types;
/// Unified statistics collection across engine components
pub mod unified_statistics;

// Re-export critical types for API layer
pub use calculator_integration::CalculatorRegistry;
pub use engine::BingoEngine;
pub use rete_nodes::{ActionResult, RuleExecutionResult};
pub use serialization::{
    SerializationContext, SerializationStats, deserialize_fact, deserialize_facts,
    get_serialization_stats, serialize_fact, serialize_facts,
};
pub use types::{
    Action, ActionType, Condition, Fact, FactData, FactValue, LogicalOperator, Operator, Rule,
};

// Additional re-exports required by benchmarks and external crates
pub use fact_store::arena_store::ArenaFactStore;
pub use memory::MemoryTracker;

/// Initialize the core engine components
#[instrument]
pub fn init() -> anyhow::Result<()> {
    debug!("Initializing Bingo core engine");
    Ok(())
}
