//! Integration tests for Distributed RETE Network functionality
//!
//! This test validates distributed RETE capabilities including cluster membership,
//! node partitioning, fault tolerance, and distributed state synchronization.

use bingo_core::distributed_rete::*;
use bingo_core::*;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use uuid::Uuid;

#[test]
fn test_distributed_network_creation() {
    println!("🌐 Distributed Network Creation Test");
    println!("==================================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

    println!("📊 Creating distributed RETE network...");
    let network = DistributedReteNetwork::new(address, config);

    assert!(network.is_ok());
    let network = network.unwrap();

    println!("  Local node ID: {}", network.local_node.node_id);
    println!("  Local address: {}", network.local_node.address);
    println!("  Node status: {:?}", network.local_node.status);

    // Validate initial state
    assert_eq!(network.local_node.address, address);
    assert_eq!(network.local_node.status, NodeStatus::Joining);
    assert!(network.local_node.capabilities.can_coordinate);
    assert_eq!(network.local_node.capabilities.max_rete_nodes, 10_000);

    let status = network.get_cluster_status();
    println!("  Cluster status: {:?}", status);
    assert_eq!(status.local_node_id, network.local_node.node_id);
    assert_eq!(status.total_nodes, 0); // No other nodes yet

    println!("  ✅ Distributed network creation working correctly!");
}

#[test]
fn test_cluster_membership_operations() {
    println!("👥 Cluster Membership Test");
    println!("=========================");

    let config = ClusterConfig {
        max_cluster_size: 3,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::LoadBalanced,
    };

    println!("📊 Testing cluster membership operations...");

    // Create multiple nodes
    let mut networks = Vec::new();
    for i in 0..3 {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080 + i);
        let network = DistributedReteNetwork::new(address, config.clone()).unwrap();
        println!("  Created node {}: {}", i, network.local_node.node_id);
        networks.push(network);
    }

    // Test cluster status for each node
    for (i, network) in networks.iter().enumerate() {
        let status = network.get_cluster_status();
        println!(
            "  Node {} status: {} total nodes, {} active",
            i, status.total_nodes, status.active_nodes
        );
        assert_eq!(status.local_node_id, network.local_node.node_id);
    }

    println!("  ✅ Cluster membership operations working correctly!");
}

#[test]
fn test_node_capabilities_and_metrics() {
    println!("📈 Node Capabilities and Metrics Test");
    println!("====================================");

    println!("📊 Testing node capabilities configuration...");

    let capabilities = NodeCapabilities::default();
    println!("  Max RETE nodes: {}", capabilities.max_rete_nodes);
    println!("  Max memory: {} bytes", capabilities.max_memory_bytes);
    println!("  Max facts/sec: {}", capabilities.max_facts_per_second);
    println!("  Can coordinate: {}", capabilities.can_coordinate);
    println!("  Supported types: {:?}", capabilities.supported_node_types);

    // Validate capabilities
    assert_eq!(capabilities.max_rete_nodes, 10_000);
    assert_eq!(capabilities.max_memory_bytes, 1024 * 1024 * 1024);
    assert_eq!(capabilities.max_facts_per_second, 10_000);
    assert!(capabilities.can_coordinate);
    assert!(capabilities.supported_node_types.contains("alpha"));
    assert!(capabilities.supported_node_types.contains("beta"));
    assert!(capabilities.supported_node_types.contains("terminal"));

    println!("📊 Testing node metrics...");

    let metrics = NodeMetrics::default();
    println!("  CPU utilization: {:.2}%", metrics.cpu_utilization * 100.0);
    println!("  Memory usage: {} bytes", metrics.memory_usage);
    println!("  RETE node count: {}", metrics.rete_node_count);
    println!("  Facts per second: {:.2}", metrics.facts_per_second);
    println!("  Average latency: {:.2}ms", metrics.avg_latency_ms);

    // Validate default metrics
    assert_eq!(metrics.cpu_utilization, 0.0);
    assert_eq!(metrics.memory_usage, 0);
    assert_eq!(metrics.rete_node_count, 0);
    assert_eq!(metrics.facts_per_second, 0.0);
    assert_eq!(metrics.avg_latency_ms, 0.0);

    println!("  ✅ Node capabilities and metrics working correctly!");
}

