use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct EventMeta {
    pub first_seen: Instant,
    pub last_seen: Instant,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct DebounceManager {
    debounce_duration: Duration,
    burst_threshold: usize,
    pending: HashMap<PathBuf, EventMeta>,
}

impl DebounceManager {
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            debounce_duration: Duration::from_millis(debounce_ms),
            burst_threshold: 50,
            pending: HashMap::new(),
        }
    }

    pub fn with_burst_threshold(mut self, threshold: usize) -> Self {
        self.burst_threshold = threshold;
        self
    }

    pub fn set_burst_threshold(&mut self, threshold: usize) {
        self.burst_threshold = threshold;
    }

    pub fn debounce_duration(&self) -> Duration {
        self.debounce_duration
    }

    /// Record a file change event for a given path. Returns the cumulative event count for this path.
    pub fn record_event(&mut self, path: PathBuf) -> usize {
        let now = Instant::now();
        let entry = self.pending.entry(path).or_insert(EventMeta {
            first_seen: now,
            last_seen: now,
            count: 0,
        });
        entry.last_seen = now;
        entry.count += 1;
        entry.count
    }

    /// Check if any events are ready to process.
    /// Ready when `now - last_seen >= debounce_duration`, OR when pending count exceeds the burst threshold.
    pub fn take_ready(&mut self) -> Vec<PathBuf> {
        let now = Instant::now();
        let is_burst = self.pending.len() >= self.burst_threshold;

        let ready_keys: Vec<PathBuf> = self
            .pending
            .iter()
            .filter(|(_, meta)| {
                is_burst || now.duration_since(meta.last_seen) >= self.debounce_duration
            })
            .map(|(p, _)| p.clone())
            .collect();

        for key in &ready_keys {
            self.pending.remove(key);
        }

        ready_keys
    }

    /// Flush all pending events regardless of debounce timeout (useful on shutdown).
    pub fn take_all(&mut self) -> Vec<PathBuf> {
        let keys: Vec<PathBuf> = self.pending.keys().cloned().collect();
        self.pending.clear();
        keys
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn get_event_count(&self, path: &Path) -> usize {
        self.pending.get(path).map(|m| m.count).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_debounce_merges_rapid_events() {
        let mut dm = DebounceManager::new(50);
        let path = PathBuf::from("src/main.rs");

        let c1 = dm.record_event(path.clone());
        let c2 = dm.record_event(path.clone());
        let c3 = dm.record_event(path.clone());

        assert_eq!(c1, 1);
        assert_eq!(c2, 2);
        assert_eq!(c3, 3);
        assert_eq!(dm.pending_count(), 1);

        // Immediate check should return nothing
        let ready = dm.take_ready();
        assert!(ready.is_empty());

        // Wait for debounce window
        sleep(Duration::from_millis(60));
        let ready = dm.take_ready();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0], path);
        assert!(dm.is_empty());
    }

    #[test]
    fn test_burst_protection_triggers_immediately() {
        let mut dm = DebounceManager::new(5000).with_burst_threshold(10);
        for i in 0..15 {
            dm.record_event(PathBuf::from(format!("file_{}.rs", i)));
        }

        assert_eq!(dm.pending_count(), 15);
        // Even though 5000ms haven't elapsed, burst threshold (10) triggers processing
        let ready = dm.take_ready();
        assert_eq!(ready.len(), 15);
        assert!(dm.is_empty());
    }
}
