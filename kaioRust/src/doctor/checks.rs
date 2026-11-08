use super::{CheckStatus, DiagnosticCheck};
use crate::config::Config;
use crate::scanner::git::GitScanner;
use crate::scanner::languages::Language;
use crate::scanner::lock::STALE_LOCK_TIMEOUT_SECS;
use crate::scanner::tree_sitter::AstParser;
use std::path::Path;
use std::process::Command;

/// 1. Validate Git availability and version (>= 2.25)
pub fn check_git_availability() -> DiagnosticCheck {
    let output = match Command::new("git").arg("--version").output() {
        Ok(out) => out,
        Err(_) => {
            return DiagnosticCheck {
                name: "git_availability".to_string(),
                display_name: "Git Availability".to_string(),
                status: CheckStatus::Fail,
                detail: "git not found in PATH".to_string(),
            };
        }
    };

    if !output.status.success() {
        return DiagnosticCheck {
            name: "git_availability".to_string(),
            display_name: "Git Availability".to_string(),
            status: CheckStatus::Fail,
            detail: "git --version exited with non-zero status".to_string(),
        };
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ver_str = parse_git_version(stdout.trim());
    let (major, minor) = parse_version_numbers(&ver_str);

    let is_supported = major > 2 || (major == 2 && minor >= 25);
    if is_supported {
        DiagnosticCheck {
            name: "git_availability".to_string(),
            display_name: "Git Availability".to_string(),
            status: CheckStatus::Pass,
            detail: format!("git {}", ver_str),
        }
    } else {
        DiagnosticCheck {
            name: "git_availability".to_string(),
            display_name: "Git Availability".to_string(),
            status: CheckStatus::Fail,
            detail: format!("git {} (< 2.25 required)", ver_str),
        }
    }
}

/// 2. Validate current directory is inside a Git repository
pub fn check_git_repository(workspace_root: &Path) -> DiagnosticCheck {
    match GitScanner::find_repo_root(workspace_root) {
        Ok(root) => {
            let normalized = root.to_string_lossy().replace('\\', "/");
            DiagnosticCheck {
                name: "git_repository".to_string(),
                display_name: "Git Repository".to_string(),
                status: CheckStatus::Pass,
                detail: normalized,
            }
        }
        Err(_) => DiagnosticCheck {
            name: "git_repository".to_string(),
            display_name: "Git Repository".to_string(),
            status: CheckStatus::Fail,
            detail: "Not inside a Git working tree".to_string(),
        },
    }
}

/// 3. Validate Rust toolchain (`rustc` and `cargo`)
pub fn check_rust_toolchain() -> DiagnosticCheck {
    let rustc_out = Command::new("rustc").arg("--version").output();
    let cargo_out = Command::new("cargo").arg("--version").output();

    match (rustc_out, cargo_out) {
        (Ok(r), Ok(c)) if r.status.success() && c.status.success() => {
            let r_str = String::from_utf8_lossy(&r.stdout);
            let short_r = r_str.split('(').next().unwrap_or(r_str.trim()).trim();
            DiagnosticCheck {
                name: "rust_toolchain".to_string(),
                display_name: "Rust Toolchain".to_string(),
                status: CheckStatus::Pass,
                detail: format!("{} (cargo operational)", short_r),
            }
        }
        (Ok(r), _) if r.status.success() => {
            let r_str = String::from_utf8_lossy(&r.stdout);
            let short_r = r_str.split('(').next().unwrap_or(r_str.trim()).trim();
            DiagnosticCheck {
                name: "rust_toolchain".to_string(),
                display_name: "Rust Toolchain".to_string(),
                status: CheckStatus::Warn,
                detail: format!("{} (cargo missing or failed)", short_r),
            }
        }
        _ => DiagnosticCheck {
            name: "rust_toolchain".to_string(),
            display_name: "Rust Toolchain".to_string(),
            status: CheckStatus::Warn,
            detail: "rustc/cargo not found in PATH (not required for all operations)".to_string(),
        },
    }
}

/// 4. Validate SQLite runtime & database connectivity
pub fn check_sqlite_runtime(workspace_root: &Path) -> DiagnosticCheck {
    let db_path = workspace_root.join(".agent").join("memory.sqlite");
    let sqlite_ver = rusqlite::version();

    if db_path.exists() {
        match rusqlite::Connection::open(&db_path) {
            Ok(conn) => {
                let journal_mode: String = conn
                    .query_row("PRAGMA journal_mode;", [], |r| r.get(0))
                    .unwrap_or_else(|_| "unknown".to_string());
                DiagnosticCheck {
                    name: "sqlite_runtime".to_string(),
                    display_name: "SQLite Runtime".to_string(),
                    status: CheckStatus::Pass,
                    detail: format!(
                        ".agent/memory.sqlite ({}, {})",
                        journal_mode.to_uppercase(),
                        sqlite_ver
                    ),
                }
            }
            Err(e) => DiagnosticCheck {
                name: "sqlite_runtime".to_string(),
                display_name: "SQLite Runtime".to_string(),
                status: CheckStatus::Fail,
                detail: format!("Failed to open .agent/memory.sqlite: {}", e),
            },
        }
    } else {
        // Test in-memory capability
        match rusqlite::Connection::open_in_memory() {
            Ok(_) => DiagnosticCheck {
                name: "sqlite_runtime".to_string(),
                display_name: "SQLite Runtime".to_string(),
                status: CheckStatus::Pass,
                detail: format!("SQLite runtime operational (WAL ready, {})", sqlite_ver),
            },
            Err(e) => DiagnosticCheck {
                name: "sqlite_runtime".to_string(),
                display_name: "SQLite Runtime".to_string(),
                status: CheckStatus::Fail,
                detail: format!("SQLite in-memory test failed: {}", e),
            },
        }
    }
}

/// 5. Validate SQLite schema completeness
pub fn check_sqlite_schema(workspace_root: &Path) -> DiagnosticCheck {
    let db_path = workspace_root.join(".agent").join("memory.sqlite");

    if !db_path.exists() {
        return DiagnosticCheck {
            name: "sqlite_schema".to_string(),
            display_name: "SQLite Schema".to_string(),
            status: CheckStatus::Warn,
            detail:
                "Database not yet initialized at .agent/memory.sqlite (run `kaioken memory init`)"
                    .to_string(),
        };
    }

    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            return DiagnosticCheck {
                name: "sqlite_schema".to_string(),
                display_name: "SQLite Schema".to_string(),
                status: CheckStatus::Fail,
                detail: format!("Cannot read tables: {}", e),
            };
        }
    };

    let expected_tables = [
        "files",
        "symbols",
        "import_raw",
        "edges",
        "commits",
        "commit_changes",
        "docs",
        "failures",
        "decisions",
        "agent_runs",
        "tool_calls",
        "audit_logs",
        "retrieval_cache",
    ];

    let mut stmt = match conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';")
    {
        Ok(s) => s,
        Err(e) => {
            return DiagnosticCheck {
                name: "sqlite_schema".to_string(),
                display_name: "SQLite Schema".to_string(),
                status: CheckStatus::Fail,
                detail: format!("Failed to query tables: {}", e),
            };
        }
    };

    let existing: Vec<String> = stmt
        .query_map([], |r| r.get(0))
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default();

    let missing: Vec<&str> = expected_tables
        .iter()
        .copied()
        .filter(|t| !existing.iter().any(|e| e == t))
        .collect();

    if missing.is_empty() {
        DiagnosticCheck {
            name: "sqlite_schema".to_string(),
            display_name: "SQLite Schema".to_string(),
            status: CheckStatus::Pass,
            detail: format!(
                "{}/{} tables present",
                expected_tables.len(),
                expected_tables.len()
            ),
        }
    } else {
        DiagnosticCheck {
            name: "sqlite_schema".to_string(),
            display_name: "SQLite Schema".to_string(),
            status: CheckStatus::Fail,
            detail: format!("Missing tables: {:?} (run `kaioken memory init`)", missing),
        }
    }
}

