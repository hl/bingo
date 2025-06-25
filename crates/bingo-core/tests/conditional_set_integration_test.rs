//! Integration test for conditional set logic in calculator DSL
//!
//! This test demonstrates the conditional set functionality working with
//! the calculator DSL, parser, and evaluator in an end-to-end scenario.

use bingo_core::calculator::EvaluationContext;
use bingo_core::calculator::evaluator::evaluate_expression;
use bingo_core::calculator::functions::FunctionRegistry;
use bingo_core::calculator::parser::parse_expression;
use bingo_core::types::{Fact, FactData, FactValue};
use std::collections::HashMap;

/// Test end-to-end conditional set logic for performance-based bonus calculation
#[test]
fn test_performance_bonus_conditional_set() {
    let functions = FunctionRegistry::with_builtins();

    // Test cases with different performance ratings
    let test_cases = vec![
        (4.7, 0.15), // Should get 15% bonus (>= 4.5)
        (4.2, 0.10), // Should get 10% bonus (>= 4.0 but < 4.5)
        (3.8, 0.05), // Should get 5% bonus (>= 3.5 but < 4.0)
        (3.0, 0.0),  // Should get no bonus (< 3.5, default)
    ];

    for (rating, expected_bonus) in test_cases {
        // Create fact with performance rating
        let mut fields = HashMap::new();
        fields.insert("performance_rating".to_string(), FactValue::Float(rating));
        fields.insert("base_salary".to_string(), FactValue::Float(100000.0));

        let fact = Fact { id: 1, data: FactData { fields } };

        let context =
            EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

        // Parse conditional set expression
        let expr_str = "cond when performance_rating >= 4.5 then 0.15 when performance_rating >= 4.0 then 0.10 when performance_rating >= 3.5 then 0.05 default 0.0";
        let expr = parse_expression(expr_str).expect("Failed to parse conditional set expression");

        // Evaluate the expression
        let result = evaluate_expression(&expr, &context, &functions)
            .expect("Failed to evaluate conditional set expression");

        if let bingo_core::calculator::CalculatorResult::Value(FactValue::Float(bonus_rate)) =
            result
        {
            assert!(
                (bonus_rate - expected_bonus).abs() < f64::EPSILON,
                "Rating {}: expected {}, got {}",
                rating,
                expected_bonus,
                bonus_rate
            );
        } else {
            panic!("Expected float result for rating {}", rating);
        }
    }
}

/// Test conditional set with different data types and complex conditions
#[test]
fn test_employee_tier_classification() {
    let functions = FunctionRegistry::with_builtins();

    // Test cases for employee tier classification
    let test_cases = vec![
        (25, 8, "senior"), // High tenure, high rating -> senior
        (15, 6, "mid"),    // Medium tenure, medium rating -> mid
        (5, 4, "junior"),  // Low tenure, low rating -> junior
        (2, 3, "trainee"), // Very low tenure and rating -> trainee
    ];

    for (tenure_years, performance_score, expected_tier) in test_cases {
        let mut fields = HashMap::new();
        fields.insert("tenure_years".to_string(), FactValue::Integer(tenure_years));
        fields.insert(
            "performance_score".to_string(),
            FactValue::Integer(performance_score),
        );

        let fact = Fact { id: 1, data: FactData { fields } };

        let context =
            EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

        // Complex conditional set with multiple criteria
        let expr_str = r#"cond 
            when tenure_years >= 20 && performance_score >= 7 then "senior"
            when tenure_years >= 10 && performance_score >= 5 then "mid" 
            when tenure_years >= 3 && performance_score >= 4 then "junior"
            default "trainee""#;

        let expr =
            parse_expression(expr_str).expect("Failed to parse tier classification expression");

        let result = evaluate_expression(&expr, &context, &functions)
            .expect("Failed to evaluate tier classification");

        if let bingo_core::calculator::CalculatorResult::Value(FactValue::String(tier)) = result {
            assert_eq!(
                tier, expected_tier,
                "Tenure {}, Score {}: expected {}, got {}",
                tenure_years, performance_score, expected_tier, tier
            );
        } else {
            panic!(
                "Expected string result for tenure {} score {}",
                tenure_years, performance_score
            );
        }
    }
}

/// Test conditional set without default value (should error when no conditions match)
#[test]
fn test_conditional_set_no_default_error() {
    let functions = FunctionRegistry::with_builtins();

    let mut fields = HashMap::new();
    fields.insert("score".to_string(), FactValue::Integer(50));

    let fact = Fact { id: 1, data: FactData { fields } };

    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Conditional set with no matching conditions and no default
    let expr_str = "cond when score > 100 then \"high\" when score > 90 then \"medium\"";
    let expr = parse_expression(expr_str).expect("Failed to parse conditional set");

    let result = evaluate_expression(&expr, &context, &functions);
    assert!(
        result.is_err(),
        "Expected error when no conditions match and no default provided"
    );
}

/// Test nested conditional sets and complex expressions
#[test]
fn test_nested_conditional_logic() {
    let functions = FunctionRegistry::with_builtins();

    let mut fields = HashMap::new();
    fields.insert(
        "department".to_string(),
        FactValue::String("engineering".to_string()),
    );
    fields.insert("level".to_string(), FactValue::Integer(5));
    fields.insert("location".to_string(), FactValue::String("sf".to_string()));

    let fact = Fact { id: 1, data: FactData { fields } };

    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Conditional set with nested logic for salary calculation
    let expr_str = r#"cond 
        when department == "engineering" && level >= 5 then 
            cond when location == "sf" then 200000 when location == "ny" then 180000 default 150000
        when department == "sales" && level >= 3 then 120000
        default 80000"#;

    let expr = parse_expression(expr_str).expect("Failed to parse nested conditional set");

    let result = evaluate_expression(&expr, &context, &functions)
        .expect("Failed to evaluate nested conditional set");

    if let bingo_core::calculator::CalculatorResult::Value(FactValue::Integer(salary)) = result {
        assert_eq!(
            salary, 200000,
            "Expected SF engineering L5 salary to be 200000, got {}",
            salary
        );
    } else {
        panic!("Expected integer result for nested conditional set");
    }
}
