use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::process::Command;
use crate::error::{KaiokenError, Result};
use super::models::{ChatCompletionRequest, ChatCompletionResponse};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait LlmProvider: Send + Sync {
    fn chat<'a>(&'a self, req: &'a ChatCompletionRequest) -> BoxFuture<'a, Result<ChatCompletionResponse>>;
}

pub struct OpenRouterClient {
    pub api_key: String,
    pub base_url: String,
}

impl OpenRouterClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://openrouter.ai/api/v1/chat/completions".to_string(),
        }
    }
}

impl LlmProvider for OpenRouterClient {
    fn chat<'a>(&'a self, req: &'a ChatCompletionRequest) -> BoxFuture<'a, Result<ChatCompletionResponse>> {
        Box::pin(async move {
            let json_body = serde_json::to_string(req)
                .map_err(|e| KaiokenError::Other(anyhow::anyhow!("Failed to serialize request: {}", e)))?;

            // Execute via native curl with exponential backoff
            let mut backoff = Duration::from_millis(500);
            for attempt in 1..=3 {
                let mut cmd = Command::new("curl");
                cmd.arg("-s")
                   .arg("-S")
                   .arg("-X").arg("POST")
                   .arg(&self.base_url)
                   .arg("-H").arg("Content-Type: application/json")
                   .arg("-H").arg(format!("Authorization: Bearer {}", self.api_key))
                   .arg("-H").arg("HTTP-Referer: https://kaioken.dev")
                   .arg("-H").arg("X-Title: KAIOKEN Agent")
                   .arg("--max-time").arg("60")
                   .arg("--data-binary").arg(&json_body);

                let output = match cmd.output().await {
                    Ok(o) => o,
                    Err(_e) if attempt < 3 => {
                        tokio::time::sleep(backoff).await;
                        backoff *= 2;
                        continue;
                    }
                    Err(e) => return Err(KaiokenError::Other(anyhow::anyhow!("Failed to execute curl: {}", e))),
                };

                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                if output.status.success() {
                    let parsed: ChatCompletionResponse = serde_json::from_str(&stdout)
                        .map_err(|e| KaiokenError::Other(anyhow::anyhow!("Failed to parse OpenRouter response (raw: {}): {}", stdout, e)))?;
                    return Ok(parsed);
                } else if attempt < 3 {
                    tokio::time::sleep(backoff).await;
                    backoff *= 2;
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    return Err(KaiokenError::Other(anyhow::anyhow!("OpenRouter request failed (exit code {}): {} {}", output.status, stdout, stderr)));
                }
            }

            Err(KaiokenError::Other(anyhow::anyhow!("OpenRouter request failed after retries.")))
        })
    }
}

/// MockLlmClient allows deterministic, offline simulation of LLM responses in automated tests.
pub struct MockLlmClient {
    canned_responses: Arc<Mutex<Vec<ChatCompletionResponse>>>,
}

impl MockLlmClient {
    pub fn new() -> Self {
        Self {
            canned_responses: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_responses(responses: Vec<ChatCompletionResponse>) -> Self {
        Self {
            canned_responses: Arc::new(Mutex::new(responses)),
        }
    }

    pub fn push_response(&self, resp: ChatCompletionResponse) {
        self.canned_responses.lock().unwrap().push(resp);
    }
}

impl Default for MockLlmClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmProvider for MockLlmClient {
    fn chat<'a>(&'a self, _req: &'a ChatCompletionRequest) -> BoxFuture<'a, Result<ChatCompletionResponse>> {
        Box::pin(async move {
            let mut list = self.canned_responses.lock().unwrap();
            if !list.is_empty() {
                Ok(list.remove(0))
            } else {
                // Default fallback response
                Ok(ChatCompletionResponse {
                    id: "mock-id".to_string(),
                    choices: vec![
                        super::models::ChatChoice {
                            index: Some(0),
                            message: super::models::ChatMessage::assistant(Some("Task complete.".to_string()), None),
                            finish_reason: Some("stop".to_string()),
                        }
                    ],
                    usage: Some(super::models::UsageInfo {
                        prompt_tokens: 50,
                        completion_tokens: 20,
                        total_tokens: 70,
                    }),
                })
            }
        })
    }
}
