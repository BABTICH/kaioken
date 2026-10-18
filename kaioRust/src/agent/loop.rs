use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use crate::config::Config;
use crate::error::Result;
use crate::llm::{
    ChatCompletionRequest, ChatMessage, FunctionDefinition,
    LlmProvider, ToolDefinition,
};
use crate::memory::{MemoryEngine, ToolCallRecord};
use crate::safety::audit::AuditLogger;
use crate::safety::command_guard::CommandGuard;
use crate::agent::tools::{ToolContext, ToolRegistry, ToolResult};

#[derive(Debug, Clone)]
pub struct AgentStepNotification {
    pub step: usize,
    pub max_steps: usize,
    pub tool_name: String,
    pub arguments_summary: String,
    pub duration_ms: u128,
    pub success: bool,
    pub output_snippet: String,
}

#[derive(Debug, Clone)]
pub struct AgentExecutionSummary {
    pub run_id: i64,
    pub task: String,
    pub status: String,
    pub total_steps: usize,
    pub tool_calls_count: usize,
    pub total_tokens: usize,
    pub final_response: String,
}

pub struct AgentLoop {
    tools: ToolRegistry,
    llm: Arc<dyn LlmProvider>,
    memory: Option<Arc<MemoryEngine>>,
    repo_root: PathBuf,
    config: Config,
    max_steps: usize,
    max_tool_calls: usize,
}

impl AgentLoop {
    pub fn new(
        tools: ToolRegistry,
        llm: Arc<dyn LlmProvider>,
        memory: Option<Arc<MemoryEngine>>,
        repo_root: PathBuf,
        config: Config,
    ) -> Self {
        Self {
            tools,
            llm,
            memory,
            repo_root,
            config,
            max_steps: 20,
            max_tool_calls: 100,
        }
    }

    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    pub fn with_max_tool_calls(mut self, max_tool_calls: usize) -> Self {
        self.max_tool_calls = max_tool_calls;
        self
    }

    pub async fn execute_task(
        &self,
        task: &str,
        listener: Option<Arc<dyn Fn(&AgentStepNotification) + Send + Sync>>,
    ) -> Result<AgentExecutionSummary> {
        let command_guard = Arc::new(CommandGuard::new(&self.config.permissions.blocked_commands));
        let audit_logger = AuditLogger::new(Some(&self.repo_root));

        let tool_context = ToolContext {
            repo_root: self.repo_root.clone(),
            command_guard,
            audit_logger,
        };

        // 1. Initialize agent run in SQLite
        let model_name = &self.config.llm.coder_model;
        let run_id = if let Some(mem) = &self.memory {
            mem.start_agent_run(task, model_name)?
        } else {
            0
        };

        // 2. Build initial conversation prompt
        let system_prompt = format!(
            "You are KAIOKEN Agent, an autonomous software engineering assistant.\n\
             Workspace Root: {}\n\
             You solve programming tasks by thinking step-by-step and using available tools.\n\
             Guidelines:\n\
             - Use read_file to inspect code before modifying it.\n\
             - Use edit_file for precise search-and-replace edits.\n\
             - Use create_file to author new files.\n\
             - Use bash to run test suites or compile checks.\n\
             - When your implementation is complete and verified, output a concise concluding summary without calling further tools.",
            self.repo_root.display()
        );

        let mut messages = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(task),
        ];

        let mut total_tokens = 0;
        let mut tool_calls_count = 0;
        let mut step = 0;
        let mut final_response = String::new();
        let mut final_status = "completed";

