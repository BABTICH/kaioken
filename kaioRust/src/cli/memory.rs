use crate::config::Config;
use crate::error::KaiokenError;
use crate::memory::{CommitAnalyzer, MemoryEngine};
use crate::scanner::{ChangeType, FileInventory, FileInventoryOptions, GitScanner, ScanLock};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum MemoryCommands {
    /// Initialize the .agent directory and default configuration
    Init,
    /// Show the current status of the memory engine and configuration
    Status,
    /// Scan repository and detect changes
    Scan {
        /// Scan only files changed in latest commit or working tree
        #[arg(long)]
        changed: bool,

        /// Run deterministic scan only (zero LLM API cost)
        #[arg(long)]
        no_llm: bool,

        /// Output scan results in structured JSON format
        #[arg(long)]
        json: bool,
    },
    /// Analyze and record commit memory for HEAD or a specific commit
    Commit {
        /// Commit hash or reference to analyze (defaults to HEAD)
        #[arg(long)]
        hash: Option<String>,

        /// Run deterministic analysis only (zero LLM API cost)
        #[arg(long)]
        no_llm: bool,

        /// Executed from git post-commit hook; non-blocking lock acquisition (will not fail or block git)
        #[arg(long)]
        hook: bool,

        /// Output commit analysis in structured JSON format
        #[arg(long)]
        json: bool,
    },
    /// Manage git post-commit hook for automated memory updates
    Hook {
        /// Install git post-commit hook (.git/hooks/post-commit)
        #[arg(long)]
        install: bool,

        /// Uninstall git post-commit hook (.git/hooks/post-commit)
        #[arg(long)]
        uninstall: bool,
    },
    /// Explain the agent's internal memory and understanding of a file
    Explain(MemoryExplainArgs),
    /// Show recent commit memories and learned changes
    Recent(MemoryRecentArgs),
}

#[derive(clap::Args, Debug, Clone, Default)]
pub struct MemoryExplainArgs {
    /// File path to explain (relative or absolute)
    pub path: String,

    /// Output as structured JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args, Debug, Clone, Default)]
pub struct MemoryRecentArgs {
    /// Number of recent commits to show
    #[arg(long, default_value = "10")]
    pub last: usize,

    /// Filter by minimum risk level (low, medium, high, critical)
    #[arg(long)]
    pub min_risk: Option<String>,

    /// Show only commits touching a specific file
    #[arg(long)]
    pub file: Option<String>,

