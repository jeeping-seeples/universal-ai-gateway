use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

use universal_ai_gateway::{
    api::{create_router, AppState},
    config::AppConfig,
    providers::{MockProvider, ProviderRegistry},
};

fn setup_test_app() -> axum::Router {
    let config = AppConfig::default();
    let mut registry = ProviderRegistry::new();
    let mock_provider = Arc::new(MockProvider::default());
    registry.register(
        mock_provider,
        vec!["mock-gpt".to_string(), "mock-fast".to_string()],
    );

    let state = Arc::new(AppState { config, registry });
    create_router(state)
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = setup_test_app();

    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["status"], "ok");
    assert_eq!(body_json["version"], env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn test_list_models_endpoint() {
    let app = setup_test_app();

    let request = Request::builder()
        .uri("/v1/models")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["object"], "list");
    let data = body_json["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0]["id"], "mock-fast");
    assert_eq!(data[1]["id"], "mock-gpt");
}

#[tokio::test]
async fn test_chat_completions_endpoint_success() {
    let app = setup_test_app();

    let req_payload = json!({
        "model": "mock-gpt",
        "messages": [
            {
                "role": "user",
                "content": "Hello, how are you?"
            }
        ]
    });

    let request = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(req_payload.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["object"], "chat.completion");
    assert_eq!(body_json["model"], "mock-gpt");
    assert_eq!(
        body_json["choices"][0]["message"]["content"],
        "Mock response to: Hello, how are you?"
    );
    assert_eq!(body_json["choices"][0]["finish_reason"], "stop");
}

#[tokio::test]
async fn test_chat_completions_model_not_found() {
    let app = setup_test_app();

    let req_payload = json!({
        "model": "non-existent-model",
        "messages": [
            {
                "role": "user",
                "content": "Test"
            }
        ]
    });

    let request = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(req_payload.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["error"]["type"], "not_found_error");
    assert!(body_json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("not found"));
}

#[tokio::test]
async fn test_chat_completions_empty_messages() {
    let app = setup_test_app();

    let req_payload = json!({
        "model": "mock-gpt",
        "messages": []
    });

    let request = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(req_payload.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["error"]["type"], "invalid_request_error");
    assert_eq!(
        body_json["error"]["message"],
        "messages array cannot be empty"
    );
}
