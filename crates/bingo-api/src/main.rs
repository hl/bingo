#![deny(warnings)]
#![allow(missing_docs)]

use std::env;
use std::sync::Arc;

use tonic::transport::Server;
use tracing::info;

use bingo_api::AppState;
use bingo_api::generated::rules_engine_service_server::RulesEngineServiceServer;
use bingo_api::grpc::service::RulesEngineServiceImpl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize distributed tracing
    let tracing_config = bingo_api::tracing_setup::TracingConfig::from_environment();
    bingo_api::tracing_setup::init_tracing(tracing_config)?;

    info!(
        version = "0.1.0",
        edition = "2024",
        "Starting Bingo RETE Rules Engine (gRPC)"
    );

    // Modern command line argument handling
    let args = env::args().collect::<Vec<_>>();
    if let Some(cmd) = args.get(1) {
        match cmd.as_str() {
            "explain" => {
                explain_command().await?;
                return Ok(());
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown command: {cmd}");
                print_help();
                return Ok(());
            }
        }
    }

    // Start gRPC server
    start_grpc_server().await
}

async fn explain_command() -> anyhow::Result<()> {
    println!("Bingo RETE Rules Engine - gRPC Mode");
    println!("This engine processes facts through a RETE network for efficient rule evaluation.");
    println!("\nFeatures:");
    println!("  - High-performance RETE algorithm with streaming gRPC interface");
    println!("  - Support for 3M+ facts with O(1) memory usage");
    println!("  - Two-phase processing: compile rules, then stream facts");
    println!("  - Concurrent client support with session isolation");
    println!("  - Built with Rust 2024 edition");
    Ok(())
}

fn print_help() {
    println!("Bingo RETE Rules Engine v0.1.0 (gRPC)");
    println!("Usage: bingo [COMMAND]");
    println!();
    println!("Commands:");
    println!("  explain    Show explanation of the rules engine");
    println!("  --help     Show this help message");
    println!();
    println!("If no command is provided, starts the gRPC server.");
}

async fn start_grpc_server() -> anyhow::Result<()> {
    // Environment-based configuration for gRPC
    let grpc_addr = env::var("GRPC_LISTEN_ADDRESS").unwrap_or_else(|_| "0.0.0.0:50051".to_string());

    info!(?grpc_addr, "Configuring gRPC server");

    // Initialize application state
    let app_state = AppState::new().await?;

    // Create gRPC service
    let grpc_service = RulesEngineServiceImpl::new(Arc::new(app_state));

    let server = Server::builder()
        .add_service(RulesEngineServiceServer::new(grpc_service))
        .serve(grpc_addr.parse()?);

    println!("🚀 Bingo RETE gRPC server starting on {grpc_addr}");
    info!("gRPC server started successfully");

    server.await?;

    // Gracefully shutdown tracing
    bingo_api::tracing_setup::shutdown_tracing();

    Ok(())
}
