use std::fs;
use std::path::Path;
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

use crate::docs::frontmatter::DocFrontmatter;
use crate::error::Result;
use crate::memory::{DocRecord, MemoryEngine};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocStalenessStatus {
    Fresh,
    MissingSource,
    MissingDocFile,
    HashMismatch {
        doc_hash: String,
        source_hash: String,
    },
    CorruptedFrontmatter(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocVerificationItem {
    pub file_path: String,
    pub doc_path: String,
    pub status: DocStalenessStatus,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocVerificationReport {
    pub total_docs: usize,
    pub valid_docs: usize,
    pub stale_docs: usize,
    pub orphaned_docs: usize,
    pub cleaned_docs: usize,
    pub items: Vec<DocVerificationItem>,
}

pub struct StalenessDetector;

impl StalenessDetector {
    /// Verify a single documentation record against the filesystem.
    pub fn verify_doc(
        doc: &DocRecord,
        repo_root: &Path,
        docs_root: &Path,
        memory: &MemoryEngine,
    ) -> Result<DocVerificationItem> {
        let file_rec = memory.get_file_by_id(doc.file_id)?;

        let (file_path_str, source_full_path) = match file_rec {
            Some(ref f) => (f.path.clone(), repo_root.join(&f.path)),
            None => {
                return Ok(DocVerificationItem {
                    file_path: "unknown".to_string(),
                    doc_path: doc.doc_path.clone(),
                    status: DocStalenessStatus::MissingSource,
                    is_valid: false,
                });
            }
        };

        if !source_full_path.exists() {
            return Ok(DocVerificationItem {
                file_path: file_path_str,
                doc_path: doc.doc_path.clone(),
                status: DocStalenessStatus::MissingSource,
                is_valid: false,
            });
        }

        let doc_full_path = docs_root.join(&doc.doc_path);
        if !doc_full_path.exists() {
            return Ok(DocVerificationItem {
                file_path: file_path_str,
                doc_path: doc.doc_path.clone(),
                status: DocStalenessStatus::MissingDocFile,
                is_valid: false,
            });
        }

        // Read and parse frontmatter from markdown file
        let doc_content = fs::read_to_string(&doc_full_path)?;
        let (frontmatter, _) = match DocFrontmatter::parse_from_markdown(&doc_content) {
            Some(pair) => pair,
            None => {
                return Ok(DocVerificationItem {
                    file_path: file_path_str,
                    doc_path: doc.doc_path.clone(),
                    status: DocStalenessStatus::CorruptedFrontmatter("Missing or invalid YAML header".to_string()),
                    is_valid: false,
                });
            }
        };

        // Compute current SHA-256 hash of the source file on disk
        let source_content = fs::read(&source_full_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&source_content);
        let actual_source_hash = format!("{:x}", hasher.finalize());

        if frontmatter.source_hash != actual_source_hash {
            return Ok(DocVerificationItem {
                file_path: file_path_str,
                doc_path: doc.doc_path.clone(),
                status: DocStalenessStatus::HashMismatch {
                    doc_hash: frontmatter.source_hash,
                    source_hash: actual_source_hash,
                },
                is_valid: false,
            });
        }

        Ok(DocVerificationItem {
            file_path: file_path_str,
            doc_path: doc.doc_path.clone(),
            status: DocStalenessStatus::Fresh,
            is_valid: true,
        })
    }

    /// Audit all documentation in SQLite against the repository on disk.
    pub fn audit_all(
        repo_root: &Path,
        docs_root: &Path,
        memory: &MemoryEngine,
    ) -> Result<DocVerificationReport> {
        Self::audit_and_clean(repo_root, docs_root, memory, false)
    }

    /// Audit all documentation in SQLite against the repository on disk, with optional orphan cleanup.
    pub fn audit_and_clean(
        repo_root: &Path,
        docs_root: &Path,
        memory: &MemoryEngine,
        clean: bool,
    ) -> Result<DocVerificationReport> {
        let docs = memory.list_docs()?;
        let mut report = DocVerificationReport::default();
        report.total_docs = docs.len();

        for d in docs {
            match Self::verify_doc(&d, repo_root, docs_root, memory) {
                Ok(item) => {
                    if item.is_valid {
                        report.valid_docs += 1;
                    } else {
                        match item.status {
                            DocStalenessStatus::MissingSource => {
                                report.orphaned_docs += 1;
                                if clean {
                                    // Remove orphaned doc markdown file from disk
                                    let doc_full = docs_root.join(&d.doc_path);
                                    let _ = fs::remove_file(&doc_full);
                                    // Remove orphaned doc record from SQLite
                                    if let Some(doc_id) = d.id {
                                        let _ = memory.delete_doc_by_id(doc_id);
                                    }
                                    report.cleaned_docs += 1;
                                }
                            }
                            _ => report.stale_docs += 1,
                        }
                    }
                    report.items.push(item);
                }
                Err(e) => {
                    tracing::warn!("Failed to verify doc {}: {}", d.doc_path, e);
                }
            }
        }

        Ok(report)
    }
}
