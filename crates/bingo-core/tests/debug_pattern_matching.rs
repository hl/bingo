//! Debug test for pattern matching issue

use bingo_core::alpha_memory::FactPattern;
use bingo_core::types::{Condition, Fact, FactData, FactValue, Operator};
use std::collections::HashMap;

#[test]
fn test_pattern_matching_debug() {
    // Create the problematic condition: score > 90
    let condition = Condition::Simple {
        field: "score".to_string(),
        operator: Operator::GreaterThan,
        value: FactValue::Integer(90),
    };

    // Create the problematic fact: {score: 85, status: "active"}
    let mut fields = HashMap::new();
    fields.insert("score".to_string(), FactValue::Integer(85));
    fields.insert(
        "status".to_string(),
        FactValue::String("active".to_string()),
    );
    let fact = Fact::new(1, FactData { fields });

    // Test the pattern matching
    if let Some(pattern) = FactPattern::from_condition(&condition) {
        let matches = pattern.matches_fact(&fact);
        println!("Condition: score > 90");
        println!("Fact score: 85");
        println!("Pattern matches fact: {matches}");
        println!("Expected: false");

        // This should be false since 85 is NOT > 90
        assert!(!matches, "Pattern should not match because 85 is not > 90");
        println!("✅ Pattern correctly does not match");
    } else {
        panic!("❌ Failed to create pattern from condition");
    }
}

#[test]
fn test_pattern_matching_should_match() {
    // Create the same condition: score > 90
    let condition = Condition::Simple {
        field: "score".to_string(),
        operator: Operator::GreaterThan,
        value: FactValue::Integer(90),
    };

    // Create a fact that SHOULD match: {score: 95, status: "active"}
    let mut fields = HashMap::new();
    fields.insert("score".to_string(), FactValue::Integer(95));
    fields.insert(
        "status".to_string(),
        FactValue::String("active".to_string()),
    );
    let fact = Fact::new(1, FactData { fields });

    // Test the pattern matching
    if let Some(pattern) = FactPattern::from_condition(&condition) {
        let matches = pattern.matches_fact(&fact);
        println!("Condition: score > 90");
        println!("Fact score: 95");
        println!("Pattern matches fact: {matches}");
        println!("Expected: true");

        // This should be true since 95 > 90
        assert!(matches, "Pattern should match because 95 > 90");
        println!("✅ Pattern correctly matches");
    } else {
        panic!("❌ Failed to create pattern from condition");
    }
}
