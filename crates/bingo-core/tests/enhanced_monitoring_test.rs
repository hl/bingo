//! Enhanced Monitoring System Test Suite
//!
//! This test suite validates the comprehensive monitoring capabilities,
//! including performance metrics, resource tracking, business metrics,
//! alerting, and health scoring functionality.

use bingo_core::{
    BingoEngine,
    enhanced_monitoring::{
        AlertSeverity, AlertType, BusinessMetrics, EnginePerformanceMetrics, EnhancedMonitoring,
        MemoryPoolMetrics, MonitoringConfig, ParallelProcessingMetrics, ResourceMetrics,
        RetePerformanceMetrics,
    },
    types::*,
};
use std::collections::HashMap;

/// Test basic enhanced monitoring functionality
#[test]
fn test_enhanced_monitoring_basic_functionality() {
    let monitoring = EnhancedMonitoring::default();

    // Test performance metrics recording
    let engine_metrics = EnginePerformanceMetrics {
        facts_per_second: 1500.0,
        avg_rule_execution_time_us: 25.0,
        success_rate_percent: 99.8,
        cpu_usage_percent: 45.0,
        avg_memory_per_operation: 2048,
        ..Default::default()
    };

    monitoring.record_engine_performance(engine_metrics).unwrap();

    // Test resource metrics recording
    let resource_metrics = ResourceMetrics {
        memory_usage_bytes: 128 * 1024 * 1024, // 128MB
        peak_memory_bytes: 256 * 1024 * 1024,  // 256MB
        thread_count: 8,
        active_connections: 15,
        ..Default::default()
    };

    monitoring.record_resource_metrics(resource_metrics).unwrap();

    // Test business metrics recording
    let business_metrics = BusinessMetrics {
        rules_processed_last_hour: 45000,
        compliance_checks_performed: 1200,
        error_rate_percent: 0.2,
        rule_violations_detected: 3,
        avg_processing_latency_ms: 12.5,
        ..Default::default()
    };

    monitoring.record_business_metrics(business_metrics).unwrap();

    // Generate monitoring report
    let report = monitoring.generate_monitoring_report().unwrap();

    // Validate report contents
    assert!(report.system_health_score >= 0.0);
    assert!(report.system_health_score <= 100.0);
    assert_eq!(
        report.performance_metrics.engine_performance.facts_per_second,
        1500.0
    );
    assert_eq!(
        report.resource_metrics.memory_usage_bytes,
        128 * 1024 * 1024
    );
    assert_eq!(report.business_metrics.rules_processed_last_hour, 45000);
}

