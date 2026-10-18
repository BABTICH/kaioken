use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompilerDiagnostic {
    pub file_path: Option<String>,
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub error_code: Option<String>, // e.g. "E0308", "E0425"
    pub message: String,
    pub severity: String, // "error" | "warning"
    pub snippet: Option<String>,
}

impl CompilerDiagnostic {
    pub fn format_summary(&self) -> String {
        let loc = match (&self.file_path, self.line) {
            (Some(f), Some(l)) => format!("{}:{}", f, l),
            (Some(f), None) => f.clone(),
            (None, _) => "unknown location".to_string(),
        };

        let code_str = match &self.error_code {
            Some(c) => format!("[{}] ", c),
            None => String::new(),
        };

        format!("- {}: {}{}{}", loc, self.severity, if !code_str.is_empty() { format!(" {}", code_str) } else { ": ".to_string() }, self.message)
    }
}

pub struct DiagnosticParser;

impl DiagnosticParser {
    /// Parses raw compiler / linter / test runner output into structured diagnostics.
    pub fn parse(output: &str) -> Vec<CompilerDiagnostic> {
        let mut diagnostics = Vec::new();

        // 1. Regex for Rust rustc structured errors:
        // error[E0308]: mismatched types
        //   --> src/math.rs:14:5
        let rust_err_re = Regex::new(
            r"(?m)^(error|warning)(?:\[([A-Z0-9]+)\])?:\s*([^\r\n]+)\r?\n\s*-->\s*([^:\r\n]+):(\d+):(\d+)"
        ).unwrap();

        for cap in rust_err_re.captures_iter(output) {
            let severity = cap.get(1).map(|m| m.as_str().to_lowercase()).unwrap_or_else(|| "error".to_string());
            let error_code = cap.get(2).map(|m| m.as_str().to_string());
            let message = cap.get(3).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
            let file_path = cap.get(4).map(|m| m.as_str().replace('\\', "/"));
            let line = cap.get(5).and_then(|m| m.as_str().parse::<usize>().ok());
            let col = cap.get(6).and_then(|m| m.as_str().parse::<usize>().ok());

            diagnostics.push(CompilerDiagnostic {
                file_path,
                line,
                col,
                error_code,
                message,
                severity,
                snippet: None,
            });
        }

        // 2. Regex for standard file:line:col error format (e.g. gcc, python, typescript):
        // src/math.rs:14:5: error: mismatched types
        // src/App.tsx:14:5 - error TS2322: Type 'string' is not assignable to type 'number'.
        let standard_re = Regex::new(
            r"(?m)^([a-zA-Z0-9_\-\./\\]+\.(?:rs|ts|tsx|js|jsx|py|go|c|cpp|h)):(\d+)(?::(\d+))?(?: -)?\s*(error|warning)(?:[ :]+([A-Z0-9]+))?:?\s*([^\r\n]+)"
        ).unwrap();

        for cap in standard_re.captures_iter(output) {
            let file_path = cap.get(1).map(|m| m.as_str().replace('\\', "/"));
            let line = cap.get(2).and_then(|m| m.as_str().parse::<usize>().ok());
            let col = cap.get(3).and_then(|m| m.as_str().parse::<usize>().ok());
            let severity = cap.get(4).map(|m| m.as_str().to_lowercase()).unwrap_or_else(|| "error".to_string());
            let error_code = cap.get(5).map(|m| m.as_str().to_string());
            let message = cap.get(6).map(|m| m.as_str().trim().to_string()).unwrap_or_default();

            // Avoid duplicates if already captured
            let already_exists = diagnostics.iter().any(|d| {
                d.file_path == file_path && d.line == line && d.message == message
            });

            if !already_exists {
                diagnostics.push(CompilerDiagnostic {
                    file_path,
                    line,
                    col,
                    error_code,
                    message,
                    severity,
                    snippet: None,
                });
            }
        }

        // 3. Regex for test panics / assertion failures:
        // Format A: thread 'main' panicked at 'assertion failed: `(left == right)`', tests/unit_tests.rs:55:9
        let panic_quoted_re = Regex::new(
            r"(?s)panicked at\s+'(.*?)',\s*([^:\r\n]+):(\d+):(\d+)"
        ).unwrap();

        for cap in panic_quoted_re.captures_iter(output) {
            let message = cap.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
            let file_path = cap.get(2).map(|m| m.as_str().replace('\\', "/"));
            let line = cap.get(3).and_then(|m| m.as_str().parse::<usize>().ok());
            let col = cap.get(4).and_then(|m| m.as_str().parse::<usize>().ok());

            diagnostics.push(CompilerDiagnostic {
                file_path,
                line,
                col,
                error_code: Some("PANIC".to_string()),
                message,
                severity: "panic".to_string(),
                snippet: None,
            });
        }

        // Format B: thread 'test_add' panicked at tests/math_tests.rs:12:5: assertion `left == right` failed
        let panic_loc_re = Regex::new(
            r"(?m)panicked at ([^:\r\n]+):(\d+):(\d+):\s*([^\r\n]+)"
        ).unwrap();

        for cap in panic_loc_re.captures_iter(output) {
            let file_path = cap.get(1).map(|m| m.as_str().replace('\\', "/"));
            let line = cap.get(2).and_then(|m| m.as_str().parse::<usize>().ok());
            let col = cap.get(3).and_then(|m| m.as_str().parse::<usize>().ok());
            let message = cap.get(4).map(|m| m.as_str().trim().to_string()).unwrap_or_default();

            diagnostics.push(CompilerDiagnostic {
                file_path,
                line,
                col,
                error_code: Some("PANIC".to_string()),
                message,
                severity: "panic".to_string(),
                snippet: None,
            });
        }

        diagnostics
    }

