use std::path::Path;
use regex::Regex;

/// RedactionEngine sanitizes API keys, private keys, bearer tokens,
/// and credentials before context packets or audit logs are stored or emitted.
pub struct RedactionEngine {
    api_key_regex: Regex,
    openrouter_key_regex: Regex,
    github_token_regex: Regex,
    aws_key_regex: Regex,
    bearer_token_regex: Regex,
    private_key_regex: Regex,
    slack_token_regex: Regex,
    uri_password_regex: Regex,
    env_secret_regex: Regex,
}

impl Default for RedactionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RedactionEngine {
    pub fn new() -> Self {
        Self {
            api_key_regex: Regex::new(r"sk-[a-zA-Z0-9_\-]{20,}").unwrap(),
            openrouter_key_regex: Regex::new(r"sk-or-v1-[a-zA-Z0-9]{20,}").unwrap(),
            github_token_regex: Regex::new(r"(ghp|gho|ghu|ghs|ghr|github_pat)_[a-zA-Z0-9_]{20,}").unwrap(),
            aws_key_regex: Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
            bearer_token_regex: Regex::new(r"(?i)bearer\s+[a-zA-Z0-9_\-\.]{20,}").unwrap(),
            private_key_regex: Regex::new(r"-----BEGIN [A-Z0-9_ ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z0-9_ ]*PRIVATE KEY-----").unwrap(),
            slack_token_regex: Regex::new(r"xox[baprs]-[0-9a-zA-Z]{10,}").unwrap(),
            uri_password_regex: Regex::new(r"://([^:]+):([^@]+)@").unwrap(),
            env_secret_regex: Regex::new(r#"([A-Z0-9_]*(?:KEY|SECRET|TOKEN|PASSWORD|AUTH|CREDENTIALS)[A-Z0-9_]*\s*=\s*)(['"][^'"\[]{6,}['"]|[^'"\s\r\n\[]{6,})"#).unwrap(),
        }
    }

    /// Redacts all detected credentials and secrets in the provided text.
    pub fn redact(&self, input: &str) -> String {
        let mut result = input.to_string();

        result = self.private_key_regex.replace_all(&result, "[REDACTED_PRIVATE_KEY]").to_string();
        result = self.openrouter_key_regex.replace_all(&result, "[REDACTED_API_KEY]").to_string();
        result = self.api_key_regex.replace_all(&result, "[REDACTED_API_KEY]").to_string();
        result = self.github_token_regex.replace_all(&result, "[REDACTED_GITHUB_TOKEN]").to_string();
        result = self.aws_key_regex.replace_all(&result, "[REDACTED_AWS_KEY]").to_string();
        result = self.slack_token_regex.replace_all(&result, "[REDACTED_SLACK_TOKEN]").to_string();
        result = self.bearer_token_regex.replace_all(&result, "Bearer [REDACTED_BEARER_TOKEN]").to_string();
        result = self.uri_password_regex.replace_all(&result, "://$1:[REDACTED_PASSWORD]@").to_string();
        result = self.env_secret_regex.replace_all(&result, "$1[REDACTED_SECRET]").to_string();

        result
    }

    /// Checks if a file path is considered a sensitive or credential file
    /// that must not be ingested into retrieval context.
    pub fn is_sensitive_file(path: &Path) -> bool {
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_lowercase(),
            None => return false,
        };

        if file_name == ".env" || file_name.starts_with(".env.") {
            return true;
        }

        if file_name == "credentials.json"
            || file_name == "service-account.json"
            || file_name == "id_rsa"
            || file_name == "id_ed25519"
            || file_name == "id_dsa"
        {
            return true;
        }

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            if ext_lower == "pem" || ext_lower == "key" || ext_lower == "pfx" || ext_lower == "p12" {
                return true;
            }
        }

        false
    }

    /// Validates whether a file path is within the workspace boundary
    /// and does not attempt directory traversal outside workspace root.
    pub fn is_within_workspace(path: &Path, workspace_root: &Path) -> bool {
        // Strip relative components or canonicalize
        let norm_workspace = match workspace_root.canonicalize() {
            Ok(p) => p,
            Err(_) => workspace_root.to_path_buf(),
        };

        let target = if path.is_absolute() {
            match path.canonicalize() {
                Ok(p) => p,
                Err(_) => path.to_path_buf(),
            }
        } else {
            match workspace_root.join(path).canonicalize() {
                Ok(p) => p,
                Err(_) => workspace_root.join(path),
            }
        };

        target.starts_with(&norm_workspace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_secrets() {
        let engine = RedactionEngine::new();
        
        let text = "Here is my key: sk-abcdef1234567890abcdef123456 and token: ghp_123456789012345678901234567890123456";
        let redacted = engine.redact(text);
        assert!(!redacted.contains("sk-abcdef"));
        assert!(!redacted.contains("ghp_123456"));
        assert!(redacted.contains("[REDACTED_API_KEY]"));
        assert!(redacted.contains("[REDACTED_GITHUB_TOKEN]"));

        let openrouter_text = "OPENROUTER_API_KEY=sk-or-v1-abcdef1234567890abcdef1234567890";
        let redacted_or = engine.redact(openrouter_text);
        assert!(!redacted_or.contains("sk-or-v1-abcdef"));
        assert!(redacted_or.contains("[REDACTED_SECRET]") || redacted_or.contains("[REDACTED_API_KEY]"));

        let pem_text = "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0\n-----END RSA PRIVATE KEY-----";
        let redacted_pem = engine.redact(pem_text);
        assert!(redacted_pem.contains("[REDACTED_PRIVATE_KEY]"));
    }

    #[test]
    fn test_sensitive_files() {
        assert!(RedactionEngine::is_sensitive_file(Path::new(".env")));
        assert!(RedactionEngine::is_sensitive_file(Path::new(".env.local")));
        assert!(RedactionEngine::is_sensitive_file(Path::new("secret.pem")));
        assert!(RedactionEngine::is_sensitive_file(Path::new("credentials.json")));
        assert!(!RedactionEngine::is_sensitive_file(Path::new("src/main.rs")));
    }
}
