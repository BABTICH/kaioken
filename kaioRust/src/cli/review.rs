use crate::config::Config;
use crate::error::KaiokenError;
use crate::memory::MemoryEngine;
pub use crate::review::ReviewArgs;
use crate::review::ReviewEngine;
use crate::scanner::GitScanner;

pub async fn handle_review_command(args: ReviewArgs) -> Result<(), KaiokenError> {
    let current_dir = std::env::current_dir()?;
    let repo_root = if let Some(ref dir) = args.dir {
        std::path::PathBuf::from(dir)
    } else {
        GitScanner::find_repo_root(&current_dir).unwrap_or(current_dir)
    };

    let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());
    let db_path = repo_root.join(&config.memory.database);

    let memory = if db_path.exists() {
        MemoryEngine::open(&db_path)?
    } else {
        MemoryEngine::open_in_memory()?
    };

    let mut report = ReviewEngine::review(&repo_root, &memory, &args)?;

    // Severity filtering: "info" (all), "warn" (breaking + gaps + stale), "breaking" (breaking only)
    let sev_lower = args.severity.to_lowercase();
    if sev_lower == "breaking" {
        report.test_gaps.clear();
        report.stale_docs.clear();
    }

    if args.json {
        let json_str = serde_json::to_string_pretty(&report)
            .map_err(|e| KaiokenError::ScannerError(e.to_string()))?;
        println!("{}", json_str);
    } else {
        println!("{}", report.render_terminal());
    }

    Ok(())
}
