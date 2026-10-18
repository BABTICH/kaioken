use std::collections::{HashMap, HashSet};
use std::path::Path;
use chrono::Utc;

use crate::error::Result;
use crate::health::report::{
    ArchitectureSummary, DeadCodeSummary, DocHealthSummary, FailureMemorySummary,
    GraphIntegritySummary, HealthReport, HealthScore, ModuleBreakdown, RepoOverview,
    ScoreDeduction, TestCoverageSummary, UnreferencedSymbol,
};
use crate::memory::MemoryEngine;

pub struct HealthAuditEngine;

impl HealthAuditEngine {
    pub fn audit(
        repo_root: &Path,
        memory: &MemoryEngine,
        _docs_root: &Path,
    ) -> Result<HealthReport> {
        let files = memory.list_files()?;
        let symbols = memory.get_all_symbols()?;
        let edges = memory.get_all_edges()?;
        let docs = memory.list_docs()?;
        let unresolved_failures = memory.get_unresolved_failures()?;
        let recent_failures = memory.get_recent_failures(50)?;
        let unresolved_imports = memory.get_unresolved_imports()?;

        let mut deductions = Vec::new();
        let mut score: i32 = 100;

        // 1. Overview & Language Analysis
        let mut languages: HashMap<String, usize> = HashMap::new();
        let mut test_files = HashSet::new();
        let mut impl_files = HashSet::new();
        let mut module_file_counts: HashMap<String, usize> = HashMap::new();
        let mut module_symbol_counts: HashMap<String, usize> = HashMap::new();

        let file_id_to_path: HashMap<i64, String> = files
            .iter()
            .filter_map(|f| f.id.map(|id| (id, f.path.clone())))
            .collect();

        for f in &files {
            let lang = f.language.clone().unwrap_or_else(|| "other".to_string());
            *languages.entry(lang).or_insert(0) += 1;

            let is_test = f.path.starts_with("tests/")
                || f.path.contains("_test")
                || f.path.contains(".test.")
                || f.path.contains(".spec.");

            if is_test {
                test_files.insert(f.path.clone());
            } else if f.path.starts_with("src/") {
                impl_files.insert(f.path.clone());
            }

            // Group module by top-level src component
            let parts: Vec<&str> = f.path.split('/').collect();
            let mod_name = if parts.len() >= 2 && parts[0] == "src" {
                format!("src/{}", parts[1])
            } else if !parts.is_empty() {
                parts[0].to_string()
            } else {
                "root".to_string()
            };
            *module_file_counts.entry(mod_name).or_insert(0) += 1;
        }

        for s in &symbols {
            if let Some(path) = file_id_to_path.get(&s.file_id) {
                let parts: Vec<&str> = path.split('/').collect();
                let mod_name = if parts.len() >= 2 && parts[0] == "src" {
                    format!("src/{}", parts[1])
                } else if !parts.is_empty() {
                    parts[0].to_string()
                } else {
                    "root".to_string()
                };
                *module_symbol_counts.entry(mod_name).or_insert(0) += 1;
            }
        }

        let mut modules: Vec<ModuleBreakdown> = module_file_counts
            .into_iter()
            .map(|(name, f_count)| {
                let s_count = module_symbol_counts.get(&name).copied().unwrap_or(0);
                ModuleBreakdown {
                    module_name: name,
                    file_count: f_count,
                    symbol_count: s_count,
                }
            })
            .collect();
        modules.sort_by(|a, b| b.file_count.cmp(&a.file_count));

        // 2. Documentation Health
        let mut fresh_docs = 0;
        let mut stale_docs = 0;
        let mut maybe_stale_docs = 0;
        let mut stale_doc_paths = Vec::new();
        let mut documented_file_ids = HashSet::new();

        for d in &docs {
            documented_file_ids.insert(d.file_id);
            match d.status.as_str() {
                "fresh" | "synced" => fresh_docs += 1,
                "stale" => {
                    stale_docs += 1;
                    stale_doc_paths.push(d.doc_path.clone());
                }
                "maybe_stale" => {
                    maybe_stale_docs += 1;
                    stale_doc_paths.push(d.doc_path.clone());
                }
                _ => fresh_docs += 1,
            }
        }

        let mut missing_doc_files = Vec::new();
        for f in &files {
            if f.path.starts_with("src/") && !f.path.contains("test") {
                if let Some(fid) = f.id {
                    if !documented_file_ids.contains(&fid) && f.doc_path.is_none() {
                        missing_doc_files.push(f.path.clone());
                    }
                }
            }
        }

        if stale_docs > 0 {
            let pts = (stale_docs as u32 * 5).min(20);
            deductions.push(ScoreDeduction {
                reason: format!("{} stale documentation file(s) require regeneration", stale_docs),
                points: pts,
            });
            score -= pts as i32;
        }

        if !missing_doc_files.is_empty() {
            let pts = (missing_doc_files.len() as u32 * 3).min(15);
            deductions.push(ScoreDeduction {
                reason: format!("{} source file(s) lack mirrored documentation", missing_doc_files.len()),
                points: pts,
            });
            score -= pts as i32;
        }

        // 3. Dead Code Detection
        let mut inbound_edge_counts: HashMap<i64, usize> = HashMap::new();
        let mut target_symbol_ids = HashSet::new();

        for e in &edges {
            if let Some(target_id) = e.target_symbol_id {
                *inbound_edge_counts.entry(target_id).or_insert(0) += 1;
                target_symbol_ids.insert(target_id);
            }
        }

        let mut unreferenced_symbols = Vec::new();
        for s in &symbols {
            if let Some(sid) = s.id {
                let inbound = inbound_edge_counts.get(&sid).copied().unwrap_or(0);
                let is_pub = s.visibility.as_deref() == Some("pub");
                let file_path = file_id_to_path.get(&s.file_id).cloned().unwrap_or_default();
                let is_entrypoint = file_path.ends_with("main.rs") || file_path.ends_with("lib.rs");

                // Flag internal non-entrypoint symbols with 0 inbound callers
                if inbound == 0 && !is_pub && !is_entrypoint && s.kind == "function" {
                    unreferenced_symbols.push(UnreferencedSymbol {
                        name: s.name.clone(),
                        kind: s.kind.clone(),
                        file_path,
                        line: s.line_start,
                    });
                }
            }
        }

        if !unreferenced_symbols.is_empty() {
            let pts = (unreferenced_symbols.len() as u32 * 2).min(15);
            deductions.push(ScoreDeduction {
                reason: format!("{} unreferenced internal symbol(s) detected as dead code candidates", unreferenced_symbols.len()),
                points: pts,
            });
            score -= pts as i32;
        }

        // 4. Test Coverage Hints
        let mut untested_files = Vec::new();
        for impl_file in &impl_files {
            let base_name = Path::new(impl_file)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            let has_matching_test = test_files.iter().any(|t| {
                t.contains(base_name)
            });

            if !has_matching_test && !impl_file.ends_with("main.rs") {
                untested_files.push(impl_file.clone());
            }
        }

        if !untested_files.is_empty() {
            let pts = (untested_files.len() as u32 * 2).min(15);
            deductions.push(ScoreDeduction {
                reason: format!("{} source file(s) lack corresponding test coverage", untested_files.len()),
                points: pts,
            });
            score -= pts as i32;
        }

        // 5. Graph Integrity
        if !unresolved_imports.is_empty() {
            let pts = (unresolved_imports.len() as u32 * 2).min(10);
            deductions.push(ScoreDeduction {
                reason: format!("{} unresolved raw import(s) detected in dependency graph", unresolved_imports.len()),
                points: pts,
            });
            score -= pts as i32;
        }

        // 6. Failure Memory
        if !unresolved_failures.is_empty() {
            let pts = (unresolved_failures.len() as u32 * 5).min(20);
            deductions.push(ScoreDeduction {
                reason: format!("{} unresolved compiler/test failure(s) pending resolution", unresolved_failures.len()),
                points: pts,
            });
            score -= pts as i32;
        }

        // Score bounding and grading
        let final_score = score.clamp(0, 100) as u32;
        let grade = match final_score {
            90..=100 => "A",
            75..=89 => "B",
            60..=74 => "C",
            40..=59 => "D",
            _ => "F",
        }.to_string();

        // 7. Actionable Recommendations
        let mut recommendations = Vec::new();
        if stale_docs > 0 {
            recommendations.push(format!("Run `kaioken docs generate --stale` to update {} stale doc(s).", stale_docs));
        }
        if !missing_doc_files.is_empty() {
            recommendations.push(format!("Run `kaioken docs generate` to create initial documentation mirrors for {} file(s).", missing_doc_files.len()));
        }
        if !unresolved_failures.is_empty() {
            recommendations.push(format!("Run `kaioken critic failures` to inspect and address {} active failure(s).", unresolved_failures.len()));
        }
        if !untested_files.is_empty() {
            recommendations.push(format!("Add unit tests under `tests/` for {} untested module(s).", untested_files.len()));
        }
        if !unreferenced_symbols.is_empty() {
            recommendations.push(format!("Review and remove or test {} dead code symbol candidate(s).", unreferenced_symbols.len()));
        }

        Ok(HealthReport {
            generated_at: Utc::now().to_rfc3339(),
            repo_root: repo_root.display().to_string(),
            score: HealthScore {
                score: final_score,
                grade,
                deductions,
            },
            overview: RepoOverview {
                total_files: files.len(),
                total_symbols: symbols.len(),
                total_edges: edges.len(),
                total_docs: docs.len(),
                total_failures: recent_failures.len(),
                languages,
            },
            architecture: ArchitectureSummary {
                modules,
                core_dependencies: vec!["rusqlite".into(), "serde".into(), "clap".into()],
                max_dependency_depth: 3,
            },
            doc_health: DocHealthSummary {
                total_docs: docs.len(),
                fresh_count: fresh_docs,
                stale_count: stale_docs,
                maybe_stale_count: maybe_stale_docs,
                missing_docs_count: missing_doc_files.len(),
                stale_doc_paths,
                missing_doc_files,
            },
            dead_code: DeadCodeSummary {
                unreferenced_symbols,
                orphan_files: Vec::new(),
            },
            test_coverage: TestCoverageSummary {
                untested_files,
                untested_public_symbols: Vec::new(),
                test_file_count: test_files.len(),
                implementation_file_count: impl_files.len(),
            },
            graph_integrity: GraphIntegritySummary {
                total_edges: edges.len(),
                unresolved_imports: unresolved_imports.clone(),
                broken_link_count: unresolved_imports.len(),
            },
            failure_memory: FailureMemorySummary {
                unresolved_count: unresolved_failures.len(),
                resolved_count: recent_failures.len() - unresolved_failures.len(),
                unresolved_failures: unresolved_failures.iter().map(|f| {
                    format!("[{}] {}", f.error_type, f.message)
                }).collect(),
            },
            recommendations,
        })
    }
}