    /// Output as structured JSON
    #[arg(long)]
    pub json: bool,
}

pub async fn handle_memory_command(cmd: MemoryCommands) -> Result<(), KaiokenError> {
    match cmd {
        MemoryCommands::Init => {
            let current_dir = std::env::current_dir()?;
            let repo_root = GitScanner::find_repo_root(&current_dir).unwrap_or(current_dir);
            let config = Config::default_config();
            config.save(&repo_root)?;

            tracing::info!(
                "Initialized .agent directory and config.toml at {}",
                repo_root.display()
            );
            println!("✅ KAIOKEN initialized successfully!");
            println!(
                "   Config created at: {}/.agent/config.toml",
                repo_root.display()
            );
            println!("   Next step: Run `kaioken memory status` to verify.");
        }
        MemoryCommands::Status => {
            let current_dir = std::env::current_dir()?;
            let repo_root = GitScanner::find_repo_root(&current_dir).unwrap_or(current_dir);
            let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());

            println!("✅ KAIOKEN Status:");
            println!("   Repository:    {}", repo_root.display());
            println!("   Project Name:  {}", config.project.name);
            println!("   Database Path: {}", config.memory.database);
            println!("   Docs Output:   {}", config.docs.output);
            println!("   LLM Provider:  {}", config.llm.provider);
            println!("   Planner Model: {}", config.llm.planner_model);

            let db_path = repo_root.join(&config.memory.database);
            if db_path.exists() {
                match MemoryEngine::open(&db_path) {
                    Ok(mem) => {
                        let f_count = mem.count_files().unwrap_or(0);
                        let s_count = mem.count_symbols().unwrap_or(0);
                        let imp_count = mem.count_raw_imports().unwrap_or(0);
                        let c_count = mem.count_commits().unwrap_or(0);
                        let recent = mem.get_recent_commits(1).unwrap_or_default();
                        let commit_str = if let Some(latest) = recent.first() {
                            let short_h = if latest.hash.len() >= 8 {
                                &latest.hash[0..8]
                            } else {
                                &latest.hash
                            };
                            format!(
                                "{} commits indexed (latest: {} - \"{}\")",
                                c_count, short_h, latest.message
                            )
                        } else {
                            format!("{} commits indexed", c_count)
                        };
                        println!(
                            "   Memory DB:     ✅ Connected ({} files, {} symbols, {} imports, {})",
                            f_count, s_count, imp_count, commit_str
                        );
                    }
                    Err(e) => {
                        println!("   Memory DB:     ⚠️  Exists but failed to open: {}", e);
                    }
                }
            } else {
                println!(
                    "   Memory DB:     ⚠️  Not found (Run `kaioken memory scan` to initialize)"
                );
            }

            // Post-commit hook status
            let hook_path = repo_root.join(".git").join("hooks").join("post-commit");
            if hook_path.exists() {
                let content = std::fs::read_to_string(&hook_path).unwrap_or_default();
                if content.contains("kaioken") {
                    println!("   Git Hook:      ✅ Installed (post-commit automatic memory updates active)");
                } else {
                    println!("   Git Hook:      ⚠️  Custom post-commit hook present (does not invoke kaioken)");
                }
            } else {
                println!("   Git Hook:      ⚪ Not installed (run `kaioken memory hook --install` to enable)");
            }
        }
        MemoryCommands::Commit {
            hash,
            no_llm,
            hook,
            json,
        } => {
            let current_dir = std::env::current_dir()?;
            let repo_root = GitScanner::find_repo_root(&current_dir).unwrap_or(current_dir);
            let agent_dir = repo_root.join(Config::AGENT_DIR);
            let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());

            // Lock handling: non-blocking if invoked from git hook
            let _lock_guard = if hook {
                match ScanLock::try_acquire(&agent_dir)? {
                    Some(guard) => guard,
                    None => {
                        println!("⚠️  KAIOKEN: Scan lock active. Skipping hook execution to avoid blocking git.");
                        return Ok(());
                    }
                }
            } else {
                ScanLock::acquire(&agent_dir)?
            };

            let commit_ref = hash.as_deref().unwrap_or("HEAD");
            let commit_info = match GitScanner::get_commit_by_hash(&repo_root, commit_ref)? {
                Some(ci) => ci,
                None => {
                    if !hook {
                        println!(
                            "⚠️  Commit '{}' not found or repository has no commits yet.",
                            commit_ref
                        );
                    }
                    return Ok(());
                }
            };

            let db_path = repo_root.join(&config.memory.database);
            let memory = MemoryEngine::open(&db_path)?;

            let analysis = CommitAnalyzer::analyze(&repo_root, &commit_info, &memory)?;
            let commit_id = memory.record_commit_analysis(&analysis)?;

            if json {
                let json_obj = serde_json::json!({
                    "commit_id": commit_id,
                    "analysis": analysis,
                    "scan_mode": if no_llm { "deterministic" } else { "standard" }
                });
                let json_str = serde_json::to_string_pretty(&json_obj)
                    .map_err(|e| KaiokenError::ScannerError(e.to_string()))?;
                println!("{}", json_str);
                return Ok(());
            }

            let hash_short = if commit_info.hash.len() >= 8 {
                &commit_info.hash[0..8]
            } else {
                &commit_info.hash
            };

            println!("┌────────────────────────────────────────────────────────┐");
            println!("│           KAIOKEN AGENT: COMMIT MEMORY GENERATOR       │");
            println!("├────────────────────────────────────────────────────────┤");
            println!("│ Commit Hash:     {:<38} │", hash_short);
            println!(
                "│ Author:          {:<38} │",
                if commit_info.author_name.len() > 38 {
                    &commit_info.author_name[0..35]
                } else {
                    &commit_info.author_name
                }
            );
            println!(
                "│ Risk Level:      {:<38} │",
                analysis.risk_level.to_uppercase()
            );
            println!("│ Changed Files:   {:<38} │", analysis.changed_files_count);
            println!(
                "│ Affected Symbols:{:<38} │",
                analysis.affected_symbols_count
            );
            println!("│ Database ID:     {:<38} │", commit_id);
            println!(
                "│ Mode:            {:<38} │",
                if no_llm {
                    "Deterministic (--no-llm)"
                } else {
                    "Standard"
                }
            );
            println!("└────────────────────────────────────────────────────────┘");

            if !analysis.touched_symbols.is_empty() {
                println!("\nAffected Symbols in Commit:");
                for sym in &analysis.touched_symbols {
                    println!(
                        "  - `{}` ({} in `{}`:L{}-L{}, {})",
                        sym.symbol_name,
                        sym.symbol_kind,
                        sym.file_path,
                        sym.line_start,
                        sym.line_end,
                        sym.change_type
                    );
                }
            }

            println!(
                "\n✅ Commit memory indexed to SQLite database (id: {}).",
                commit_id
            );
            if hook {
                println!("   (Generated automatically via git post-commit hook)");
            }
        }
        MemoryCommands::Hook { install, uninstall } => {
            let current_dir = std::env::current_dir()?;
            let repo_root = GitScanner::find_repo_root(&current_dir).unwrap_or(current_dir);

            if uninstall {
                let outcome = crate::hook::HookManager::uninstall(&repo_root)?;
                match outcome {
                    crate::hook::UninstallOutcome::RemovedAndDeletedFile(path) => {
                        println!("✅ Git post-commit hook removed from {}", path.display());
                    }
                    crate::hook::UninstallOutcome::RemovedBlock(path) => {
                        println!(
                            "✅ Kaioken hook block removed from {} (other hooks preserved)",
                            path.display()
                        );
                    }
                    crate::hook::UninstallOutcome::NotInstalled(path) => {
                        println!("ℹ️  No Kaioken hook was found at {}", path.display());
                    }
                    crate::hook::UninstallOutcome::FileNotFound(path) => {
                        println!("ℹ️  No post-commit hook was found at {}", path.display());
                    }
                }
                return Ok(());
            }

            if install {
                let outcome = crate::hook::HookManager::install(&repo_root, false, "/bin/sh")?;
                match outcome {
                    crate::hook::InstallOutcome::InstalledFresh(path)
                    | crate::hook::InstallOutcome::Appended(path)
                    | crate::hook::InstallOutcome::Replaced(path) => {
                        println!("✅ Git post-commit hook installed successfully!");
                        println!("   Location: {}", path.display());
                        println!("   After each commit, KAIOKEN will automatically update commit memory and touched symbols.");
                    }
                    crate::hook::InstallOutcome::AlreadyInstalled(path) => {
                        println!(
                            "ℹ️  Git post-commit hook is already installed at {}",
                            path.display()
                        );
                    }
                }
                return Ok(());
            }

            // Status check
            let status = crate::hook::HookManager::status(&repo_root)?;
            println!("{}", status.render_terminal());
        }
        MemoryCommands::Scan {
            changed,
            no_llm,
            json,
        } => {
            let current_dir = std::env::current_dir()?;
            let repo_root = GitScanner::find_repo_root(&current_dir).unwrap_or(current_dir);
            let agent_dir = repo_root.join(Config::AGENT_DIR);
            let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());

