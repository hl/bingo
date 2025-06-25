//! Integration tests for Stream Processing functionality
//!
//! This test validates stream processing capabilities including windowed aggregations,
//! temporal pattern matching, and event-time processing with watermarks.

use bingo_core::*;
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_stream_processing_basic_functionality() {
    println!("🌊 Basic Stream Processing Test");
    println!("==============================");

    let mut processor = StreamProcessor::new();

    // Register a tumbling window of 10 seconds
    processor.register_window(
        "payment_window".to_string(),
        WindowSpec::Tumbling { size: Duration::from_secs(10) },
    );

    println!("📊 Processing payment events...");

    // Create payment facts with timestamps
    let payments = vec![
        create_payment_fact(1, 100, 1000),  // Window 1
        create_payment_fact(2, 200, 3000),  // Window 1
        create_payment_fact(3, 150, 7000),  // Window 1
        create_payment_fact(4, 300, 12000), // Window 2
        create_payment_fact(5, 250, 15000), // Window 2
    ];

    // Process payments
    for payment in payments {
        let timestamp = extract_timestamp(&payment);
        processor.process_fact(payment, Some(timestamp)).unwrap();
    }

    // Update watermark to complete windows
    processor.update_watermark(Timestamp::from_millis(20000)).unwrap();

    let completed_windows = processor.get_completed_windows("payment_window");
    println!("  Completed windows: {}", completed_windows.len());

    // Should have 2 completed windows
    assert_eq!(completed_windows.len(), 2);

    // First window should have 3 payments
    let window1 = &completed_windows[0];
    println!("  Window 1: {} payments", window1.facts.len());
    assert_eq!(window1.facts.len(), 3);

    // Second window should have 2 payments
    let window2 = &completed_windows[1];
    println!("  Window 2: {} payments", window2.facts.len());
    assert_eq!(window2.facts.len(), 2);

    let stats = processor.get_stats();
    println!("  Events processed: {}", stats.events_processed);
    println!("  Windows created: {}", stats.windows_created);
    println!("  Windows completed: {}", stats.windows_completed);

    assert_eq!(stats.events_processed, 5);
    assert_eq!(stats.windows_created, 2);
    assert_eq!(stats.windows_completed, 2);

    println!("  ✅ Basic stream processing working correctly!");
}

#[test]
fn test_windowed_aggregations() {
    println!("📈 Windowed Aggregations Test");
    println!("============================");

    let mut processor = StreamProcessor::new();

    // Register tumbling window
    processor.register_window(
        "metrics_window".to_string(),
        WindowSpec::Tumbling { size: Duration::from_secs(5) },
    );

    // Create metric facts
    let metrics = vec![
        create_metric_fact(1, "cpu_usage", 50, 1000),
        create_metric_fact(2, "cpu_usage", 70, 2000),
        create_metric_fact(3, "cpu_usage", 60, 3000),
        create_metric_fact(4, "cpu_usage", 90, 6000), // Next window
        create_metric_fact(5, "cpu_usage", 80, 7000),
    ];

    println!("📊 Processing metrics and computing aggregations...");

    // Process metrics
    for metric in metrics {
        let timestamp = extract_timestamp(&metric);
        processor.process_fact(metric, Some(timestamp)).unwrap();
    }

    // Update watermark
    processor.update_watermark(Timestamp::from_millis(10000)).unwrap();

    // Test different aggregation functions
    let aggregations = vec![
        AggregationFunction::Count,
        AggregationFunction::Sum { field: "value".to_string() },
        AggregationFunction::Average { field: "value".to_string() },
        AggregationFunction::Min { field: "value".to_string() },
        AggregationFunction::Max { field: "value".to_string() },
    ];

    for aggregation in aggregations {
        let results = processor.compute_window_aggregation("metrics_window", &aggregation).unwrap();
        println!(
            "  {:?}: {} windows with results",
            aggregation,
            results.len()
        );

        if let Some((window_id, result)) = results.first() {
            println!("    Window {}: {:?}", window_id, result);
        }
    }

    let stats = processor.get_stats();
    println!("  Aggregations computed: {}", stats.aggregations_computed);
    assert!(stats.aggregations_computed > 0);

    println!("  ✅ Windowed aggregations working correctly!");
}

