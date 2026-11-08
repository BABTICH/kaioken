# 13. `kaioken audit show` — Audit Log Inspector

> **Status**: 🟢 Implemented (Phase 18)
> **CLI Entry**: `kaioken audit show`, `kaioken audit stats`
> **Source Modules**: `src/cli/audit.rs`, `src/safety/audit.rs`, `src/safety/redact.rs`
> **Tests**: `tests/phase18_audit_tests.rs` (8 unit/integration tests)

---

## 1. Purpose & Motivation

`kaioken audit show` provides a **human-readable and machine-queryable interface** to inspect the agent's audit trail. Every tool call, safety intercept, approval decision, and permission escalation is recorded in both SQLite (`audit_logs` table) and disk (`.agent/logs/audit.log`). This command surfaces that data for debugging, compliance, and trust-building.

Without a dedicated viewer, users must manually `cat` log files or write SQL queries — `audit show` abstracts this into an ergonomic CLI.

---

## 2. Audit Data Sources

Kaioken records audit data in two locations:

### 2.1 SQLite `audit_logs` Table (Structured)

```sql
CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    event_type TEXT NOT NULL,       -- 'tool_call', 'safety_block', 'approval', 'rejection', etc.
    tool_name TEXT,                 -- 'read_file', 'edit_file', 'bash', etc.
    action TEXT NOT NULL,           -- Human-readable description
    details TEXT,                   -- JSON blob with extra context
    agent_run_id TEXT,              -- Links to agent_runs.id
    severity TEXT DEFAULT 'info'    -- 'info', 'warn', 'critical'
);
```

### 2.2 Disk Log `.agent/logs/audit.log` (Append-Only)

```text
[2026-10-19T09:15:00Z] [INFO] tool_call: read_file path="src/main.rs" run_id="abc-123"
[2026-10-19T09:15:01Z] [INFO] tool_call: edit_file path="src/lib.rs" run_id="abc-123"
[2026-10-19T09:15:02Z] [WARN] safety_block: path_traversal blocked path="../../../etc/passwd" run_id="abc-123"
[2026-10-19T09:15:03Z] [INFO] approval: user approved bash command "cargo test" run_id="abc-123"
```

---

## 3. CLI Interface

```text
kaioken audit
├── show                    # Display recent audit events
│   ├── --last <N>          # Show last N events (default: 50)
│   ├── --since <datetime>  # Filter events after ISO 8601 timestamp
│   ├── --type <event_type> # Filter by event type (tool_call, safety_block, approval, etc.)
│   ├── --severity <level>  # Filter by severity (info, warn, critical)
│   ├── --run <run_id>      # Filter by specific agent run
│   ├── --tool <tool_name>  # Filter by tool name
│   ├── --json              # Output as JSON array
│   └── --source <src>      # Read from 'sqlite' (default) or 'disk'
└── stats                   # Summary statistics of audit events
```

### 3.1 CLI Args

```rust
#[derive(Subcommand)]
pub enum AuditCommands {
    /// Display recent audit log events
    Show(AuditShowArgs),
    /// Display audit summary statistics
    Stats(AuditStatsArgs),
}

#[derive(clap::Args)]
pub struct AuditShowArgs {
    /// Number of recent events to show
    #[arg(long, default_value = "50")]
    pub last: usize,
    /// Filter events after this ISO 8601 timestamp
    #[arg(long)]
    pub since: Option<String>,
    /// Filter by event type
    #[arg(long)]
    pub r#type: Option<String>,
    /// Filter by severity level
    #[arg(long)]
    pub severity: Option<String>,
    /// Filter by agent run ID
    #[arg(long)]
    pub run: Option<String>,
    /// Filter by tool name
    #[arg(long)]
    pub tool: Option<String>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
    /// Data source: sqlite or disk
    #[arg(long, default_value = "sqlite")]
    pub source: String,
}
```

---

## 4. Output Formats

### 4.1 Terminal Output (Default)

```text
📋 Audit Log — Last 5 Events
═══════════════════════════════════════════════════════════════════════

  #1  2026-10-19 09:15:00  [INFO]  tool_call
      Tool: read_file  |  Path: src/main.rs  |  Run: abc-123

  #2  2026-10-19 09:15:01  [INFO]  tool_call
      Tool: edit_file  |  Path: src/lib.rs  |  Run: abc-123
      Detail: Inserted 12 lines at line 45

  #3  2026-10-19 09:15:02  [WARN]  safety_block
      Tool: read_file  |  Path: ../../../etc/passwd  |  Run: abc-123
      Detail: Path traversal blocked by sandbox guard

  #4  2026-10-19 09:15:03  [INFO]  approval
      Tool: bash  |  Command: cargo test  |  Run: abc-123
      Detail: User approved via interactive prompt

  #5  2026-10-19 09:15:10  [INFO]  tool_call
      Tool: bash  |  Command: cargo test  |  Run: abc-123
      Detail: Exit code 0, 12 tests passed

═══════════════════════════════════════════════════════════════════════
  Showing 5 of 247 total events
```

