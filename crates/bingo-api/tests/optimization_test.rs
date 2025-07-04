use axum_test::TestServer;
use bingo_api::create_app;
use serde_json::json;

#[tokio::test]
async fn test_ruleset_registration_and_caching() {
    let app = create_app().await.unwrap();
    let server = TestServer::new(app).unwrap();

    // Test ruleset registration
    let response = server
        .post("/rulesets")
        .json(&json!({
            "ruleset_id": "test_cache",
            "ttl_seconds": 300,
            "description": "Test caching ruleset",
            "rules": [{
                "id": "cache_rule",
                "name": "Cache Test Rule",
                "description": "Test rule for caching",
                "conditions": [{
                    "type": "simple",
                    "field": "test_field",
                    "operator": "equal",
                    "value": "test_value"
                }],
                "actions": [{
                    "type": "log",
                    "level": "info",
                    "message": "Cache test rule fired"
                }],
                "priority": 100,
                "enabled": true,
                "tags": ["test"],
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }]
        }))
        .await;

    assert_eq!(response.status_code(), 201);
    let body: serde_json::Value = response.json();
    assert_eq!(body["ruleset_id"], "test_cache");
    assert_eq!(body["compiled"], true);

    // Test traditional evaluation (with rules in request)
    let response = server
        .post("/evaluate")
        .json(&json!({
            "rules": [{
                "id": "direct_rule",
                "name": "Direct Rule",
                "description": "Direct test rule",
                "conditions": [{
                    "type": "simple",
                    "field": "test_field",
                    "operator": "equal",
                    "value": "test_value"
                }],
                "actions": [{
                    "type": "log",
                    "level": "info",
                    "message": "Direct rule fired"
                }],
                "priority": 100,
                "enabled": true,
                "tags": ["test"],
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }],
            "facts": [{
                "id": "test_fact",
                "data": {
                    "test_field": "test_value"
                },
                "created_at": "2024-01-01T00:00:00Z"
            }]
        }))
        .await;

    assert_eq!(response.status_code(), 200);
    let body: serde_json::Value = response.json();
    assert_eq!(body["rules_fired"], 1);

    // Test cached evaluation (with ruleset_id)
    let response = server
        .post("/evaluate")
        .json(&json!({
            "rules": null,
            "ruleset_id": "test_cache",
            "facts": [{
                "id": "test_fact",
                "data": {
                    "test_field": "test_value"
                },
                "created_at": "2024-01-01T00:00:00Z"
            }]
        }))
        .await;

    assert_eq!(response.status_code(), 200);
    let body: serde_json::Value = response.json();
    assert_eq!(body["rules_fired"], 1);

    // Check cache stats
    let response = server.get("/cache/stats").await;
    assert_eq!(response.status_code(), 200);
    let body: serde_json::Value = response.json();
    assert_eq!(body["ruleset_cache"]["total_entries"], 1);
    assert_eq!(body["ruleset_cache"]["cache_hits"], 1); // Ruleset cache hit is 1 because it's retrieved from ruleset cache
    assert_eq!(body["engine_cache"]["total_entries"], 1);
    assert_eq!(body["engine_cache"]["cache_hits"], 0);
}
