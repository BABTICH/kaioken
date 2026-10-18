pub mod models;
pub mod prompts;
pub mod openrouter;

pub use models::{ChatMessage, ChatCompletionRequest, ChatCompletionResponse};
pub use openrouter::OpenRouterClient;
pub use prompts::{SYSTEM_PROMPT_CODER, SYSTEM_PROMPT_PLANNER, SYSTEM_PROMPT_SUMMARIZER};
