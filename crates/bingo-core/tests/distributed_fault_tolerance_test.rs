//! Test distributed fault tolerance and node failure recovery functionality

use bingo_core::distributed_rete::*;
use bingo_core::*;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use uuid::Uuid;

#[tokio::test]
async fn test_fault_tolerance_manager_initialization() {
    println!("🔧 Fault Tolerance Manager Initialization Test");
    println!("=============================================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing fault tolerance manager initialization...");

    // Check that fault tolerance manager was initialized
    let initial_stats = &network.stats.fault_tolerance_stats;
    assert_eq!(initial_stats.node_failures_detected, 0);
    assert_eq!(initial_stats.successful_recoveries, 0);
    assert_eq!(initial_stats.failed_recoveries, 0);
    assert_eq!(initial_stats.availability_percentage, 100.0);

    println!(
        "  Node failures detected: {}",
        initial_stats.node_failures_detected
    );
    println!(
        "  Successful recoveries: {}",
        initial_stats.successful_recoveries
    );
    println!("  Failed recoveries: {}", initial_stats.failed_recoveries);
    println!(
        "  Availability: {:.2}%",
        initial_stats.availability_percentage
    );
    println!("  ✅ Fault tolerance manager initialization working correctly!");
}

#[tokio::test]
async fn test_node_failure_recovery_strategies() {
    println!("🚨 Node Failure Recovery Strategies Test");
    println!("=======================================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing different recovery strategies...");

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    // Create test cluster nodes for recovery scenarios
    let mut cluster_nodes = HashMap::new();

    // Add healthy nodes
    for i in 1..=3 {
        let node_id = Uuid::new_v4();
        let node = ClusterNode {
            node_id,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080 + i),
            capabilities: NodeCapabilities::default(),
            status: NodeStatus::Active,
            last_heartbeat: distributed_rete::current_timestamp(),
            metrics: NodeMetrics::default(),
        };
        cluster_nodes.insert(node_id, node);
    }

    // Add a failed node to test recovery
    let failed_node_id = Uuid::new_v4();
    let failed_node = ClusterNode {
        node_id: failed_node_id,
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9000),
        capabilities: NodeCapabilities::default(),
        status: NodeStatus::Failed,
        last_heartbeat: distributed_rete::current_timestamp() - 120_000, // 2 minutes ago
        metrics: NodeMetrics::default(),
    };
    cluster_nodes.insert(failed_node_id, failed_node);

    // Test node failure recovery
    let result = network.handle_node_failure(failed_node_id).await;
    assert!(result.is_ok());

    let recovery_result = result.unwrap();
    println!("  Recovery result: {:?}", recovery_result);

    match recovery_result {
        RecoveryResult::Recovered { recovery_time_ms, data_loss, actions_taken } => {
            println!("  Recovery completed successfully:");
            println!("    Recovery time: {}ms", recovery_time_ms);
            println!("    Data loss: {}", data_loss);
            println!("    Actions taken: {:?}", actions_taken);
            assert!(recovery_time_ms > 0);
            assert!(!actions_taken.is_empty());
        }
        _ => panic!("Expected successful recovery"),
    }

    println!("  ✅ Node failure recovery strategies working correctly!");
}

#[tokio::test]
async fn test_circuit_breaker_functionality() {
    println!("🔄 Circuit Breaker Functionality Test");
    println!("====================================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing circuit breaker states and transitions...");

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    let test_node_id = Uuid::new_v4();

    // Test initial state (should allow operations)
    assert!(network.is_operation_allowed(test_node_id));

    let cb_state = network.get_circuit_breaker_state(test_node_id);
    let cb_failures = network.get_circuit_breaker_failure_count(test_node_id);
    assert_eq!(cb_state, CircuitBreakerState::Closed);
    assert_eq!(cb_failures, 0);

    println!("  Initial circuit breaker state: {:?}", cb_state);

    // Record successful operations
    for _ in 0..3 {
        network.record_operation_result(test_node_id, true);
    }

    let cb_state = network.get_circuit_breaker_state(test_node_id);
    let cb_failures = network.get_circuit_breaker_failure_count(test_node_id);
    assert_eq!(cb_state, CircuitBreakerState::Closed);
    assert_eq!(cb_failures, 0);

    println!("  After successful operations: {:?}", cb_state);

    // Record failures to trigger circuit breaker
    for i in 0..5 {
        network.record_operation_result(test_node_id, false);
        let state = network.get_circuit_breaker_state(test_node_id);
        let failures = network.get_circuit_breaker_failure_count(test_node_id);
        println!(
            "    Failure {}: state={:?}, failures={}",
            i + 1,
            state,
            failures
        );
    }

    // Circuit breaker should now be open
    let cb_state = network.get_circuit_breaker_state(test_node_id);
    assert_eq!(cb_state, CircuitBreakerState::Open);

    // Operations should now be blocked
    assert!(!network.is_operation_allowed(test_node_id));

    println!("  Circuit breaker opened after failures: {:?}", cb_state);
    println!("  ✅ Circuit breaker functionality working correctly!");
}

