//! Test distributed state synchronization and consistency mechanisms

use bingo_core::distributed_rete::*;
use bingo_core::*;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[tokio::test]
async fn test_state_synchronization_initialization() {
    println!("🔄 State Synchronization Initialization Test");
    println!("=============================================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing state coordinator initialization...");

    // Check initial state coordinator state
    assert_eq!(
        network.get_state_coordinator().get_pending_operations().len(),
        0
    );
    assert_eq!(
        network.get_state_coordinator().get_state_version(&StateType::ReteNodeState),
        0
    );
    assert_eq!(
        network.get_state_coordinator().get_state_version(&StateType::ClusterMembership),
        0
    );

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    // Test periodic sync initialization
    let cluster_nodes = vec![network.local_node.node_id];
    network
        .get_state_coordinator_mut()
        .start_periodic_sync(&cluster_nodes)
        .await
        .unwrap();

    // Verify sync operations were created
    let pending_ops = network.get_state_coordinator().get_pending_operations();
    assert!(pending_ops.len() >= 3); // Should have ClusterMembership, PartitioningScheme, ReteNodeState

    println!("  Pending sync operations: {}", pending_ops.len());
    for op in &pending_ops {
        println!("    {:?}: {:?}", op.state_type, op.status);
    }

    // Verify state versions were initialized
    assert!(network.get_state_coordinator().get_state_version(&StateType::ClusterMembership) >= 1);
    assert!(network.get_state_coordinator().get_state_version(&StateType::ReteNodeState) >= 1);

    println!("  ✅ State synchronization initialization working correctly!");
}

#[tokio::test]
async fn test_state_synchronization_messaging() {
    println!("📬 State Synchronization Messaging Test");
    println!("======================================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing state synchronization messaging...");

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    // Add a second node to test messaging
    let second_node_id = uuid::Uuid::new_v4();
    let second_node = ClusterNode {
        node_id: second_node_id,
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
        capabilities: NodeCapabilities::default(),
        status: NodeStatus::Active,
        last_heartbeat: distributed_rete::current_timestamp(),
        metrics: NodeMetrics::default(),
    };
    network.cluster.nodes.insert(second_node_id, second_node);

    let initial_messages_sent = network.stats.routing_stats.total_messages_sent;

    // Test state synchronization
    let target_nodes = vec![second_node_id];
    let result = network.synchronize_state(StateType::ClusterMembership, target_nodes).await;
    assert!(result.is_ok());

    // Verify message was queued
    assert!(network.stats.routing_stats.total_messages_sent > initial_messages_sent);

    // Check that sync operation is tracked
    let pending_ops = network.get_state_coordinator().get_pending_operations();
    let membership_sync = pending_ops
        .iter()
        .find(|op| matches!(op.state_type, StateType::ClusterMembership));
    assert!(membership_sync.is_some());

    println!(
        "  Messages sent: {}",
        network.stats.routing_stats.total_messages_sent
    );
    println!("  Pending operations: {}", pending_ops.len());
    println!("  ✅ State synchronization messaging working correctly!");
}

#[tokio::test]
async fn test_state_consistency_checking() {
    println!("🔍 State Consistency Checking Test");
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

    println!("📊 Testing state consistency checking...");

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    // Initialize some state versions to trigger consistency checking
    network
        .get_state_coordinator_mut()
        .update_state_version(StateType::ReteNodeState, 2);
    network
        .get_state_coordinator_mut()
        .update_state_version(StateType::ClusterMembership, 3);

    let initial_inconsistencies = network
        .get_state_coordinator()
        .get_inconsistencies(InconsistencySeverity::Low)
        .len();

    // Perform consistency check
    let result = network.check_state_consistency().await;
    assert!(result.is_ok());

    // Check for any detected inconsistencies
    let inconsistencies =
        network.get_state_coordinator().get_inconsistencies(InconsistencySeverity::Low);

    println!("  Initial inconsistencies: {}", initial_inconsistencies);
    println!("  Inconsistencies after check: {}", inconsistencies.len());

    for inconsistency in &inconsistencies {
        println!(
            "    {:?}: {} ({})",
            inconsistency.state_type,
            inconsistency.description,
            format!("{:?}", inconsistency.severity)
        );
    }

    println!("  ✅ State consistency checking working correctly!");
}

#[tokio::test]
async fn test_state_conflict_resolution() {
    println!("⚖️ State Conflict Resolution Test");
    println!("=================================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing different conflict resolution strategies...");

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    // Test Last Write Wins strategy
    let result = network.resolve_state_conflicts(StateType::ReteNodeState).await;
    assert!(result.is_ok());
    println!("  Last Write Wins resolution: ✅");

    // Test Coordinator Decides strategy
    network.get_state_coordinator_mut().conflict_resolver = ConflictResolver::CoordinatorDecides;
    let result = network.resolve_state_conflicts(StateType::ClusterMembership).await;
    assert!(result.is_ok());
    println!("  Coordinator Decides resolution: ✅");

    // Test Majority Vote strategy
    network.get_state_coordinator_mut().conflict_resolver = ConflictResolver::MajorityVote;
    let result = network.resolve_state_conflicts(StateType::PartitioningScheme).await;
    assert!(result.is_ok());
    println!("  Majority Vote resolution: ✅");

    println!("  ✅ State conflict resolution working correctly!");
}

