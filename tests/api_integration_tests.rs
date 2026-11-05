use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use universal_ai_gateway::{
    api::{create_router, AppState},
    config::{AppConfig, HostedProviderConfig, SecretString},
    models::{ChatCompletionRequest, ChatMessage},
    providers::{MockProvider, OpenAICompatibleProvider, Provider, ProviderRegistry},
};

fn setup_test_app() -> axum::Router {
    let mut config = AppConfig::default();
    config.gateway.fallback_model = Some("mock-gpt".to_string());
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
    assert!(response.headers().get("x-request-id").is_some());

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["status"], "ok");
    assert_eq!(body_json["version"], env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn test_request_id_propagation() {
    let app = setup_test_app();

    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .header("x-request-id", "custom-trace-id-123")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("x-request-id").unwrap(),
        "custom-trace-id-123"
    );
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
    assert!(response.headers().get("x-request-id").is_some());

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
async fn test_chat_completions_endpoint_streaming() {
    let app = setup_test_app();

    let req_payload = json!({
        "model": "mock-gpt",
        "messages": [
            {
                "role": "user",
                "content": "Hello stream"
            }
        ],
        "stream": true
    });

    let request = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(req_payload.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap(),
        "text/event-stream"
    );

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body);

    assert!(body_str.contains("data:"));
    assert!(body_str.contains("[DONE]"));
    assert!(body_str.contains("chat.completion.chunk"));
}

#[tokio::test]
async fn test_chat_completions_fallback_routing() {
    let app = setup_test_app();

    // Request non-existent model; should fall back to mock-gpt
    let req_payload = json!({
        "model": "non-existent-fallback-candidate",
        "messages": [
            {
                "role": "user",
                "content": "Test fallback"
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
    assert_eq!(body_json["model"], "mock-gpt");
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

#[tokio::test]
async fn test_openai_compatible_provider_success() {
    let mock_server = MockServer::start().await;

    let upstream_response = json!({
        "id": "chatcmpl-upstream-123",
        "object": "chat.completion",
        "created": 1700000000,
        "model": "gpt-4o",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello from upstream OpenAI-compatible provider!"
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 12,
            "completion_tokens": 8,
            "total_tokens": 20
        }
    });

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(upstream_response))
        .mount(&mock_server)
        .await;

    let config = HostedProviderConfig {
        name: "test-openai".to_string(),
        base_url: mock_server.uri(),
        api_key: Some(SecretString::new("test-api-key")),
        models: vec!["gpt-4o".to_string()],
        enabled: true,
    };

    let provider =
        OpenAICompatibleProvider::new(&config, Duration::from_secs(5), 2).expect("provider");

    let req = ChatCompletionRequest {
        model: "gpt-4o".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Hi!".to_string(),
            name: None,
        }],
        temperature: None,
        max_tokens: None,
        stream: None,
    };

    let res = provider.complete_chat(&req).await.unwrap();
    assert_eq!(res.id, "chatcmpl-upstream-123");
    assert_eq!(
        res.choices[0].message.content,
        "Hello from upstream OpenAI-compatible provider!"
    );
}

#[tokio::test]
async fn test_openai_compatible_provider_bounded_retries_recovers() {
    let mock_server = MockServer::start().await;

    // First attempt returns 429
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(429).set_body_string("Rate limited"))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Second attempt recovers with 200
    let success_json = json!({
        "id": "chatcmpl-recovered",
        "object": "chat.completion",
        "created": 1700000000,
        "model": "gpt-4o",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Recovered after retry"
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 5,
            "completion_tokens": 5,
            "total_tokens": 10
        }
    });

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(success_json))
        .mount(&mock_server)
        .await;

    let config = HostedProviderConfig {
        name: "retry-provider".to_string(),
        base_url: mock_server.uri(),
        api_key: None,
        models: vec!["gpt-4o".to_string()],
        enabled: true,
    };

    let provider =
        OpenAICompatibleProvider::new(&config, Duration::from_secs(5), 2).expect("provider");

    let req = ChatCompletionRequest {
        model: "gpt-4o".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Retry me".to_string(),
            name: None,
        }],
        temperature: None,
        max_tokens: None,
        stream: None,
    };

    let res = provider.complete_chat(&req).await.unwrap();
    assert_eq!(res.id, "chatcmpl-recovered");
    assert_eq!(res.choices[0].message.content, "Recovered after retry");
}

#[tokio::test]
async fn test_openai_compatible_provider_timeout() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(300))
                .set_body_string("Delayed"),
        )
        .mount(&mock_server)
        .await;

    let config = HostedProviderConfig {
        name: "slow-provider".to_string(),
        base_url: mock_server.uri(),
        api_key: None,
        models: vec!["gpt-4o".to_string()],
        enabled: true,
    };

    let provider =
        OpenAICompatibleProvider::new(&config, Duration::from_millis(50), 0).expect("provider");

    let req = ChatCompletionRequest {
        model: "gpt-4o".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Hi!".to_string(),
            name: None,
        }],
        temperature: None,
        max_tokens: None,
        stream: None,
    };

    let err = provider.complete_chat(&req).await.unwrap_err();
    assert!(err.to_string().contains("timed out"));
}