/// 6. Validate Tree-sitter Rust parser operational status
pub fn check_treesitter_rust() -> DiagnosticCheck {
    match AstParser::parse_source("fn main() {}", Language::Rust) {
        Ok(res) if !res.has_syntax_errors && !res.symbols.is_empty() => DiagnosticCheck {
            name: "treesitter_rust".to_string(),
            display_name: "Tree-sitter Rust".to_string(),
            status: CheckStatus::Pass,
            detail: "Parser operational".to_string(),
        },
        Ok(_) => DiagnosticCheck {
            name: "treesitter_rust".to_string(),
            display_name: "Tree-sitter Rust".to_string(),
            status: CheckStatus::Fail,
            detail: "Parser produced unexpected AST on trivial snippet".to_string(),
        },
        Err(e) => DiagnosticCheck {
            name: "treesitter_rust".to_string(),
            display_name: "Tree-sitter Rust".to_string(),
            status: CheckStatus::Fail,
            detail: format!("Parser failed: {}", e),
        },
    }
}

/// 7. Tree-sitter TypeScript/JavaScript support
pub fn check_treesitter_ts_js() -> DiagnosticCheck {
    DiagnosticCheck {
        name: "treesitter_ts_js".to_string(),
        display_name: "Tree-sitter TS/JS".to_string(),
        status: CheckStatus::Warn,
        detail: "Not yet installed (multi-lang feature, see 17_multi_language_parsers.md)"
            .to_string(),
    }
}