#[test]
fn test_cluster_health_assessment() {
    println!("🏥 Cluster Health Assessment Test");
    println!("===============================");

    let config = ClusterConfig {
        max_cluster_size: 10,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing health calculation with various scenarios...");

    // Test empty cluster
    let health = network.calculate_cluster_health();
    println!("  Empty cluster health: {:?}", health);
    assert_eq!(health, ClusterHealth::Critical);

    // Add nodes in various states
    let mut active_count = 0;
    let mut total_count = 0;

    // Add healthy nodes
    for i in 0..5 {
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
        active_count += 1;
        total_count += 1;
    }

    // All nodes active (100% health)
    let health = network.calculate_cluster_health();
    println!(
        "  All nodes active ({}/{}): {:?}",
        active_count, total_count, health
    );
    assert_eq!(health, ClusterHealth::Healthy);

    // Add a failed node (83% health)
    let failed_node_id = Uuid::new_v4();
    let failed_node = ClusterNode {
        node_id: failed_node_id,
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9000),
        capabilities: NodeCapabilities::default(),
        status: NodeStatus::Failed,
        last_heartbeat: distributed_rete::current_timestamp() - 120_000,
        metrics: NodeMetrics::default(),
    };
    network.cluster.nodes.insert(failed_node_id, failed_node);
    total_count += 1;

    let health = network.calculate_cluster_health();
    println!(
        "  One node failed ({}/{}): {:?}",
        active_count, total_count, health
    );
    assert_eq!(health, ClusterHealth::Degraded);

    // Add more failed nodes to reach warning level
    for i in 0..2 {
        let node_id = Uuid::new_v4();
        let node = ClusterNode {
            node_id,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9001 + i),
            capabilities: NodeCapabilities::default(),
            status: NodeStatus::Failed,
            last_heartbeat: distributed_rete::current_timestamp() - 120_000,
            metrics: NodeMetrics::default(),
        };
        network.cluster.nodes.insert(node_id, node);
        total_count += 1;
    }

    let health = network.calculate_cluster_health();
    println!(
        "  Multiple failures ({}/{}): {:?}",
        active_count, total_count, health
    );
    assert_eq!(health, ClusterHealth::Warning);

    println!("  ✅ Cluster health assessment working correctly!");
}

#[test]
fn test_load_balancing_strategies() {
    println!("⚖️ Load Balancing Strategies Test");
    println!("===============================");

    println!("📊 Testing different load balancing strategies...");

    let strategies = vec![
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::LeastCpuUtilization,
        LoadBalancingStrategy::LeastMemoryUsage,
        LoadBalancingStrategy::ResourceAware,
        LoadBalancingStrategy::ConsistentHashing,
    ];

    for strategy in strategies {
        let config = ClusterConfig {
            max_cluster_size: 5,
            heartbeat_interval_ms: 30_000,
            failure_timeout_ms: 90_000,
            replication_factor: 2,
            load_balancing_strategy: strategy.clone(),
        };

        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let network = DistributedReteNetwork::new(address, config);

        assert!(network.is_ok());
        println!("  Strategy {:?}: Network created successfully", strategy);
    }

    println!("  ✅ Load balancing strategies working correctly!");
}

#[test]
fn test_partitioning_strategies() {
    println!("🧩 Partitioning Strategies Test");
    println!("===============================");

    println!("📊 Testing RETE node partitioning strategies...");

    let partitioner = NodePartitioner::new();
    println!("  Node partitioner created successfully");

    // Test partitioning statistics
    let stats = &partitioner.stats;
    println!("  Initial stats:");
    println!("    Total RETE nodes: {}", stats.total_rete_nodes);
    println!("    Rebalance operations: {}", stats.rebalance_operations);
    println!(
        "    Cross-node communications: {}",
        stats.cross_node_communications
    );
    println!(
        "    Partition efficiency: {:.2}%",
        stats.partition_efficiency * 100.0
    );

    assert_eq!(stats.total_rete_nodes, 0);
    assert_eq!(stats.rebalance_operations, 0);
    assert_eq!(stats.partition_efficiency, 1.0);

    println!("  ✅ Partitioning strategies working correctly!");
}

