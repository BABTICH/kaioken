use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use crate::error::{KaiokenError, Result};

pub const SCAN_LOCK_FILE: &str = "scan.lock";
pub const STALE_LOCK_TIMEOUT_SECS: u64 = 300; // 5 minutes

pub struct LockGuard {
    lock_path: PathBuf,
}

impl LockGuard {
    pub fn lock_path(&self) -> &Path {
        &self.lock_path
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        if self.lock_path.exists() {
            let _ = fs::remove_file(&self.lock_path);
        }
    }
}

pub struct ScanLock;

impl ScanLock {
    pub const SCAN_LOCK_FILE: &str = "scan.lock";
    pub const STALE_LOCK_TIMEOUT_SECS: u64 = 300; // 5 minutes

    /// Attempt to acquire the lock immediately.
    /// Returns `Ok(Some(guard))` if acquired, or `Ok(None)` if locked by an active process.
    pub fn try_acquire(agent_dir: &Path) -> Result<Option<LockGuard>> {
        if !agent_dir.exists() {
            fs::create_dir_all(agent_dir)?;
        }

        let lock_path = agent_dir.join(Self::SCAN_LOCK_FILE);

        if lock_path.exists() {
            if let Ok(metadata) = fs::metadata(&lock_path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                        if elapsed.as_secs() > Self::STALE_LOCK_TIMEOUT_SECS {
                            tracing::warn!(
                                "Found stale scan.lock (age: {}s), reclaiming lock.",
                                elapsed.as_secs()
                            );
                            let _ = fs::remove_file(&lock_path);
                        } else {
                            // Active lock held by another process
                            return Ok(None);
                        }
                    }
                }
            } else {
                return Err(KaiokenError::LockError(format!(
                    "Cannot inspect existing lockfile at {}",
                    lock_path.display()
                )));
            }
        }

        let pid = std::process::id();
        let timestamp = chrono::Utc::now().to_rfc3339();
        let content = format!("pid={}\ntimestamp={}\n", pid, timestamp);

        fs::write(&lock_path, content)?;

        Ok(Some(LockGuard { lock_path }))
    }

    /// Acquire the lock, returning an error if held by another active process.
    pub fn acquire(agent_dir: &Path) -> Result<LockGuard> {
        match Self::try_acquire(agent_dir)? {
            Some(guard) => Ok(guard),
            None => {
                let lock_path = agent_dir.join(Self::SCAN_LOCK_FILE);
                let age_str = if let Ok(metadata) = fs::metadata(&lock_path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                            format!("{}s", elapsed.as_secs())
                        } else {
                            "unknown".to_string()
                        }
                    } else {
                        "unknown".to_string()
                    }
                } else {
                    "unknown".to_string()
                };

                Err(KaiokenError::LockError(format!(
                    "Scan locked by another active process (lock: {}, age: {}). Try again later.",
                    lock_path.display(),
                    age_str
                )))
            }
        }
    }

    /// Acquire the lock with a timeout, polling at regular intervals.
    pub fn acquire_with_timeout(
        agent_dir: &Path,
        timeout: std::time::Duration,
    ) -> Result<LockGuard> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if let Some(guard) = Self::try_acquire(agent_dir)? {
                return Ok(guard);
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        Self::acquire(agent_dir)
    }

    /// Check if the lock is currently held and not stale.
    pub fn is_locked(agent_dir: &Path) -> bool {
        let lock_path = agent_dir.join(Self::SCAN_LOCK_FILE);
        if !lock_path.exists() {
            return false;
        }

        if let Ok(metadata) = fs::metadata(&lock_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    return elapsed.as_secs() <= Self::STALE_LOCK_TIMEOUT_SECS;
                }
            }
        }
        false
    }
}