/// 8. Tree-sitter Python support
pub fn check_treesitter_python() -> DiagnosticCheck {
    DiagnosticCheck {
        name: "treesitter_python".to_string(),
        display_name: "Tree-sitter Python".to_string(),
        status: CheckStatus::Warn,
        detail: "Not yet installed (multi-lang feature, see 17_multi_language_parsers.md)"
            .to_string(),
    }
}

/// 9. Validate `.agent/config.toml` presence and validity
pub fn check_config_file(workspace_root: &Path) -> DiagnosticCheck {
    let config_path = workspace_root.join(".agent").join("config.toml");
    if config_path.exists() {
        match Config::load(workspace_root) {
            Ok(_) => DiagnosticCheck {
                name: "config_file".to_string(),
                display_name: "Config File".to_string(),
                status: CheckStatus::Pass,
                detail: ".agent/config.toml valid".to_string(),
            },
            Err(e) => DiagnosticCheck {
                name: "config_file".to_string(),
                display_name: "Config File".to_string(),
                status: CheckStatus::Warn,
                detail: format!("Invalid config at .agent/config.toml: {}", e),
            },
        }
    } else {
        DiagnosticCheck {
            name: "config_file".to_string(),
            display_name: "Config File".to_string(),
            status: CheckStatus::Warn,
            detail: ".agent/config.toml not found (using defaults)".to_string(),
        }
    }
}

/// 10. Validate OPENROUTER_API_KEY environment variable
pub fn check_api_key_openrouter() -> DiagnosticCheck {
    match std::env::var("OPENROUTER_API_KEY") {
        Ok(key) if !key.trim().is_empty() => {
            let prefix = if key.len() >= 8 { &key[..8] } else { &key[..] };
            DiagnosticCheck {
                name: "api_key_openrouter".to_string(),
                display_name: "API Key (OpenRouter)".to_string(),
                status: CheckStatus::Pass,
                detail: format!("Configured (length: {}, prefix: {}...)", key.len(), prefix),
            }
        }
        _ => DiagnosticCheck {
            name: "api_key_openrouter".to_string(),
            display_name: "API Key (OpenRouter)".to_string(),
            status: CheckStatus::Warn,
            detail: "Not set — agent tasks will be unavailable".to_string(),
        },
    }
}

/// 11. Validate `.agent-docs/` directory status
pub fn check_docs_directory(workspace_root: &Path) -> DiagnosticCheck {
    let docs_dir = workspace_root.join(".agent-docs");
    if docs_dir.exists() && docs_dir.is_dir() {
        let mut file_count = 0;
        count_files_recursive(&docs_dir, &mut file_count);
        DiagnosticCheck {
            name: "docs_directory".to_string(),
            display_name: "Docs Directory".to_string(),
            status: CheckStatus::Pass,
            detail: format!(".agent-docs/ exists ({} files)", file_count),
        }
    } else {
        DiagnosticCheck {
            name: "docs_directory".to_string(),
            display_name: "Docs Directory".to_string(),
            status: CheckStatus::Warn,
            detail: ".agent-docs/ does not exist (will be created on first docs generate)"
                .to_string(),
        }
    }
}