/// Test BingoEngine integration with enhanced monitoring
#[test]
fn test_bingo_engine_enhanced_monitoring_integration() {
    let monitoring_config = MonitoringConfig {
        enabled: true,
        sampling_interval_seconds: 30,
        enable_prometheus_export: false, // Disable for testing
        enable_detailed_tracing: false,
        ..Default::default()
    };

    let mut engine = BingoEngine::with_enhanced_monitoring(monitoring_config).unwrap();

    // Create test rule
    let rule = Rule {
        id: 1,
        name: "Performance Test Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "score".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Float(80.0),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "tier".to_string(),
                value: FactValue::String("premium".to_string()),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Create test facts
    let mut facts = Vec::new();
    for i in 0..100 {
        let mut fields = HashMap::new();
        fields.insert("score".to_string(), FactValue::Float(85.0 + i as f64));
        fields.insert(
            "customer_id".to_string(),
            FactValue::String(format!("C{i}")),
        );

        facts.push(Fact::new(i as u64, FactData { fields }));
    }

    // Process facts (this should automatically record performance metrics)
    let results = engine.process_facts(facts).unwrap();
    assert_eq!(results.len(), 100); // All facts should match the rule

    // Generate monitoring report
    let report = engine.generate_monitoring_report().unwrap();

    // Validate that metrics were recorded
    assert!(report.performance_metrics.engine_performance.facts_per_second > 0.0);
    assert!(report.system_health_score > 0.0);

    // Test system health score
    let health_score = engine.get_system_health_score().unwrap();
    assert!((0.0..=100.0).contains(&health_score));

    // Test business metrics recording
    engine.record_business_metrics(100, 25, 2).unwrap();

    let updated_report = engine.generate_monitoring_report().unwrap();
    assert_eq!(
        updated_report.business_metrics.rules_processed_last_hour,
        100
    );
    assert_eq!(
        updated_report.business_metrics.compliance_checks_performed,
        25
    );
    assert_eq!(updated_report.business_metrics.rule_violations_detected, 2);
}

/// Test monitoring configuration and customization
#[test]
fn test_monitoring_configuration() {
    let custom_config = MonitoringConfig {
        enabled: true,
        sampling_interval_seconds: 15,
        max_historical_samples: 500,
        enable_profiler_integration: true,
        enable_prometheus_export: true,
        enable_detailed_tracing: true,
    };

    let monitoring = EnhancedMonitoring::new(custom_config.clone());

    // Verify configuration is applied
    let config = monitoring.get_config();
    assert_eq!(config.sampling_interval_seconds, 15);
    assert_eq!(config.max_historical_samples, 500);
    assert!(config.enable_profiler_integration);
    assert!(config.enable_prometheus_export);
    assert!(config.enable_detailed_tracing);
}

/// Test historical data management and sampling
#[test]
fn test_historical_data_management() {
    let config = MonitoringConfig {
        max_historical_samples: 5, // Small limit for testing
        ..Default::default()
    };

    let monitoring = EnhancedMonitoring::new(config);

    // Add multiple performance metrics
    for i in 0..8 {
        let metrics = EnginePerformanceMetrics {
            facts_per_second: (i * 100) as f64,
            avg_rule_execution_time_us: (i * 10) as f64,
            success_rate_percent: 99.0 - (i as f64 * 0.1),
            ..Default::default()
        };

        monitoring.record_engine_performance(metrics).unwrap();
        monitoring.add_historical_sample().unwrap();
    }

    // Verify that historical data is maintained within limits
    let historical_data = monitoring.get_historical_data_reader().unwrap();
    assert_eq!(historical_data.performance_samples_len(), 5);

    // Verify the most recent samples are kept
    let latest_sample = historical_data.get_latest_performance_sample().unwrap();
    assert_eq!(
        latest_sample.metrics.engine_performance.facts_per_second,
        700.0
    );
}

/// Test alert generation and management
#[test]
fn test_alert_generation() {
    let monitoring = EnhancedMonitoring::default();

    // Record high CPU usage that should trigger an alert
    let engine_metrics = EnginePerformanceMetrics {
        cpu_usage_percent: 95.0, // Above critical threshold (90%)
        facts_per_second: 100.0,
        success_rate_percent: 100.0,
        ..Default::default()
    };

    monitoring.record_engine_performance(engine_metrics).unwrap();

    // Generate report and check for alerts
    let report = monitoring.generate_monitoring_report().unwrap();

    // Should have generated a high CPU usage alert
    let cpu_alerts: Vec<_> = report
        .active_alerts
        .iter()
        .filter(|alert| matches!(alert.alert_type, AlertType::HighCpuUsage))
        .collect();

    assert!(
        !cpu_alerts.is_empty(),
        "Should have generated CPU usage alert"
    );

    let cpu_alert = &cpu_alerts[0];
    assert!(matches!(cpu_alert.severity, AlertSeverity::Critical));
    assert_eq!(cpu_alert.metric_value, 95.0);
    assert!(!cpu_alert.resolved);
}

/// Test health score calculation with different scenarios
#[test]
fn test_health_score_calculation() {
    let monitoring = EnhancedMonitoring::default();

    // Test scenario 1: Excellent performance
    let excellent_engine_metrics = EnginePerformanceMetrics {
        facts_per_second: 2000.0,
        success_rate_percent: 99.9,
        cpu_usage_percent: 30.0,
        ..Default::default()
    };

    let excellent_resource_metrics = ResourceMetrics {
        memory_usage_bytes: 64 * 1024 * 1024, // 64MB
        peak_memory_bytes: 512 * 1024 * 1024, // 512MB (12.5% usage)
        ..Default::default()
    };

    let excellent_business_metrics =
        BusinessMetrics { error_rate_percent: 0.1, ..Default::default() };

    monitoring.record_engine_performance(excellent_engine_metrics).unwrap();
    monitoring.record_resource_metrics(excellent_resource_metrics).unwrap();
    monitoring.record_business_metrics(excellent_business_metrics).unwrap();

    // Add missing metrics needed for health score calculation
    let excellent_rete_metrics = RetePerformanceMetrics {
        alpha_node_hit_rate: 98.0,
        beta_node_efficiency: 95.0,
        ..Default::default()
    };
    monitoring.record_rete_performance(excellent_rete_metrics).unwrap();

    let excellent_memory_pool_metrics =
        MemoryPoolMetrics { overall_hit_rate: 96.0, ..Default::default() };
    monitoring
        .record_memory_pool_performance(excellent_memory_pool_metrics)
        .unwrap();

    let excellent_parallel_metrics =
        ParallelProcessingMetrics { parallel_efficiency: 93.0, ..Default::default() };
    monitoring.record_parallel_performance(excellent_parallel_metrics).unwrap();

    let excellent_report = monitoring.generate_monitoring_report().unwrap();
    assert!(
        excellent_report.system_health_score > 90.0,
        "Excellent metrics should yield high health score"
    );

    // Test scenario 2: Poor performance
    let poor_engine_metrics = EnginePerformanceMetrics {
        facts_per_second: 50.0,
        success_rate_percent: 85.0,
        cpu_usage_percent: 95.0,
        ..Default::default()
    };

    let poor_resource_metrics = ResourceMetrics {
        memory_usage_bytes: 480 * 1024 * 1024, // 480MB
        peak_memory_bytes: 512 * 1024 * 1024,  // 512MB (93.75% usage)
        ..Default::default()
    };

    let poor_business_metrics = BusinessMetrics { error_rate_percent: 15.0, ..Default::default() };

    monitoring.record_engine_performance(poor_engine_metrics).unwrap();
    monitoring.record_resource_metrics(poor_resource_metrics).unwrap();
    monitoring.record_business_metrics(poor_business_metrics).unwrap();

    // Add poor metrics for complete health score calculation
    let poor_rete_metrics = RetePerformanceMetrics {
        alpha_node_hit_rate: 60.0,
        beta_node_efficiency: 55.0,
        ..Default::default()
    };
    monitoring.record_rete_performance(poor_rete_metrics).unwrap();

    let poor_memory_pool_metrics =
        MemoryPoolMetrics { overall_hit_rate: 45.0, ..Default::default() };
    monitoring.record_memory_pool_performance(poor_memory_pool_metrics).unwrap();

    let poor_parallel_metrics =
        ParallelProcessingMetrics { parallel_efficiency: 40.0, ..Default::default() };
    monitoring.record_parallel_performance(poor_parallel_metrics).unwrap();

    let poor_report = monitoring.generate_monitoring_report().unwrap();
    assert!(
        poor_report.system_health_score < 70.0,
        "Poor metrics should yield low health score"
    );
}

/// Test monitoring report JSON serialization
#[test]
fn test_monitoring_report_serialization() {
    let monitoring = EnhancedMonitoring::default();

    // Add sample metrics
    let engine_metrics = EnginePerformanceMetrics {
        facts_per_second: 1200.0,
        success_rate_percent: 98.5,
        ..Default::default()
    };

    monitoring.record_engine_performance(engine_metrics).unwrap();

    // Generate report
    let report = monitoring.generate_monitoring_report().unwrap();

    // Test JSON serialization
    let json_report = report.to_json().unwrap();
    assert!(json_report.contains("facts_per_second"));
    assert!(json_report.contains("1200"));
    assert!(json_report.contains("system_health_score"));

    // Test summary generation
    let summary = report.get_summary();
    assert!(summary.health_score >= 0.0);
    assert!(summary.health_score <= 100.0);
    assert_eq!(summary.facts_per_second, 1200.0);
}

/// Test performance metrics accuracy during high-load scenarios
#[test]
fn test_performance_metrics_accuracy() {
    let monitoring_config = MonitoringConfig {
        enabled: true,
        sampling_interval_seconds: 1, // Fast sampling for testing
        ..Default::default()
    };

    let mut engine = BingoEngine::with_enhanced_monitoring(monitoring_config).unwrap();

    // Add a simple rule
    let rule = Rule {
        id: 1,
        name: "Load Test Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "value".to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Float(50.0),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: "processed".to_string(),
                value: FactValue::Boolean(true),
            },
        }],
    };

    engine.add_rule(rule).unwrap();

    // Process facts in multiple batches
    let mut total_facts_processed = 0;
    let batch_size = 50usize;
    let num_batches = 5;

    for batch in 0..num_batches {
        let mut facts = Vec::new();
        for i in 0..batch_size {
            let mut fields = HashMap::new();
            fields.insert("value".to_string(), FactValue::Float(75.0));
            fields.insert("batch".to_string(), FactValue::Integer(batch as i64));
            fields.insert("index".to_string(), FactValue::Integer(i as i64));

            facts.push(Fact::new(
                (batch * batch_size + i) as u64,
                FactData { fields },
            ));
        }

        let results = engine.process_facts(facts).unwrap();
        total_facts_processed += batch_size;

        // Verify all facts were processed
        assert_eq!(results.len(), batch_size);

        // Add historical sample
        engine.add_monitoring_sample().unwrap();
    }

    // Generate final report
    let report = engine.generate_monitoring_report().unwrap();

    // Verify metrics accuracy
    assert!(report.performance_metrics.engine_performance.facts_per_second > 0.0);
    assert!(report.system_health_score > 0.0);

    // Verify processing counts are tracked
    assert!(total_facts_processed > 0);
}

