use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::error::Result;
use super::languages::Language;

use super::tree_sitter::{AstParser, ExtractedSymbol};

pub const DEFAULT_MAX_FILE_SIZE_BYTES: usize = 1024 * 1024; // 1 MB
pub const BINARY_CHECK_BUFFER_SIZE: usize = 8192; // 8 KB

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParseStatus {
    Ok,
    Partial,
    Failed,
    Binary,
    Ignored,
    TooLarge,
}

impl ParseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ParseStatus::Ok => "ok",
            ParseStatus::Partial => "partial",
            ParseStatus::Failed => "failed",
            ParseStatus::Binary => "binary",
            ParseStatus::Ignored => "ignored",
            ParseStatus::TooLarge => "too_large",
        }
    }

    pub fn badge(&self) -> &'static str {
        match self {
            ParseStatus::Ok => "[OK]",
            ParseStatus::Partial => "[PARTIAL]",
            ParseStatus::Failed => "[FAIL]",
            ParseStatus::Binary => "[BIN]",
            ParseStatus::Ignored => "[IGN]",
            ParseStatus::TooLarge => "[LARGE]",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredFile {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub language: Language,
    pub size: usize,
    pub file_hash: Option<String>,
    pub parse_status: ParseStatus,
    pub last_modified: Option<String>,
    pub symbols: Vec<ExtractedSymbol>,
    pub imports_raw: Vec<String>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FileInventoryOptions {
    pub max_file_size_bytes: usize,
    pub ignore_patterns: Vec<String>,
    pub compute_hashes: bool,
    pub extract_ast: bool,
}

impl Default for FileInventoryOptions {
    fn default() -> Self {
        Self {
            max_file_size_bytes: DEFAULT_MAX_FILE_SIZE_BYTES,
            ignore_patterns: vec![
                "node_modules".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".git".to_string(),
                ".agent".to_string(),
                ".idea".to_string(),
                ".vscode".to_string(),
                ".kaio".to_string(),
                ".vercel".to_string(),
            ],
            compute_hashes: true,
            extract_ast: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InventorySummary {
    pub total_files: usize,
    pub total_symbols: usize,
    pub total_size_bytes: usize,
    pub files_by_language: BTreeMap<String, usize>,
    pub files_by_status: BTreeMap<String, usize>,
    pub files: Vec<DiscoveredFile>,
}

pub struct FileInventory;

impl FileInventory {
    pub fn scan_dir(root: &Path, ignore_patterns: &[String]) -> Result<InventorySummary> {
        let mut options = FileInventoryOptions::default();
        if !ignore_patterns.is_empty() {
            for pattern in ignore_patterns {
                if !options.ignore_patterns.contains(pattern) {
                    options.ignore_patterns.push(pattern.clone());
                }
            }
        }
        Self::scan_dir_with_options(root, &options)
    }

    pub fn scan_dir_with_options(
        root: &Path,
        options: &FileInventoryOptions,
    ) -> Result<InventorySummary> {
        let mut files = Vec::new();
        Self::walk(root, root, options, &mut files)?;

        // Sort files deterministically by relative path
        files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

        let total_files = files.len();
        let total_symbols = files.iter().map(|f| f.symbols.len()).sum();
        let total_size_bytes = files.iter().map(|f| f.size).sum();

        let mut files_by_language = BTreeMap::new();
        let mut files_by_status = BTreeMap::new();

        for file in &files {
            *files_by_language
                .entry(file.language.as_str().to_string())
                .or_insert(0) += 1;
            *files_by_status
                .entry(file.parse_status.as_str().to_string())
                .or_insert(0) += 1;
        }

        Ok(InventorySummary {
            total_files,
            total_symbols,
            total_size_bytes,
            files_by_language,
            files_by_status,
            files,
        })
    }

    pub fn is_ignored(path: &Path, base: &Path, ignore_patterns: &[String]) -> bool {
        let file_name = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        let rel_path = path
            .strip_prefix(base)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        for pat in ignore_patterns {
            let pat_clean = pat.trim().trim_end_matches('/');
            if pat_clean.is_empty() {
                continue;
            }
            // Exact file/directory name match
            if file_name == pat_clean {
                return true;
            }
            // Relative path component or prefix match
            if rel_path == pat_clean
                || rel_path.starts_with(&format!("{}/", pat_clean))
                || rel_path.ends_with(&format!("/{}", pat_clean))
                || rel_path.contains(&format!("/{}/", pat_clean))
            {
                return true;
            }
            // Simple wildcard e.g. *.lock, *.tmp, *.log
            if pat_clean.starts_with('*') && file_name.ends_with(&pat_clean[1..]) {
                return true;
            }
        }
        false
    }

    fn walk(
        base: &Path,
        current: &Path,
        options: &FileInventoryOptions,
        out: &mut Vec<DiscoveredFile>,
    ) -> Result<()> {
        if !current.exists() {
            return Ok(());
        }

        let entries = match fs::read_dir(current) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("Skipping directory {}: {}", current.display(), e);
                return Ok(());
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("Skipping entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();
            if Self::is_ignored(&path, base, &options.ignore_patterns) {
                continue;
            }

            if path.is_dir() {
                Self::walk(base, &path, options, out)?;
            } else if path.is_file() {
                let discovered = Self::inspect_file(base, &path, options)?;
                out.push(discovered);
            }
        }

        Ok(())
    }

    pub fn inspect_file(
        base: &Path,
        path: &Path,
        options: &FileInventoryOptions,
    ) -> Result<DiscoveredFile> {
        let metadata = fs::metadata(path)?;
        let size = metadata.len() as usize;
        let relative_path = path.strip_prefix(base).unwrap_or(path).to_path_buf();
        let language = Language::detect_from_path(path);

        let last_modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| {
                chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default()
            });

        // 1. Check size limit
        if size > options.max_file_size_bytes {
            return Ok(DiscoveredFile {
                path: path.to_path_buf(),
                relative_path,
                language,
                size,
                file_hash: None,
                parse_status: ParseStatus::TooLarge,
                last_modified,
                symbols: Vec::new(),
                imports_raw: Vec::new(),
                imports: Vec::new(),
                exports: Vec::new(),
            });
        }

        // 2. Binary pre-filter & content inspection
        let mut file = File::open(path)?;
        let mut head_buf = vec![0u8; BINARY_CHECK_BUFFER_SIZE.min(size)];
        let bytes_read = file.read(&mut head_buf)?;

        if Self::is_binary(&head_buf[..bytes_read]) {
            return Ok(DiscoveredFile {
                path: path.to_path_buf(),
                relative_path,
                language,
                size,
                file_hash: None,
                parse_status: ParseStatus::Binary,
                last_modified,
                symbols: Vec::new(),
                imports_raw: Vec::new(),
                imports: Vec::new(),
                exports: Vec::new(),
            });
        }

        // 3. Compute SHA-256 hash, validate UTF-8, and extract AST symbols
        let mut full_bytes = Vec::with_capacity(size);
        full_bytes.extend_from_slice(&head_buf[..bytes_read]);
        file.read_to_end(&mut full_bytes)?;

        let file_hash = if options.compute_hashes {
            let mut hasher = Sha256::new();
            hasher.update(&full_bytes);
            Some(format!("{:x}", hasher.finalize()))
        } else {
            None
        };

        let mut parse_status = ParseStatus::Ok;
        let mut symbols = Vec::new();
        let mut imports_raw = Vec::new();
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        match std::str::from_utf8(&full_bytes) {
            Ok(utf8_str) => {
                if options.extract_ast && language == Language::Rust {
                    match AstParser::parse_source(utf8_str, language) {
                        Ok(ast_res) => {
                            parse_status = ast_res.parse_status;
                            symbols = ast_res.symbols;
                            imports_raw = ast_res.imports_raw;
                            imports = ast_res.imports;
                            exports = ast_res.exports;
                        }
                        Err(e) => {
                            tracing::warn!("AST parsing failed for {}: {}", path.display(), e);
                            parse_status = ParseStatus::Partial;
                        }
                    }
                }
            }
            Err(_) => {
                parse_status = ParseStatus::Failed;
            }
        }

        Ok(DiscoveredFile {
            path: path.to_path_buf(),
            relative_path,
            language,
            size,
            file_hash,
            parse_status,
            last_modified,
            symbols,
            imports_raw,
            imports,
            exports,
        })
    }

    /// Check if byte slice contains null byte (0x00), indicating binary content.
    pub fn is_binary(bytes: &[u8]) -> bool {
        bytes.iter().any(|&b| b == 0x00)
    }
}
