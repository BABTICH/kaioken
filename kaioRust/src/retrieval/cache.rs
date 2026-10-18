use std::path::Path;
use chrono::{Duration, Utc};
use rusqlite::params;
use sha2::{Digest, Sha256};
use crate::error::{KaiokenError, Result};
use crate::memory::MemoryEngine;
use crate::retrieval::context_packet::ContextPacket;

pub struct RetrievalCacheStore<'a> {
    memory: &'a MemoryEngine,
}

impl<'a> RetrievalCacheStore<'a> {
    pub fn new(memory: &'a MemoryEngine) -> Self {
        Self { memory }
    }

    /// Computes deterministic SHA-256 key for a task, repo head commit, and retrieval options.
    pub fn compute_key(task: &str, head_commit: Option<&str>, flags: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(task.trim().as_bytes());
        hasher.update(b":");
        hasher.update(head_commit.unwrap_or("uncommitted").as_bytes());
        hasher.update(b":");
        hasher.update(flags.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Attempts to retrieve a cached ContextPacket, validating TTL and disk file hash freshness.
    pub fn get(&self, query_key: &str, repo_root: &Path) -> Result<Option<ContextPacket>> {
        let conn = self.memory.db().connection();
        let now = Utc::now().to_rfc3339();

        let row: Option<(String, String)> = conn
            .query_row(
                "SELECT context_json, expires_at FROM retrieval_cache WHERE query_hash = ?1",
                params![query_key],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();

        let (context_json, expires_at) = match row {
            Some(data) => data,
            None => return Ok(None),
        };

        // Check TTL expiration
        if expires_at < now {
            let _ = conn.execute("DELETE FROM retrieval_cache WHERE query_hash = ?1", params![query_key]);
            return Ok(None);
        }

        // Deserialize context packet
        let packet: ContextPacket = match serde_json::from_str(&context_json) {
            Ok(p) => p,
            Err(_) => {
                let _ = conn.execute("DELETE FROM retrieval_cache WHERE query_hash = ?1", params![query_key]);
                return Ok(None);
            }
        };

        // Validate hash freshness for relevant files
        for file in &packet.relevant_files {
            let disk_path = repo_root.join(&file.path);
            if disk_path.exists() {
                if let Ok(content) = std::fs::read(&disk_path) {
                    let mut hasher = Sha256::new();
                    hasher.update(&content);
                    let current_hash = format!("{:x}", hasher.finalize());
                    if current_hash != file.source_hash {
                        // Invalidate cache entry due to file modification
                        let _ = conn.execute("DELETE FROM retrieval_cache WHERE query_hash = ?1", params![query_key]);
                        return Ok(None);
                    }
                }
            } else {
                // File deleted or moved
                let _ = conn.execute("DELETE FROM retrieval_cache WHERE query_hash = ?1", params![query_key]);
                return Ok(None);
            }
        }

        Ok(Some(packet))
    }

    /// Stores a ContextPacket in SQLite retrieval_cache with configured TTL seconds.
    pub fn put(&self, query_key: &str, packet: &ContextPacket, ttl_seconds: i64) -> Result<()> {
        let conn = self.memory.db().connection();
        let now = Utc::now();
        let created_at = now.to_rfc3339();
        let expires_at = (now + Duration::seconds(ttl_seconds)).to_rfc3339();
        let context_json = packet.to_json()
            .map_err(|e| KaiokenError::RetrievalError(format!("Failed to serialize context packet: {}", e)))?;

        conn.execute(
            "INSERT INTO retrieval_cache (query_hash, context_json, created_at, expires_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(query_hash) DO UPDATE SET
               context_json = excluded.context_json,
               created_at = excluded.created_at,
               expires_at = excluded.expires_at",
            params![query_key, context_json, created_at, expires_at],
        )
        .map_err(|e| KaiokenError::MemoryError(format!("Failed to insert retrieval cache: {}", e)))?;

        Ok(())
    }

    /// Clears all entries in the retrieval_cache table.
    pub fn clear(&self) -> Result<usize> {
        let conn = self.memory.db().connection();
        let deleted = conn.execute("DELETE FROM retrieval_cache", [])
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to clear retrieval cache: {}", e)))?;
        Ok(deleted)
    }

    /// Deletes expired cache entries.
    pub fn clear_expired(&self) -> Result<usize> {
        let conn = self.memory.db().connection();
        let now = Utc::now().to_rfc3339();
        let deleted = conn.execute("DELETE FROM retrieval_cache WHERE expires_at < ?1", params![now])
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to clear expired cache: {}", e)))?;
        Ok(deleted)
    }
}
