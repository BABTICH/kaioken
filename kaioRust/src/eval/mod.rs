pub mod benchmarks;
pub mod metrics;

pub use benchmarks::{BenchmarkRegistry, BenchmarkTask};
pub use metrics::{BenchmarkResult, EvaluationScorecard};

use std::time::Instant;
use crate::error::Result;

pub struct EvaluationRunner {
    registry: BenchmarkRegistry,
}

impl Default for EvaluationRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl EvaluationRunner {
    pub fn new() -> Self {
        Self {
            registry: BenchmarkRegistry::new(),
        }
    }

    pub async fn run_all(
        &self,
        filter: Option<&str>,
        progress_callback: Option<&(dyn Fn(&BenchmarkResult) + Send + Sync)>,
    ) -> Result<EvaluationScorecard> {
        let start = Instant::now();
        let mut results = Vec::new();

        for task in self.registry.list() {
            if let Some(f) = filter {
                if !task.name().contains(f) {
                    continue;
                }
            }

            let res = task.run().await?;
            if let Some(cb) = progress_callback {
                cb(&res);
            }
            results.push(res);
        }

        let total_duration = start.elapsed().as_millis();
        Ok(EvaluationScorecard::compute(results, total_duration))
    }
}