/// 12. Validate absence of stale lockfile
pub fn check_lockfile_safety(workspace_root: &Path) -> DiagnosticCheck {
    let lock_path = workspace_root.join(".agent").join("scan.lock");
    if !lock_path.exists() {
        DiagnosticCheck {
            name: "lockfile_safety".to_string(),
            display_name: "Lockfile Safety".to_string(),
            status: CheckStatus::Pass,
            detail: "No stale lock".to_string(),
        }
    } else {
        let age_secs = if let Ok(metadata) = std::fs::metadata(&lock_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = std::time::SystemTime::now().duration_since(modified) {
                    elapsed.as_secs()
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        };

        if age_secs > STALE_LOCK_TIMEOUT_SECS {
            DiagnosticCheck {
                name: "lockfile_safety".to_string(),
                display_name: "Lockfile Safety".to_string(),
                status: CheckStatus::Warn,
                detail: format!(
                    "Stale scan lock detected (age: {}s, > {}s)",
                    age_secs, STALE_LOCK_TIMEOUT_SECS
                ),
            }
        } else {
            DiagnosticCheck {
                name: "lockfile_safety".to_string(),
                display_name: "Lockfile Safety".to_string(),
                status: CheckStatus::Warn,
                detail: format!("Active scan lock held (age: {}s)", age_secs),
            }
        }
    }
}

/// 13. Validate available disk space on workspace drive
pub fn check_disk_space(workspace_root: &Path) -> DiagnosticCheck {
    match get_available_disk_space(workspace_root) {
        Some(bytes) => {
            let mb = bytes as f64 / 1_048_576.0;
            let gb = bytes as f64 / 1_073_741_824.0;
            let formatted = if gb >= 1.0 {
                format!("{:.1} GB free", gb)
            } else {
                format!("{:.1} MB free", mb)
            };

            if bytes > 100 * 1024 * 1024 {
                DiagnosticCheck {
                    name: "disk_space".to_string(),
                    display_name: "Disk Space".to_string(),
                    status: CheckStatus::Pass,
                    detail: formatted,
                }
            } else {
                DiagnosticCheck {
                    name: "disk_space".to_string(),
                    display_name: "Disk Space".to_string(),
                    status: CheckStatus::Warn,
                    detail: format!("Low disk space: {} (< 100 MB recommended)", formatted),
                }
            }
        }
        None => DiagnosticCheck {
            name: "disk_space".to_string(),
            display_name: "Disk Space".to_string(),
            status: CheckStatus::Pass,
            detail: "Disk space check skipped (platform unsupported)".to_string(),
        },
    }
}

// ==========================================
// Internal Helpers
// ==========================================

fn parse_git_version(text: &str) -> String {
    for part in text.split_whitespace() {
        let trimmed = part.trim_matches(|c: char| !c.is_numeric() && c != '.');
        if !trimmed.is_empty()
            && trimmed.contains('.')
            && trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
        {
            return trimmed.to_string();
        }
    }
    "unknown".to_string()
}

fn parse_version_numbers(ver: &str) -> (u32, u32) {
    let parts: Vec<&str> = ver.split('.').collect();
    let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor)
}

fn count_files_recursive(dir: &Path, count: &mut usize) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                *count += 1;
            } else if path.is_dir() {
                count_files_recursive(&path, count);
            }
        }
    }
}

#[cfg(windows)]
fn get_available_disk_space(dir: &Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;

    // Use absolute path or fallback to current dir
    let abs_path = if dir.is_absolute() {
        dir.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_else(|_| dir.to_path_buf())
    };

    let mut path_u16: Vec<u16> = abs_path.as_os_str().encode_wide().collect();
    path_u16.push(0);

    #[link(name = "kernel32")]
    extern "system" {
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }

    let mut free_bytes_available: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free_bytes: u64 = 0;

    let ret = unsafe {
        GetDiskFreeSpaceExW(
            path_u16.as_ptr(),
            &mut free_bytes_available,
            &mut total_bytes,
            &mut total_free_bytes,
        )
    };

    if ret != 0 {
        Some(free_bytes_available)
    } else {
        None
    }
}

#[cfg(not(windows))]
fn get_available_disk_space(_dir: &Path) -> Option<u64> {
    None
}