            // Acquire lock to prevent concurrent scan processes
            let _lock_guard = ScanLock::acquire(&agent_dir)?;
            tracing::info!("Acquired scan.lock in {}", agent_dir.display());

            if changed {
                let commit_info = GitScanner::get_head_commit(&repo_root)?;
                let changed_files = GitScanner::get_all_changed_files(&repo_root)?;

                // Phase 6: Index commit memory if HEAD commit is available
                let mut commit_analysis = None;
                let db_path = repo_root.join(&config.memory.database);
                if let Some(ref ci) = commit_info {
                    if let Ok(memory) = MemoryEngine::open(&db_path) {
                        if let Ok(analysis) = CommitAnalyzer::analyze(&repo_root, ci, &memory) {
                            let _ = memory.record_commit_analysis(&analysis);
                            commit_analysis = Some(analysis);
                        }
                    }
                }

                if json {
                    let json_obj = serde_json::json!({
                        "repo_root": repo_root.display().to_string(),
                        "head_commit": commit_info,
                        "commit_analysis": commit_analysis,
                        "changed_files": changed_files,
                        "scan_mode": if no_llm { "deterministic" } else { "standard" }
                    });
                    let json_str = serde_json::to_string_pretty(&json_obj)
                        .map_err(|e| KaiokenError::ScannerError(e.to_string()))?;
                    println!("{}", json_str);
                    return Ok(());
                }

                println!("┌────────────────────────────────────────────────────────┐");
                println!("│           KAIOKEN AGENT: GIT CHANGE DETECTION          │");
                println!("├────────────────────────────────────────────────────────┤");
                println!(
                    "│ Repo Root:       {:<38} │",
                    repo_root.file_name().unwrap_or_default().to_string_lossy()
                );
                if let Some(c) = &commit_info {
                    println!(
                        "│ HEAD Commit:     {:<38} │",
                        &c.hash[0..8.min(c.hash.len())]
                    );
                    println!("│ Author:          {:<38} │", c.author_name);
                    println!(
                        "│ Commit Msg:      {:<38} │",
                        if c.message.len() > 38 {
                            &c.message[0..35]
                        } else {
                            &c.message
                        }
                    );
                } else {
                    println!(
                        "│ HEAD Commit:     {:<38} │",
                        "No commits yet (Fresh repository)"
                    );
                }
                println!("│ Changed Files:   {:<38} │", changed_files.len());
                if let Some(ref ca) = commit_analysis {
                    println!("│ Risk Level:      {:<38} │", ca.risk_level.to_uppercase());
                    println!("│ Affected Symbols:{:<38} │", ca.affected_symbols_count);
                }
                println!(
                    "│ Scan Mode:       {:<38} │",
                    if no_llm {
                        "Deterministic (--no-llm)"
                    } else {
                        "Standard"
                    }
                );
                println!("└────────────────────────────────────────────────────────┘");

                if changed_files.is_empty() {
                    println!(
                        "\nNo modified, staged, or untracked files detected in this repository."
                    );
                } else {
                    println!("\nDetected Changed Files (Working Tree & Recent Commit):");
                    for f in &changed_files {
                        let path_str = f.path.to_string_lossy();
                        match f.change_type {
                            ChangeType::Renamed => {
                                let old_str = f
                                    .old_path
                                    .as_ref()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_default();
                                println!(
                                    "  {} {} (renamed from {})",
                                    f.change_type.badge(),
                                    path_str,
                                    old_str
                                );
                            }
                            _ => {
                                println!(
                                    "  {} {} ({})",
                                    f.change_type.badge(),
                                    path_str,
                                    f.change_type.description()
                                );
                            }
                        }
                    }
                }

                if let Some(ref ca) = commit_analysis {
                    if !ca.touched_symbols.is_empty() {
                        println!("\nAffected AST Symbols in Recent Commit:");
                        for sym in &ca.touched_symbols {
                            println!(
                                "  - `{}` ({} in `{}`:L{}-L{}, {})",
                                sym.symbol_name,
                                sym.symbol_kind,
                                sym.file_path,
                                sym.line_start,
                                sym.line_end,
                                sym.change_type
                            );
                        }
                    }
                }
                println!(
                    "\n✓ Commit memory and changed files synchronized to SQLite memory database."
                );
            } else {
                // Full repository inventory walk
                let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());
                let mut options = FileInventoryOptions::default();
                for pat in config.scanner.ignore {
                    if !options.ignore_patterns.contains(&pat) {
                        options.ignore_patterns.push(pat);
                    }
                }