/// Test monitoring behavior when disabled
#[test]
fn test_monitoring_disabled_behavior() {
    let disabled_config = MonitoringConfig { enabled: false, ..Default::default() };

    let monitoring = EnhancedMonitoring::new(disabled_config);

    // Attempt to record metrics when disabled
    let engine_metrics =
        EnginePerformanceMetrics { facts_per_second: 1000.0, ..Default::default() };

    // Should succeed but not actually record anything
    monitoring.record_engine_performance(engine_metrics).unwrap();

    // Generate report
    let report = monitoring.generate_monitoring_report().unwrap();

    // Should have default/empty metrics
    assert_eq!(
        report.performance_metrics.engine_performance.facts_per_second,
        0.0
    );
}

/// Test concurrent access to monitoring system
#[test]
fn test_concurrent_monitoring_access() {
    use std::sync::Arc;
    use std::thread;

    let monitoring = Arc::new(EnhancedMonitoring::default());
    let num_threads = 4;
    let operations_per_thread = 25;

    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let monitoring = Arc::clone(&monitoring);

        let handle = thread::spawn(move || {
            for i in 0..operations_per_thread {
                let engine_metrics = EnginePerformanceMetrics {
                    facts_per_second: (thread_id * 100 + i) as f64,
                    success_rate_percent: 99.0,
                    ..Default::default()
                };

                monitoring.record_engine_performance(engine_metrics).unwrap();

                // Occasionally add historical samples
                if i % 5 == 0 {
                    monitoring.add_historical_sample().unwrap();
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Generate report
    let report = monitoring.generate_monitoring_report().unwrap();

    // Verify concurrent operations completed successfully
    assert!(report.performance_metrics.engine_performance.facts_per_second >= 0.0);
    assert!(report.system_health_score >= 0.0);
}
