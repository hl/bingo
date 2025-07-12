use askama::Template;
use axum::{Router, routing::get};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // build our application with a few routes
    let app = Router::new()
        .route("/", get(dashboard))
        .route("/rules", get(rules))
        .route("/health", get(|| async { "OK" }));

    // run it with hyper on localhost:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

struct Metrics {
    total_facts: u64,
    rules_loaded: usize,
    processing_time: f64,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    metrics: Metrics,
}

async fn dashboard() -> DashboardTemplate {
    let metrics = Metrics { total_facts: 1_234_567, rules_loaded: 42, processing_time: 0.123 };
    DashboardTemplate { metrics }
}

#[derive(Template)]
#[template(path = "rules.html")]
struct RulesTemplate;

async fn rules() -> RulesTemplate {
    RulesTemplate
}
