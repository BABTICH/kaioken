use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn badge(&self) -> &'static str {
        match self {
            RiskLevel::Low => "\x1b[32m[LOW]\x1b[0m",
            RiskLevel::Medium => "\x1b[33m[MEDIUM ⚠️]\x1b[0m",
            RiskLevel::High => "\x1b[31m[HIGH ⚠️]\x1b[0m",
            RiskLevel::Critical => "\x1b[1;31m[CRITICAL 🚨]\x1b[0m",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RiskFactor {
    BreakingSignatureChange {
        symbol: String,
        weight: f64,
    },
    PublicApiChange {
        symbol: String,
        weight: f64,
    },
    HighFanOutSymbol {
        symbol: String,
        callers: usize,
        weight: f64,
    },
    NoTestCoverage {
        file: String,
        weight: f64,
    },
    PreviousFailure {
        error_sig: String,
        weight: f64,
    },
    StaleDocumentation {
        doc_path: String,
        weight: f64,
    },
    LargeChangeSet {
        files_changed: usize,
        weight: f64,
    },
}

impl RiskFactor {
    pub fn weight(&self) -> f64 {
        match *self {
            RiskFactor::BreakingSignatureChange { weight, .. } => weight,
            RiskFactor::PublicApiChange { weight, .. } => weight,
            RiskFactor::HighFanOutSymbol { weight, .. } => weight,
            RiskFactor::NoTestCoverage { weight, .. } => weight,
            RiskFactor::PreviousFailure { weight, .. } => weight,
            RiskFactor::StaleDocumentation { weight, .. } => weight,
            RiskFactor::LargeChangeSet { weight, .. } => weight,
        }
    }

    pub fn description(&self) -> String {
        match self {
            RiskFactor::BreakingSignatureChange { symbol, .. } => {
                format!("Breaking signature change ({})", symbol)
            }
            RiskFactor::PublicApiChange { symbol, .. } => {
                format!("Public API change ({})", symbol)
            }
            RiskFactor::HighFanOutSymbol {
                symbol, callers, ..
            } => {
                format!(
                    "High fan-out symbol ({} has {} callers affected)",
                    symbol, callers
                )
            }
            RiskFactor::NoTestCoverage { file, .. } => {
                format!("No test coverage ({})", file)
            }
            RiskFactor::PreviousFailure { error_sig, .. } => {
                format!("Previous unresolved failure in file ({})", error_sig)
            }
            RiskFactor::StaleDocumentation { doc_path, .. } => {
                format!("Stale documentation ({})", doc_path)
            }
            RiskFactor::LargeChangeSet { files_changed, .. } => {
                format!("Large change set ({} files touched)", files_changed)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskAssessment {
    pub score: f64,
    pub level: RiskLevel,
    pub factors: Vec<RiskFactor>,
}

impl RiskAssessment {
    pub fn calculate(factors: Vec<RiskFactor>) -> Self {
        let raw_score: f64 = factors.iter().map(|f| f.weight()).sum();
        let score = (raw_score * 100.0).round() / 100.0;
        let clamped = score.clamp(0.0, 1.0);

        let level = if clamped >= 0.75 {
            RiskLevel::Critical
        } else if clamped >= 0.50 {
            RiskLevel::High
        } else if clamped >= 0.25 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        Self {
            score: clamped,
            level,
            factors,
        }
    }
}