                let summary = FileInventory::scan_dir_with_options(&repo_root, &options)?;

                // Phase 5: Persist inventory and extracted symbols into SQLite Memory Database
                let db_path = repo_root.join(&config.memory.database);
                let memory_engine = MemoryEngine::open(&db_path)?;
                let sync_stats = memory_engine.sync_inventory(&summary)?;

                if json {
                    let json_obj = serde_json::json!({
                        "summary": summary,
                        "sync_stats": sync_stats,
                        "database_path": db_path.display().to_string(),
                    });
                    let json_str = serde_json::to_string_pretty(&json_obj)
                        .map_err(|e| KaiokenError::ScannerError(e.to_string()))?;
                    println!("{}", json_str);
                    return Ok(());
                }

                let kb_total = (summary.total_size_bytes as f64) / 1024.0;
                let size_str = if kb_total > 1024.0 {
                    format!("{:.2} MB", kb_total / 1024.0)
                } else {
                    format!("{:.1} KB", kb_total)
                };

                println!("┌────────────────────────────────────────────────────────┐");
                println!("│           KAIOKEN AGENT: REPOSITORY INVENTORY          │");
                println!("├────────────────────────────────────────────────────────┤");
                println!(
                    "│ Workspace Root:  {:<38} │",
                    repo_root.file_name().unwrap_or_default().to_string_lossy()
                );
                println!("│ Total Files:     {:<38} │", summary.total_files);
                println!("│ Total Symbols:   {:<38} │", summary.total_symbols);
                println!("│ Total Size:      {:<38} │", size_str);
                println!(
                    "│ Scan Mode:       {:<38} │",
                    if no_llm {
                        "Deterministic (--no-llm)"
                    } else {
                        "Standard"
                    }
                );
                println!("├────────────────────────────────────────────────────────┤");
                println!("│ Breakdown by Language:                                 │");
                for (lang, count) in &summary.files_by_language {
                    println!("│   - {:<12} {:>36} │", lang, format!("{} files", count));
                }
                println!("├────────────────────────────────────────────────────────┤");
                println!("│ Breakdown by Parse Status:                             │");
                for (status, count) in &summary.files_by_status {
                    println!("│   - {:<12} {:>36} │", status, format!("{} files", count));
                }
                println!("└────────────────────────────────────────────────────────┘");

