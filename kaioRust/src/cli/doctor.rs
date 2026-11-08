use crate::doctor::DoctorEngine;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {
    /// Output results as formatted JSON
    #[arg(long)]
    pub json: bool,

    /// Run only specific checks (comma-separated, e.g. "git_availability,sqlite_runtime")
    #[arg(long)]
    pub only: Option<String>,

    /// Override the workspace directory to diagnose
    #[arg(long)]
    pub dir: Option<PathBuf>,
}

pub async fn handle_doctor_command(args: DoctorArgs) -> anyhow::Result<()> {
    let workspace_root = match args.dir {
        Some(d) => d,
        None => std::env::current_dir()?,
    };

    let report = DoctorEngine::run(&workspace_root, args.only.as_deref())?;

    if args.json {
        let json_str = serde_json::to_string_pretty(&report)?;
        println!("{}", json_str);
    } else {
        println!("{}", report.render_terminal());
    }

    if report.has_failures() {
        std::process::exit(1);
    }

    Ok(())
}
