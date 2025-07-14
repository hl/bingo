use bingo_core::engine::BingoEngine;
use bingo_core::types::{Fact, FactData, FactValue};
use std::collections::HashMap;

#[test]
fn test_fact_id_lookup() {
    let engine = BingoEngine::new().unwrap();

    let mut fields = HashMap::new();
    fields.insert(
        "name".to_string(),
        FactValue::String("John Doe".to_string()),
    );
    fields.insert("age".to_string(), FactValue::Integer(42));

    let fact = Fact {
        id: 0, // Internal ID, will be overwritten by the fact store
        external_id: Some("user-123".to_string()),
        timestamp: chrono::Utc::now(),
        data: FactData { fields },
    };

    engine.process_facts(vec![fact]).unwrap();

    // 1. Test lookup_fact_by_id
    let looked_up_fact = engine.lookup_fact_by_id("user-123");
    assert!(
        looked_up_fact.is_some(),
        "Fact should be found by external ID"
    );
    let looked_up_fact = looked_up_fact.unwrap();
    assert_eq!(looked_up_fact.external_id.as_deref(), Some("user-123"));
    assert_eq!(
        *looked_up_fact.data.fields.get("name").unwrap(),
        FactValue::String("John Doe".to_string())
    );

    // 2. Test get_field_by_id
    let name_field = engine.get_field_by_id("user-123", "name");
    assert!(name_field.is_some(), "Field 'name' should be found");
    assert_eq!(
        name_field.unwrap(),
        FactValue::String("John Doe".to_string())
    );

    let age_field = engine.get_field_by_id("user-123", "age");
    assert!(age_field.is_some(), "Field 'age' should be found");
    assert_eq!(age_field.unwrap(), FactValue::Integer(42));

    // 3. Test non-existent fact and field
    let non_existent_fact = engine.lookup_fact_by_id("non-existent-id");
    assert!(
        non_existent_fact.is_none(),
        "Should not find non-existent fact"
    );

    let non_existent_field = engine.get_field_by_id("user-123", "non-existent-field");
    assert!(
        non_existent_field.is_none(),
        "Should not find non-existent field"
    );
}
