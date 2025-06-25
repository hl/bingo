//! Tests for multi-fact calculator context integration

use bingo_core::calculator::{Calculator, CalculatorResult, EvaluationContext};
use bingo_core::types::{Fact, FactData, FactValue};
use std::collections::HashMap;

fn create_multi_fact_context() -> (Fact, Vec<Fact>) {
    // Create primary fact
    let mut primary_fields = HashMap::new();
    primary_fields.insert("type".to_string(), FactValue::String("order".to_string()));
    primary_fields.insert("amount".to_string(), FactValue::Float(100.0));
    primary_fields.insert("customer_id".to_string(), FactValue::Integer(1));
    let primary_fact = Fact { id: 1, data: FactData { fields: primary_fields } };

    // Create additional facts for context
    let mut fact2_fields = HashMap::new();
    fact2_fields.insert(
        "type".to_string(),
        FactValue::String("customer".to_string()),
    );
    fact2_fields.insert("id".to_string(), FactValue::Integer(1));
    fact2_fields.insert("discount_rate".to_string(), FactValue::Float(0.1));
    fact2_fields.insert(
        "status".to_string(),
        FactValue::String("premium".to_string()),
    );
    let fact2 = Fact { id: 2, data: FactData { fields: fact2_fields } };

    let mut fact3_fields = HashMap::new();
    fact3_fields.insert("type".to_string(), FactValue::String("order".to_string()));
    fact3_fields.insert("amount".to_string(), FactValue::Float(50.0));
    fact3_fields.insert("customer_id".to_string(), FactValue::Integer(1));
    let fact3 = Fact { id: 3, data: FactData { fields: fact3_fields } };

    let mut fact4_fields = HashMap::new();
    fact4_fields.insert("type".to_string(), FactValue::String("order".to_string()));
    fact4_fields.insert("amount".to_string(), FactValue::Float(75.0));
    fact4_fields.insert("customer_id".to_string(), FactValue::Integer(2));
    let fact4 = Fact { id: 4, data: FactData { fields: fact4_fields } };

    (primary_fact, vec![fact2, fact3, fact4])
}

#[test]
fn test_fact_count_function() {
    let mut calc = Calculator::new();
    let (primary_fact, context_facts) = create_multi_fact_context();

    let context = EvaluationContext {
        current_fact: &primary_fact,
        facts: &context_facts,
        globals: HashMap::new(),
    };

    // Test fact count
    let result = calc.eval("fact_count()", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Integer(count)) => {
            assert_eq!(count, 3); // Should count 3 facts in context
        }
        _ => panic!("Expected integer result for fact_count"),
    }
}

#[test]
fn test_fact_sum_function() {
    let mut calc = Calculator::new();
    let (primary_fact, context_facts) = create_multi_fact_context();

    let context = EvaluationContext {
        current_fact: &primary_fact,
        facts: &context_facts,
        globals: HashMap::new(),
    };

    // Test summing amounts across all order facts
    let result = calc.eval("fact_sum(\"amount\")", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Float(sum)) => {
            // Should sum: 50.0 + 75.0 = 125.0 (customer fact doesn't have amount)
            assert!((sum - 125.0).abs() < f64::EPSILON);
        }
        _ => panic!("Expected float result for fact_sum"),
    }
}

#[test]
fn test_fact_avg_function() {
    let mut calc = Calculator::new();
    let (primary_fact, context_facts) = create_multi_fact_context();

    let context = EvaluationContext {
        current_fact: &primary_fact,
        facts: &context_facts,
        globals: HashMap::new(),
    };

    // Test averaging amounts
    let result = calc.eval("fact_avg(\"amount\")", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Float(avg)) => {
            // Should average: (50.0 + 75.0) / 2 = 62.5
            assert!((avg - 62.5).abs() < f64::EPSILON);
        }
        _ => panic!("Expected float result for fact_avg"),
    }
}

#[test]
fn test_fact_min_max_functions() {
    let mut calc = Calculator::new();
    let (primary_fact, context_facts) = create_multi_fact_context();

    let context = EvaluationContext {
        current_fact: &primary_fact,
        facts: &context_facts,
        globals: HashMap::new(),
    };

    // Test minimum amount
    let result = calc.eval("fact_min(\"amount\")", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Float(min)) => {
            assert!((min - 50.0).abs() < f64::EPSILON);
        }
        _ => panic!("Expected float result for fact_min"),
    }

    // Test maximum amount
    let result = calc.eval("fact_max(\"amount\")", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Float(max)) => {
            assert!((max - 75.0).abs() < f64::EPSILON);
        }
        _ => panic!("Expected float result for fact_max"),
    }
}

