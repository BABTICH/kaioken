use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocFrontmatter {
    pub source_path: String,
    pub source_hash: String,
    pub doc_level: u8,
    pub generated_at: String,
    pub generator: String, // "deterministic" or "llm"
    pub status: String,    // "fresh", "stale", "maybe_stale"
}

impl DocFrontmatter {
    pub fn to_yaml_header(&self) -> String {
        format!(
            "---\nsource_path: {}\nsource_hash: {}\ndoc_level: {}\ngenerated_at: {}\ngenerator: {}\nstatus: {}\n---\n\n",
            self.source_path,
            self.source_hash,
            self.doc_level,
            self.generated_at,
            self.generator,
            self.status,
        )
    }

    pub fn parse_from_markdown(content: &str) -> Option<(Self, &str)> {
        let trimmed = content.trim_start();
        if !trimmed.starts_with("---") {
            return None;
        }

        // Find closing "---"
        let after_first = &trimmed[3..];
        let after_first_trimmed = after_first.trim_start_matches(|c| c == '\r' || c == '\n');
        let close_idx = after_first_trimmed.find("---")?;
        let yaml_block = &after_first_trimmed[..close_idx];
        let body = after_first_trimmed[close_idx + 3..].trim_start_matches(|c| c == '\r' || c == '\n');

        let mut source_path = None;
        let mut source_hash = None;
        let mut doc_level = 0;
        let mut generated_at = None;
        let mut generator = "deterministic".to_string();
        let mut status = "fresh".to_string();

        for line in yaml_block.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some((k, v)) = line.split_once(':') {
                let key = k.trim();
                let val = v.trim().trim_matches('"').trim_matches('\'');
                match key {
                    "source_path" => source_path = Some(val.to_string()),
                    "source_hash" => source_hash = Some(val.to_string()),
                    "doc_level" => {
                        if let Ok(lvl) = val.parse::<u8>() {
                            doc_level = lvl;
                        }
                    }
                    "generated_at" => generated_at = Some(val.to_string()),
                    "generator" => generator = val.to_string(),
                    "status" => status = val.to_string(),
                    _ => {}
                }
            }
        }

        let frontmatter = DocFrontmatter {
            source_path: source_path?,
            source_hash: source_hash.unwrap_or_default(),
            doc_level,
            generated_at: generated_at.unwrap_or_default(),
            generator,
            status,
        };

        Some((frontmatter, body))
    }
}
