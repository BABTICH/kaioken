use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use crate::error::{KaiokenError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitCommitInfo {
    pub hash: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Untracked,
}

impl ChangeType {
    pub fn badge(&self) -> &'static str {
        match self {
            ChangeType::Added => "[A]",
            ChangeType::Modified => "[M]",
            ChangeType::Deleted => "[D]",
            ChangeType::Renamed => "[R]",
            ChangeType::Untracked => "[?]",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ChangeType::Added => "added",
            ChangeType::Modified => "modified",
            ChangeType::Deleted => "deleted",
            ChangeType::Renamed => "renamed",
            ChangeType::Untracked => "untracked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangedFile {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub old_path: Option<PathBuf>,
}

pub struct GitScanner;

impl GitScanner {
    /// Locate the top-level root directory of the enclosing git repository.
    pub fn find_repo_root(start_dir: &Path) -> Result<PathBuf> {
        let output = Command::new("git")
            .arg("-C")
            .arg(start_dir)
            .arg("rev-parse")
            .arg("--show-toplevel")
            .output()
            .map_err(|e| KaiokenError::ScannerError(format!("Failed to execute git: {}", e)))?;

        if !output.status.success() {
            return Err(KaiokenError::ScannerError(format!(
                "Directory {} is not within a valid git repository",
                start_dir.display()
            )));
        }

        let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(PathBuf::from(raw))
    }

    /// Retrieve HEAD commit information, or Ok(None) if repository has no commits yet.
    pub fn get_head_commit(repo_root: &Path) -> Result<Option<GitCommitInfo>> {
        Self::get_commit_by_hash(repo_root, "HEAD")
    }

    /// Retrieve commit information for a specific commit hash (or reference like HEAD).
    pub fn get_commit_by_hash(repo_root: &Path, commit_ref: &str) -> Result<Option<GitCommitInfo>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo_root)
            .arg("log")
            .arg("-1")
            .arg("--format=%H%x1f%an%x1f%ae%x1f%aI%x1f%s")
            .arg(commit_ref)
            .output()
            .map_err(|e| KaiokenError::ScannerError(format!("Failed to run git log: {}", e)))?;

        if !output.status.success() {
            // Commit doesn't exist or repo has no commits
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            return Ok(None);
        }

        let parts: Vec<&str> = stdout.split('\x1f').collect();
        if parts.len() >= 5 {
            Ok(Some(GitCommitInfo {
                hash: parts[0].to_string(),
                author_name: parts[1].to_string(),
                author_email: parts[2].to_string(),
                timestamp: parts[3].to_string(),
                message: parts[4].to_string(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Retrieve changed files in the latest HEAD commit.
    pub fn get_commit_changed_files(repo_root: &Path) -> Result<Vec<ChangedFile>> {
        Self::get_commit_changed_files_for_hash(repo_root, "HEAD")
    }

    /// Retrieve changed files for a specific commit hash or reference.
    pub fn get_commit_changed_files_for_hash(
        repo_root: &Path,
        commit_ref: &str,
    ) -> Result<Vec<ChangedFile>> {
        if Self::get_commit_by_hash(repo_root, commit_ref)?.is_none() {
            return Ok(Vec::new());
        }

        let parents_output = Command::new("git")
            .arg("-C")
            .arg(repo_root)
            .arg("rev-list")
            .arg("--parents")
            .arg("-n")
            .arg("1")
            .arg(commit_ref)
            .output()
            .map_err(|e| KaiokenError::ScannerError(format!("Failed to check commit parents: {}", e)))?;

        let stdout = String::from_utf8_lossy(&parents_output.stdout);
        let hashes: Vec<&str> = stdout.split_whitespace().collect();

        let output = if hashes.len() <= 1 {
            // Initial commit (no parent) -> diff-tree against empty tree
            Command::new("git")
                .arg("-C")
                .arg(repo_root)
                .arg("diff-tree")
                .arg("--no-commit-id")
                .arg("--name-status")
                .arg("-r")
                .arg("--root")
                .arg(commit_ref)
                .output()
        } else {
            // Normal commit -> diff against parent
            Command::new("git")
                .arg("-C")
                .arg(repo_root)
                .arg("diff-tree")
                .arg("--no-commit-id")
                .arg("--name-status")
                .arg("-r")
                .arg(format!("{}^", commit_ref))
                .arg(commit_ref)
                .output()
        }
        .map_err(|e| KaiokenError::ScannerError(format!("Failed to run git diff-tree: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        Self::parse_name_status_output(&String::from_utf8_lossy(&output.stdout))
    }

    /// Retrieve uncommitted working tree changes (staged, unstaged, untracked).
    pub fn get_working_tree_changed_files(repo_root: &Path) -> Result<Vec<ChangedFile>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo_root)
            .arg("status")
            .arg("--porcelain=v1")
            .arg("-uall")
            .output()
            .map_err(|e| KaiokenError::ScannerError(format!("Failed to run git status: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let mut changed_files = Vec::new();
        let raw = String::from_utf8_lossy(&output.stdout);

        for line in raw.lines() {
            if line.len() < 4 {
                continue;
            }
            let status = &line[0..2];
            let raw_path = line[3..].trim();
            let clean_path = raw_path.trim_matches('"');

            let (change_type, path, old_path) = if status == "??" {
                (ChangeType::Untracked, PathBuf::from(clean_path), None)
            } else if status.contains('D') {
                (ChangeType::Deleted, PathBuf::from(clean_path), None)
            } else if status.contains('A') {
                (ChangeType::Added, PathBuf::from(clean_path), None)
            } else if status.contains('R') {
                if let Some((old_p, new_p)) = clean_path.split_once(" -> ") {
                    (
                        ChangeType::Renamed,
                        PathBuf::from(new_p.trim_matches('"')),
                        Some(PathBuf::from(old_p.trim_matches('"'))),
                    )
                } else {
                    (ChangeType::Renamed, PathBuf::from(clean_path), None)
                }
            } else {
                (ChangeType::Modified, PathBuf::from(clean_path), None)
            };

            changed_files.push(ChangedFile {
                path,
                change_type,
                old_path,
            });
        }

        Ok(changed_files)
    }

    /// Retrieve combined changed files (latest commit + active working tree modifications),
    /// deduplicated with working tree status taking precedence.
    pub fn get_all_changed_files(repo_root: &Path) -> Result<Vec<ChangedFile>> {
        let mut results = Self::get_working_tree_changed_files(repo_root)?;
        let commit_changes = Self::get_commit_changed_files(repo_root)?;

        for file in commit_changes {
            if !results.iter().any(|f| f.path == file.path) {
                results.push(file);
            }
        }

        // Sort by path for deterministic output
        results.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(results)
    }

    /// Retrieve unified diff text for a specific file or the entire repository.
    pub fn get_diff(repo_root: &Path, file_path: Option<&Path>) -> Result<String> {
        let mut cmd = Command::new("git");
        cmd.arg("-C").arg(repo_root).arg("diff");

        if let Some(path) = file_path {
            cmd.arg("--").arg(path);
        }

        let output = cmd
            .output()
            .map_err(|e| KaiokenError::ScannerError(format!("Failed to run git diff: {}", e)))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Retrieve unified diff text for a specific commit hash or reference.
    pub fn get_commit_diff(repo_root: &Path, commit_ref: &str) -> Result<String> {
        let parents_output = Command::new("git")
            .arg("-C")
            .arg(repo_root)
            .arg("rev-list")
            .arg("--parents")
            .arg("-n")
            .arg("1")
            .arg(commit_ref)
            .output()
            .map_err(|e| KaiokenError::ScannerError(format!("Failed to check commit parents: {}", e)))?;

        let stdout = String::from_utf8_lossy(&parents_output.stdout);
        let hashes: Vec<&str> = stdout.split_whitespace().collect();

        let output = if hashes.len() <= 1 {
            // Initial commit -> diff-tree against empty tree with patch
            Command::new("git")
                .arg("-C")
                .arg(repo_root)
                .arg("diff-tree")
                .arg("-p")
                .arg("--unified=0")
                .arg("--root")
                .arg(commit_ref)
                .output()
        } else {
            // Normal commit -> diff against parent
            Command::new("git")
                .arg("-C")
                .arg(repo_root)
                .arg("diff")
                .arg("--unified=0")
                .arg(hashes[1])
                .arg(commit_ref)
                .output()
        }
        .map_err(|e| KaiokenError::ScannerError(format!("Failed to run git diff for commit: {}", e)))?;

        if !output.status.success() {
            return Err(KaiokenError::ScannerError(format!(
                "git diff failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Parse unified diff text to extract changed line ranges `(start_line, end_line)` for each file.
    pub fn parse_diff_hunks_changed_lines(
        diff_text: &str,
    ) -> std::collections::HashMap<PathBuf, Vec<(usize, usize)>> {
        let mut map: std::collections::HashMap<PathBuf, Vec<(usize, usize)>> =
            std::collections::HashMap::new();
        let mut current_file: Option<PathBuf> = None;

        for line in diff_text.lines() {
            if line.starts_with("+++ b/") {
                let path_str = line["+++ b/".len()..].trim().trim_matches('"');
                current_file = Some(PathBuf::from(path_str));
            } else if line.starts_with("+++ /dev/null") {
                current_file = None;
            } else if line.starts_with("--- ") {
                continue;
            } else if line.starts_with("diff --git ") {
                current_file = None;
            } else if line.starts_with("@@ ") {
                if let Some(ref path) = current_file {
                    if let Some((start, count)) = Self::parse_hunk_header_range(line) {
                        let end = if count == 0 {
                            start
                        } else {
                            start + count.saturating_sub(1)
                        };
                        map.entry(path.clone()).or_default().push((start, end));
                    }
                }
            }
        }

        map
    }

    fn parse_hunk_header_range(line: &str) -> Option<(usize, usize)> {
        // Line format: "@@ -old,old_cnt +new_start,new_cnt @@ optional text"
        if !line.starts_with("@@ ") {
            return None;
        }
        let end_idx = line[3..].find("@@")? + 3;
        let content = line[3..end_idx].trim();
        for part in content.split_whitespace() {
            if part.starts_with('+') {
                let num_str = &part[1..];
                if let Some((start_s, count_s)) = num_str.split_once(',') {
                    let start = start_s.parse::<usize>().ok()?;
                    let count = count_s.parse::<usize>().ok()?;
                    return Some((start, count));
                } else {
                    let start = num_str.parse::<usize>().ok()?;
                    return Some((start, 1));
                }
            }
        }
        None
    }

    fn parse_name_status_output(raw: &str) -> Result<Vec<ChangedFile>> {
        let mut files = Vec::new();
        for line in raw.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.is_empty() {
                continue;
            }

            let status = parts[0];
            if parts.len() == 2 {
                let path = PathBuf::from(parts[1].trim_matches('"'));
                let change_type = match status.chars().next() {
                    Some('A') => ChangeType::Added,
                    Some('D') => ChangeType::Deleted,
                    Some('M') => ChangeType::Modified,
                    _ => ChangeType::Modified,
                };
                files.push(ChangedFile {
                    path,
                    change_type,
                    old_path: None,
                });
            } else if parts.len() >= 3 && status.starts_with('R') {
                let old_path = PathBuf::from(parts[1].trim_matches('"'));
                let path = PathBuf::from(parts[2].trim_matches('"'));
                files.push(ChangedFile {
                    path,
                    change_type: ChangeType::Renamed,
                    old_path: Some(old_path),
                });
            }
        }
        Ok(files)
    }
}