    /// Formats a list of diagnostics into a markdown feedback block for the LLM.
    pub fn format_feedback_block(command: &str, exit_code: i32, diagnostics: &[CompilerDiagnostic], raw_snippet: &str) -> String {
        let mut out = format!(
            "⚠️ CRITIC FEEDBACK: Deterministic verification failed\n\
             Command: `{}` (Exit Code: {})\n\n\
             The following compiler/test failures were detected:\n",
            command, exit_code
        );

        if diagnostics.is_empty() {
            out.push_str("- Build/Test failed with non-zero exit code.\n\n");
        } else {
            for diag in diagnostics {
                out.push_str(&diag.format_summary());
                out.push('\n');
            }
            out.push('\n');
        }

        if !raw_snippet.trim().is_empty() {
            out.push_str("Diagnostic Output:\n```text\n");
            // Limit snippet to ~15 lines for prompt efficiency
            let snippet_lines: Vec<&str> = raw_snippet.lines().take(25).collect();
            out.push_str(&snippet_lines.join("\n"));
            out.push_str("\n```\n\n");
        }

        out.push_str("Please inspect these errors, apply necessary code changes using `edit_file`, and ensure the code compiles cleanly.");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rustc_error() {
        let rustc_output = r#"
error[E0308]: mismatched types
  --> src/math.rs:14:5
   |
14 |     n
   |     ^ expected `u64`, found `i32`
"#;
        let diags = DiagnosticParser::parse(rustc_output);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].file_path.as_deref(), Some("src/math.rs"));
        assert_eq!(diags[0].line, Some(14));
        assert_eq!(diags[0].col, Some(5));
        assert_eq!(diags[0].error_code.as_deref(), Some("E0308"));
        assert_eq!(diags[0].message, "mismatched types");
        assert_eq!(diags[0].severity, "error");
    }

    #[test]
    fn test_parse_test_panic() {
        let test_output = r#"
---- test_fib stdout ----
thread 'test_fib' panicked at tests/fib_tests.rs:25:9:
assertion `left == right` failed
  left: 0
 right: 1
"#;
        let diags = DiagnosticParser::parse(test_output);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].file_path.as_deref(), Some("tests/fib_tests.rs"));
        assert_eq!(diags[0].line, Some(25));
        assert!(diags[0].message.contains("assertion `left == right` failed"));
    }
}
