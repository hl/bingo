//! Comprehensive integration test for new ActionType handlers
//!
//! This test validates Task 3.2: Implement missing ActionType handlers
//! Tests UpdateFact, DeleteFact, IncrementField, AppendToArray, and SendNotification

use bingo_core::BingoEngine;
use bingo_core::types::{
    Action, ActionType, Condition, Fact, FactData, FactValue, NotificationType, Operator, Rule,
};
use chrono::Utc;
use std::collections::HashMap;

/// Create test facts for action type testing
fn create_test_facts() -> Vec<Fact> {
    vec![
        create_user_fact(1, "Alice", 1000.0, 50),
        create_user_fact(2, "Bob", 2000.0, 75),
        create_user_fact(3, "Charlie", 1500.0, 60),
    ]
}

fn create_user_fact(id: u64, name: &str, balance: f64, score: i64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), FactValue::Integer(id as i64));
    fields.insert("name".to_string(), FactValue::String(name.to_string()));
    fields.insert("balance".to_string(), FactValue::Float(balance));
    fields.insert("score".to_string(), FactValue::Integer(score));
    fields.insert(
        "tags".to_string(),
        FactValue::Array(vec![FactValue::String("active".to_string())]),
    );

    Fact {
        id,
        external_id: Some(format!("user-{id}")),
        timestamp: Utc::now(),
        data: FactData { fields },
    }
}

