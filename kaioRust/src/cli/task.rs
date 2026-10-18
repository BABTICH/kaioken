use std::sync::Arc;
use clap::{Args, Subcommand};
use crate::agent::tools::ToolRegistry;
use crate::agent::{AgentLoop, Planner};
use crate::config::Config;
use crate::llm::{LlmProvider, MockLlmClient, OpenRouterClient};
use crate::memory::MemoryEngine;

#[derive(Subcommand, Debug)]
pub enum TaskCommands {
    /// Execute an autonomous coding task with safety and tool execution
    Run(RunArgs),
    /// Generate an execution plan for a goal without executing modifications
    Plan(PlanArgs),
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Task description or prompt for the autonomous agent
    pub prompt: String,

    /// LLM model to use (overrides config)
    #[arg(long)]
    pub model: Option<String>,

    /// Maximum loop steps/iterations (default: 20)
    #[arg(long, default_value = "20")]
    pub max_steps: usize,

    /// Maximum tool calls allowed (default: 100)
    #[arg(long, default_value = "100")]
    pub max_tool_calls: usize,

    /// Run in offline mock mode without calling external LLM API
    #[arg(long)]
    pub mock: bool,
}

#[derive(Args, Debug)]
pub struct PlanArgs {
    /// Task goal to plan without applying changes
    pub prompt: String,
}

pub async fn handle_task_command(cmd: TaskCommands) -> anyhow::Result<()> {
    let repo_root = std::env::current_dir()?;
    let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());
    let db_path = repo_root.join(&config.memory.database);
    let memory = MemoryEngine::open(&db_path).ok().map(Arc::new);

    match cmd {
        TaskCommands::Run(args) => {
            println!("🤖 KAIOKEN Agent starting task: '{}'", args.prompt);
            println!("   Repository: {}", repo_root.display());
            let model_name = args.model.unwrap_or_else(|| config.llm.coder_model.clone());
            println!("   Model: {}", model_name);

            let api_key = std::env::var("OPENROUTER_API_KEY").ok();
            let llm: Arc<dyn LlmProvider> = if args.mock {
                println!("   [Offline Mock mode: deterministic simulation active]");
                Arc::new(MockLlmClient::new())
            } else if let Some(key) = api_key {
                Arc::new(OpenRouterClient::new(key))
            } else {
                println!("   [Notice: OPENROUTER_API_KEY not set; using deterministic mock client]");
                Arc::new(MockLlmClient::new())
            };

            let tools = ToolRegistry::default_registry();
            let agent_loop = AgentLoop::new(tools, llm, memory, repo_root, config)
                .with_max_steps(args.max_steps)
                .with_max_tool_calls(args.max_tool_calls);

            let listener = Arc::new(|n: &crate::agent::AgentStepNotification| {
                let badge = if n.success { "✅ OK" } else { "❌ ERROR" };
                println!(
                    "   [Step {}/{}] {} ({} ms) -> {}",
                    n.step, n.max_steps, n.tool_name, n.duration_ms, badge
                );
                if !n.output_snippet.is_empty() {
                    let preview = n.output_snippet.replace('\n', " ");
                    println!("          Output: {}", if preview.len() > 80 { format!("{}...", &preview[..80]) } else { preview });
                }
            });

            let summary = agent_loop.execute_task(&args.prompt, Some(listener)).await?;

            println!("\n✨ Task Finished (Status: {})", summary.status);
            println!("   Total Steps: {}", summary.total_steps);
            println!("   Total Tool Calls: {}", summary.tool_calls_count);
            println!("   Tokens Processed: {}", summary.total_tokens);
            if !summary.final_response.is_empty() {
                println!("\nSummary Conclusion:\n{}", summary.final_response);
            }
        }
        TaskCommands::Plan(args) => {
            let plan = Planner::create_plan(&args.prompt);
            println!("Generated Execution Plan for: '{}'\n", plan.goal);
            for step in plan.steps {
                println!("  {}. {}", step.step_number, step.description);
            }
        }
    }

    Ok(())
}