### 4.2 Statistics Output (`audit stats`)

```text
📊 Audit Statistics
═══════════════════
  Total Events:        247
  Tool Calls:          198
  Safety Blocks:       3
  Approvals:           12
  Rejections:          0
  Critical Events:     1

  Top Tools:
    read_file          87 (43.9%)
    edit_file          54 (27.3%)
    bash               32 (16.2%)
    grep               25 (12.6%)

  Agent Runs:          4
  Avg Events/Run:      61.8
  Last Event:          2026-10-19 09:15:10Z
```

### 4.3 JSON Output

```json
{
  "events": [
    {
      "id": 1,
      "timestamp": "2026-10-19T09:15:00Z",
      "event_type": "tool_call",
      "tool_name": "read_file",
      "action": "Read file src/main.rs",
      "details": { "path": "src/main.rs", "bytes_read": 1393 },
      "agent_run_id": "abc-123",
      "severity": "info"
    }
  ],
  "total": 247,
  "filtered": 5
}
```

---

## 5. Architecture & Integration Points

### 5.1 Module Layout

```text
src/
├── cli/
│   ├── mod.rs       ← Add `Audit(AuditCommands)` variant to Commands enum
│   └── audit.rs     ← NEW: CLI handler for audit show/stats
├── safety/
│   └── audit.rs     ← EXISTING: AuditLogger (write-path) — extend with query methods
├── health/
│   └── audit.rs     ← EXISTING: HealthAuditEngine (reads DB for health reports)
├── memory/
│   └── mod.rs       ← EXISTING: MemoryEngine — add audit query helpers
```

### 5.2 Query Architecture

```rust
impl MemoryEngine {
    /// Query audit logs with flexible filters
    pub fn query_audit_logs(&self, filter: &AuditFilter) -> Result<Vec<AuditEvent>> {
        // Dynamic SQL WHERE clause construction based on filter fields
    }

    /// Get audit event count grouped by event_type
    pub fn audit_stats(&self) -> Result<AuditStats> {
        // GROUP BY queries on audit_logs table
    }
}

pub struct AuditFilter {
    pub limit: usize,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub event_type: Option<String>,
    pub severity: Option<String>,
    pub run_id: Option<String>,
    pub tool_name: Option<String>,
}
```

### 5.3 Disk Log Parsing

When `--source disk` is used, the command parses `.agent/logs/audit.log`:

```rust
pub fn parse_disk_audit_log(path: &Path) -> Result<Vec<AuditEvent>> {
    // Parse structured log lines:
    // [timestamp] [level] event_type: details key=value pairs
    // Using regex: r"\[(.+?)\] \[(.+?)\] (\w+): (.+)"
}
```

### 5.4 Dependencies

- **Existing**: `MemoryEngine`, `AuditLogger`, `rusqlite`, `chrono`, `regex`
- **No new crate dependencies**

---

## 6. Behavioral Rules

1. **Read-only**: `audit show` never modifies the audit log or database.
2. **Graceful degradation**: If SQLite is unavailable, fall back to disk log parsing.
3. **Privacy**: Redact any secret values that may appear in tool call details using the existing `SecretRedactor`.
4. **Performance**: Queries use indexed `timestamp DESC` ordering with `LIMIT` for efficient pagination.

---

## 7. Testing Strategy

| Test | Description |
|------|-------------|
| `test_audit_show_default` | Insert test events → verify last 50 returned |
| `test_audit_show_filtered` | Filter by type/severity → verify correct subset |
| `test_audit_show_since` | Filter by timestamp → verify time-bounded results |
| `test_audit_show_json` | Verify JSON output schema |
| `test_audit_stats` | Insert mixed events → verify grouped counts |
| `test_audit_disk_fallback` | Parse .agent/logs/audit.log file → verify parsing |
| `test_audit_empty` | No events → graceful empty output |

---

## 8. Open Questions

- [ ] Should `audit show` support **log rotation** (reading from archived `.audit.log.1`, `.audit.log.2`)?
- [ ] Should there be a `kaioken audit export` command for CSV/JSON export of full history?
- [ ] Should `--watch` flag enable live-tailing of audit events (like `tail -f`)?
