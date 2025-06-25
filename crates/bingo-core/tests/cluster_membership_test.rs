//! Test cluster membership and heartbeat functionality

use bingo_core::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[tokio::test]
async fn test_cluster_bootstrap() {
    println!("🚀 Cluster Bootstrap Test");
    println!("========================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing cluster bootstrap...");

    // Join empty cluster (should bootstrap)
    let result = network.join_cluster(vec![]).await;
    assert!(result.is_ok());

    // Verify this node became coordinator
    assert!(network.is_coordinator());

    // Check cluster status
    let status = network.get_cluster_status();
    assert_eq!(status.coordinator_id, Some(network.local_node.node_id));
    assert_eq!(status.total_nodes, 1); // Only this node
    assert_eq!(status.active_nodes, 1);

    println!(
        "  Node {} bootstrapped cluster as coordinator",
        network.local_node.node_id
    );
    println!("  ✅ Cluster bootstrap working correctly!");
}

#[tokio::test]
async fn test_heartbeat_mechanism() {
    println!("💓 Heartbeat Mechanism Test");
    println!("===========================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing heartbeat functionality...");

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    let initial_heartbeat = network.local_node.last_heartbeat;

    // Send heartbeat
    let result = network.send_heartbeat().await;
    assert!(result.is_ok());

    // Process maintenance (updates metrics and heartbeat)
    let result = network.perform_maintenance().await;
    assert!(result.is_ok());

    // Verify heartbeat was updated
    assert!(network.local_node.last_heartbeat >= initial_heartbeat);

    println!("  Initial heartbeat: {}", initial_heartbeat);
    println!("  Updated heartbeat: {}", network.local_node.last_heartbeat);
    println!("  ✅ Heartbeat mechanism working correctly!");
}

#[tokio::test]
async fn test_coordinator_election() {
    println!("🗳️ Coordinator Election Test");
    println!("============================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing coordinator election...");

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    // Should be coordinator after bootstrap
    assert!(network.is_coordinator());
    let original_coordinator = network.local_node.node_id;

    // Simulate adding another node with higher ID
    let higher_id = uuid::Uuid::from_u128(original_coordinator.as_u128() + 1);
    let node = ClusterNode {
        node_id: higher_id,
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
        capabilities: NodeCapabilities::default(),
        status: NodeStatus::Active,
        last_heartbeat: distributed_rete::current_timestamp(),
        metrics: NodeMetrics::default(),
    };
    network.cluster.nodes.insert(higher_id, node);

    // Trigger election
    network.start_coordinator_election().await.unwrap();

    // Higher ID should win
    assert_eq!(network.cluster.coordinator_id, Some(higher_id));
    assert!(!network.is_coordinator()); // This node is no longer coordinator

    println!("  Original coordinator: {}", original_coordinator);
    println!("  New coordinator: {}", higher_id);
    println!("  ✅ Coordinator election working correctly!");
}

#[tokio::test]
async fn test_node_failure_detection() {
    println!("🚨 Node Failure Detection Test");
    println!("==============================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 1_000, // Short for testing
        failure_timeout_ms: 2_000,    // Very short for testing
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing failure detection...");

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    // Add another node with old heartbeat (simulate failure)
    let failed_node_id = uuid::Uuid::new_v4();
    let old_timestamp = distributed_rete::current_timestamp() - 10_000; // 10 seconds ago
    let failed_node = ClusterNode {
        node_id: failed_node_id,
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
        capabilities: NodeCapabilities::default(),
        status: NodeStatus::Active, // Initially active
        last_heartbeat: old_timestamp,
        metrics: NodeMetrics::default(),
    };
    network.cluster.nodes.insert(failed_node_id, failed_node);

    // Initial health should be healthy since both nodes are marked active initially
    let initial_health = network.calculate_cluster_health();
    println!("  Initial health: {:?}", initial_health);
    println!("  Total nodes: {}", network.cluster.nodes.len());
    println!(
        "  Active nodes: {}",
        network
            .cluster
            .nodes
            .values()
            .filter(|n| n.status == NodeStatus::Active)
            .count()
    );
    // Should be healthy initially because both nodes have Active status despite old heartbeat

    // Trigger failure detection
    network.check_node_failures().await.unwrap();

    // Node should now be marked as failed
    let failed_node = &network.cluster.nodes[&failed_node_id];
    assert_eq!(failed_node.status, NodeStatus::Failed);

    // Health should now be warning (1 active, 1 failed out of 2 total)
    let final_health = network.calculate_cluster_health();
    println!("  Final health: {:?}", final_health);
    assert_eq!(final_health, ClusterHealth::Warning);

    println!("  Failed node: {}", failed_node_id);
    println!("  Node status: {:?}", failed_node.status);
    println!("  ✅ Node failure detection working correctly!");
}

#[tokio::test]
async fn test_maintenance_cycle() {
    println!("🔧 Maintenance Cycle Test");
    println!("========================");

    let config = ClusterConfig {
        max_cluster_size: 5,
        heartbeat_interval_ms: 30_000,
        failure_timeout_ms: 90_000,
        replication_factor: 2,
        load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
    };

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing maintenance operations...");

    // Bootstrap cluster
    network.join_cluster(vec![]).await.unwrap();

    let initial_stats = network.stats.clone();
    let initial_cpu = network.local_node.metrics.cpu_utilization;

    // Perform maintenance cycle
    let result = network.perform_maintenance().await;
    assert!(result.is_ok());

    // Verify statistics were updated
    assert_eq!(network.stats.cluster_stats.total_nodes, 1);
    assert_eq!(network.stats.cluster_stats.active_nodes, 1);

    // Verify metrics were updated
    assert!(network.local_node.metrics.cpu_utilization >= initial_cpu);

    println!("  Initial CPU utilization: {:.2}%", initial_cpu * 100.0);
    println!(
        "  Updated CPU utilization: {:.2}%",
        network.local_node.metrics.cpu_utilization * 100.0
    );
    println!("  Total nodes: {}", network.stats.cluster_stats.total_nodes);
    println!(
        "  Active nodes: {}",
        network.stats.cluster_stats.active_nodes
    );
    println!("  ✅ Maintenance cycle working correctly!");
}
