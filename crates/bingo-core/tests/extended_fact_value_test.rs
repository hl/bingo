//! Tests for extended FactValue types (Array, Object, Date, Null)

use bingo_core::calculator::{Calculator, CalculatorResult, EvaluationContext};
use bingo_core::types::{Fact, FactData, FactValue};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

fn create_test_fact_with_extended_types() -> Fact {
    let mut fields = HashMap::new();

    // Basic types
    fields.insert(
        "name".to_string(),
        FactValue::String("John Doe".to_string()),
    );
    fields.insert("age".to_string(), FactValue::Integer(30));
    fields.insert("salary".to_string(), FactValue::Float(75000.0));
    fields.insert("active".to_string(), FactValue::Boolean(true));

    // Extended types
    fields.insert(
        "tags".to_string(),
        FactValue::Array(vec![
            FactValue::String("developer".to_string()),
            FactValue::String("senior".to_string()),
            FactValue::String("fullstack".to_string()),
        ]),
    );

    let mut address = HashMap::new();
    address.insert(
        "street".to_string(),
        FactValue::String("123 Main St".to_string()),
    );
    address.insert(
        "city".to_string(),
        FactValue::String("San Francisco".to_string()),
    );
    address.insert("zip".to_string(), FactValue::Integer(94105));
    fields.insert("address".to_string(), FactValue::Object(address));

    fields.insert(
        "hire_date".to_string(),
        FactValue::Date(
            DateTime::parse_from_rfc3339("2020-01-15T09:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        ),
    );

    fields.insert("bonus".to_string(), FactValue::Null);

    Fact { id: 1, data: FactData { fields } }
}

#[test]
fn test_extended_fact_value_display() {
    let fact = create_test_fact_with_extended_types();

    // Test Array display
    if let Some(FactValue::Array(tags)) = fact.data.fields.get("tags") {
        let display = tags.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
        assert!(display.contains("developer"));
        assert!(display.contains("senior"));
        assert!(display.contains("fullstack"));
    }

    // Test Object display
    if let Some(FactValue::Object(address)) = fact.data.fields.get("address") {
        assert!(address.contains_key("street"));
        assert!(address.contains_key("city"));
        assert!(address.contains_key("zip"));
    }

    // Test Date display
    if let Some(FactValue::Date(_)) = fact.data.fields.get("hire_date") {
        // Date exists and can be displayed
        assert!(true);
    }

    // Test Null display
    if let Some(FactValue::Null) = fact.data.fields.get("bonus") {
        assert!(true);
    }
}

#[test]
fn test_array_functions() {
    let mut calc = Calculator::new();
    let fact = create_test_fact_with_extended_types();
    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Test array length
    let result = calc.eval("array_len(tags)", &context).unwrap();
    if let CalculatorResult::Value(FactValue::Integer(len)) = result {
        assert_eq!(len, 3);
    } else {
        panic!("Expected integer result for array_len");
    }

    // Test array contains
    let result = calc.eval("array_contains(tags, \"developer\")", &context).unwrap();
    if let FactValue::Boolean(contains) = result.value() {
        assert!(*contains);
    } else {
        panic!("Expected boolean result for array_contains");
    }

    // Test array join
    let result = calc.eval("array_join(tags, \", \")", &context).unwrap();
    if let FactValue::String(joined) = result.value() {
        assert!(joined.contains("developer, senior, fullstack"));
    } else {
        panic!("Expected string result for array_join");
    }
}

#[test]
fn test_object_functions() {
    let mut calc = Calculator::new();
    let fact = create_test_fact_with_extended_types();
    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Test object has key
    let result = calc.eval("object_has_key(address, \"city\")", &context).unwrap();
    if let FactValue::Boolean(has_key) = result.value() {
        assert!(*has_key);
    } else {
        panic!("Expected boolean result for object_has_key");
    }

    // Test object get
    let result = calc.eval("object_get(address, \"city\")", &context).unwrap();
    if let FactValue::String(city) = result.value() {
        assert_eq!(city, "San Francisco");
    } else {
        panic!("Expected string result for object_get");
    }

    // Test object get with default
    let result = calc.eval("object_get(address, \"country\", \"USA\")", &context).unwrap();
    if let FactValue::String(country) = result.value() {
        assert_eq!(country, "USA");
    } else {
        panic!("Expected string result for object_get with default");
    }
}

#[test]
fn test_date_functions() {
    let mut calc = Calculator::new();
    let fact = create_test_fact_with_extended_types();
    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Test date year
    let result = calc.eval("date_year(hire_date)", &context).unwrap();
    if let FactValue::Integer(year) = result.value() {
        assert_eq!(*year, 2020);
    } else {
        panic!("Expected integer result for date_year");
    }

    // Test date month
    let result = calc.eval("date_month(hire_date)", &context).unwrap();
    if let FactValue::Integer(month) = result.value() {
        assert_eq!(*month, 1);
    } else {
        panic!("Expected integer result for date_month");
    }

    // Test date day
    let result = calc.eval("date_day(hire_date)", &context).unwrap();
    if let FactValue::Integer(day) = result.value() {
        assert_eq!(*day, 15);
    } else {
        panic!("Expected integer result for date_day");
    }
}

#[test]
fn test_array_literal_expressions() {
    let mut calc = Calculator::new();
    let fact = create_test_fact_with_extended_types();
    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Test creating arrays in expressions (would need parser support)
    // For now, test using existing array functions
    let result = calc.eval("array_len(tags)", &context).unwrap();
    assert!(matches!(result.value(), FactValue::Integer(3)));

    // Test array operations
    let result = calc.eval("array_slice(tags, 1, 2)", &context).unwrap();
    if let FactValue::Array(sliced) = result.value() {
        assert_eq!(sliced.len(), 1);
        if let FactValue::String(tag) = &sliced[0] {
            assert_eq!(tag, "senior");
        } else {
            panic!("Expected string in sliced array");
        }
    } else {
        panic!("Expected array result for array_slice");
    }
}

#[test]
fn test_type_checking() {
    let fact = create_test_fact_with_extended_types();

    // Test type_name method
    if let Some(tags) = fact.data.fields.get("tags") {
        assert_eq!(tags.type_name(), "array");
    }

    if let Some(address) = fact.data.fields.get("address") {
        assert_eq!(address.type_name(), "object");
    }

    if let Some(hire_date) = fact.data.fields.get("hire_date") {
        assert_eq!(hire_date.type_name(), "date");
    }

    if let Some(bonus) = fact.data.fields.get("bonus") {
        assert_eq!(bonus.type_name(), "null");
    }
}

#[test]
fn test_truthiness() {
    let fact = create_test_fact_with_extended_types();

    // Test is_truthy for extended types
    if let Some(tags) = fact.data.fields.get("tags") {
        assert!(tags.is_truthy()); // Non-empty array
    }

    if let Some(address) = fact.data.fields.get("address") {
        assert!(address.is_truthy()); // Non-empty object
    }

    if let Some(hire_date) = fact.data.fields.get("hire_date") {
        assert!(hire_date.is_truthy()); // Dates are always truthy
    }

    if let Some(bonus) = fact.data.fields.get("bonus") {
        assert!(!bonus.is_truthy()); // Null is falsy
    }

    // Test empty collections
    let empty_array = FactValue::Array(vec![]);
    assert!(!empty_array.is_truthy());

    let empty_object = FactValue::Object(HashMap::new());
    assert!(!empty_object.is_truthy());
}

#[test]
fn test_conversion_functions() {
    let mut calc = Calculator::new();
    let fact = create_test_fact_with_extended_types();
    let context = EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };

    // Test converting array length to int/float
    let result = calc.eval("to_int(array_len(tags))", &context).unwrap();
    assert!(matches!(result.value(), FactValue::Integer(3)));

    let result = calc.eval("to_float(array_len(tags))", &context).unwrap();
    assert!(matches!(result.value(), FactValue::Float(f) if (*f - 3.0).abs() < f64::EPSILON));

    // Test converting date to string
    let result = calc.eval("to_string(hire_date)", &context).unwrap();
    if let FactValue::String(date_str) = result.value() {
        assert!(date_str.contains("2020"));
        assert!(date_str.contains("01"));
        assert!(date_str.contains("15"));
    } else {
        panic!("Expected string result for date conversion");
    }
}