#[test]
fn test_fact_field_function() {
    let mut calc = Calculator::new();
    let (primary_fact, context_facts) = create_multi_fact_context();

    let context = EvaluationContext {
        current_fact: &primary_fact,
        facts: &context_facts,
        globals: HashMap::new(),
    };

    // Test getting specific field from specific fact
    let result = calc.eval("fact_field(2, \"discount_rate\")", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Float(rate)) => {
            assert!((rate - 0.1).abs() < f64::EPSILON);
        }
        _ => panic!("Expected float result for fact_field"),
    }

    // Test getting string field
    let result = calc.eval("fact_field(2, \"status\")", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::String(status)) => {
            assert_eq!(status, "premium");
        }
        _ => panic!("Expected string result for fact_field"),
    }
}

#[test]
fn test_fact_exists_function() {
    let mut calc = Calculator::new();
    let (primary_fact, context_facts) = create_multi_fact_context();

    let context = EvaluationContext {
        current_fact: &primary_fact,
        facts: &context_facts,
        globals: HashMap::new(),
    };

    // Test existing fact
    let result = calc.eval("fact_exists(2)", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Boolean(exists)) => {
            assert!(exists);
        }
        _ => panic!("Expected boolean result for fact_exists"),
    }

    // Test non-existing fact
    let result = calc.eval("fact_exists(999)", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Boolean(exists)) => {
            assert!(!exists);
        }
        _ => panic!("Expected boolean result for fact_exists"),
    }
}

#[test]
fn test_field_access_by_fact_id() {
    let mut calc = Calculator::new();
    let (primary_fact, context_facts) = create_multi_fact_context();

    let context = EvaluationContext {
        current_fact: &primary_fact,
        facts: &context_facts,
        globals: HashMap::new(),
    };

    // Test field access using existing syntax: "fact_id".field_name
    let result = calc.eval("\"2\".discount_rate", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Float(rate)) => {
            assert!((rate - 0.1).abs() < f64::EPSILON);
        }
        _ => panic!("Expected float result for field access"),
    }

    // Test field access with string field
    let result = calc.eval("\"2\".status", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::String(status)) => {
            assert_eq!(status, "premium");
        }
        _ => panic!("Expected string result for field access"),
    }
}

#[test]
fn test_complex_multi_fact_expression() {
    let mut calc = Calculator::new();
    let (primary_fact, context_facts) = create_multi_fact_context();

    let context = EvaluationContext {
        current_fact: &primary_fact,
        facts: &context_facts,
        globals: HashMap::new(),
    };

    // Test complex expression using multiple fact functions
    // Calculate discounted total: current amount * (1 - customer discount rate)
    let result = calc.eval("amount * (1 - fact_field(2, \"discount_rate\"))", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Float(discounted)) => {
            // 100.0 * (1 - 0.1) = 90.0
            assert!((discounted - 90.0).abs() < f64::EPSILON);
        }
        _ => panic!("Expected float result for complex expression"),
    }

    // Test conditional logic with fact existence
    let result = calc
        .eval(
            "if fact_exists(2) then fact_field(2, \"discount_rate\") else 0.0",
            &context,
        )
        .unwrap();
    match result {
        CalculatorResult::Value(FactValue::Float(rate)) => {
            assert!((rate - 0.1).abs() < f64::EPSILON);
        }
        _ => panic!("Expected float result for conditional expression"),
    }

    // Test aggregation with comparison
    let result = calc.eval("amount > fact_avg(\"amount\")", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Boolean(is_above_avg)) => {
            // 100.0 > 62.5 should be true
            assert!(is_above_avg);
        }
        _ => panic!("Expected boolean result for comparison"),
    }
}

#[test]
fn test_error_handling() {
    let mut calc = Calculator::new();
    let (primary_fact, context_facts) = create_multi_fact_context();

    let context = EvaluationContext {
        current_fact: &primary_fact,
        facts: &context_facts,
        globals: HashMap::new(),
    };

    // Test error for non-existent fact field access
    let result = calc.eval("fact_field(999, \"amount\")", &context);
    assert!(result.is_err());

    // Test error for non-existent field on existing fact
    let result = calc.eval("fact_field(2, \"non_existent_field\")", &context);
    assert!(result.is_err());

    // Test error for min/max on non-numeric field
    let result = calc.eval("fact_min(\"status\")", &context);
    assert!(result.is_err());
}