                // Preview first 15 files
                println!("\nSample Discovered Files (Pre-filtered, Hashed & Parsed):");
                for file in summary.files.iter().take(15) {
                    let hash_preview = file
                        .file_hash
                        .as_ref()
                        .map(|h| &h[0..8.min(h.len())])
                        .unwrap_or("-");
                    let size_kb = (file.size as f64) / 1024.0;
                    let sym_info = if !file.symbols.is_empty() {
                        format!(", {} symbols", file.symbols.len())
                    } else {
                        String::new()
                    };
                    println!(
                        "  {:<8} {:<35} ({}, {:.1} KB, hash: {}{})",
                        file.parse_status.badge(),
                        file.relative_path.to_string_lossy(),
                        file.language.as_str(),
                        size_kb,
                        hash_preview,
                        sym_info
                    );
                }

                if summary.total_files > 15 {
                    println!("  ... and {} more files", summary.total_files - 15);
                }

                println!(
                    "\n✓ {} files, {} AST symbols, and {} raw imports synchronized to SQLite memory ({}).",
                    sync_stats.files_synced,
                    sync_stats.symbols_synced,
                    sync_stats.imports_synced,
                    config.memory.database
                );
            }
        }
        MemoryCommands::Explain(args) => {
            let current_dir = std::env::current_dir()?;
            let repo_root = GitScanner::find_repo_root(&current_dir).unwrap_or(current_dir);
            let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());
            let db_path = repo_root.join(&config.memory.database);

            if !db_path.exists() {
                if args.json {
                    let err_json = serde_json::json!({
                        "error": "not_scanned",
                        "message": "File not yet scanned — run `kaioken memory scan` first."
                    });
                    println!("{}", serde_json::to_string_pretty(&err_json).unwrap());
                } else {
                    println!("⚠️  File not yet scanned — run `kaioken memory scan` first.");
                }
                return Ok(());
            }

            let memory = MemoryEngine::open(&db_path)?;
            let normalized_path = crate::memory::normalize_file_path(&args.path, &repo_root);

            match memory.explain_file(&normalized_path) {
                Ok(explanation) => {
                    if args.json {
                        let json_str = serde_json::to_string_pretty(&explanation)
                            .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
                        println!("{}", json_str);
                    } else {
                        println!("{}", explanation.render_terminal());
                    }
                }
                Err(_) => {
                    if args.json {
                        let err_json = serde_json::json!({
                            "error": "file_not_found",
                            "file": normalized_path,
                            "message": "File not yet scanned — run `kaioken memory scan` first."
                        });
                        println!("{}", serde_json::to_string_pretty(&err_json).unwrap());
                    } else {
                        println!("⚠️  File not yet scanned — run `kaioken memory scan` first.");
                    }
                }
            }
        }
        MemoryCommands::Recent(args) => {
            let current_dir = std::env::current_dir()?;
            let repo_root = GitScanner::find_repo_root(&current_dir).unwrap_or(current_dir);
            let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());
            let db_path = repo_root.join(&config.memory.database);

            if !db_path.exists() {
                if args.json {
                    let empty_res = crate::memory::MemoryRecentResult {
                        total_commits: 0,
                        shown_commits: 0,
                        commits: Vec::new(),
                    };
                    println!("{}", serde_json::to_string_pretty(&empty_res).unwrap());
                } else {
                    println!("⚠️  Memory database not found. Run `kaioken memory scan` or `kaioken memory commit` first.");
                }
                return Ok(());
            }

            let memory = MemoryEngine::open(&db_path)?;
            let norm_file = args
                .file
                .as_deref()
                .map(|f| crate::memory::normalize_file_path(f, &repo_root));
            let result = memory.get_recent_commits_filtered(
                args.last,
                args.min_risk.as_deref(),
                norm_file.as_deref(),
            )?;

            if args.json {
                let json_str = serde_json::to_string_pretty(&result)
                    .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
                println!("{}", json_str);
            } else {
                println!("{}", result.render_terminal());
            }
        }
    }
    Ok(())
}