        // Convert registry tools to tool definitions
        let tool_definitions: Vec<ToolDefinition> = self.tools.list().iter().map(|t| {
            ToolDefinition {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: t.name().to_string(),
                    description: t.description().to_string(),
                    parameters: t.schema(),
                },
            }
        }).collect();

        // 3. ReAct Autonomous Loop
        while step < self.max_steps {
            step += 1;

            let req = ChatCompletionRequest {
                model: model_name.clone(),
                messages: messages.clone(),
                tools: Some(tool_definitions.clone()),
                temperature: Some(self.config.llm.temperature),
                max_tokens: Some(self.config.llm.max_tokens as usize),
            };

            let resp = match self.llm.chat(&req).await {
                Ok(r) => r,
                Err(e) => {
                    final_status = "failed";
                    final_response = format!("LLM request failed at step {}: {}", step, e);
                    break;
                }
            };

            if let Some(usage) = &resp.usage {
                total_tokens += usage.total_tokens;
            }

            let choice = match resp.choices.first() {
                Some(c) => c,
                None => {
                    final_status = "failed";
                    final_response = format!("Received empty response choices from LLM at step {}", step);
                    break;
                }
            };

            let assistant_msg = &choice.message;

            // Check if assistant called any tools
            if let Some(tool_calls) = &assistant_msg.tool_calls {
                if tool_calls.is_empty() {
                    // No tool calls -> task complete
                    final_response = assistant_msg.content.clone().unwrap_or_default();
                    break;
                }

                // Append assistant message with tool calls to conversation
                messages.push(assistant_msg.clone());

                // Execute each tool call
                for tc in tool_calls {
                    tool_calls_count += 1;
                    let tool_start = Instant::now();
                    let tool_name = &tc.function.name;
                    let tool_args = &tc.function.arguments;

                    let tool_res = if let Some(tool) = self.tools.find(tool_name) {
                        match tool.execute(tool_args, &tool_context).await {
                            Ok(res) => res,
                            Err(e) => ToolResult::error(format!("Tool execution error: {}", e)),
                        }
                    } else {
                        ToolResult::error(format!("Tool '{}' is not registered.", tool_name))
                    };

                    let duration_ms = tool_start.elapsed().as_millis();

                    // Record tool execution in SQLite
                    if let Some(mem) = &self.memory {
                        let _ = mem.record_tool_call(&ToolCallRecord {
                            id: None,
                            agent_run_id: run_id,
                            tool_name: tool_name.clone(),
                            input_json: tool_args.clone(),
                            output_summary: Some(tool_res.output.chars().take(200).collect()),
                            status: if tool_res.success { "success".to_string() } else { "error".to_string() },
                            duration_ms: duration_ms as i64,
                            error: if !tool_res.success { Some(tool_res.output.clone()) } else { None },
                            created_at: chrono::Utc::now().to_rfc3339(),
                        });
                    }

                    // Notify listener
                    if let Some(cb) = &listener {
                        cb(&AgentStepNotification {
                            step,
                            max_steps: self.max_steps,
                            tool_name: tool_name.clone(),
                            arguments_summary: tool_args.chars().take(80).collect(),
                            duration_ms,
                            success: tool_res.success,
                            output_snippet: tool_res.output.chars().take(120).collect(),
                        });
                    }

                    // Append tool response to conversation
                    messages.push(ChatMessage::tool_response(&tc.id, tool_name, tool_res.output));

                    if tool_calls_count >= self.max_tool_calls {
                        final_status = "aborted";
                        final_response = format!("Max tool calls limit ({}) reached.", self.max_tool_calls);
                        break;
                    }
                }

                if final_status == "aborted" {
                    break;
                }
            } else {
                // Assistant responded with plain text / final conclusion
                final_response = assistant_msg.content.clone().unwrap_or_default();
                break;
            }
        }

        if step >= self.max_steps && final_response.is_empty() {
            final_status = "aborted";
            final_response = format!("Max iterations limit ({}) reached without final answer.", self.max_steps);
        }

        // 4. Update agent run in SQLite
        if let Some(mem) = &self.memory {
            let estimated_cost = (total_tokens as f64 / 1_000_000.0) * 3.0; // ~$3 per 1M tokens approximation
            let error_msg = if final_status != "completed" { Some(final_response.as_str()) } else { None };
            let _ = mem.finish_agent_run(run_id, final_status, total_tokens, estimated_cost, error_msg);
        }

        Ok(AgentExecutionSummary {
            run_id,
            task: task.to_string(),
            status: final_status.to_string(),
            total_steps: step,
            tool_calls_count,
            total_tokens,
            final_response,
        })
    }
}
