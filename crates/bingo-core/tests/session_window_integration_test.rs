// Comprehensive integration test for session window support in stream processing.
//
// This test validates Task 3.3: Add session window support for stream processing
// and exercises Stream conditions plus Aggregation conditions with session windows.

use bingo_core::BingoEngine;
use bingo_core::types::{
    Action, ActionType, AggregationCondition, AggregationType, AggregationWindow, Condition, Fact,
    FactData, FactValue, Operator, Rule, StreamAggregation, StreamCondition, StreamWindowSpec,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Create test facts with timestamps for session window testing
fn create_session_facts() -> Vec<Fact> {
    let base_time = Utc::now();

    // Create facts that form two sessions with a gap larger than timeout
    vec![
        // Session 1: Facts at times 0, 1, 2 seconds (continuous session)
        create_fact_with_timestamp(1, "user_123", "login", base_time),
        create_fact_with_timestamp(
            2,
            "user_123",
            "browse",
            base_time + chrono::Duration::seconds(1),
        ),
        create_fact_with_timestamp(
            3,
            "user_123",
            "purchase",
            base_time + chrono::Duration::seconds(2),
        ),
        // Gap of 10 seconds (larger than 5-second timeout)

        // Session 2: Facts at times 12, 13, 14 seconds (new session)
        create_fact_with_timestamp(
            4,
            "user_123",
            "login",
            base_time + chrono::Duration::seconds(12),
        ),
        create_fact_with_timestamp(
            5,
            "user_123",
            "browse",
            base_time + chrono::Duration::seconds(13),
        ),
        create_fact_with_timestamp(
            6,
            "user_123",
            "add_to_cart",
            base_time + chrono::Duration::seconds(14),
        ),
    ]
}

fn create_fact_with_timestamp(
    id: u64,
    user_id: &str,
    action: &str,
    timestamp: DateTime<Utc>,
) -> Fact {
    let mut fields = HashMap::new();
    fields.insert(
        "user_id".to_string(),
        FactValue::String(user_id.to_string()),
    );
    fields.insert("action".to_string(), FactValue::String(action.to_string()));
    fields.insert("session_value".to_string(), FactValue::Integer(1)); // For counting in session

    Fact {
        id,
        external_id: Some(format!("event-{}", id)),
        timestamp,
        data: FactData { fields },
    }
}

#[test]
fn test_stream_condition_with_session_windows() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Stream Condition with Session Windows");

    // Create a rule that counts events in a session window
    // Session timeout: 5 seconds
    let rule = Rule {
        id: 1,
        name: "Session Activity Counter".to_string(),
        conditions: vec![Condition::Stream(StreamCondition {
            alias: "session_count".to_string(),
            aggregation: StreamAggregation::Count,
            window_spec: StreamWindowSpec::Session { gap_timeout_ms: 5000 }, // 5 second gap timeout
            filter: None, // Count all events in session
            having: Some(Box::new(Condition::Simple {
                field: "session_count".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Integer(2), // Trigger when session has > 2 events
            })),
        })],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Session with >2 events detected".to_string() },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts
    let facts = create_session_facts();
    let results = engine.process_facts(facts).unwrap();

    println!(
        "🎯 Session window rule results: {} rules fired",
        results.len()
    );

    // In RETE, rules fire for each fact that triggers them
    // For session windows with >2 events, this will fire multiple times per session
    // Session 1: fires on fact 2 and fact 3 (when count becomes > 2)
    // Session 2: fires on fact 5 and fact 6 (when count becomes > 2)
    assert!(
        results.len() >= 2,
        "Should fire at least for facts that complete sessions with >2 events"
    );

    // Verify action results
    let log_results: Vec<_> = results
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

    assert!(log_results.len() >= 2, "Should have at least 2 log results");

    for message in log_results {
        println!("📊 Session detected: {}", message);
        assert!(message.contains("Session with >2 events detected"));
    }

    println!("✅ Stream condition session window test passed");
}

#[test]
fn test_aggregation_condition_with_session_windows() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Aggregation Condition with Session Windows");

    // Create a rule that uses session window aggregation
    let rule = Rule {
        id: 1,
        name: "Session Sum Aggregation".to_string(),
        conditions: vec![Condition::Aggregation(AggregationCondition {
            alias: "session_sum".to_string(),
            aggregation_type: AggregationType::Sum,
            source_field: "session_value".to_string(),
            window: Some(AggregationWindow::Session { timeout_ms: 5000 }), // 5 second session timeout
            group_by: vec!["user_id".to_string()],
            having: Some(Box::new(Condition::Simple {
                field: "session_sum".to_string(),
                operator: Operator::GreaterThanOrEqual,
                value: FactValue::Integer(3), // Trigger when session sum >= 3
            })),
        })],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Session sum >= 3 detected".to_string() },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts
    let facts = create_session_facts();
    let results = engine.process_facts(facts).unwrap();

    println!(
        "🎯 Session aggregation rule results: {} rules fired",
        results.len()
    );

    // In RETE, rules fire for each fact that meets the aggregation condition
    // Each fact in sessions where sum >= 3 will trigger the rule
    assert!(
        results.len() >= 2,
        "Should fire for facts in sessions with sum >= 3"
    );

    // Verify action results
    let log_results: Vec<_> = results
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

    assert!(log_results.len() >= 2, "Should have at least 2 log results");

    for message in log_results {
        println!("📊 Session aggregation: {}", message);
        assert!(message.contains("Session sum >= 3 detected"));
    }

    println!("✅ Aggregation condition session window test passed");
}

#[test]
fn test_session_window_gap_timeout_behavior() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Session Window Gap Timeout Behavior");

    // Create a rule with strict session timeout (1 second)
    let rule = Rule {
        id: 1,
        name: "Strict Session Counter".to_string(),
        conditions: vec![Condition::Stream(StreamCondition {
            alias: "strict_session_count".to_string(),
            aggregation: StreamAggregation::Count,
            window_spec: StreamWindowSpec::Session { gap_timeout_ms: 1000 }, // 1 second gap timeout
            filter: None,
            having: Some(Box::new(Condition::Simple {
                field: "strict_session_count".to_string(),
                operator: Operator::GreaterThanOrEqual,
                value: FactValue::Integer(2), // Trigger when session has >= 2 events
            })),
        })],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Strict session detected".to_string() },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Create facts with varying gaps
    let base_time = Utc::now();
    let facts = vec![
        // Session 1: 0s and 0.5s (gap < 1s, should be in same session)
        create_fact_with_timestamp(1, "user_456", "start", base_time),
        create_fact_with_timestamp(
            2,
            "user_456",
            "continue",
            base_time + chrono::Duration::milliseconds(500),
        ),
        // Session 2: 3s (gap > 1s from previous, new session)
        create_fact_with_timestamp(
            3,
            "user_456",
            "restart",
            base_time + chrono::Duration::seconds(3),
        ),
        // Session 3: 4s and 4.8s (gap < 1s, should be in same session)
        create_fact_with_timestamp(
            4,
            "user_456",
            "action1",
            base_time + chrono::Duration::seconds(4),
        ),
        create_fact_with_timestamp(
            5,
            "user_456",
            "action2",
            base_time + chrono::Duration::milliseconds(4800),
        ),
    ];

    let results = engine.process_facts(facts).unwrap();

    println!("🎯 Strict session results: {} rules fired", results.len());

    // Should fire for facts in sessions with >= 2 events
    // Session 1: facts 1,2 -> fires on fact 2
    // Session 2: facts 4,5 -> fires on fact 5
    // Fact 3 is alone and doesn't meet >= 2 threshold
    assert!(
        results.len() >= 2,
        "Should fire for sessions with >= 2 events each"
    );

    let log_results: Vec<_> = results
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

    assert!(log_results.len() >= 2, "Should have at least 2 log results");

    for message in log_results {
        println!("📊 Strict session: {}", message);
        assert!(message.contains("Strict session detected"));
    }

    println!("✅ Session window gap timeout test passed");
}

#[test]
fn test_session_window_with_filter_condition() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Session Window with Filter Condition");

    // Create a rule that only counts "purchase" events in session windows
    let rule = Rule {
        id: 1,
        name: "Purchase Session Counter".to_string(),
        conditions: vec![Condition::Stream(StreamCondition {
            alias: "purchase_session_count".to_string(),
            aggregation: StreamAggregation::Count,
            window_spec: StreamWindowSpec::Session { gap_timeout_ms: 5000 }, // 5 second gap timeout
            filter: Some(Box::new(Condition::Simple {
                field: "action".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("purchase".to_string()),
            })),
            having: Some(Box::new(Condition::Simple {
                field: "purchase_session_count".to_string(),
                operator: Operator::GreaterThanOrEqual,
                value: FactValue::Integer(1), // Trigger when session has >= 1 purchase
            })),
        })],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Session with purchases detected".to_string() },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts (only first session has a purchase)
    let facts = create_session_facts();
    let results = engine.process_facts(facts).unwrap();

    println!("🎯 Filtered session results: {} rules fired", results.len());

    // Should fire for facts in sessions containing purchase events
    // Only the first session has a "purchase" event
    assert!(
        !results.is_empty(),
        "Should fire for session containing purchase"
    );

    let log_results: Vec<_> = results
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

    assert!(!log_results.is_empty(), "Should have at least 1 log result");
    assert!(log_results.iter().any(|msg| msg.contains("Session with purchases detected")));

    println!("📊 Filtered session result: {}", log_results[0]);
    println!("✅ Session window filter test passed");
}

#[test]
fn test_multiple_users_session_windows() {
    let mut engine = BingoEngine::new().unwrap();

    println!("🧪 Testing Session Windows with Multiple Users");

    // Create a rule that groups by user_id
    let rule = Rule {
        id: 1,
        name: "User Session Aggregation".to_string(),
        conditions: vec![Condition::Aggregation(AggregationCondition {
            alias: "user_session_count".to_string(),
            aggregation_type: AggregationType::Count,
            source_field: "session_value".to_string(),
            window: Some(AggregationWindow::Session { timeout_ms: 3000 }), // 3 second session timeout
            group_by: vec!["user_id".to_string()],
            having: Some(Box::new(Condition::Simple {
                field: "user_session_count".to_string(),
                operator: Operator::GreaterThanOrEqual,
                value: FactValue::Integer(2), // Trigger when session has >= 2 events
            })),
        })],
        actions: vec![Action {
            action_type: ActionType::Log { message: "User session with >=2 events".to_string() },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Create facts for different users
    let base_time = Utc::now();
    let facts = vec![
        // User A: session with 3 events (0s, 1s, 2s)
        create_fact_with_timestamp(1, "user_A", "login", base_time),
        create_fact_with_timestamp(
            2,
            "user_A",
            "browse",
            base_time + chrono::Duration::seconds(1),
        ),
        create_fact_with_timestamp(
            3,
            "user_A",
            "purchase",
            base_time + chrono::Duration::seconds(2),
        ),
        // User B: session with 2 events (1s, 2s)
        create_fact_with_timestamp(
            4,
            "user_B",
            "login",
            base_time + chrono::Duration::seconds(1),
        ),
        create_fact_with_timestamp(
            5,
            "user_B",
            "browse",
            base_time + chrono::Duration::seconds(2),
        ),
        // User C: only 1 event (should not trigger)
        create_fact_with_timestamp(
            6,
            "user_C",
            "login",
            base_time + chrono::Duration::seconds(1),
        ),
    ];

    let results = engine.process_facts(facts).unwrap();

    println!(
        "🎯 Multi-user session results: {} rules fired",
        results.len()
    );

    // Should fire for facts from users A and B (both have >= 2 events in their sessions)
    // User C has only 1 event and should not trigger
    assert!(
        results.len() >= 2,
        "Should fire for users with >= 2 events each"
    );

    let log_results: Vec<_> = results
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

    assert!(log_results.len() >= 2, "Should have at least 2 log results");

    for message in log_results {
        println!("📊 Multi-user session: {}", message);
        assert!(message.contains("User session with >=2 events"));
    }

    println!("✅ Multi-user session window test passed");
}
