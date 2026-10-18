use crate::error::Result;
use super::models::{ChatCompletionRequest, ChatCompletionResponse};

pub struct OpenRouterClient {
    pub api_key: String,
    pub base_url: String,
}

impl OpenRouterClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
        }
    }

    pub async fn complete(&self, req: &ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        // Stub for Phase 10: async-openai or reqwest HTTP call
        Ok(ChatCompletionResponse {
            id: "stub-id".to_string(),
            content: format!("OpenRouter response stub for model {}", req.model),
            prompt_tokens: 10,
            completion_tokens: 15,
        })
    }
}
