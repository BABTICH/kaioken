use std::collections::HashSet;
use crate::error::Result;
use crate::memory::MemoryEngine;
use crate::retrieval::context_packet::FailureMemorySnippet;

pub struct FailureLookup<'a> {
    memory: &'a MemoryEngine,
}

impl<'a> FailureLookup<'a> {
    pub fn new(memory: &'a MemoryEngine) -> Self {
        Self { memory }
    }

    /// Finds relevant failure memory records matching targeted files, symbols, or general context.
    pub fn find_failures(
        &self,
        file_ids: &[i64],
        symbol_ids: &[i64],
        task_query: &str,
        limit: usize,
    ) -> Result<Vec<FailureMemorySnippet>> {
        let mut results = Vec::new();
        let mut seen_ids = HashSet::new();

        // 1. Check specific symbols
        for &sym_id in symbol_ids {
            let failures = self.memory.get_failures_for_symbol(sym_id)?;
            for f in failures {
                if let Some(id) = f.id {
                    if seen_ids.insert(id) {
                        let (file_path, sym_name) = self.resolve_provenance(f.file_id, f.symbol_id);
                        results.push(FailureMemorySnippet {
                            error_type: f.error_type,
                            message: f.message,
                            file_path,
                            symbol_name: sym_name.or(Some("target_symbol".to_string())),
                            fix_summary: f.fix_summary,
                            created_at: f.created_at,
                        });
                    }
                }
            }
        }

        // 2. Check specific files
        for &fid in file_ids {
            let failures = self.memory.get_failures_for_file(fid)?;
            for f in failures {
                if let Some(id) = f.id {
                    if seen_ids.insert(id) {
                        let (file_path, sym_name) = self.resolve_provenance(f.file_id, f.symbol_id);
                        results.push(FailureMemorySnippet {
                            error_type: f.error_type,
                            message: f.message,
                            file_path,
                            symbol_name: sym_name,
                            fix_summary: f.fix_summary,
                            created_at: f.created_at,
                        });
                    }
                }
            }
        }

        // 3. Fallback: match task query against recent failures
        let recent = self.memory.get_recent_failures(20)?;
        let query_terms: Vec<&str> = task_query.split_whitespace().collect();

        for f in recent {
            if let Some(id) = f.id {
                if seen_ids.contains(&id) {
                    continue;
                }

                let matches_query = query_terms.iter().any(|term| {
                    if term.len() < 3 {
                        return false;
                    }
                    f.message.to_lowercase().contains(&term.to_lowercase())
                        || f.error_type.to_lowercase().contains(&term.to_lowercase())
                });

                if matches_query {
                    seen_ids.insert(id);
                    let (file_path, sym_name) = self.resolve_provenance(f.file_id, f.symbol_id);
                    results.push(FailureMemorySnippet {
                        error_type: f.error_type,
                        message: f.message,
                        file_path,
                        symbol_name: sym_name,
                        fix_summary: f.fix_summary,
                        created_at: f.created_at,
                    });
                }
            }
        }

        results.truncate(limit);
        Ok(results)
    }

    fn resolve_provenance(&self, file_id: Option<i64>, symbol_id: Option<i64>) -> (Option<String>, Option<String>) {
        let file_path = file_id.and_then(|id| {
            self.memory.get_file_by_id(id).ok().flatten().map(|f| f.path)
        });

        let symbol_name = symbol_id.and_then(|id| {
            let conn = self.memory.db().connection();
            conn.query_row(
                "SELECT name FROM symbols WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            ).ok()
        });

        (file_path, symbol_name)
    }
}
