use bingo_core::*;
use std::collections::HashMap;

#[test]
fn test_calculator_basic_expressions() {
    let mut calculator = CalculatorEngine::new();

    // Create test fact
    let mut fields = HashMap::new();
    fields.insert("amount".to_string(), FactValue::Float(100.0));
    fields.insert("rate".to_string(), FactValue::Float(0.15));
    fields.insert(
        "status".to_string(),
        FactValue::String("active".to_string()),
    );

    let fact = Fact { id: 1, data: FactData { fields } };

    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Test simple arithmetic
    let result = calculator.eval("100 + 50", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Float(value)) => {
            assert!((value - 150.0).abs() < f64::EPSILON);
        }
        CalculatorResult::Value(FactValue::Integer(value)) => {
            assert_eq!(value, 150);
        }
        _ => panic!("Expected numeric result"),
    }

    // Test variable access
    let result = calculator.eval("amount * rate", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Float(value)) => {
            assert!((value - 15.0).abs() < f64::EPSILON);
        }
        _ => panic!("Expected float result"),
    }

    // Test string operations
    let result = calculator.eval("status", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::String(value)) => {
            assert_eq!(value, "active");
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_calculator_functions() {
    let mut calculator = CalculatorEngine::new();

    // Create test fact
    let mut fields = HashMap::new();
    fields.insert("value1".to_string(), FactValue::Float(10.5));
    fields.insert("value2".to_string(), FactValue::Float(20.2));

    let fact = Fact { id: 1, data: FactData { fields } };

    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Test max function
    let result = calculator.eval("max(value1, value2)", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Float(value)) => {
            assert!((value - 20.2).abs() < f64::EPSILON);
        }
        _ => panic!("Expected float result"),
    }

    // Test abs function
    let result = calculator.eval("abs(-42)", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Integer(value)) => {
            assert_eq!(value, 42);
        }
        _ => panic!("Expected integer result"),
    }
}

#[test]
fn test_calculator_conditional_expressions() {
    let mut calculator = CalculatorEngine::new();

    // Create test fact
    let mut fields = HashMap::new();
    fields.insert("balance".to_string(), FactValue::Float(1000.0));
    fields.insert("min_balance".to_string(), FactValue::Float(500.0));

    let fact = Fact { id: 1, data: FactData { fields } };

    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Test conditional expression
    let result = calculator
        .eval(
            "if balance > min_balance then balance * 0.02 else 0",
            &context,
        )
        .unwrap();
    match result {
        CalculatorResult::Value(FactValue::Float(value)) => {
            assert!((value - 20.0).abs() < f64::EPSILON);
        }
        _ => panic!("Expected float result"),
    }
}

#[test]
fn test_calculator_compilation_caching() {
    let mut calculator = CalculatorEngine::new();

    // Create test fact
    let mut fields = HashMap::new();
    fields.insert("x".to_string(), FactValue::Integer(10));

    let fact = Fact { id: 1, data: FactData { fields } };

    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Compile expression manually
    let expr = calculator.compile("x * 2 + 1").unwrap();
    assert_eq!(expr.source, "x * 2 + 1");
    assert_eq!(expr.variables, vec!["x"]);

    // Evaluate compiled expression
    let result = calculator.evaluate(&expr, &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Integer(value)) => {
            assert_eq!(value, 21);
        }
        _ => panic!("Expected integer result"),
    }

    // Test that eval uses caching
    let result2 = calculator.eval("x * 2 + 1", &context).unwrap();
    match result2 {
        CalculatorResult::Value(FactValue::Integer(value)) => {
            assert_eq!(value, 21);
        }
        _ => panic!("Expected integer result"),
    }
}
