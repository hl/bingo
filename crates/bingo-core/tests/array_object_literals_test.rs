//! Tests for array and object literal parsing and evaluation

use bingo_core::calculator::{Calculator, CalculatorResult, EvaluationContext};
use bingo_core::types::{Fact, FactData, FactValue};
use std::collections::HashMap;

fn create_test_fact() -> Fact {
    let mut fields = HashMap::new();
    fields.insert("value".to_string(), FactValue::Integer(42));
    fields.insert("name".to_string(), FactValue::String("test".to_string()));

    Fact { id: 1, data: FactData { fields } }
}

#[test]
fn test_array_literal_parsing_and_evaluation() {
    let mut calc = Calculator::new();
    let fact = create_test_fact();
    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Test simple array literal
    let result = calc.eval("[1, 2, 3]", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Array(arr)) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], FactValue::Integer(1));
            assert_eq!(arr[1], FactValue::Integer(2));
            assert_eq!(arr[2], FactValue::Integer(3));
        }
        _ => panic!("Expected array result"),
    }

    // Test empty array literal
    let result = calc.eval("[]", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Array(arr)) => {
            assert_eq!(arr.len(), 0);
        }
        _ => panic!("Expected empty array result"),
    }

    // Test array with expressions
    let result = calc.eval("[value, value + 1, value * 2]", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Array(arr)) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], FactValue::Integer(42));
            assert_eq!(arr[1], FactValue::Integer(43));
            assert_eq!(arr[2], FactValue::Integer(84));
        }
        _ => panic!("Expected array with expressions result"),
    }
}

#[test]
fn test_object_literal_parsing_and_evaluation() {
    let mut calc = Calculator::new();
    let fact = create_test_fact();
    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Test simple object literal
    let result = calc.eval(r#"{"count": 5, "active": true}"#, &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Object(obj)) => {
            assert_eq!(obj.len(), 2);
            assert_eq!(obj.get("count"), Some(&FactValue::Integer(5)));
            assert_eq!(obj.get("active"), Some(&FactValue::Boolean(true)));
        }
        _ => panic!("Expected object result"),
    }

    // Test empty object literal
    let result = calc.eval("{}", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Object(obj)) => {
            assert_eq!(obj.len(), 0);
        }
        _ => panic!("Expected empty object result"),
    }

    // Test object with expressions
    let result = calc
        .eval(
            r#"{"original": value, "doubled": value * 2, "name": name}"#,
            &context,
        )
        .unwrap();
    match result {
        CalculatorResult::Value(FactValue::Object(obj)) => {
            assert_eq!(obj.len(), 3);
            assert_eq!(obj.get("original"), Some(&FactValue::Integer(42)));
            assert_eq!(obj.get("doubled"), Some(&FactValue::Integer(84)));
            assert_eq!(
                obj.get("name"),
                Some(&FactValue::String("test".to_string()))
            );
        }
        _ => panic!("Expected object with expressions result"),
    }
}

#[test]
fn test_array_indexing_parsing_and_evaluation() {
    let mut calc = Calculator::new();
    let mut fields = HashMap::new();
    fields.insert(
        "arr".to_string(),
        FactValue::Array(vec![
            FactValue::String("first".to_string()),
            FactValue::String("second".to_string()),
            FactValue::String("third".to_string()),
        ]),
    );

    let fact = Fact { id: 1, data: FactData { fields } };
    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Test array indexing
    let result = calc.eval("arr[0]", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::String(s)) => {
            assert_eq!(s, "first");
        }
        _ => panic!("Expected string result from array indexing"),
    }

    let result = calc.eval("arr[1]", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::String(s)) => {
            assert_eq!(s, "second");
        }
        _ => panic!("Expected string result from array indexing"),
    }
}

#[test]
fn test_nested_structures() {
    let mut calc = Calculator::new();
    let fact = create_test_fact();
    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Test nested array in object
    let result = calc
        .eval(
            r#"{"numbers": [1, 2, 3], "info": {"count": 3, "type": "test"}}"#,
            &context,
        )
        .unwrap();
    match result {
        CalculatorResult::Value(FactValue::Object(obj)) => {
            assert_eq!(obj.len(), 2);

            // Check numbers array
            if let Some(FactValue::Array(numbers)) = obj.get("numbers") {
                assert_eq!(numbers.len(), 3);
                assert_eq!(numbers[0], FactValue::Integer(1));
                assert_eq!(numbers[1], FactValue::Integer(2));
                assert_eq!(numbers[2], FactValue::Integer(3));
            } else {
                panic!("Expected numbers array in object");
            }

            // Check nested info object
            if let Some(FactValue::Object(info)) = obj.get("info") {
                assert_eq!(info.len(), 2);
                assert_eq!(info.get("count"), Some(&FactValue::Integer(3)));
                assert_eq!(
                    info.get("type"),
                    Some(&FactValue::String("test".to_string()))
                );
            } else {
                panic!("Expected info object in object");
            }
        }
        _ => panic!("Expected nested object result"),
    }
}

#[test]
fn test_array_and_object_functions_with_literals() {
    let mut calc = Calculator::new();
    let fact = create_test_fact();
    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Test array functions with literals
    let result = calc.eval("array_len([1, 2, 3, 4])", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Integer(len)) => {
            assert_eq!(len, 4);
        }
        _ => panic!("Expected integer result for array_len"),
    }

    let result = calc.eval("array_contains([1, 2, 3], 2)", &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Boolean(contains)) => {
            assert!(contains);
        }
        _ => panic!("Expected boolean result for array_contains"),
    }

    // Test object functions with literals
    let result = calc.eval(r#"object_has_key({"a": 1, "b": 2}, "a")"#, &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Boolean(has_key)) => {
            assert!(has_key);
        }
        _ => panic!("Expected boolean result for object_has_key"),
    }

    let result = calc.eval(r#"object_get({"x": 10, "y": 20}, "x")"#, &context).unwrap();
    match result {
        CalculatorResult::Value(FactValue::Integer(val)) => {
            assert_eq!(val, 10);
        }
        _ => panic!("Expected integer result for object_get"),
    }
}
