use crate::docs::frontmatter::DocFrontmatter;

#[derive(Debug, Clone)]
pub struct SymbolDocEntry {
    pub name: String,
    pub kind: String,
    pub visibility: String,
    pub signature: Option<String>,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Debug, Clone)]
pub struct CommitDocEntry {
    pub hash_short: String,
    pub message: String,
    pub author: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileDocTemplate {
    pub frontmatter: DocFrontmatter,
    pub file_path: String,
    pub language: String,
    pub purpose: String,
    pub public_api: Vec<SymbolDocEntry>,
    pub internal_symbols: Vec<SymbolDocEntry>,
    pub dependencies: Vec<String>,
    pub used_by: Vec<String>,
    pub tests: Vec<String>,
    pub recent_changes: Vec<CommitDocEntry>,
    pub risks: Vec<String>,
    pub notes: Option<String>,
}

impl FileDocTemplate {
    pub fn render(&self) -> String {
        let mut out = String::new();
        // 1. YAML frontmatter header
        out.push_str(&self.frontmatter.to_yaml_header());

        // 2. Title & Language header
        out.push_str(&format!("# {}\n\n", self.file_path));
        out.push_str(&format!(
            "> **Language:** {} | **Level:** {} | **Status:** {}\n\n",
            self.language, self.frontmatter.doc_level, self.frontmatter.status
        ));

        // 3. Purpose
        out.push_str("## Purpose\n\n");
        out.push_str(&format!("{}\n\n", self.purpose));

        // 4. Public API
        out.push_str("## Public API\n\n");
        if self.public_api.is_empty() {
            out.push_str("*(No public symbols exported)*\n\n");
        } else {
            for api in &self.public_api {
                if let Some(ref sig) = api.signature {
                    out.push_str(&format!(
                        "- `{}` ({} in L{}-L{})\n",
                        sig, api.kind, api.line_start, api.line_end
                    ));
                } else {
                    out.push_str(&format!(
                        "- `{}` ({} in L{}-L{})\n",
                        api.name, api.kind, api.line_start, api.line_end
                    ));
                }
            }
            out.push('\n');
        }

        // 5. Internal Symbols
        if !self.internal_symbols.is_empty() {
            out.push_str("## Internal Symbols\n\n");
            for sym in &self.internal_symbols {
                out.push_str(&format!(
                    "- `{}` ({} in L{}-L{})\n",
                    sym.name, sym.kind, sym.line_start, sym.line_end
                ));
            }
            out.push('\n');
        }

        // 6. Dependencies / Imports
        out.push_str("## Dependencies\n\n");
        if self.dependencies.is_empty() {
            out.push_str("*(No external dependencies detected)*\n\n");
        } else {
            for dep in &self.dependencies {
                out.push_str(&format!("- `{}`\n", dep));
            }
            out.push('\n');
        }

        // 7. Used By / Direct Dependents
        out.push_str("## Used By\n\n");
        if self.used_by.is_empty() {
            out.push_str("*(Direct dependents unknown or none)*\n\n");
        } else {
            for u in &self.used_by {
                out.push_str(&format!("- `{}`\n", u));
            }
            out.push('\n');
        }

        // 8. Associated Tests
        out.push_str("## Tests\n\n");
        if self.tests.is_empty() {
            out.push_str("*(No linked unit tests detected)*\n\n");
        } else {
            for t in &self.tests {
                out.push_str(&format!("- `{}`\n", t));
            }
            out.push('\n');
        }

        // 9. Recent Changes
        out.push_str("## Recent Changes\n\n");
        if self.recent_changes.is_empty() {
            out.push_str("*(Initial indexing / No recent commit changes)*\n\n");
        } else {
            for c in &self.recent_changes {
                let date_str = c.date.as_deref().unwrap_or("recent");
                out.push_str(&format!(
                    "- Commit `{}` ({}): {}\n",
                    c.hash_short, date_str, c.message
                ));
            }
            out.push('\n');
        }

        // 10. Risks
        out.push_str("## Risks\n\n");
        if self.risks.is_empty() {
            out.push_str("*(No elevated risk patterns identified)*\n\n");
        } else {
            for r in &self.risks {
                out.push_str(&format!("- ⚠️ {}\n", r));
            }
            out.push('\n');
        }

        if let Some(notes) = &self.notes {
            out.push_str("## Notes\n\n");
            out.push_str(&format!("{}\n\n", notes));
        }

        out
    }
}
