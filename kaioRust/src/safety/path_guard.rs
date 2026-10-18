use std::path::{Component, Path, PathBuf};
use crate::error::{KaiokenError, Result};
use crate::safety::redact::RedactionEngine;

pub struct PathGuard;

fn strip_verbatim(p: &Path) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        p.to_path_buf()
    }
}

impl PathGuard {
    /// Validates that `target_path` resolves strictly within `repo_root`,
    /// does not perform path traversal, does not access secret files,
    /// and does not attempt direct write operations to protected repositories or databases.
    pub fn validate_path(target_path: &Path, repo_root: &Path, is_write: bool) -> Result<PathBuf> {
        // 1. Secret File Exclusion (checked immediately on input path)
        if RedactionEngine::is_sensitive_file(target_path) {
            return Err(KaiokenError::SafetyError(format!(
                "Access to sensitive credential or secret file '{}' is blocked",
                target_path.display()
            )));
        }

        // 2. Check for explicit parent directory traversal in components
        for comp in target_path.components() {
            if comp == Component::ParentDir {
                // components with .. require careful boundary verification
            }
        }

        let canonical_root = strip_verbatim(&repo_root.canonicalize().unwrap_or_else(|_| repo_root.to_path_buf()));

        // Resolve absolute target
        let resolved = if target_path.is_absolute() {
            target_path.to_path_buf()
        } else {
            canonical_root.join(target_path)
        };

        // Find closest existing ancestor to canonicalize properly
        let mut curr = resolved.clone();
        let mut components_after = Vec::new();
        while !curr.exists() {
            if let Some(name) = curr.file_name() {
                components_after.push(name.to_os_string());
                match curr.parent() {
                    Some(p) => curr = p.to_path_buf(),
                    None => break,
                }
            } else {
                break;
            }
        }

        let mut normalized = if curr.exists() {
            strip_verbatim(&curr.canonicalize().unwrap_or(curr))
        } else {
            strip_verbatim(&curr)
        };

        for comp in components_after.into_iter().rev() {
            normalized.push(comp);
        }

        // 3. Strict Workspace Boundary Check
        if !normalized.starts_with(&canonical_root) {
            return Err(KaiokenError::SafetyError(format!(
                "Path traversal blocked: '{}' resolves outside workspace root '{}'",
                target_path.display(),
                repo_root.display()
            )));
        }

        // Relative path inside workspace
        let rel_path = normalized.strip_prefix(&canonical_root).unwrap_or(&normalized);
        let rel_str = rel_path.to_string_lossy().replace('\\', "/");

        // 4. Protected Directory Quarantine
        if is_write {
            if rel_str == ".git" || rel_str.starts_with(".git/") {
                return Err(KaiokenError::SafetyError(
                    "Direct modifications to .git directory are strictly prohibited".to_string(),
                ));
            }

            if rel_str == ".agent/memory.sqlite" || rel_str.ends_with("/memory.sqlite") {
                return Err(KaiokenError::SafetyError(
                    "Direct modifications to .agent/memory.sqlite are strictly prohibited".to_string(),
                ));
            }
        }

        // 5. Sensitive File Exclusion on resolved path
        if RedactionEngine::is_sensitive_file(&normalized) {
            return Err(KaiokenError::SafetyError(format!(
                "Access to sensitive credential or secret file '{}' is blocked",
                target_path.display()
            )));
        }

        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_valid_path_within_workspace() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        let file = root.join("src/lib.rs");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "test").unwrap();

        let validated = PathGuard::validate_path(Path::new("src/lib.rs"), root, false);
        assert!(validated.is_ok());
    }

    #[test]
    fn test_path_traversal_blocked() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        let outside = Path::new("../../etc/passwd");
        let result = PathGuard::validate_path(outside, root, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("outside workspace root"));
    }

    #[test]
    fn test_secret_file_blocked() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        let env_file = Path::new(".env");
        let result = PathGuard::validate_path(env_file, root, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("sensitive credential"));
    }

    #[test]
    fn test_git_dir_write_blocked() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        let git_hook = Path::new(".git/hooks/pre-commit");
        let result = PathGuard::validate_path(git_hook, root, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(".git directory"));
    }
}