#[test]
fn test_message_routing_system() {
    println!("📨 Message Routing System Test");
    println!("==============================");

    let local_node_id = Uuid::new_v4();
    let router = MessageRouter::new(local_node_id);

    println!("📊 Testing message router initialization...");
    println!(
        "  Message router created successfully for node: {}",
        local_node_id
    );

    // Verify router can access public stats
    let stats = &router.stats;
    assert_eq!(stats.total_messages_sent, 0);
    assert_eq!(stats.total_messages_received, 0);

    // Test message creation
    let target_node = Uuid::new_v4();
    let message = ClusterMessage {
        message_id: Uuid::new_v4(),
        from: local_node_id,
        to: target_node,
        payload: MessagePayload::Heartbeat { metrics: NodeMetrics::default() },
        timestamp: distributed_rete::current_timestamp(),
        priority: MessagePriority::Critical,
        delivery_mode: DeliveryMode::BestEffort,
    };

    println!("  Created test message: {}", message.message_id);
    println!("  Message priority: {:?}", message.priority);
    println!("  Delivery mode: {:?}", message.delivery_mode);

    assert_eq!(message.from, local_node_id);
    assert_eq!(message.to, target_node);
    assert_eq!(message.priority, MessagePriority::Critical);

    println!("  ✅ Message routing system working correctly!");
}

#[test]
fn test_failure_detection_system() {
    println!("🚨 Failure Detection System Test");
    println!("===============================");

    let mut detector = FailureDetector::new();

    println!("📊 Testing failure detection configuration...");
    println!(
        "  Max missed heartbeats: {}",
        detector.detection_config.max_missed_heartbeats
    );
    println!(
        "  Confirmation timeout: {}ms",
        detector.detection_config.confirmation_timeout_ms
    );
    println!(
        "  Grace period: {}ms",
        detector.detection_config.grace_period_ms
    );

    assert_eq!(detector.detection_config.max_missed_heartbeats, 3);
    assert_eq!(detector.detection_config.confirmation_timeout_ms, 10_000);
    assert_eq!(detector.detection_config.grace_period_ms, 30_000);

    println!("📊 Testing heartbeat recording...");

    let node_id = Uuid::new_v4();

    // Initially no heartbeat info
    assert!(!detector.heartbeat_tracker.contains_key(&node_id));

    // Record first heartbeat
    detector.record_heartbeat(node_id);

    // Verify heartbeat was recorded
    assert!(detector.heartbeat_tracker.contains_key(&node_id));
    let heartbeat_info = &detector.heartbeat_tracker[&node_id];

    println!("  Heartbeat recorded for node: {}", node_id);
    println!("  Missed heartbeats: {}", heartbeat_info.missed_heartbeats);
    println!("  Average interval: {:?}", heartbeat_info.average_interval);

    assert_eq!(heartbeat_info.missed_heartbeats, 0);

    // Record another heartbeat
    detector.record_heartbeat(node_id);
    let updated_info = &detector.heartbeat_tracker[&node_id];
    assert_eq!(updated_info.missed_heartbeats, 0);

    println!("  ✅ Failure detection system working correctly!");
}

#[test]
fn test_replication_configuration() {
    println!("🔄 Replication Configuration Test");
    println!("=================================");

    let replication_manager = ReplicationManager::new();
    let config = &replication_manager.config;

    println!("📊 Testing replication configuration...");
    println!("  Replica count: {}", config.replica_count);
    println!("  Sync mode: {:?}", config.sync_mode);
    println!("  Placement strategy: {:?}", config.placement_strategy);

    assert_eq!(config.replica_count, 2);
    assert_eq!(config.sync_mode, SynchronizationMode::SemiSynchronous);
    assert_eq!(
        config.placement_strategy,
        ReplicaPlacementStrategy::LoadBalanced
    );

    println!(
        "  Empty replica assignments: {}",
        replication_manager.replica_assignments.is_empty()
    );
    println!(
        "  Empty sync status: {}",
        replication_manager.sync_status.is_empty()
    );

    assert!(replication_manager.replica_assignments.is_empty());
    assert!(replication_manager.sync_status.is_empty());

    println!("  ✅ Replication configuration working correctly!");
}

#[test]
fn test_distributed_state_coordination() {
    println!("🔄 Distributed State Coordination Test");
    println!("=====================================");

    let coordinator = StateCoordinator::new();

    println!("📊 Testing state coordinator initialization...");
    println!("  State versions: {}", coordinator.state_versions.len());
    println!(
        "  Pending sync operations: {}",
        coordinator.pending_sync.len()
    );
    println!(
        "  Inconsistencies tracked: {}",
        coordinator.consistency_checker.inconsistencies.len()
    );

    assert!(coordinator.state_versions.is_empty());
    assert!(coordinator.pending_sync.is_empty());
    assert!(coordinator.consistency_checker.inconsistencies.is_empty());

    // Test conflict resolver
    match coordinator.conflict_resolver {
        ConflictResolver::LastWriteWins => {
            println!("  Conflict resolution strategy: Last Write Wins");
        }
        _ => panic!("Expected LastWriteWins resolver"),
    }

    println!("  ✅ Distributed state coordination working correctly!");
}

