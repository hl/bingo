//! Bingo Prelude
//!
//! This crate re-exports the most frequently used public items from the Bingo
//! ecosystem (currently `bingo-core` and `bingo-calculator`).  Down-stream
//! applications can depend on `bingo-prelude` to avoid long import lists and
//! to stay insulated from internal module reshuffles.

#![deny(warnings)]
#![deny(missing_docs)]

// Re-export core engine & type system ---------------------------------------------------------

pub use bingo_core::{
    BingoEngine,
    // Fundamental data types
    Fact, FactData, FactValue,
    // Rule structure
    Rule, Condition, Operator, LogicalOperator, Action, ActionType,
    // Runtime results
    rete_nodes::RuleExecutionResult,
};

// Calculator trait & a few helpers ------------------------------------------------------------

pub use bingo_calculator::{
    Calculator, CalculatorInputs,
};

// Commonly used built-in calculators ----------------------------------------------------------

pub use bingo_calculator::built_in::weighted_average::WeightedAverageCalculator;

// When new crates expose stable public APIs, add re-exports here in a backwards-compatible
// manner.
