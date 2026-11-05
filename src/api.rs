use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use futures_util::stream::StreamExt;
use serde::Serialize;
use std::convert::Infallible;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::error::AppError;
use crate::middleware::request_id_middleware;
use crate::models::{ChatCompletionRequest, ModelListResponse};
use crate::providers::{Provider, ProviderRegistry};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub registry: ProviderRegistry,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

pub async fn health_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok",
            version: env!("CARGO_PKG_VERSION"),
        }),
    )
}

pub async fn list_models_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ModelListResponse>, AppError> {
    let models = state.registry.list_all_models().await?;
    Ok(Json(models))
}

fn resolve_provider_and_model(
    state: &AppState,
    request: &ChatCompletionRequest,
) -> Result<(Arc<dyn Provider>, String), AppError> {
    let requested_model = if request.model.trim().is_empty() {
        state.config.gateway.default_model.as_str()
    } else {
        request.model.as_str()
    };

    if let Some(provider) = state.registry.get_provider_for_model(requested_model) {
        return Ok((provider, requested_model.to_string()));
    }

    if let Some(ref fallback) = state.config.gateway.fallback_model {
        if let Some(provider) = state.registry.get_provider_for_model(fallback) {
            tracing::info!(
                "Model '{}' not found, falling back to model '{}'",
                requested_model,
                fallback
            );
            return Ok((provider, fallback.clone()));
        }
    }

    Err(AppError::NotFound(format!(
        "Model '{}' not found",
        requested_model
    )))
}

pub async fn chat_completions_handler(
    State(state): State<Arc<AppState>>,
    Json(mut request): Json<ChatCompletionRequest>,
) -> Result<Response, AppError> {
    if request.messages.is_empty() {
        return Err(AppError::BadRequest(
            "messages array cannot be empty".to_string(),
        ));
    }

    let (provider, effective_model) = resolve_provider_and_model(&state, &request)?;
    request.model = effective_model;

    let is_streaming = request.stream.unwrap_or(false);

    if is_streaming {
        let stream = provider.complete_chat_stream(&request).await?;

        let sse_stream = async_stream::stream! {
            let mut inner = stream;
            while let Some(chunk_res) = inner.next().await {
                match chunk_res {
                    Ok(chunk) => {
                        if let Ok(json_str) = serde_json::to_string(&chunk) {
                            yield Ok::<Event, Infallible>(Event::default().data(json_str));
                        }
                    }
                    Err(err) => {
                        tracing::error!("Error in chat stream: {}", err);
                        yield Ok::<Event, Infallible>(Event::default().data(format!("[ERROR: {}]", err)));
                        break;
                    }
                }
            }
            yield Ok::<Event, Infallible>(Event::default().data("[DONE]"));
        };

        let response = Sse::new(sse_stream)
            .keep_alive(KeepAlive::default())
            .into_response();

        return Ok(response);
    }

    let response = provider.complete_chat(&request).await?;
    Ok((StatusCode::OK, Json(response)).into_response())
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/models", get(list_models_handler))
        .route("/v1/chat/completions", post(chat_completions_handler))
        .layer(middleware::from_fn(request_id_middleware))
        .with_state(state)
}