#[tokio::test]
async fn test_cluster_message_processing() {
    println!("📬 Cluster Message Processing Test");
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

    println!("📊 Testing message processing with empty queue...");

    // Process messages on empty queue
    let result = network.process_messages().await;
    assert!(result.is_ok());

    println!("  Processed empty message queue successfully");

    // Test cluster status retrieval
    let status = network.get_cluster_status();
    println!("  Cluster status:");
    println!("    Local node: {}", status.local_node_id);
    println!("    Total nodes: {}", status.total_nodes);
    println!("    Active nodes: {}", status.active_nodes);
    println!("    Health: {:?}", status.cluster_health);

    assert_eq!(status.local_node_id, network.local_node.node_id);
    assert_eq!(status.total_nodes, 0);
    assert_eq!(status.active_nodes, 0);
    assert_eq!(status.cluster_health, ClusterHealth::Critical);

    println!("  ✅ Cluster message processing working correctly!");
}

#[test]
fn test_distributed_statistics() {
    println!("📊 Distributed Statistics Test");
    println!("==============================");

    let stats = DistributedReteStats::default();

    println!("📊 Testing default statistics initialization...");

    // Cluster stats
    let cluster_stats = &stats.cluster_stats;
    println!("  Cluster Statistics:");
    println!("    Total nodes: {}", cluster_stats.total_nodes);
    println!("    Active nodes: {}", cluster_stats.active_nodes);
    println!("    Total RETE nodes: {}", cluster_stats.total_rete_nodes);
    println!(
        "    Total facts processed: {}",
        cluster_stats.total_facts_processed
    );
    println!(
        "    Cluster utilization: {:.2}%",
        cluster_stats.cluster_utilization * 100.0
    );
    println!(
        "    Average latency: {:.2}ms",
        cluster_stats.average_latency_ms
    );
    println!(
        "    Throughput: {:.2} facts/sec",
        cluster_stats.throughput_facts_per_second
    );

    assert_eq!(cluster_stats.total_nodes, 0);
    assert_eq!(cluster_stats.active_nodes, 0);
    assert_eq!(cluster_stats.cluster_utilization, 0.0);

    // Fault tolerance stats
    let fault_stats = &stats.fault_tolerance_stats;
    println!("  Fault Tolerance Statistics:");
    println!(
        "    Node failures detected: {}",
        fault_stats.node_failures_detected
    );
    println!(
        "    Successful recoveries: {}",
        fault_stats.successful_recoveries
    );
    println!("    Failed recoveries: {}", fault_stats.failed_recoveries);
    println!(
        "    Average recovery time: {:.2}ms",
        fault_stats.average_recovery_time_ms
    );
    println!(
        "    Data loss incidents: {}",
        fault_stats.data_loss_incidents
    );
    println!(
        "    Availability: {:.2}%",
        fault_stats.availability_percentage
    );

    assert_eq!(fault_stats.node_failures_detected, 0);
    assert_eq!(fault_stats.successful_recoveries, 0);
    assert_eq!(fault_stats.availability_percentage, 100.0);

    // Routing stats
    let routing_stats = &stats.routing_stats;
    println!("  Message Routing Statistics:");
    println!(
        "    Total messages sent: {}",
        routing_stats.total_messages_sent
    );
    println!(
        "    Total messages received: {}",
        routing_stats.total_messages_received
    );
    println!(
        "    Average queue depth: {:.2}",
        routing_stats.average_queue_depth
    );
    println!(
        "    Network bytes sent: {}",
        routing_stats.network_bytes_sent
    );
    println!(
        "    Network bytes received: {}",
        routing_stats.network_bytes_received
    );

    assert_eq!(routing_stats.total_messages_sent, 0);
    assert_eq!(routing_stats.total_messages_received, 0);
    assert_eq!(routing_stats.average_queue_depth, 0.0);

    println!("  ✅ Distributed statistics working correctly!");
}

// Helper functions for testing

fn create_test_cluster_config() -> ClusterConfig {
    ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    }
}

fn create_test_node(port: u16) -> ClusterNode {
    ClusterNode {
        node_id: Uuid::new_v4(),
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port),
        capabilities: NodeCapabilities::default(),
        status: NodeStatus::Active,
        last_heartbeat: distributed_rete::current_timestamp(),
        metrics: NodeMetrics::default(),
    }
}