#[test]
fn test_update_fact_action() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing UpdateFact Action");

    // Create a rule that updates another user's balance based on current user
    let rule = Rule {
        id: 1,
        name: "Update Target User Balance".to_string(),
        conditions: vec![Condition::Simple {
            field: "name".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("Alice".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::UpdateFact {
                fact_id_field: "user_id".to_string(), // Use Alice's user_id field (1) to target fact ID 1
                updates: {
                    let mut updates = HashMap::new();
                    updates.insert("balance".to_string(), FactValue::Float(5000.0));
                    updates.insert(
                        "status".to_string(),
                        FactValue::String("updated".to_string()),
                    );
                    updates
                },
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts
    let facts = create_test_facts();
    let results = engine.process_facts(facts).unwrap();

    println!(
        "🎯 UpdateFact action results: {} rules fired",
        results.len()
    );

    // Verify UpdateFact results
    let update_results: Vec<_> = results
        .iter()
        .filter_map(|r| {
            r.actions_executed.iter().find_map(|action| {
                if let bingo_core::rete_nodes::ActionResult::FactUpdated {
                    fact_id,
                    updated_fields,
                } = action
                {
                    Some((fact_id, updated_fields))
                } else {
                    None
                }
            })
        })
        .collect();

    assert!(!update_results.is_empty(), "Should have UpdateFact results");

    for (fact_id, updated_fields) in update_results {
        println!("📊 Updated Fact ID: {fact_id}, Fields: {updated_fields:?}");
        assert_eq!(*fact_id, 1); // Alice's user_id
        assert!(updated_fields.contains(&"balance".to_string()));
        assert!(updated_fields.contains(&"status".to_string()));
    }

    println!("✅ UpdateFact action test passed");
}

#[test]
fn test_delete_fact_action() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing DeleteFact Action");

    // Create a rule that deletes another user based on current user
    let rule = Rule {
        id: 1,
        name: "Delete Target User".to_string(),
        conditions: vec![Condition::Simple {
            field: "name".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("Bob".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::DeleteFact {
                fact_id_field: "user_id".to_string(), // Use Bob's user_id field (2) to target fact ID 2
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts
    let facts = create_test_facts();
    let results = engine.process_facts(facts).unwrap();

    println!(
        "🎯 DeleteFact action results: {} rules fired",
        results.len()
    );

    // Verify DeleteFact results
    let delete_results: Vec<_> = results
        .iter()
        .filter_map(|r| {
            r.actions_executed.iter().find_map(|action| {
                if let bingo_core::rete_nodes::ActionResult::FactDeleted { fact_id } = action {
                    Some(fact_id)
                } else {
                    None
                }
            })
        })
        .collect();

    assert!(!delete_results.is_empty(), "Should have DeleteFact results");

    for fact_id in delete_results {
        println!("📊 Deleted Fact ID: {fact_id}");
        assert_eq!(*fact_id, 2); // Bob's user_id
    }

    println!("✅ DeleteFact action test passed");
}

#[test]
fn test_increment_field_action() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing IncrementField Action");

    // Test integer increment
    let rule1 = Rule {
        id: 1,
        name: "Increment Score".to_string(),
        conditions: vec![Condition::Simple {
            field: "name".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("Alice".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::IncrementField {
                field: "score".to_string(),
                increment: FactValue::Integer(10),
            },
        }],
    };

    // Test float increment
    let rule2 = Rule {
        id: 2,
        name: "Increment Balance".to_string(),
        conditions: vec![Condition::Simple {
            field: "name".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("Bob".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::IncrementField {
                field: "balance".to_string(),
                increment: FactValue::Float(500.5),
            },
        }],
    };

    engine.add_rule(rule1).unwrap();
    engine.add_rule(rule2).unwrap();

    // Process facts
    let facts = create_test_facts();
    let results = engine.process_facts(facts).unwrap();

    println!(
        "🎯 IncrementField action results: {} rules fired",
        results.len()
    );

    // Debug: Print what actions were actually executed
    for (i, result) in results.iter().enumerate() {
        println!(
            "🔍 Rule {}: {} actions executed",
            i,
            result.actions_executed.len()
        );
        for (j, action) in result.actions_executed.iter().enumerate() {
            println!("   Action {j}: {action:?}");
        }
    }

    // Verify IncrementField results
    let increment_results: Vec<_> = results
        .iter()
        .filter_map(|r| {
            r.actions_executed.iter().find_map(|action| {
                if let bingo_core::rete_nodes::ActionResult::FieldIncremented {
                    fact_id,
                    field,
                    old_value,
                    new_value,
                } = action
                {
                    Some((fact_id, field, old_value, new_value))
                } else {
                    None
                }
            })
        })
        .collect();

    assert!(
        !increment_results.is_empty(),
        "Should have IncrementField results"
    );

    for (fact_id, field, old_value, new_value) in increment_results {
        println!(
            "📊 Incremented Fact ID: {fact_id}, Field: {field}, Old: {old_value:?}, New: {new_value:?}"
        );

        match field.as_str() {
            "score" => {
                assert_eq!(*fact_id, 1); // Alice's fact
                if let FactValue::Integer(old) = old_value {
                    if let FactValue::Integer(new) = new_value {
                        assert_eq!(*new, *old + 10);
                    }
                }
            }
            "balance" => {
                assert_eq!(*fact_id, 2); // Bob's fact
                if let FactValue::Float(old) = old_value {
                    if let FactValue::Float(new) = new_value {
                        assert!((new - (old + 500.5)).abs() < 0.01);
                    }
                }
            }
            _ => {}
        }
    }

    println!("✅ IncrementField action test passed");
}

#[test]
fn test_append_to_array_action() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing AppendToArray Action");

    // Create a rule that appends to the tags array
    let rule = Rule {
        id: 1,
        name: "Add Premium Tag".to_string(),
        conditions: vec![Condition::Simple {
            field: "name".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("Charlie".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::AppendToArray {
                field: "tags".to_string(),
                value: FactValue::String("premium".to_string()),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts
    let facts = create_test_facts();
    let results = engine.process_facts(facts).unwrap();

    println!(
        "🎯 AppendToArray action results: {} rules fired",
        results.len()
    );

    // Verify AppendToArray results
    let append_results: Vec<_> = results
        .iter()
        .filter_map(|r| {
            r.actions_executed.iter().find_map(|action| {
                if let bingo_core::rete_nodes::ActionResult::ArrayAppended {
                    fact_id,
                    field,
                    appended_value,
                    new_length,
                } = action
                {
                    Some((fact_id, field, appended_value, new_length))
                } else {
                    None
                }
            })
        })
        .collect();

    assert!(
        !append_results.is_empty(),
        "Should have AppendToArray results"
    );

    for (fact_id, field, appended_value, new_length) in append_results {
        println!(
            "📊 Appended to Fact ID: {fact_id}, Field: {field}, Value: {appended_value:?}, New Length: {new_length}"
        );
        assert_eq!(*fact_id, 3); // Charlie's fact
        assert_eq!(field, "tags");
        assert_eq!(appended_value, &FactValue::String("premium".to_string()));
        assert_eq!(*new_length, 2); // Original "active" + new "premium"
    }

    println!("✅ AppendToArray action test passed");
}

#[test]
fn test_send_notification_action() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing SendNotification Action");

    // Create a rule that sends notifications
    let rule = Rule {
        id: 1,
        name: "Send Welcome Email".to_string(),
        conditions: vec![Condition::Simple {
            field: "name".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("Alice".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::SendNotification {
                recipient: "alice@example.com".to_string(),
                subject: "Welcome to our platform!".to_string(),
                message: "Thank you for joining us.".to_string(),
                notification_type: NotificationType::Email,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert(
                        "template_id".to_string(),
                        FactValue::String("welcome".to_string()),
                    );
                    meta.insert(
                        "priority".to_string(),
                        FactValue::String("high".to_string()),
                    );
                    meta
                },
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts
    let facts = create_test_facts();
    let results = engine.process_facts(facts).unwrap();

    println!(
        "🎯 SendNotification action results: {} rules fired",
        results.len()
    );

    // Verify SendNotification results
    let notification_results: Vec<_> = results
        .iter()
        .filter_map(|r| {
            r.actions_executed.iter().find_map(|action| {
                if let bingo_core::rete_nodes::ActionResult::NotificationSent {
                    recipient,
                    notification_type,
                    subject,
                } = action
                {
                    Some((recipient, notification_type, subject))
                } else {
                    None
                }
            })
        })
        .collect();

    assert!(
        !notification_results.is_empty(),
        "Should have SendNotification results"
    );

    for (recipient, notification_type, subject) in notification_results {
        println!(
            "📊 Notification sent to: {recipient}, Type: {notification_type:?}, Subject: {subject}"
        );
        assert_eq!(recipient, "alice@example.com");
        assert_eq!(notification_type, &NotificationType::Email);
        assert_eq!(subject, "Welcome to our platform!");
    }

    println!("✅ SendNotification action test passed");
}

#[test]
fn test_error_handling_for_action_types() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Error Handling for New Action Types");

    // Test UpdateFact with non-existent field
    let rule1 = Rule {
        id: 1,
        name: "Update with missing field".to_string(),
        conditions: vec![Condition::Simple {
            field: "name".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("Alice".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::UpdateFact {
                fact_id_field: "non_existent_field".to_string(),
                updates: {
                    let mut updates = HashMap::new();
                    updates.insert("balance".to_string(), FactValue::Float(5000.0));
                    updates
                },
            },
        }],
    };

    // Test AppendToArray on non-array field
    let rule2 = Rule {
        id: 2,
        name: "Append to non-array".to_string(),
        conditions: vec![Condition::Simple {
            field: "name".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("Bob".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::AppendToArray {
                field: "balance".to_string(), // This is a Float, not an Array
                value: FactValue::String("invalid".to_string()),
            },
        }],
    };

    engine.add_rule(rule1).unwrap();
    engine.add_rule(rule2).unwrap();

    // Process facts
    let facts = create_test_facts();
    let results = engine.process_facts(facts).unwrap();

    println!("🎯 Error handling results: {} rules fired", results.len());

    // Verify error handling through logged messages
    let error_results: Vec<_> = results
        .iter()
        .filter_map(|r| {
            r.actions_executed.iter().find_map(|action| {
                if let bingo_core::rete_nodes::ActionResult::Logged { message } = action {
                    if message.contains("failed") {
                        Some(message)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
        .collect();

    assert!(
        !error_results.is_empty(),
        "Should have error handling results"
    );

    for error_message in error_results {
        println!("⚠️  Error handled: {error_message}");
        // Verify specific error patterns
        assert!(
            error_message.contains("UpdateFact failed")
                || error_message.contains("AppendToArray failed")
        );
    }

    println!("✅ Error handling test passed");
}
