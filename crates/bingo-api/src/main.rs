use std::env;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing with structured logging
    tracing_subscriber::fmt()
        .with_env_filter("bingo=debug,info")
        .with_target(false)
        .json()
        .init();

    info!(
        version = "0.1.0",
        edition = "2024",
        "Starting Bingo RETE Rules Engine"
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
                eprintln!("Unknown command: {}", cmd);
                print_help();
                return Ok(());
            }
        }
    }

    // Start web server with environment-based configuration
    start_server().await
}

async fn explain_command() -> anyhow::Result<()> {
    println!("Bingo RETE Rules Engine - Explain Mode");
    println!("This engine processes facts through a RETE network for efficient rule evaluation.");
    println!("\nFeatures:");
    println!("  - High-performance RETE algorithm with modern optimizations");
    println!("  - Support for 3M+ facts with sub-second processing");
    println!("  - Hybrid rules: Built-in + JSON API + Calculator DSL");
    println!("  - Generic business rule processing");
    println!("  - Built with Rust 2024 edition");
    Ok(())
}

fn print_help() {
    println!("Bingo RETE Rules Engine v0.1.0");
    println!("Usage: bingo [COMMAND]");
    println!();
    println!("Commands:");
    println!("  explain    Show explanation of the rules engine");
    println!("  --help     Show this help message");
    println!();
    println!("If no command is provided, starts the web server.");
}

async fn start_server() -> anyhow::Result<()> {
    // Environment-based configuration
    let host = env::var("BINGO_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("BINGO_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    info!(?host, ?port, "Configuring web server");

    let app = bingo_api::create_app()?;
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("🚀 Bingo RETE server starting on {}", addr);
    info!("Web server started successfully");

    axum::serve(listener, app).await?;

    Ok(())
}
