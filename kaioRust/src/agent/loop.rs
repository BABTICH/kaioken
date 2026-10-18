use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use sha2::{Digest, Sha256};
use crate::config::Config;
use crate::critic::{Critic, CriticEngine};
use crate::docs::cascade::StalenessCascadeEngine;
use crate::error::Result;
use crate::llm::{
    ChatCompletionRequest, ChatMessage, FunctionDefinition,
    LlmProvider, ToolDefinition,
};
use crate::memory::{FailureRecord, MemoryEngine, ToolCallRecord};
use crate::retrieval::{RetrievalEngine, RetrievalOptions};
use crate::safety::audit::AuditLogger;
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
    pub critic_iterations: usize,
    pub context_retrieved: bool,
    pub touched_files: Vec<String>,
}

pub struct AgentLoop {
    tools: ToolRegistry,
    llm: Arc<dyn LlmProvider>,
    memory: Option<Arc<MemoryEngine>>,
    critic: Arc<dyn Critic>,
    repo_root: PathBuf,
    config: Config,
    max_steps: usize,
    max_tool_calls: usize,
    max_critic_retries: usize,
    retrieval_options: Option<RetrievalOptions>,
    enable_retrieval: bool,
    safety_policy: Option<Arc<crate::safety::SafetyPolicy>>,
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
            critic: Arc::new(CriticEngine::new()),
            repo_root,
            config,
            max_steps: 20,
            max_tool_calls: 100,
            max_critic_retries: 3,
            retrieval_options: None,
            enable_retrieval: true,
            safety_policy: None,
        }
    }

    pub fn with_critic(mut self, critic: Arc<dyn Critic>) -> Self {
        self.critic = critic;
        self
    }

    pub fn with_safety_policy(mut self, policy: Arc<crate::safety::SafetyPolicy>) -> Self {
        self.safety_policy = Some(policy);
        self
    }

    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    pub fn with_max_tool_calls(mut self, max_tool_calls: usize) -> Self {
        self.max_tool_calls = max_tool_calls;
        self
    }

    pub fn with_max_critic_retries(mut self, max_critic_retries: usize) -> Self {
        self.max_critic_retries = max_critic_retries;
        self
    }

    pub fn with_retrieval(mut self, enable: bool) -> Self {
        self.enable_retrieval = enable;
        self
    }

    pub fn with_retrieval_options(mut self, options: RetrievalOptions) -> Self {
        self.retrieval_options = Some(options);
        self
    }

    pub async fn execute_task(
        &self,
        task: &str,
        listener: Option<Arc<dyn Fn(&AgentStepNotification) + Send + Sync>>,
    ) -> Result<AgentExecutionSummary> {
        let audit_logger = AuditLogger::new(Some(&self.repo_root));
        let safety_policy = self.safety_policy.clone().unwrap_or_else(|| {
            let default_handler: Arc<dyn crate::safety::approval::ApprovalHandler> =
                Arc::new(crate::safety::approval::AutoApprovalHandler);
            Arc::new(crate::safety::SafetyPolicy::new(
                self.repo_root.clone(),
                self.config.permissions.mode,
                false,
                true,
                self.config.permissions.allowed_commands.clone(),
                self.config.permissions.blocked_commands.clone(),
                Some(default_handler),
                Some(audit_logger.clone()),
            ))
        });

        let tool_context = ToolContext {
            repo_root: self.repo_root.clone(),
            command_guard: safety_policy.command_guard.clone(),
            audit_logger,
            safety_policy: Some(safety_policy),
        };

        // 1. Initialize agent run in SQLite
        let model_name = &self.config.llm.coder_model;
        let run_id = if let Some(mem) = &self.memory {
            mem.start_agent_run(task, model_name)?
        } else {
            0
        };

        // 2. Pre-Planning Context Retrieval (Layer 4 integration)
        let mut context_retrieved = false;
        let context_packet = if self.enable_retrieval {
            if let Some(mem) = &self.memory {
                let opts = self.retrieval_options.clone().unwrap_or_default();
                let engine = RetrievalEngine::new(mem, &self.config, &self.repo_root);
                match engine.retrieve_context(task, &opts) {
                    Ok(packet) => {
                        context_retrieved = true;
                        Some(packet)
                    }
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        // 3. Build initial conversation prompt
        let system_prompt = format!(
            "You are KAIOKEN Agent, an autonomous repo-aware software engineering assistant.\n\
             Workspace Root: {}\n\
             You solve programming tasks by thinking step-by-step, using available tools, and adhering to strict compiler verification.\n\
             Guidelines:\n\
             - Inspect the provided Repository Context Packet before modifying code.\n\
             - If past failures are noted, explain how you will avoid them in your plan.\n\
             - Use read_file to inspect code before modifying it.\n\
             - Use edit_file for precise search-and-replace edits.\n\
             - Use create_file to author new files.\n\
             - Use bash to run test suites or compile checks.\n\
             - When your implementation is complete and verified, output a concise concluding summary without calling further tools.\n\
             - A deterministic critic will verify your changes. If verification fails, inspect the diagnostic errors and fix them.",
            self.repo_root.display()
        );

        let mut messages = vec![
            ChatMessage::system(system_prompt),
        ];

        if let Some(packet) = &context_packet {
            messages.push(ChatMessage::user(format!(
                "Repository Context from Memory Layer:\n\n{}\n\nTask Goal:\n{}",
                packet.to_markdown(),
                task
            )));
        } else {
            messages.push(ChatMessage::user(task));
        }

        let mut total_tokens = 0;
        let mut tool_calls_count = 0;
        let mut step = 0;
        let mut final_response = String::new();
        let mut final_status = "completed";
        let mut critic_iterations = 0;
        let mut active_failure_id: Option<i64> = None;
        let mut touched_files: HashSet<String> = HashSet::new();

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

            // Check if assistant called tools or believes task is complete
            let is_conclusion = match &assistant_msg.tool_calls {
                None => true,
                Some(calls) => calls.is_empty(),
            };

            if is_conclusion {
                let conclusion = assistant_msg.content.clone().unwrap_or_default();
                let critic_report = self.critic.evaluate(&self.repo_root).await?;

                if critic_report.passed {
                    // Critic passed -> resolve any tracked failure from this run
                    if let Some(fid) = active_failure_id {
                        if let Some(mem) = &self.memory {
                            let fix_summary = format!("Resolved in step {}: {}", step, conclusion);
                            let _ = mem.resolve_failure(fid, &fix_summary);
                        }
                    }
                    final_response = conclusion;
                    break;
                } else {
                    // Critic failed -> self-correction feedback loop
                    critic_iterations += 1;

                    // Log failure to SQLite failures table if not tracked yet
                    if active_failure_id.is_none() {
                        if let Some(mem) = &self.memory {
                            let first_msg = critic_report.diagnostics.first()
                                .map(|d| d.message.clone())
                                .unwrap_or_else(|| "Compiler/test verification failed".to_string());
                            let failure_record = FailureRecord {
                                id: None,
                                error_type: "compiler".to_string(),
                                message: format!("{} failed: {}", critic_report.command, first_msg),
                                file_id: None,
                                symbol_id: None,
                                fix_summary: None,
                                status: Some("unresolved".to_string()),
                                resolved_at: None,
                                created_at: chrono::Utc::now().to_rfc3339(),
                            };
                            if let Ok(fid) = mem.insert_failure(&failure_record) {
                                active_failure_id = Some(fid);
                            }
                        }
                    }

                    if critic_iterations >= self.max_critic_retries {
                        final_status = "failed";
                        final_response = format!(
                            "Task aborted: Deterministic verification checks failed after {} attempts.\n{}",
                            self.max_critic_retries,
                            critic_report.feedback_message
                        );
                        break;
                    }

                    // Feed critic feedback back into conversation
                    messages.push(assistant_msg.clone());
                    let feedback = format!(
                        "{}\n\n[Critic Attempt {} of {}]",
                        critic_report.feedback_message,
                        critic_iterations,
                        self.max_critic_retries
                    );
                    messages.push(ChatMessage::user(feedback));
                    continue;
                }
            }

            // Assistant emitted tool calls -> execute them
            let tool_calls = assistant_msg.tool_calls.as_ref().unwrap();

            // Append assistant message with tool calls to conversation
            messages.push(assistant_msg.clone());

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

                if tool_res.success && (tool_name == "create_file" || tool_name == "edit_file") {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(tool_args) {
                        if let Some(p) = val.get("path").and_then(|s| s.as_str()) {
                            touched_files.insert(p.replace('\\', "/"));
                        }
                    }
                }

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
        }

        if step >= self.max_steps && final_response.is_empty() {
            final_status = "aborted";
            final_response = format!("Max iterations limit ({}) reached without final answer.", self.max_steps);
        }

        let touched_list: Vec<String> = touched_files.into_iter().collect();

        // 4. Update agent run in SQLite
        if let Some(mem) = &self.memory {
            let estimated_cost = (total_tokens as f64 / 1_000_000.0) * 3.0; // ~$3 per 1M tokens approximation
            let error_msg = if final_status != "completed" { Some(final_response.as_str()) } else { None };
            let _ = mem.finish_agent_run(run_id, final_status, total_tokens, estimated_cost, error_msg);

            // 5. Post-task memory and doc staleness synchronization (Phase 12)
            if final_status == "completed" && !touched_list.is_empty() {
                // Cascade staleness to mirrored documentation
                let _ = StalenessCascadeEngine::cascade_changes(&self.repo_root, &touched_list, mem);

                // Update file inventory hashes in SQLite files table
                for rel_path in &touched_list {
                    let full_path = self.repo_root.join(rel_path);
                    if full_path.exists() {
                        if let Ok(bytes) = std::fs::read(&full_path) {
                            let mut hasher = Sha256::new();
                            hasher.update(&bytes);
                            let new_hash = format!("{:x}", hasher.finalize());
                            let _ = mem.update_file_hash(rel_path, &new_hash, bytes.len());
                        }
                    }
                }
            }
        }

        Ok(AgentExecutionSummary {
            run_id,
            task: task.to_string(),
            status: final_status.to_string(),
            total_steps: step,
            tool_calls_count,
            total_tokens,
            final_response,
            critic_iterations,
            context_retrieved,
            touched_files: touched_list,
        })
    }
}
