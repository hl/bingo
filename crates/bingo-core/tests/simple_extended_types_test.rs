//! Simple tests for extended FactValue types

use bingo_core::types::FactValue;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[test]
fn test_basic_extended_types() {
    // Test Array creation and operations
    let arr = FactValue::Array(vec![
        FactValue::Integer(1),
        FactValue::Integer(2),
        FactValue::Integer(3),
    ]);

    assert_eq!(arr.type_name(), "array");
    assert!(arr.is_truthy());

    // Test Object creation and operations
    let mut obj_map = HashMap::new();
    obj_map.insert("name".to_string(), FactValue::String("test".to_string()));
    obj_map.insert("value".to_string(), FactValue::Integer(42));

    let obj = FactValue::Object(obj_map);
    assert_eq!(obj.type_name(), "object");
    assert!(obj.is_truthy());

    // Test Date creation
    let date = FactValue::Date(
        DateTime::parse_from_rfc3339("2020-01-15T09:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
    );
    assert_eq!(date.type_name(), "date");
    assert!(date.is_truthy());

    // Test Null
    let null_val = FactValue::Null;
    assert_eq!(null_val.type_name(), "null");
    assert!(!null_val.is_truthy());
}

#[test]
fn test_extended_type_conversions() {
    // Test array length conversion
    let arr = FactValue::Array(vec![FactValue::Integer(1), FactValue::Integer(2)]);
    assert_eq!(arr.as_integer(), Some(2)); // length
    assert_eq!(arr.as_float(), Some(2.0)); // length as float

    // Test object length conversion
    let mut obj_map = HashMap::new();
    obj_map.insert("key1".to_string(), FactValue::Integer(1));
    obj_map.insert("key2".to_string(), FactValue::Integer(2));
    let obj = FactValue::Object(obj_map);
    assert_eq!(obj.as_integer(), Some(2)); // length

    // Test date conversion
    let date = FactValue::Date(DateTime::from_timestamp(1580000000, 0).unwrap());
    assert!(date.as_integer().is_some()); // timestamp

    // Test null conversion
    let null_val = FactValue::Null;
    assert_eq!(null_val.as_integer(), Some(0));
    assert_eq!(null_val.as_float(), Some(0.0));
    assert_eq!(null_val.as_string(), "null");
}

#[test]
fn test_extended_type_display() {
    // Test Array display
    let arr = FactValue::Array(vec![
        FactValue::Integer(1),
        FactValue::String("test".to_string()),
    ]);
    let display = arr.to_string();
    assert!(display.contains("["));
    assert!(display.contains("]"));
    assert!(display.contains("1"));
    assert!(display.contains("test"));

    // Test Object display
    let mut obj_map = HashMap::new();
    obj_map.insert("key".to_string(), FactValue::String("value".to_string()));
    let obj = FactValue::Object(obj_map);
    let display = obj.to_string();
    assert!(display.contains("{"));
    assert!(display.contains("}"));
    assert!(display.contains("key"));
    assert!(display.contains("value"));

    // Test Date display
    let date = FactValue::Date(
        DateTime::parse_from_rfc3339("2020-01-15T09:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
    );
    let display = date.to_string();
    assert!(display.contains("2020"));
    assert!(display.contains("01"));
    assert!(display.contains("15"));

    // Test Null display
    let null_val = FactValue::Null;
    assert_eq!(null_val.to_string(), "null");
}

#[test]
fn test_extended_type_equality() {
    // Test Array equality
    let arr1 = FactValue::Array(vec![FactValue::Integer(1), FactValue::Integer(2)]);
    let arr2 = FactValue::Array(vec![FactValue::Integer(1), FactValue::Integer(2)]);
    let arr3 = FactValue::Array(vec![FactValue::Integer(2), FactValue::Integer(1)]);

    assert_eq!(arr1, arr2);
    assert_ne!(arr1, arr3);

    // Test Object equality
    let mut obj1_map = HashMap::new();
    obj1_map.insert("key".to_string(), FactValue::Integer(1));
    let obj1 = FactValue::Object(obj1_map);

    let mut obj2_map = HashMap::new();
    obj2_map.insert("key".to_string(), FactValue::Integer(1));
    let obj2 = FactValue::Object(obj2_map);

    let mut obj3_map = HashMap::new();
    obj3_map.insert("key".to_string(), FactValue::Integer(2));
    let obj3 = FactValue::Object(obj3_map);

    assert_eq!(obj1, obj2);
    assert_ne!(obj1, obj3);

    // Test Date equality
    let date1 = FactValue::Date(
        DateTime::parse_from_rfc3339("2020-01-15T09:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
    );
    let date2 = FactValue::Date(
        DateTime::parse_from_rfc3339("2020-01-15T09:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
    );
    let date3 = FactValue::Date(
        DateTime::parse_from_rfc3339("2020-01-16T09:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
    );

    assert_eq!(date1, date2);
    assert_ne!(date1, date3);

    // Test Null equality
    let null1 = FactValue::Null;
    let null2 = FactValue::Null;
    let int_val = FactValue::Integer(0);

    assert_eq!(null1, null2);
    assert_ne!(null1, int_val);
}

#[test]
fn test_extended_type_hashing() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_value(val: &FactValue) -> u64 {
        let mut hasher = DefaultHasher::new();
        val.hash(&mut hasher);
        hasher.finish()
    }

    // Test that equal values have same hash
    let arr1 = FactValue::Array(vec![FactValue::Integer(1)]);
    let arr2 = FactValue::Array(vec![FactValue::Integer(1)]);
    assert_eq!(hash_value(&arr1), hash_value(&arr2));

    let mut obj1_map = HashMap::new();
    obj1_map.insert("key".to_string(), FactValue::Integer(1));
    let obj1 = FactValue::Object(obj1_map);

    let mut obj2_map = HashMap::new();
    obj2_map.insert("key".to_string(), FactValue::Integer(1));
    let obj2 = FactValue::Object(obj2_map);

    assert_eq!(hash_value(&obj1), hash_value(&obj2));

    let null1 = FactValue::Null;
    let null2 = FactValue::Null;
    assert_eq!(hash_value(&null1), hash_value(&null2));
}
