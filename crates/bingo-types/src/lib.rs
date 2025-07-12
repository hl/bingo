//! Bingo Types
//!
//! This crate defines the core types and data structures used throughout the Bingo
//! ecosystem (currently `bingo-core` and `bingo-calculator`). It provides shared
//! types like `FactValue` and eliminates circular dependencies between crates.

#![deny(warnings)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::cargo)]
#![deny(missing_docs)]

// Re-export types
mod types;
pub use types::FactValue;

// Re-export core engine & type system ---------------------------------------------------------
// NOTE: These will be re-exported from bingo-core once it imports bingo-types

// pub use bingo_core::{
//     Action,
//     ActionType,
//     BingoEngine,
//     Condition,
//     // Fundamental data types
//     Fact,
//     FactData,
//     LogicalOperator,
//     Operator,
//     // Rule structure
//     Rule,
//     // Runtime results
//     rete_nodes::RuleExecutionResult,
// };

// Calculator trait & a few helpers ------------------------------------------------------------
// NOTE: Calculator re-exports will be added back when circular dependencies are resolved

// When new crates expose stable public APIs, add re-exports here in a backwards-compatible
// manner.