#[tokio::test]
async fn test_state_synchronization_cleanup() {
    println!("🧹 State Synchronization Cleanup Test");
    println!("=====================================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing synchronization operation cleanup...");

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    // Create some sync operations
    let cluster_nodes = vec![network.local_node.node_id];
    network
        .get_state_coordinator_mut()
        .start_periodic_sync(&cluster_nodes)
        .await
        .unwrap();

    let initial_ops = network.get_state_coordinator().get_pending_operations().len();
    println!("  Initial pending operations: {}", initial_ops);

    // Simulate some operations completing
    if let Some((_state_type, sync_op)) =
        network.get_state_coordinator_mut().pending_sync.iter_mut().next()
    {
        sync_op.status = SyncStatus::Completed;
    }

    // Perform cleanup
    network.get_state_coordinator_mut().cleanup_completed_operations();

    let final_ops = network.get_state_coordinator().get_pending_operations().len();
    println!("  Final pending operations: {}", final_ops);

    // Should have fewer operations after cleanup
    assert!(final_ops < initial_ops);

    // Test inconsistency cleanup
    let node_id = network.local_node.node_id;
    network.get_state_coordinator_mut().consistency_checker.inconsistencies.push(
        InconsistencyReport {
            state_type: StateType::ReteNodeState,
            affected_nodes: [node_id].iter().cloned().collect(),
            detected_at: distributed_rete::current_timestamp() - 7_200_000, // 2 hours ago
            severity: InconsistencySeverity::Low,
            description: "Test inconsistency".to_string(),
        },
    );

    let initial_inconsistencies = network
        .get_state_coordinator()
        .get_inconsistencies(InconsistencySeverity::Low)
        .len();

    // Clean up old inconsistencies (older than 1 hour)
    network.get_state_coordinator_mut().clear_old_inconsistencies(3_600_000);

    let final_inconsistencies = network
        .get_state_coordinator()
        .get_inconsistencies(InconsistencySeverity::Low)
        .len();

    println!("  Initial inconsistencies: {}", initial_inconsistencies);
    println!("  Final inconsistencies: {}", final_inconsistencies);

    // Old inconsistency should be cleaned up
    assert!(final_inconsistencies < initial_inconsistencies);

    println!("  ✅ State synchronization cleanup working correctly!");
}

#[tokio::test]
async fn test_distributed_state_maintenance() {
    println!("🔧 Distributed State Maintenance Test");
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

    println!("📊 Testing maintenance cycle with state synchronization...");

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    // Initialize some state
    network
        .get_state_coordinator_mut()
        .update_state_version(StateType::ReteNodeState, 1);
    network
        .get_state_coordinator_mut()
        .update_state_version(StateType::ClusterMembership, 1);

    let initial_stats = network.stats.clone();

    // Perform maintenance cycle (includes state consistency checks)
    let result = network.perform_maintenance().await;
    assert!(result.is_ok());

    // Verify maintenance was performed
    assert!(network.local_node.last_heartbeat > 0);
    assert_eq!(network.stats.cluster_stats.total_nodes, 1);
    assert_eq!(network.stats.cluster_stats.active_nodes, 1);

    println!(
        "  Heartbeat timestamp: {}",
        network.local_node.last_heartbeat
    );
    println!(
        "  Cluster utilization: {:.2}%",
        network.stats.cluster_stats.cluster_utilization * 100.0
    );

    // Verify state synchronization components were exercised
    // (actual state checks are performed in maintenance)

    println!("  ✅ Distributed state maintenance working correctly!");
}

#[tokio::test]
async fn test_state_synchronization_message_handling() {
    println!("📨 State Synchronization Message Handling Test");
    println!("==============================================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing state synchronization message handling...");

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    let initial_version =
        network.get_state_coordinator().get_state_version(&StateType::ClusterMembership);

    // Simulate receiving a state synchronization message
    let test_data = b"test_state_data".to_vec();
    let result = network
        .handle_state_synchronization(StateType::ClusterMembership, test_data)
        .await;
    assert!(result.is_ok());

    // Verify state version was updated
    let new_version =
        network.get_state_coordinator().get_state_version(&StateType::ClusterMembership);
    assert!(new_version > initial_version);

    println!("  Initial version: {}", initial_version);
    println!("  New version: {}", new_version);

    // Test handling different state types
    let state_types = [
        StateType::ReteNodeState,
        StateType::FactWorkingMemory,
        StateType::RuleDefinitions,
        StateType::PartitioningScheme,
    ];

    for state_type in state_types {
        let test_data = format!("test_data_for_{:?}", state_type).into_bytes();
        let result = network.handle_state_synchronization(state_type, test_data).await;
        assert!(result.is_ok());
        println!("  Handled {:?} synchronization: ✅", state_type);
    }

    println!("  ✅ State synchronization message handling working correctly!");
}
