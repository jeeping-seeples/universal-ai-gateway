use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::AppError;
use crate::models::{
    ChatChoice, ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ModelCard,
    ModelListResponse, Usage,
};

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;

    async fn list_models(&self) -> Result<Vec<ModelCard>, AppError>;

    async fn complete_chat(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AppError>;
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
