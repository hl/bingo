#![deny(warnings)]
//! The calculator ecosystem for the Bingo Rules Engine.
//!
//! This crate provides the `Calculator` trait and the `CalculatorInputs` struct
//! for creating custom, high-performance business logic that can be invoked
//! from rule actions.

pub mod built_in;
pub mod calculator;
pub mod limit_validator;
pub mod plugin;
pub mod plugin_manager;
pub mod threshold_check;

// Re-exports for convenience
pub use bingo_types::FactValue;
pub use calculator::Calculator;
pub use limit_validator::LimitValidateCalculator;
pub use plugin::CalculatorInputs;
pub use threshold_check::ThresholdCheckCalculator;