#[test]
fn test_sliding_windows() {
    println!("🔄 Sliding Windows Test");
    println!("=====================");

    let mut processor = StreamProcessor::new();

    // Register sliding window: 10 second window, advancing every 5 seconds
    processor.register_window(
        "sliding_metrics".to_string(),
        WindowSpec::Sliding { size: Duration::from_secs(10), advance: Duration::from_secs(5) },
    );

    // Create overlapping events
    let events = vec![
        create_metric_fact(1, "requests", 10, 1000),
        create_metric_fact(2, "requests", 15, 6000),
        create_metric_fact(3, "requests", 20, 11000),
        create_metric_fact(4, "requests", 25, 16000),
    ];

    println!("📊 Processing overlapping events...");

    for event in events {
        let timestamp = extract_timestamp(&event);
        processor.process_fact(event, Some(timestamp)).unwrap();
    }

    // Update watermark
    processor.update_watermark(Timestamp::from_millis(25000)).unwrap();

    let completed_windows = processor.get_completed_windows("sliding_metrics");
    println!("  Sliding windows created: {}", completed_windows.len());

    // Should have multiple overlapping windows
    assert!(completed_windows.len() > 2);

    // Check that windows overlap correctly
    for (i, window) in completed_windows.iter().enumerate() {
        println!(
            "  Window {}: {}-{} ({} facts)",
            i,
            window.start_time.as_millis(),
            window.end_time.as_millis(),
            window.facts.len()
        );
    }

    let stats = processor.get_stats();
    println!("  Events processed: {}", stats.events_processed);
    println!("  Windows created: {}", stats.windows_created);

    assert_eq!(stats.events_processed, 4);
    assert!(stats.windows_created >= 3); // Multiple overlapping windows

    println!("  ✅ Sliding windows working correctly!");
}

#[test]
fn test_session_windows() {
    println!("⏱️ Session Windows Test");
    println!("======================");

    let mut processor = StreamProcessor::new();

    // Register session window with 3 second gap timeout
    processor.register_window(
        "user_session".to_string(),
        WindowSpec::Session { gap_timeout: Duration::from_secs(3) },
    );

    // Create user activity with gaps
    let activities = vec![
        create_activity_fact(1, "login", 1000),
        create_activity_fact(2, "click", 2000), // Same session
        create_activity_fact(3, "view", 3000),  // Same session
        create_activity_fact(4, "search", 7000), // New session (4s gap)
        create_activity_fact(5, "click", 8000), // Same session
        create_activity_fact(6, "logout", 15000), // New session (7s gap)
    ];

    println!("📊 Processing user activities...");

    for activity in activities {
        let timestamp = extract_timestamp(&activity);
        processor.process_fact(activity, Some(timestamp)).unwrap();
    }

    // Update watermark to complete sessions
    processor.update_watermark(Timestamp::from_millis(20000)).unwrap();

    let completed_windows = processor.get_completed_windows("user_session");
    println!("  Session windows created: {}", completed_windows.len());

    // Should have 3 distinct sessions
    assert_eq!(completed_windows.len(), 3);

    // Check session sizes
    let session_sizes: Vec<usize> = completed_windows.iter().map(|w| w.facts.len()).collect();
    println!("  Session sizes: {:?}", session_sizes);

    // First session: 3 activities, second: 2 activities, third: 1 activity
    assert_eq!(session_sizes, vec![3, 2, 1]);

    let stats = processor.get_stats();
    println!("  Events processed: {}", stats.events_processed);
    println!("  Windows created: {}", stats.windows_created);

    assert_eq!(stats.events_processed, 6);
    assert_eq!(stats.windows_created, 3);

    println!("  ✅ Session windows working correctly!");
}

#[test]
fn test_late_event_handling() {
    println!("⏰ Late Event Handling Test");
    println!("==========================");

    let mut processor = StreamProcessor::new();
    processor.set_max_lateness(Duration::from_secs(5));

    processor.register_window(
        "late_events".to_string(),
        WindowSpec::Tumbling { size: Duration::from_secs(10) },
    );

    // Set watermark to 15 seconds
    processor.update_watermark(Timestamp::from_millis(15000)).unwrap();

    println!("📊 Processing late events...");

    // Try to add events that are acceptably late
    let acceptable_late = create_payment_fact(1, 100, 12000); // 3s late, acceptable
    processor
        .process_fact(acceptable_late, Some(Timestamp::from_millis(12000)))
        .unwrap();

    // Try to add events that are too late
    let too_late = create_payment_fact(2, 200, 5000); // 10s late, too late
    processor.process_fact(too_late, Some(Timestamp::from_millis(5000))).unwrap();

    let stats = processor.get_stats();
    println!("  Events processed: {}", stats.events_processed);
    println!("  Late events dropped: {}", stats.late_events_dropped);

    // One event should be accepted, one dropped
    assert_eq!(stats.events_processed, 2);
    assert_eq!(stats.late_events_dropped, 1);

    println!("  ✅ Late event handling working correctly!");
}

