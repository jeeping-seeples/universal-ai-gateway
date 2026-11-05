use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{ChatCompletionRequest, ModelListResponse};
use crate::providers::ProviderRegistry;

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

pub async fn chat_completions_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<impl IntoResponse, AppError> {
    if request.messages.is_empty() {
        return Err(AppError::BadRequest(
            "messages array cannot be empty".to_string(),
        ));
    }

    let model = if request.model.trim().is_empty() {
        &state.config.gateway.default_model
    } else {
        &request.model
    };

    let provider = state
        .registry
        .get_provider_for_model(model)
        .ok_or_else(|| AppError::NotFound(format!("Model '{}' not found", model)))?;

    let response = provider.complete_chat(&request).await?;
    Ok((StatusCode::OK, Json(response)))
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/models", get(list_models_handler))
        .route("/v1/chat/completions", post(chat_completions_handler))
        .with_state(state)
}
