//! Basic integration tests for Distributed RETE Network functionality
//!
//! This test validates core distributed RETE capabilities using only the public API.

use bingo_core::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[test]
fn test_distributed_network_creation() {
    println!("🌐 Basic Distributed Network Creation Test");
    println!("==========================================");

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
        LoadBalancingStrategy::LoadBalanced,
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
    let network = DistributedReteNetwork::new(address, config).unwrap();

    println!("📊 Testing health calculation with empty cluster...");

    // Test empty cluster
    let health = network.calculate_cluster_health();
    println!("  Empty cluster health: {:?}", health);
    assert_eq!(health, ClusterHealth::Critical);

    // Test cluster status
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

    println!("  ✅ Cluster health assessment working correctly!");
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

#[test]
fn test_cluster_configuration() {
    println!("⚙️ Cluster Configuration Test");
    println!("=============================");

    let config = ClusterConfig {
        max_cluster_size: 8,
        heartbeat_interval_ms: 45_000,
        failure_timeout_ms: 120_000,
        replication_factor: 3,
        load_balancing_strategy: LoadBalancingStrategy::ConsistentHashing,
    };

    println!("📊 Testing cluster configuration...");
    println!("  Max cluster size: {}", config.max_cluster_size);
    println!("  Heartbeat interval: {}ms", config.heartbeat_interval_ms);
    println!("  Failure timeout: {}ms", config.failure_timeout_ms);
    println!("  Replication factor: {}", config.replication_factor);
    println!("  Load balancing: {:?}", config.load_balancing_strategy);

    // Create network with custom configuration
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let network = DistributedReteNetwork::new(address, config);

    assert!(network.is_ok());
    let network = network.unwrap();

    // Verify the network was created successfully
    assert_eq!(network.local_node.address, address);
    assert_eq!(network.local_node.status, NodeStatus::Joining);

    println!("  ✅ Cluster configuration working correctly!");
}

#[test]
fn test_multiple_network_creation() {
    println!("🌐 Multiple Network Creation Test");
    println!("=================================");

    println!("📊 Creating multiple distributed networks...");

    let mut networks = Vec::new();
    for i in 0..3 {
        let config = ClusterConfig {
            max_cluster_size: 5,
            heartbeat_interval_ms: 30_000,
            failure_timeout_ms: 90_000,
            replication_factor: 2,
            load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
        };

        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080 + i);
        let network = DistributedReteNetwork::new(address, config);

        assert!(network.is_ok());
        let network = network.unwrap();

        println!(
            "  Created network {}: {} on {}",
            i, network.local_node.node_id, network.local_node.address
        );

        // Each network should have unique node ID
        assert_eq!(network.local_node.address, address);
        networks.push(network);
    }

    // Verify all networks have unique node IDs
    for i in 0..networks.len() {
        for j in (i + 1)..networks.len() {
            assert_ne!(
                networks[i].local_node.node_id,
                networks[j].local_node.node_id
            );
        }
    }

    println!("  ✅ Multiple network creation working correctly!");
}
