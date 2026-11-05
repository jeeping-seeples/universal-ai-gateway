use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use universal_ai_gateway::{
    api::{create_router, AppState},
    config::AppConfig,
    providers::{MockProvider, OpenAICompatibleProvider, ProviderRegistry},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "universal_ai_gateway=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config_path = std::env::var("GATEWAY_CONFIG_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("gateway.toml"));

    let config = AppConfig::load_from_file_or_default(Some(config_path.as_path()))?;
    let addr = config.socket_addr()?;

    let mut registry = ProviderRegistry::new();

    // Register mock provider for default testing/mock models
    let mock_provider = Arc::new(MockProvider::default());
    registry.register(
        mock_provider,
        vec!["mock-gpt".to_string(), "mock-fast".to_string()],
    );

    // Register hosted provider if enabled or configured
    if let Some(ref hosted_config) = config.hosted_provider {
        if hosted_config.enabled || hosted_config.api_key.is_some() {
            let timeout = Duration::from_secs(config.gateway.request_timeout_secs);
            let hosted_provider = Arc::new(OpenAICompatibleProvider::new(hosted_config, timeout)?);
            tracing::info!(
                "Registering hosted OpenAI-compatible provider '{}' for models {:?}",
                hosted_config.name,
                hosted_config.models
            );
            registry.register(hosted_provider, hosted_config.models.clone());
        }
    }

    let state = Arc::new(AppState { config, registry });
    let router = create_router(state);

    tracing::info!("Universal AI Gateway starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
