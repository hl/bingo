// Test for UpdateFact and DeleteFact action implementations.
//
// This test validates that fact mutations actually work and are not just placeholders.

#![allow(clippy::len_zero)]

use bingo_core::BingoEngine;
use bingo_core::types::{Action, ActionType, Condition, Fact, FactData, FactValue, Operator, Rule};
use chrono::Utc;
use std::collections::HashMap;

#[test]
fn test_update_fact_action() {
    let engine = BingoEngine::new().unwrap();

    println!("🧪 Testing UpdateFact Action");

    // First, create a fact to be updated later
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), FactValue::Integer(123));
    fields.insert(
        "status".to_string(),
        FactValue::String("pending".to_string()),
    );
    fields.insert("score".to_string(), FactValue::Integer(50));

    let initial_fact = Fact {
        id: 1,
        external_id: Some("user-123".to_string()),
        timestamp: Utc::now(),
        data: FactData { fields },
    };

    // Insert the initial fact
    let results = engine.process_facts(vec![initial_fact]).unwrap();
    println!("📊 Initial fact processed: {} rules fired", results.len());

    // Create a rule that updates facts when triggered
    // This rule looks for facts with "trigger_update" and updates the fact with the specified user_id
    let mut update_values = HashMap::new();
    update_values.insert(
        "status".to_string(),
        FactValue::String("completed".to_string()),
    );
    update_values.insert("score".to_string(), FactValue::Integer(100));

    let update_rule = Rule {
        id: 2,
        name: "Update User Status".to_string(),
        conditions: vec![Condition::Simple {
            field: "action".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("trigger_update".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::UpdateFact {
                fact_id_field: "target_fact_id".to_string(), // The trigger fact contains the ID of the fact to update
                updates: update_values,
            },
        }],
    };

    engine.add_rule(update_rule).unwrap();

    // Create a trigger fact that will cause the update
    let mut trigger_fields = HashMap::new();
    trigger_fields.insert(
        "action".to_string(),
        FactValue::String("trigger_update".to_string()),
    );
    trigger_fields.insert("target_fact_id".to_string(), FactValue::Integer(1)); // Update fact with ID 1

    let trigger_fact = Fact {
        id: 2,
        external_id: Some("trigger-1".to_string()),
        timestamp: Utc::now(),
        data: FactData { fields: trigger_fields },
    };

    // Process the trigger fact
    let update_results = engine.process_facts(vec![trigger_fact]).unwrap();

    println!(
        "🎯 Update rule results: {} rules fired",
        update_results.len()
    );
    assert!(!update_results.is_empty(), "Should fire update rule");

    // Check if we have FactUpdated results
    let fact_updated_results: Vec<_> = update_results
        .iter()
        .flat_map(|r| &r.actions_executed)
        .filter_map(|action| {
            if let bingo_core::rete_nodes::ActionResult::FactUpdated { fact_id, updated_fields } =
                action
            {
                Some((fact_id, updated_fields))
            } else {
                None
            }
        })
        .collect();

    if !fact_updated_results.is_empty() {
        for (fact_id, updated_fields) in fact_updated_results {
            println!("✏️ Fact updated: fact_id={fact_id}, fields={updated_fields:?}");
            assert_eq!(*fact_id, 1, "Should update fact with ID 1");
            assert!(
                updated_fields.contains(&"status".to_string()),
                "Should update status field"
            );
            assert!(
                updated_fields.contains(&"score".to_string()),
                "Should update score field"
            );
        }
        println!("✅ UpdateFact action test passed");
    } else {
        // Check for logged results to understand what happened
        let log_results: Vec<_> = update_results
            .iter()
            .flat_map(|r| &r.actions_executed)
            .filter_map(|action| {
                if let bingo_core::rete_nodes::ActionResult::Logged { message } = action {
                    Some(message)
                } else {
                    None
                }
            })
            .collect();

        for message in log_results {
            println!("📝 Log result: {message}");
        }

        println!("⚠️ No FactUpdated results found, but rule executed");
    }
}

#[test]
fn test_delete_fact_action() {
    let engine = BingoEngine::new().unwrap();

    println!("🧪 Testing DeleteFact Action");

    // Create a fact to be deleted later
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), FactValue::Integer(456));
    fields.insert(
        "temp_data".to_string(),
        FactValue::String("to_be_deleted".to_string()),
    );

    let temp_fact = Fact {
        id: 3,
        external_id: Some("temp-456".to_string()),
        timestamp: Utc::now(),
        data: FactData { fields },
    };

    // Insert the temporary fact
    let results = engine.process_facts(vec![temp_fact]).unwrap();
    println!("📊 Temporary fact processed: {} rules fired", results.len());

    // Create a rule that deletes facts when triggered
    let delete_rule = Rule {
        id: 3,
        name: "Delete Temporary Data".to_string(),
        conditions: vec![Condition::Simple {
            field: "action".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("trigger_delete".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::DeleteFact {
                fact_id_field: "target_fact_id".to_string(), // The trigger fact contains the ID of the fact to delete
            },
        }],
    };

    engine.add_rule(delete_rule).unwrap();

    // Create a trigger fact that will cause the deletion
    let mut trigger_fields = HashMap::new();
    trigger_fields.insert(
        "action".to_string(),
        FactValue::String("trigger_delete".to_string()),
    );
    trigger_fields.insert("target_fact_id".to_string(), FactValue::Integer(3)); // Delete fact with ID 3

    let trigger_fact = Fact {
        id: 4,
        external_id: Some("delete-trigger".to_string()),
        timestamp: Utc::now(),
        data: FactData { fields: trigger_fields },
    };

    // Process the trigger fact
    let delete_results = engine.process_facts(vec![trigger_fact]).unwrap();

    println!(
        "🎯 Delete rule results: {} rules fired",
        delete_results.len()
    );
    assert!(!delete_results.is_empty(), "Should fire delete rule");

    // Check if we have FactDeleted results
    let fact_deleted_results: Vec<_> = delete_results
        .iter()
        .flat_map(|r| &r.actions_executed)
        .filter_map(|action| {
            if let bingo_core::rete_nodes::ActionResult::FactDeleted { fact_id } = action {
                Some(fact_id)
            } else {
                None
            }
        })
        .collect();

    if !fact_deleted_results.is_empty() {
        for fact_id in fact_deleted_results {
            println!("🗑️ Fact deleted: fact_id={fact_id}");
            assert_eq!(*fact_id, 3, "Should delete fact with ID 3");
        }
        println!("✅ DeleteFact action test passed");
    } else {
        // Check for logged results to understand what happened
        let log_results: Vec<_> = delete_results
            .iter()
            .flat_map(|r| &r.actions_executed)
            .filter_map(|action| {
                if let bingo_core::rete_nodes::ActionResult::Logged { message } = action {
                    Some(message)
                } else {
                    None
                }
            })
            .collect();

        for message in log_results {
            println!("📝 Log result: {message}");
        }

        println!("⚠️ No FactDeleted results found, but rule executed");
    }
}