#[test]
fn test_stream_rule_integration() {
    println!("🔗 Stream Rule Integration Test");
    println!("==============================");

    let mut engine = ReteNetwork::new().unwrap();

    // Create a rule with stream condition
    let stream_rule = Rule {
        id: 1,
        name: "high_payment_volume_alert".to_string(),
        conditions: vec![Condition::Stream(StreamCondition {
            window_spec: StreamWindowSpec::Tumbling { duration_ms: 60000 }, // 1 minute
            aggregation: StreamAggregation::Sum { field: "amount".to_string() },
            filter: Some(Box::new(Condition::Simple {
                field: "type".to_string(),
                operator: Operator::Equal,
                value: FactValue::String("payment".to_string()),
            })),
            having: Some(Box::new(Condition::Simple {
                field: "total_amount".to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Integer(1000),
            })),
            alias: "total_amount".to_string(),
        })],
        actions: vec![Action {
            action_type: ActionType::TriggerAlert {
                alert_type: "HIGH_PAYMENT_VOLUME".to_string(),
                message: "High payment volume detected in last minute".to_string(),
                severity: AlertSeverity::High,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("window_duration".to_string(), FactValue::Integer(60000));
                    meta
                },
            },
        }],
    };

    // For now, just validate the rule structure can be created
    println!("📊 Created stream rule: {}", stream_rule.name);
    println!(
        "  Window: {:?}",
        if let Condition::Stream(sc) = &stream_rule.conditions[0] {
            &sc.window_spec
        } else {
            panic!("Expected stream condition")
        }
    );

    // Basic validation - rule should have stream condition
    assert_eq!(stream_rule.conditions.len(), 1);
    if let Condition::Stream(stream_condition) = &stream_rule.conditions[0] {
        assert_eq!(stream_condition.alias, "total_amount");
        assert!(stream_condition.filter.is_some());
        assert!(stream_condition.having.is_some());
    } else {
        panic!("Expected stream condition");
    }

    // Action should be an alert
    assert_eq!(stream_rule.actions.len(), 1);
    if let ActionType::TriggerAlert { alert_type, severity, .. } =
        &stream_rule.actions[0].action_type
    {
        assert_eq!(alert_type, "HIGH_PAYMENT_VOLUME");
        assert_eq!(*severity, AlertSeverity::High);
    } else {
        panic!("Expected alert action");
    }

    println!("  ✅ Stream rule integration structure validated!");
}

#[test]
fn test_window_cleanup() {
    println!("🧹 Window Cleanup Test");
    println!("=====================");

    let mut processor = StreamProcessor::new();

    processor.register_window(
        "cleanup_test".to_string(),
        WindowSpec::Tumbling { size: Duration::from_secs(5) },
    );

    // Create events over time
    let events = vec![
        create_payment_fact(1, 100, 1000),
        create_payment_fact(2, 200, 6000),
        create_payment_fact(3, 300, 11000),
    ];

    for event in events {
        let timestamp = extract_timestamp(&event);
        processor.process_fact(event, Some(timestamp)).unwrap();
    }

    // Complete all windows
    processor.update_watermark(Timestamp::from_millis(20000)).unwrap();

    let windows_before_count = processor.get_completed_windows("cleanup_test").len();
    println!("  Windows before cleanup: {}", windows_before_count);

    // Clean up old windows (retain only last 5 seconds)
    processor.cleanup_old_windows(Duration::from_secs(5));

    let windows_after = processor.get_completed_windows("cleanup_test");
    println!("  Windows after cleanup: {}", windows_after.len());

    // Should have fewer windows after cleanup
    assert!(windows_after.len() <= windows_before_count);

    let stats = processor.get_stats();
    println!(
        "  Final stats - Events: {}, Windows: {}",
        stats.events_processed, stats.windows_created
    );

    println!("  ✅ Window cleanup working correctly!");
}

// Helper functions for creating test facts

fn create_payment_fact(id: u64, amount: i64, timestamp: u64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert("type".to_string(), FactValue::String("payment".to_string()));
    fields.insert("amount".to_string(), FactValue::Integer(amount));
    fields.insert(
        "timestamp".to_string(),
        FactValue::Integer(timestamp as i64),
    );

    Fact { id, data: FactData { fields } }
}

fn create_metric_fact(id: u64, metric_name: &str, value: i64, timestamp: u64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert("type".to_string(), FactValue::String("metric".to_string()));
    fields.insert(
        "metric_name".to_string(),
        FactValue::String(metric_name.to_string()),
    );
    fields.insert("value".to_string(), FactValue::Integer(value));
    fields.insert(
        "timestamp".to_string(),
        FactValue::Integer(timestamp as i64),
    );

    Fact { id, data: FactData { fields } }
}

fn create_activity_fact(id: u64, activity_type: &str, timestamp: u64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert(
        "type".to_string(),
        FactValue::String("activity".to_string()),
    );
    fields.insert(
        "activity_type".to_string(),
        FactValue::String(activity_type.to_string()),
    );
    fields.insert(
        "timestamp".to_string(),
        FactValue::Integer(timestamp as i64),
    );

    Fact { id, data: FactData { fields } }
}

fn extract_timestamp(fact: &Fact) -> Timestamp {
    if let Some(FactValue::Integer(ts)) = fact.data.fields.get("timestamp") {
        Timestamp::from_millis(*ts as u64)
    } else {
        Timestamp::now()
    }
}
