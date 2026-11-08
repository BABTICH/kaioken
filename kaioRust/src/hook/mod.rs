use crate::error::{KaiokenError, Result};
use crate::scanner::git::GitScanner;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const SENTINEL_START: &str = "# === KAIOKEN AGENT HOOK START ===";
pub const SENTINEL_END: &str = "# === KAIOKEN AGENT HOOK END ===";

pub const HOOK_FILE_NAME: &str = "post-commit";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallOutcome {
    InstalledFresh(PathBuf),
    Appended(PathBuf),
    Replaced(PathBuf),
    AlreadyInstalled(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UninstallOutcome {
    RemovedAndDeletedFile(PathBuf),
    RemovedBlock(PathBuf),
    NotInstalled(PathBuf),
    FileNotFound(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookStatus {
    pub hook_path: String,
    pub exists: bool,
    pub is_installed: bool,
    pub is_executable: bool,
    pub binary_on_path: bool,
    pub binary_path: Option<String>,
}

impl HookStatus {
    pub fn render_terminal(&self) -> String {
        let mut out = String::new();
        out.push_str("🔗 Git Hook Status\n");
        out.push_str("═══════════════════\n");
        out.push_str(&format!("  Hook file:      {}\n", self.hook_path));
        out.push_str(&format!(
            "  Exists:         {}\n",
            if self.exists { "✅ Yes" } else { "❌ No" }
        ));
        out.push_str(&format!(
            "  Kaioken:        {}\n",
            if self.is_installed {
                "✅ Installed (sentinel markers found)"
            } else if self.exists {
                "⚠️  Custom hook present (no Kaioken block)"
            } else {
                "⚪ Not installed"
            }
        ));
        out.push_str(&format!(
            "  Executable:     {}\n",
            if self.is_executable {
                "✅ Yes"
            } else {
                "⚠️  No"
            }
        ));

        if self.binary_on_path {
            let path_str = self.binary_path.as_deref().unwrap_or("on PATH");
            out.push_str(&format!("  Binary path:    ✅ {} (on PATH)\n", path_str));
        } else {
            out.push_str("  Binary path:    ⚠️  `kaioken` not found on PATH\n");
        }

        out
    }
}

pub struct HookManager;

impl HookManager {
    /// Resolve the path to the git post-commit hook for a given workspace
    pub fn resolve_hook_path(workspace_root: &Path) -> Result<PathBuf> {
        let repo_root = GitScanner::find_repo_root(workspace_root)
            .unwrap_or_else(|_| workspace_root.to_path_buf());
        Ok(repo_root.join(".git").join("hooks").join(HOOK_FILE_NAME))
    }

    /// Generate the standardized Kaioken post-commit script block
    pub fn generate_hook_block() -> String {
        format!(
            "{SENTINEL_START}\n\
# Auto-installed by `kaioken hook install`\n\
# Non-blocking post-commit memory synchronization\n\
# To uninstall: kaioken hook uninstall (or delete this block)\n\
\n\
if command -v kaioken >/dev/null 2>&1; then\n\
    kaioken memory scan --changed --no-llm >/dev/null 2>&1 &\n\
    disown 2>/dev/null || true\n\
fi\n\
\n\
{SENTINEL_END}\n"
        )
    }

    /// Install the post-commit hook
    pub fn install(workspace_root: &Path, force: bool, shell: &str) -> Result<InstallOutcome> {
        let hook_path = Self::resolve_hook_path(workspace_root)?;
        let hooks_dir = hook_path.parent().ok_or_else(|| {
            KaiokenError::ScannerError("Invalid hook parent directory".to_string())
        })?;

        if !hooks_dir.exists() {
            fs::create_dir_all(hooks_dir)?;
        }

        let hook_block = Self::generate_hook_block();

        if !hook_path.exists() {
            let clean_shell = if shell.trim().is_empty() {
                "/bin/sh"
            } else {
                shell.trim()
            };
            let shebang = if clean_shell.starts_with("#!") {
                clean_shell.to_string()
            } else {
                format!("#!{}", clean_shell)
            };
            let content = format!("{}\n{}", shebang, hook_block);
            fs::write(&hook_path, content)?;
            set_executable(&hook_path)?;
            return Ok(InstallOutcome::InstalledFresh(hook_path));
        }

        let existing_content = fs::read_to_string(&hook_path)?;

        if existing_content.contains(SENTINEL_START) {
            if !force {
                return Ok(InstallOutcome::AlreadyInstalled(hook_path));
            }

            // Replace existing block
            let start_idx = existing_content.find(SENTINEL_START).unwrap();
            let end_idx = match existing_content.find(SENTINEL_END) {
                Some(idx) => idx + SENTINEL_END.len(),
                None => existing_content.len(),
            };

            let before = &existing_content[..start_idx];
            let after = &existing_content[end_idx..];

            let new_content = format!(
                "{}{}{}",
                before.trim_end(),
                if before.is_empty() { "" } else { "\n\n" },
                hook_block
            );
            let final_content = if after.trim().is_empty() {
                new_content
            } else {
                format!("{}\n{}", new_content.trim_end(), after.trim_start())
            };

            fs::write(&hook_path, final_content)?;
            set_executable(&hook_path)?;
            return Ok(InstallOutcome::Replaced(hook_path));
        }

        // Append to existing file
        let mut new_content = existing_content;
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push('\n');
        new_content.push_str(&hook_block);

        fs::write(&hook_path, new_content)?;
        set_executable(&hook_path)?;
        Ok(InstallOutcome::Appended(hook_path))
    }

    /// Uninstall the Kaioken block from the post-commit hook
    pub fn uninstall(workspace_root: &Path) -> Result<UninstallOutcome> {
        let hook_path = Self::resolve_hook_path(workspace_root)?;

        if !hook_path.exists() {
            return Ok(UninstallOutcome::FileNotFound(hook_path));
        }

        let content = fs::read_to_string(&hook_path)?;

        if !content.contains(SENTINEL_START) {
            return Ok(UninstallOutcome::NotInstalled(hook_path));
        }

        let start_idx = content.find(SENTINEL_START).unwrap();
        let end_idx = match content.find(SENTINEL_END) {
            Some(idx) => idx + SENTINEL_END.len(),
            None => content.len(),
        };

        let before = &content[..start_idx];
        let after = &content[end_idx..];

        let remaining = format!("{}{}", before, after);
        let trimmed_remaining = remaining.trim();

        // Check if remaining content is empty or contains only comments/shebang
        let meaningful_lines: Vec<&str> = trimmed_remaining
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with("#!") && !l.starts_with('#'))
            .collect();

        if meaningful_lines.is_empty() {
            // Delete the file cleanly
            fs::remove_file(&hook_path)?;
            Ok(UninstallOutcome::RemovedAndDeletedFile(hook_path))
        } else {
            let mut final_content = trimmed_remaining.to_string();
            final_content.push('\n');
            fs::write(&hook_path, final_content)?;
            set_executable(&hook_path)?;
            Ok(UninstallOutcome::RemovedBlock(hook_path))
        }
    }

    /// Inspect the post-commit hook installation and health status
    pub fn status(workspace_root: &Path) -> Result<HookStatus> {
        let hook_path = Self::resolve_hook_path(workspace_root)?;
        let exists = hook_path.exists();

        let is_installed = if exists {
            let content = fs::read_to_string(&hook_path).unwrap_or_default();
            content.contains(SENTINEL_START)
        } else {
            false
        };

        let is_executable = if exists {
            check_executable(&hook_path)
        } else {
            false
        };

        let binary_path = find_binary_on_path("kaioken");
        let binary_on_path = binary_path.is_some();

        Ok(HookStatus {
            hook_path: hook_path.to_string_lossy().replace('\\', "/"),
            exists,
            is_installed,
            is_executable,
            binary_on_path,
            binary_path,
        })
    }
}

// ==========================================
// Helper Functions
// ==========================================

#[cfg(unix)]
fn set_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn check_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = fs::metadata(path) {
        metadata.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

#[cfg(not(unix))]
fn check_executable(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        !metadata.permissions().readonly()
    } else {
        false
    }
}

pub fn find_binary_on_path(binary_name: &str) -> Option<String> {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(binary_name);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().replace('\\', "/"));
            }
            #[cfg(windows)]
            {
                let candidate_exe = dir.join(format!("{}.exe", binary_name));
                if candidate_exe.is_file() {
                    return Some(candidate_exe.to_string_lossy().replace('\\', "/"));
                }
            }
        }
    }
    None
}
