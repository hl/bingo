/// Debug test to isolate beta network issue
use bingo_core::*;

#[test]
fn debug_beta_network_issue() {
    println!("🔍 Debug: Testing beta network issue...");

    let mut engine = BingoEngine::new().unwrap();

    // Simple 2-condition rule: age > 18 AND status == "active"
    let rule = Rule {
        id: 1,
        name: "Test Rule".to_string(),
        conditions: vec![
            Condition::Simple {
                field: "age".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Integer(18),
            },
            Condition::Simple {
                field: "status".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("active".to_string()),
            },
        ],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "eligible".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };
    engine.add_rule(rule).unwrap();

    println!("✅ Added rule: age > 18 AND status == 'active'");

    // Test fact that should NOT match (age=16, status=active)
    let fact = Fact {
        id: 1,
        external_id: None,
        timestamp: chrono::Utc::now(),
        data: FactData {
            fields: [
                ("user_id".to_string(), FactValue::Integer(102)), // Extra field like in original test
                ("age".to_string(), FactValue::Integer(16)),      // 16 is NOT > 18
                (
                    "status".to_string(),
                    FactValue::String("active".to_string()),
                ), // Matches second condition
            ]
            .iter()
            .cloned()
            .collect(),
        },
    };

    println!("📊 Testing fact: age=16, status=active");

    let results = engine.process_facts(vec![fact]).unwrap();

    println!("📈 Results: {} (expected: 0)", results.len());

    // This should be 0 because age=16 is not > 18
    assert_eq!(
        results.len(),
        0,
        "Fact with age=16 should not match rule requiring age > 18"
    );
}

#[test]
fn debug_single_condition_test() {
    println!("🔍 Debug: Testing single conditions separately...");

    let mut engine = BingoEngine::new().unwrap();

    // Test first condition only: age > 18
    let rule1 = Rule {
        id: 1,
        name: "Age Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "age".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(18),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "age_check".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };
    engine.add_rule(rule1).unwrap();

    // Test second condition only: status == "active"
    let rule2 = Rule {
        id: 2,
        name: "Status Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("active".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "status_check".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };
    engine.add_rule(rule2).unwrap();

    println!("✅ Added two single-condition rules");

    // Test fact: age=16, status=active
    let fact = Fact {
        id: 1,
        external_id: None,
        timestamp: chrono::Utc::now(),
        data: FactData {
            fields: [
                ("age".to_string(), FactValue::Integer(16)),
                (
                    "status".to_string(),
                    FactValue::String("active".to_string()),
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        },
    };

    let results = engine.process_facts(vec![fact]).unwrap();

    println!("📈 Single condition results: {}", results.len());
    println!("   Expected: 1 (only status rule should match)");

    // Should be 1: age=16 fails rule1 (age > 18), but status=active matches rule2
    assert_eq!(results.len(), 1, "Only the status rule should match");
}
