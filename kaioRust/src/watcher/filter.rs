use std::path::Path;

#[derive(Debug, Clone)]
pub struct PathFilter {
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl PathFilter {
    pub fn new(include: Option<&str>, exclude: Option<&str>, config_ignore: &[String]) -> Self {
        let mut includes = Vec::new();
        if let Some(inc) = include {
            for part in inc.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    includes.push(trimmed.to_string());
                }
            }
        }

        let mut excludes = Vec::new();
        // Default excludes
        let default_excludes = [
            "node_modules",
            "target",
            "dist",
            "build",
            ".git",
            ".agent",
            ".agent-docs",
            ".idea",
            ".vscode",
        ];
        for d in default_excludes {
            excludes.push(d.to_string());
        }

        for ig in config_ignore {
            if !excludes.contains(ig) {
                excludes.push(ig.clone());
            }
        }

        if let Some(exc) = exclude {
            for part in exc.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty() && !excludes.contains(&trimmed.to_string()) {
                    excludes.push(trimmed.to_string());
                }
            }
        }

        Self {
            include_patterns: includes,
            exclude_patterns: excludes,
        }
    }

    /// Check if a given file path should be processed
    pub fn should_process(&self, path: &Path, repo_root: &Path) -> bool {
        let rel_path = if let Ok(rel) = path.strip_prefix(repo_root) {
            rel
        } else {
            path
        };

        let path_str = rel_path.to_string_lossy().replace('\\', "/");

        // Skip temporary, swap, or backup files
        if path_str.ends_with('~')
            || path_str.ends_with(".swp")
            || path_str.ends_with(".tmp")
            || path_str.ends_with(".bak")
            || path_str.contains("/.#")
            || path_str.starts_with(".#")
        {
            return false;
        }

        // Check excludes
        for exc in &self.exclude_patterns {
            let exc_clean = exc.trim_matches('*').trim_matches('/');
            if exc_clean.is_empty() {
                continue;
            }
            if path_str.contains(exc_clean) {
                return false;
            }
        }

        // Check includes if specified
        if !self.include_patterns.is_empty() {
            for inc in &self.include_patterns {
                let inc_clean = inc.trim_start_matches('*').trim_start_matches('.');
                if path_str.ends_with(inc_clean) || path_str.contains(inc_clean) {
                    return true;
                }
            }
            return false;
        }

        // Default: match supported programming language files
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            matches!(
                ext.to_lowercase().as_str(),
                "rs" | "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "py"
            )
        } else {
            false
        }
    }
}
