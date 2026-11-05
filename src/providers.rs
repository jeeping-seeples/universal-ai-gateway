use async_stream::try_stream;
use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::config::HostedProviderConfig;
use crate::error::AppError;
use crate::models::{
    ChatChoice, ChatChunkChoice, ChatChunkDelta, ChatCompletionChunk, ChatCompletionRequest,
    ChatCompletionResponse, ChatMessage, ModelCard, ModelListResponse, Usage,
};

pub type ChatStream =
    Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk, AppError>> + Send + 'static>>;

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;

    async fn list_models(&self) -> Result<Vec<ModelCard>, AppError>;

    async fn complete_chat(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AppError>;

    async fn complete_chat_stream(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatStream, AppError>;
}

pub struct MockProvider {
    pub name: String,
    pub supported_models: Vec<String>,
}

impl MockProvider {
    pub fn new(name: impl Into<String>, supported_models: Vec<String>) -> Self {
        Self {
            name: name.into(),
            supported_models,
        }
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new(
            "mock-provider",
            vec!["mock-gpt".to_string(), "mock-fast".to_string()],
        )
    }
}

#[async_trait]
impl Provider for MockProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn list_models(&self) -> Result<Vec<ModelCard>, AppError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let models = self
            .supported_models
            .iter()
            .map(|model_id| ModelCard {
                id: model_id.clone(),
                object: "model".to_string(),
                created: now,
                owned_by: self.name.clone(),
            })
            .collect();

        Ok(models)
    }

    async fn complete_chat(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AppError> {
        if !self.supported_models.contains(&request.model) {
            return Err(AppError::NotFound(format!(
                "Model '{}' is not supported by mock provider '{}'",
                request.model, self.name
            )));
        }

        let last_message = request
            .messages
            .last()
            .map(|m| m.content.as_str())
            .unwrap_or("");

        let reply_content = format!("Mock response to: {}", last_message);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(ChatCompletionResponse {
            id: format!("chatcmpl-mock-{}", now),
            object: "chat.completion".to_string(),
            created: now,
            model: request.model.clone(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: reply_content,
                    name: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 15,
                total_tokens: 25,
            },
        })
    }

    async fn complete_chat_stream(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatStream, AppError> {
        if !self.supported_models.contains(&request.model) {
            return Err(AppError::NotFound(format!(
                "Model '{}' is not supported by mock provider '{}'",
                request.model, self.name
            )));
        }

        let last_message = request
            .messages
            .last()
            .map(|m| m.content.as_str())
            .unwrap_or("")
            .to_string();

        let model = request.model.clone();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let chunk_id = format!("chatcmpl-mock-stream-{}", now);

        let words: Vec<String> = format!("Mock response to: {}", last_message)
            .split_whitespace()
            .map(|s| format!("{} ", s))
            .collect();

        let s = try_stream! {
            yield ChatCompletionChunk {
                id: chunk_id.clone(),
                object: "chat.completion.chunk".to_string(),
                created: now,
                model: model.clone(),
                choices: vec![ChatChunkChoice {
                    index: 0,
                    delta: ChatChunkDelta {
                        role: Some("assistant".to_string()),
                        content: None,
                    },
                    finish_reason: None,
                }],
            };

            for word in words {
                yield ChatCompletionChunk {
                    id: chunk_id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created: now,
                    model: model.clone(),
                    choices: vec![ChatChunkChoice {
                        index: 0,
                        delta: ChatChunkDelta {
                            role: None,
                            content: Some(word),
                        },
                        finish_reason: None,
                    }],
                };
            }

            yield ChatCompletionChunk {
                id: chunk_id.clone(),
                object: "chat.completion.chunk".to_string(),
                created: now,
                model: model.clone(),
                choices: vec![ChatChunkChoice {
                    index: 0,
                    delta: ChatChunkDelta {
                        role: None,
                        content: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
            };
        };

        Ok(Box::pin(s))
    }
}

pub struct OpenAICompatibleProvider {
    name: String,
    base_url: String,
    api_key: Option<String>,
    supported_models: Vec<String>,
    client: reqwest::Client,
    timeout: Duration,
    max_retries: u32,
}

impl OpenAICompatibleProvider {
    pub fn new(
        config: &HostedProviderConfig,
        timeout: Duration,
        max_retries: u32,
    ) -> Result<Self, AppError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(timeout)
            .build()
            .map_err(|e| {
                AppError::Internal(format!("Failed to build reqwest HTTP client: {}", e))
            })?;

        let base_url = config.base_url.trim_end_matches('/').to_string();

        Ok(Self {
            name: config.name.clone(),
            base_url,
            api_key: config
                .api_key
                .as_ref()
                .map(|s| s.expose_secret().to_string()),
            supported_models: config.models.clone(),
            client,
            timeout,
            max_retries,
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }

    async fn execute_with_retries(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<reqwest::Response, AppError> {
        let endpoint = self.url("chat/completions");
        let mut attempts = 0;

        loop {
            attempts += 1;
            let mut req_builder = self.client.post(&endpoint).json(request);

            if let Some(ref key) = self.api_key {
                req_builder = req_builder.header(AUTHORIZATION, format!("Bearer {}", key));
            }

            match req_builder.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    // Retry on transient status: 429 Too Many Requests, 503 Service Unavailable, 504 Gateway Timeout
                    if (status == reqwest::StatusCode::TOO_MANY_REQUESTS
                        || status == reqwest::StatusCode::SERVICE_UNAVAILABLE
                        || status == reqwest::StatusCode::GATEWAY_TIMEOUT)
                        && attempts <= self.max_retries
                    {
                        tracing::warn!(
                            "Provider {} returned status {}, attempt {}/{}. Retrying...",
                            self.name,
                            status,
                            attempts,
                            self.max_retries
                        );
                        tokio::time::sleep(Duration::from_millis(50 * (1 << attempts))).await;
                        continue;
                    }

                    return Ok(resp);
                }
                Err(err) => {
                    if attempts <= self.max_retries && !err.is_timeout() {
                        tracing::warn!(
                            "Network error contacting provider {}, attempt {}/{}. Retrying: {}",
                            self.name,
                            attempts,
                            self.max_retries,
                            err
                        );
                        tokio::time::sleep(Duration::from_millis(50 * (1 << attempts))).await;
                        continue;
                    }

                    if err.is_timeout() {
                        return Err(AppError::ProviderError {
                            provider: self.name.clone(),
                            message: format!("Request timed out after {:?}", self.timeout),
                        });
                    } else {
                        return Err(AppError::ProviderError {
                            provider: self.name.clone(),
                            message: "Failed to send request to upstream provider".to_string(),
                        });
                    }
                }
            }
        }
    }
}

#[async_trait]
impl Provider for OpenAICompatibleProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn list_models(&self) -> Result<Vec<ModelCard>, AppError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let models = self
            .supported_models
            .iter()
            .map(|model_id| ModelCard {
                id: model_id.clone(),
                object: "model".to_string(),
                created: now,
                owned_by: self.name.clone(),
            })
            .collect();

        Ok(models)
    }

    async fn complete_chat(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AppError> {
        if !self.supported_models.is_empty() && !self.supported_models.contains(&request.model) {
            return Err(AppError::NotFound(format!(
                "Model '{}' is not supported by hosted provider '{}'",
                request.model, self.name
            )));
        }

        let resp = self.execute_with_retries(request).await?;

        let status = resp.status();
        if !status.is_success() {
            let error_text = resp
                .text()
                .await
                .unwrap_or_else(|_| "Unknown upstream error".to_string());
            return Err(AppError::ProviderError {
                provider: self.name.clone(),
                message: format!("Upstream HTTP {}: {}", status, error_text),
            });
        }

        let completion =
            resp.json::<ChatCompletionResponse>()
                .await
                .map_err(|err| AppError::ProviderError {
                    provider: self.name.clone(),
                    message: format!("Failed to parse upstream response JSON: {}", err),
                })?;

        Ok(completion)
    }

    async fn complete_chat_stream(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatStream, AppError> {
        if !self.supported_models.is_empty() && !self.supported_models.contains(&request.model) {
            return Err(AppError::NotFound(format!(
                "Model '{}' is not supported by hosted provider '{}'",
                request.model, self.name
            )));
        }

        let resp = self.execute_with_retries(request).await?;

        let status = resp.status();
        if !status.is_success() {
            let error_text = resp
                .text()
                .await
                .unwrap_or_else(|_| "Unknown upstream error".to_string());
            return Err(AppError::ProviderError {
                provider: self.name.clone(),
                message: format!("Upstream HTTP {}: {}", status, error_text),
            });
        }

        let mut byte_stream = resp.bytes_stream();
        let provider_name = self.name.clone();

        let s = try_stream! {
            let mut buffer = String::new();

            while let Some(item) = byte_stream.next().await {
                let chunk_bytes = item.map_err(|e| AppError::ProviderError {
                    provider: provider_name.clone(),
                    message: format!("Streaming read error: {}", e),
                })?;

                buffer.push_str(&String::from_utf8_lossy(&chunk_bytes));

                while let Some(pos) = buffer.find("\n\n") {
                    let event_block = buffer[..pos].to_string();
                    buffer = buffer[pos + 2..].to_string();

                    for line in event_block.lines() {
                        let line = line.trim();
                        if let Some(stripped) = line.strip_prefix("data:") {
                            let data = stripped.trim();
                            if data == "[DONE]" {
                                return;
                            }
                            if !data.is_empty() {
                                match serde_json::from_str::<ChatCompletionChunk>(data) {
                                    Ok(chunk) => yield chunk,
                                    Err(err) => {
                                        tracing::warn!("Failed to parse SSE chunk: {}", err);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };

        Ok(Box::pin(s))
    }
}

#[derive(Clone, Default)]
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn Provider>>,
    model_to_provider: HashMap<String, String>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, provider: Arc<dyn Provider>, models: Vec<String>) {
        let provider_name = provider.name().to_string();
        for model in models {
            self.model_to_provider.insert(model, provider_name.clone());
        }
        self.providers.insert(provider_name, provider);
    }

    pub fn get_provider_for_model(&self, model: &str) -> Option<Arc<dyn Provider>> {
        self.model_to_provider
            .get(model)
            .and_then(|provider_name| self.providers.get(provider_name).cloned())
    }

    pub fn get_provider_by_name(&self, name: &str) -> Option<Arc<dyn Provider>> {
        self.providers.get(name).cloned()
    }

    pub async fn list_all_models(&self) -> Result<ModelListResponse, AppError> {
        let mut all_models = Vec::new();
        for provider in self.providers.values() {
            let models = provider.list_models().await?;
            all_models.extend(models);
        }
        all_models.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(ModelListResponse {
            object: "list".to_string(),
            data: all_models,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_list_models() {
        let provider = MockProvider::default();
        let models = provider.list_models().await.unwrap();
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].id, "mock-gpt");
    }

    #[tokio::test]
    async fn test_mock_provider_chat_completion() {
        let provider = MockProvider::default();
        let req = ChatCompletionRequest {
            model: "mock-gpt".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello mock!".to_string(),
                name: None,
            }],
            temperature: None,
            max_tokens: None,
            stream: None,
        };

        let res = provider.complete_chat(&req).await.unwrap();
        assert_eq!(res.model, "mock-gpt");
        assert_eq!(res.choices.len(), 1);
        assert_eq!(
            res.choices[0].message.content,
            "Mock response to: Hello mock!"
        );
    }

    #[tokio::test]
    async fn test_mock_provider_chat_stream() {
        let provider = MockProvider::default();
        let req = ChatCompletionRequest {
            model: "mock-gpt".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Stream test".to_string(),
                name: None,
            }],
            temperature: None,
            max_tokens: None,
            stream: Some(true),
        };

        let mut stream = provider.complete_chat_stream(&req).await.unwrap();
        let mut chunks = Vec::new();
        while let Some(chunk_res) = stream.next().await {
            chunks.push(chunk_res.unwrap());
        }

        assert!(chunks.len() >= 3);
        assert_eq!(
            chunks[0].choices[0].delta.role.as_deref(),
            Some("assistant")
        );
        assert_eq!(
            chunks.last().unwrap().choices[0].finish_reason.as_deref(),
            Some("stop")
        );
    }

    #[tokio::test]
    async fn test_provider_registry() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider::default());
        registry.register(
            provider,
            vec!["mock-gpt".to_string(), "mock-fast".to_string()],
        );

        let found = registry.get_provider_for_model("mock-gpt");
        assert!(found.is_some());

        let not_found = registry.get_provider_for_model("unknown");
        assert!(not_found.is_none());

        let model_list = registry.list_all_models().await.unwrap();
        assert_eq!(model_list.data.len(), 2);
    }
}
