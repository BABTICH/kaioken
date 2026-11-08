use crate::error::{KaiokenError, Result};
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Option<i64>,
    pub actor: String,  // 'agent' | 'user'
    pub action: String, // 'edit_file' | 'bash' | 'create_file' | 'read_file', etc.
    pub target: String, // file path or command string
    pub approved: bool, // true = allowed, false = blocked
    pub reason: Option<String>,
    pub created_at: String,
}

impl AuditLogEntry {
    pub fn severity(&self) -> &'static str {
        if !self.approved {
            if let Some(r) = &self.reason {
                let lower = r.to_lowercase();
                if lower.contains("critical")
                    || lower.contains("blocked command")
                    || lower.contains("denied")
                {
                    return "critical";
                }
            }
            "warn"
        } else {
            "info"
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditFilter {
    pub limit: usize,
    pub since: Option<String>,
    pub event_type: Option<String>,
    pub severity: Option<String>,
    pub run_id: Option<String>,
    pub tool_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolStat {
    pub name: String,
    pub count: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_events: usize,
    pub tool_calls: usize,
    pub safety_blocks: usize,
    pub approvals: usize,
    pub rejections: usize,
    pub critical_events: usize,
    pub top_tools: Vec<ToolStat>,
    pub agent_runs: usize,
    pub avg_events_per_run: f64,
    pub last_event: Option<String>,
}

impl AuditStats {
    pub fn render_terminal(&self) -> String {
        let mut out = String::new();
        out.push_str("📊 Audit Statistics\n");
        out.push_str("═══════════════════\n");
        out.push_str(&format!("  Total Events:        {}\n", self.total_events));
        out.push_str(&format!("  Tool Calls:          {}\n", self.tool_calls));
        out.push_str(&format!("  Safety Blocks:       {}\n", self.safety_blocks));
        out.push_str(&format!("  Approvals:           {}\n", self.approvals));
        out.push_str(&format!("  Rejections:          {}\n", self.rejections));
        out.push_str(&format!(
            "  Critical Events:     {}\n\n",
            self.critical_events
        ));

        out.push_str("  Top Tools:\n");
        if self.top_tools.is_empty() {
            out.push_str("    (None)\n");
        } else {
            for t in &self.top_tools {
                out.push_str(&format!(
                    "    {:<18} {} ({:.1}%)\n",
                    t.name, t.count, t.percentage
                ));
            }
        }

        out.push_str(&format!("\n  Agent Runs:          {}\n", self.agent_runs));
        out.push_str(&format!(
            "  Avg Events/Run:      {:.1}\n",
            self.avg_events_per_run
        ));
        out.push_str(&format!(
            "  Last Event:          {}\n",
            self.last_event.as_deref().unwrap_or("None")
        ));

        out
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AuditQueryResult {
    pub events: Vec<AuditLogEntry>,
    pub total: usize,
    pub filtered: usize,
}

impl AuditQueryResult {
    pub fn render_terminal(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("📋 Audit Log — Last {} Events\n", self.filtered));
        out.push_str("═══════════════════════════════════════════════════════════════════════\n\n");

        if self.events.is_empty() {
            out.push_str("  No audit events matching criteria found.\n\n");
        } else {
            for (i, entry) in self.events.iter().enumerate() {
                let sev_str = match entry.severity() {
                    "critical" => "[CRITICAL]",
                    "warn" => "[WARN]",
                    _ => "[INFO]",
                };

                let status_str = if entry.approved { "Allowed" } else { "Blocked" };

                out.push_str(&format!(
                    "  #{:<2} {}  {:<10} {}\n",
                    i + 1,
                    entry.created_at,
                    sev_str,
                    entry.action
                ));
                out.push_str(&format!(
                    "      Target: {}  |  Actor: {}  |  Status: {}\n",
                    entry.target, entry.actor, status_str
                ));
                if let Some(reason) = &entry.reason {
                    out.push_str(&format!("      Detail: {}\n", reason));
                }
                out.push('\n');
            }
        }

        out.push_str("═══════════════════════════════════════════════════════════════════════\n");
        out.push_str(&format!(
            "  Showing {} of {} total events\n",
            self.filtered, self.total
        ));

        out
    }
}

#[derive(Clone, Default)]
pub struct AuditLogger {
    log_file_path: Option<PathBuf>,
    db_path: Option<PathBuf>,
    shared_conn: Option<Arc<Mutex<rusqlite::Connection>>>,
}

impl AuditLogger {
    pub fn new(repo_root: Option<&Path>) -> Self {
        let log_file_path = repo_root.map(|r| r.join(".agent").join("logs").join("audit.log"));
        let db_path = repo_root.map(|r| r.join(".agent").join("memory.sqlite"));
        Self {
            log_file_path,
            db_path,
            shared_conn: None,
        }
    }

    pub fn with_db_path(mut self, db_path: PathBuf) -> Self {
        self.db_path = Some(db_path);
        self
    }

    pub fn with_shared_conn(mut self, conn: Arc<Mutex<rusqlite::Connection>>) -> Self {
        self.shared_conn = Some(conn);
        self
    }

    pub fn with_log_file(mut self, log_path: PathBuf) -> Self {
        self.log_file_path = Some(log_path);
        self
    }

    /// Logs an action to both file disk log and SQLite audit_logs table.
    pub fn log(
        &self,
        actor: &str,
        action: &str,
        target: &str,
        approved: bool,
        reason: Option<&str>,
    ) -> Result<()> {
        let entry = AuditLogEntry {
            id: None,
            actor: actor.to_string(),
            action: action.to_string(),
            target: target.to_string(),
            approved,
            reason: reason.map(|s| s.to_string()),
            created_at: Utc::now().to_rfc3339(),
        };

        let approved_int = if approved { 1 } else { 0 };

        // 1. Write to SQLite audit_logs table via shared connection or db path
        if let Some(shared) = &self.shared_conn {
            if let Ok(conn) = shared.lock() {
                let _ = conn.execute(
                    "INSERT INTO audit_logs (actor, action, target, approved, reason, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        entry.actor,
                        entry.action,
                        entry.target,
                        approved_int,
                        entry.reason,
                        entry.created_at,
                    ],
                );
            }
        } else if let Some(p) = &self.db_path {
            if p.exists() {
                if let Ok(conn) = rusqlite::Connection::open(p) {
                    let _ = conn.execute(
                        "INSERT INTO audit_logs (actor, action, target, approved, reason, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![
                            entry.actor,
                            entry.action,
                            entry.target,
                            approved_int,
                            entry.reason,
                            entry.created_at,
                        ],
                    );
                }
            }
        }

        // 2. Append to .agent/logs/audit.log
        if let Some(path) = &self.log_file_path {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                if let Ok(json_line) = serde_json::to_string(&entry) {
                    let _ = writeln!(file, "{}", json_line);
                }
            }
        }

        Ok(())
    }

    /// Internal helper to execute a query closure across shared or file-backed SQLite
    fn execute_on_conn<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<R>,
        R: Default,
    {
        if let Some(shared) = &self.shared_conn {
            let conn = shared.lock().unwrap();
            f(&conn)
        } else if let Some(p) = &self.db_path {
            if p.exists() {
                let conn = rusqlite::Connection::open(p)
                    .map_err(|e| KaiokenError::MemoryError(format!("Failed to open DB: {}", e)))?;
                f(&conn)
            } else {
                Ok(R::default())
            }
        } else {
            Ok(R::default())
        }
    }

    /// Retrieves recent audit log entries from SQLite database.
    pub fn list_recent(&self, limit: usize) -> Result<Vec<AuditLogEntry>> {
        let filter = AuditFilter {
            limit,
            ..Default::default()
        };
        let res = self.query(&filter)?;
        Ok(res.events)
    }

    /// Query audit log entries with multi-attribute filtering, pagination, and secret redaction
    pub fn query(&self, filter: &AuditFilter) -> Result<AuditQueryResult> {
        let redactor = crate::safety::redact::RedactionEngine::new();

        self.execute_on_conn(|conn| {
            let total: usize = conn
                .query_row("SELECT count(*) FROM audit_logs", [], |r| r.get(0))
                .unwrap_or(0);

            let mut sql = String::from(
                "SELECT id, actor, action, target, approved, reason, created_at FROM audit_logs WHERE 1=1"
            );
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(since) = &filter.since {
                sql.push_str(" AND created_at >= ?");
                params.push(Box::new(since.clone()));
            }

            if let Some(evt) = &filter.event_type {
                sql.push_str(" AND (lower(action) = lower(?) OR lower(target) LIKE lower(?) OR lower(reason) LIKE lower(?))");
                params.push(Box::new(evt.clone()));
                params.push(Box::new(format!("%{}%", evt)));
                params.push(Box::new(format!("%{}%", evt)));
            }

            if let Some(tool) = &filter.tool_name {
                sql.push_str(" AND (lower(action) = lower(?) OR lower(target) LIKE lower(?))");
                params.push(Box::new(tool.clone()));
                params.push(Box::new(format!("%{}%", tool)));
            }

            if let Some(sev) = &filter.severity {
                match sev.to_lowercase().as_str() {
                    "warn" | "warning" => {
                        sql.push_str(" AND approved = 0");
                    }
                    "info" => {
                        sql.push_str(" AND approved = 1");
                    }
                    "critical" => {
                        sql.push_str(" AND approved = 0 AND (lower(reason) LIKE '%critical%' OR lower(reason) LIKE '%blocked command%' OR lower(reason) LIKE '%denied%')");
                    }
                    _ => {}
                }
            }

            if let Some(run) = &filter.run_id {
                sql.push_str(" AND (lower(reason) LIKE lower(?) OR lower(target) LIKE lower(?))");
                params.push(Box::new(format!("%{}%", run)));
                params.push(Box::new(format!("%{}%", run)));
            }

            sql.push_str(" ORDER BY id DESC LIMIT ?");
            let limit = if filter.limit > 0 { filter.limit } else { 50 };
            params.push(Box::new(limit as i64));

            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            let mut stmt = conn
                .prepare(&sql)
                .map_err(|e| KaiokenError::MemoryError(format!("Failed to prepare query: {}", e)))?;

            let rows = stmt
                .query_map(param_refs.as_slice(), |row| {
                    let approved_int: i32 = row.get(4)?;
                    Ok(AuditLogEntry {
                        id: Some(row.get(0)?),
                        actor: row.get(1)?,
                        action: row.get(2)?,
                        target: row.get(3)?,
                        approved: approved_int == 1,
                        reason: row.get(5)?,
                        created_at: row.get(6)?,
                    })
                })
                .map_err(|e| KaiokenError::MemoryError(format!("Failed to execute query: {}", e)))?;

            let mut events = Vec::new();
            for r in rows.flatten() {
                let mut entry = r;
                entry.target = redactor.redact(&entry.target);
                if let Some(reason) = &entry.reason {
                    entry.reason = Some(redactor.redact(reason));
                }
                events.push(entry);
            }

            let filtered = events.len();
            Ok(AuditQueryResult {
                events,
                total,
                filtered,
            })
        })
    }

    /// Compute summary statistics from SQLite audit_logs
    pub fn compute_stats(&self) -> Result<AuditStats> {
        self.execute_on_conn(|conn| {
            let total_events: usize = conn
                .query_row("SELECT count(*) FROM audit_logs", [], |r| r.get(0))
                .unwrap_or(0);

            if total_events == 0 {
                return Ok(AuditStats::default());
            }

            let safety_blocks: usize = conn
                .query_row("SELECT count(*) FROM audit_logs WHERE approved = 0", [], |r| r.get(0))
                .unwrap_or(0);

            let approvals: usize = conn
                .query_row("SELECT count(*) FROM audit_logs WHERE approved = 1 AND (actor = 'user' OR action = 'approval')", [], |r| r.get(0))
                .unwrap_or(0);

            let rejections: usize = conn
                .query_row("SELECT count(*) FROM audit_logs WHERE approved = 0 AND actor = 'user'", [], |r| r.get(0))
                .unwrap_or(0);

            let critical_events: usize = conn
                .query_row(
                    "SELECT count(*) FROM audit_logs WHERE approved = 0 AND (lower(reason) LIKE '%critical%' OR lower(reason) LIKE '%blocked command%' OR lower(reason) LIKE '%denied%')",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(0);

            let tool_calls: usize = conn
                .query_row(
                    "SELECT count(*) FROM audit_logs WHERE lower(action) IN ('read_file', 'edit_file', 'create_file', 'bash', 'glob', 'grep', 'git', 'task', 'tool_call')",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(0);

            let mut stmt = conn
                .prepare("SELECT action, count(*) as c FROM audit_logs GROUP BY action ORDER BY c DESC LIMIT 5")
                .map_err(|e| KaiokenError::MemoryError(format!("Failed to prepare stats query: {}", e)))?;

            let rows = stmt
                .query_map([], |row| {
                    let name: String = row.get(0)?;
                    let count: i64 = row.get(1)?;
                    Ok((name, count as usize))
                })
                .map_err(|e| KaiokenError::MemoryError(format!("Failed to query tool breakdown: {}", e)))?;

            let mut top_tools = Vec::new();
            for r in rows.flatten() {
                let pct = if total_events > 0 {
                    (r.1 as f64 / total_events as f64) * 100.0
                } else {
                    0.0
                };
                top_tools.push(ToolStat {
                    name: r.0,
                    count: r.1,
                    percentage: (pct * 10.0).round() / 10.0,
                });
            }

            let agent_runs: usize = conn
                .query_row("SELECT count(DISTINCT id) FROM agent_runs", [], |r| r.get(0))
                .unwrap_or(1);

            let avg_events_per_run = if agent_runs > 0 {
                ((total_events as f64 / agent_runs as f64) * 10.0).round() / 10.0
            } else {
                0.0
            };

            let last_event: Option<String> = conn
                .query_row("SELECT created_at FROM audit_logs ORDER BY id DESC LIMIT 1", [], |r| r.get(0))
                .ok();

            Ok(AuditStats {
                total_events,
                tool_calls,
                safety_blocks,
                approvals,
                rejections,
                critical_events,
                top_tools,
                agent_runs,
                avg_events_per_run,
                last_event,
            })
        })
    }

    /// Parse disk-based audit log (`.agent/logs/audit.log`), supporting both JSON lines and text formats
    pub fn parse_disk_log(path: &Path, filter: &AuditFilter) -> Result<AuditQueryResult> {
        if !path.exists() {
            return Ok(AuditQueryResult {
                events: Vec::new(),
                total: 0,
                filtered: 0,
            });
        }

        let redactor = crate::safety::redact::RedactionEngine::new();
        let content = std::fs::read_to_string(path)?;
        let mut all_entries = Vec::new();

        let text_regex = regex::Regex::new(
            r#"^\[(?P<ts>[^\]]+)\] \[(?P<lvl>[^\]]+)\] (?P<action>[^:]+): (?P<details>.+)$"#,
        )
        .unwrap();
        let mut id_counter = 1;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with('{') {
                if let Ok(mut entry) = serde_json::from_str::<AuditLogEntry>(trimmed) {
                    if entry.id.is_none() {
                        entry.id = Some(id_counter);
                        id_counter += 1;
                    }
                    all_entries.push(entry);
                    continue;
                }
            }

            if let Some(caps) = text_regex.captures(trimmed) {
                let ts = caps.name("ts").map_or("", |m| m.as_str()).to_string();
                let lvl = caps.name("lvl").map_or("INFO", |m| m.as_str());
                let action = caps
                    .name("action")
                    .map_or("unknown", |m| m.as_str())
                    .to_string();
                let details = caps.name("details").map_or("", |m| m.as_str()).to_string();

                let approved = lvl != "WARN" && lvl != "ERROR" && lvl != "CRITICAL";
                let reason = if !approved {
                    Some(details.clone())
                } else {
                    None
                };

                all_entries.push(AuditLogEntry {
                    id: Some(id_counter),
                    actor: "agent".to_string(),
                    action,
                    target: details,
                    approved,
                    reason,
                    created_at: ts,
                });
                id_counter += 1;
            }
        }

        let total = all_entries.len();

        let filtered_entries: Vec<AuditLogEntry> = all_entries
            .into_iter()
            .rev()
            .filter(|e| {
                if let Some(since) = &filter.since {
                    if e.created_at < *since {
                        return false;
                    }
                }
                if let Some(evt) = &filter.event_type {
                    let lower_evt = evt.to_lowercase();
                    if !e.action.to_lowercase().contains(&lower_evt)
                        && !e.target.to_lowercase().contains(&lower_evt)
                        && !e
                            .reason
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&lower_evt)
                    {
                        return false;
                    }
                }
                if let Some(tool) = &filter.tool_name {
                    let lower_tool = tool.to_lowercase();
                    if !e.action.to_lowercase().contains(&lower_tool)
                        && !e.target.to_lowercase().contains(&lower_tool)
                    {
                        return false;
                    }
                }
                if let Some(sev) = &filter.severity {
                    match sev.to_lowercase().as_str() {
                        "warn" | "warning" => {
                            if e.approved {
                                return false;
                            }
                        }
                        "info" => {
                            if !e.approved {
                                return false;
                            }
                        }
                        "critical" if e.approved || e.severity() != "critical" => {
                            return false;
                        }
                        _ => {}
                    }
                }
                if let Some(run) = &filter.run_id {
                    let lower_run = run.to_lowercase();
                    if !e.target.to_lowercase().contains(&lower_run)
                        && !e
                            .reason
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&lower_run)
                    {
                        return false;
                    }
                }
                true
            })
            .take(if filter.limit > 0 { filter.limit } else { 50 })
            .map(|mut e| {
                e.target = redactor.redact(&e.target);
                if let Some(r) = &e.reason {
                    e.reason = Some(redactor.redact(r));
                }
                e
            })
            .collect();

        let filtered = filtered_entries.len();
        Ok(AuditQueryResult {
            events: filtered_entries,
            total,
            filtered,
        })
    }