#[tokio::test]
async fn test_recovery_statistics_tracking() {
    println!("📈 Recovery Statistics Tracking Test");
    println!("===================================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing recovery statistics tracking...");

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    let initial_stats = network.stats.fault_tolerance_stats.clone();

    // Simulate a successful recovery
    let recovery_result = RecoveryResult::Recovered {
        recovery_time_ms: 150,
        data_loss: false,
        actions_taken: vec!["Test recovery action".to_string()],
    };

    let mut updated_stats = network.stats.fault_tolerance_stats.clone();
    network.update_fault_tolerance_stats(&mut updated_stats, &recovery_result);

    // Verify statistics were updated
    assert!(updated_stats.successful_recoveries > initial_stats.successful_recoveries);
    assert!(updated_stats.average_recovery_time_ms > 0.0);

    println!(
        "  Initial successful recoveries: {}",
        initial_stats.successful_recoveries
    );
    println!(
        "  Updated successful recoveries: {}",
        updated_stats.successful_recoveries
    );
    println!(
        "  Average recovery time: {:.2}ms",
        updated_stats.average_recovery_time_ms
    );

    // Simulate a failed recovery
    let failed_recovery =
        RecoveryResult::Failed { reason: "Test failure".to_string(), retry_possible: true };

    network.update_fault_tolerance_stats(&mut updated_stats, &failed_recovery);

    assert!(updated_stats.failed_recoveries > initial_stats.failed_recoveries);

    println!("  Failed recoveries: {}", updated_stats.failed_recoveries);
    println!("  ✅ Recovery statistics tracking working correctly!");
}

#[tokio::test]
async fn test_partial_recovery_scenarios() {
    println!("⚠️ Partial Recovery Scenarios Test");
    println!("==================================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing partial recovery handling...");

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    // Simulate a partial recovery result
    let partial_recovery = RecoveryResult::PartialRecovery {
        recovery_time_ms: 300,
        recovered_percentage: 75.0,
        data_loss: true,
        actions_taken: vec![
            "Partial state restoration".to_string(),
            "Limited functionality recovered".to_string(),
        ],
    };

    let mut stats = network.stats.fault_tolerance_stats.clone();
    let initial_successful = stats.successful_recoveries;
    let initial_data_loss = stats.data_loss_incidents;

    network.update_fault_tolerance_stats(&mut stats, &partial_recovery);

    // Verify partial recovery is counted as successful
    assert_eq!(stats.successful_recoveries, initial_successful + 1);
    assert_eq!(stats.data_loss_incidents, initial_data_loss + 1);

    println!("  Partial recovery handled successfully:");
    println!("    Recovery percentage: 75.0%");
    println!("    Data loss incidents: {}", stats.data_loss_incidents);
    println!("    Successful recoveries: {}", stats.successful_recoveries);

    // Test manual intervention required scenario
    let manual_intervention = RecoveryResult::ManualInterventionRequired;
    let initial_failed = stats.failed_recoveries;

    network.update_fault_tolerance_stats(&mut stats, &manual_intervention);

    // Manual intervention should be counted as failed for statistics
    assert_eq!(stats.failed_recoveries, initial_failed + 1);

    println!("  Manual intervention required handled correctly");
    println!("  ✅ Partial recovery scenarios working correctly!");
}

#[tokio::test]
async fn test_fault_tolerance_integration() {
    println!("🔗 Fault Tolerance Integration Test");
    println!("==================================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing fault tolerance integration with distributed system...");

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    // Add some nodes to the cluster
    for i in 1..=2 {
        let node_id = Uuid::new_v4();
        let node = ClusterNode {
            node_id,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080 + i),
            capabilities: NodeCapabilities::default(),
            status: NodeStatus::Active,
            last_heartbeat: distributed_rete::current_timestamp(),
            metrics: NodeMetrics::default(),
        };
        network.cluster.nodes.insert(node_id, node);
    }

    let initial_cluster_health = network.calculate_cluster_health();
    println!("  Initial cluster health: {:?}", initial_cluster_health);

    // Simulate a node failure
    if let Some(&failed_node_id) = network.cluster.nodes.keys().next() {
        if let Some(node) = network.cluster.nodes.get_mut(&failed_node_id) {
            node.status = NodeStatus::Failed;
            node.last_heartbeat = distributed_rete::current_timestamp() - 120_000;
        }

        // Test that fault tolerance can handle the failure
        let recovery_result = network.handle_node_failure(failed_node_id).await;

        assert!(recovery_result.is_ok());
        println!("  Node failure recovery triggered successfully");

        // Check that cluster health reflects the failure
        let post_failure_health = network.calculate_cluster_health();
        println!("  Post-failure cluster health: {:?}", post_failure_health);

        // Cluster health should be degraded due to the failed node
        assert_ne!(post_failure_health, ClusterHealth::Healthy);
    }

    println!("  ✅ Fault tolerance integration working correctly!");
}
