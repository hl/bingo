use axum_test::TestServer;
use bingo_api::create_app;
use serde_json::json;
#[tokio::main]
async fn main() {
    let app = create_app().await.unwrap();
    let server = TestServer::new(app).unwrap();
    let mut facts = Vec::new();
    for i in 0..1000 {
        facts.push(json!({
            "id": format!("fact-{}", i),
            "data": {
                "employee_id": i,
                "hours_worked": 40.0 + (i % 10) as f64,
                "department": "test"
            },
            "created_at": "2024-01-01T09:00:00Z"
        }));
    }
    let payload = json!({
        "facts": facts,
        "rules": [
            {
                "id": "bulk-test-rule",
                "name": "Bulk Test Rule",
                "description": "Rule for testing large fact processing",
                "conditions": [
                    {
                        "type": "simple",
                        "field": "hours_worked",
                        "operator": "GreaterThan",
                        "value": 45.0
                    }
                ],
                "actions": [
                    {
                        "type": "set_field",
                        "field": "overtime",
                        "value": true
                    }
                ],
                "priority": 100,
                "enabled": true,
                "tags": ["bulk", "test"],
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ],
        "response_format": "json",
        "streaming_config": {
            "incremental_processing": true,
            "fact_batch_size": 100,
            "memory_limit_mb": 512
        }
    });
    let response = server.post("/evaluate").json(&payload).await;
    println!("Status: {}", response.status());
    println!("Headers: {:?}", response.headers());
    println!("Body length: {}", response.text().len());
}