    /// Compute statistics in-memory from a slice of AuditLogEntry
    pub fn stats_from_entries(entries: &[AuditLogEntry]) -> AuditStats {
        let total_events = entries.len();
        if total_events == 0 {
            return AuditStats::default();
        }

        let mut tool_calls = 0;
        let mut safety_blocks = 0;
        let mut approvals = 0;
        let mut rejections = 0;
        let mut critical_events = 0;
        let mut tool_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for e in entries {
            if !e.approved {
                safety_blocks += 1;
                if e.actor == "user" {
                    rejections += 1;
                }
                if e.severity() == "critical" {
                    critical_events += 1;
                }
            } else if e.actor == "user" || e.action == "approval" {
                approvals += 1;
            }

            let is_tool = matches!(
                e.action.to_lowercase().as_str(),
                "read_file"
                    | "edit_file"
                    | "create_file"
                    | "bash"
                    | "glob"
                    | "grep"
                    | "git"
                    | "task"
                    | "tool_call"
            );
            if is_tool {
                tool_calls += 1;
            }

            *tool_counts.entry(e.action.clone()).or_insert(0) += 1;
        }

        let mut sorted_tools: Vec<(String, usize)> = tool_counts.into_iter().collect();
        sorted_tools.sort_by_key(|a| std::cmp::Reverse(a.1));

        let top_tools: Vec<ToolStat> = sorted_tools
            .into_iter()
            .take(5)
            .map(|(name, count)| {
                let pct = (count as f64 / total_events as f64) * 100.0;
                ToolStat {
                    name,
                    count,
                    percentage: (pct * 10.0).round() / 10.0,
                }
            })
            .collect();

        let last_event = entries.first().map(|e| e.created_at.clone());

        AuditStats {
            total_events,
            tool_calls,
            safety_blocks,
            approvals,
            rejections,
            critical_events,
            top_tools,
            agent_runs: 1,
            avg_events_per_run: total_events as f64,
            last_event,
        }
    }
}
