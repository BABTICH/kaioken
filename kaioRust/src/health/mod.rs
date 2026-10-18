pub mod audit;
pub mod report;

pub use audit::HealthAuditEngine;
pub use report::{
    ArchitectureSummary, DeadCodeSummary, DocHealthSummary, FailureMemorySummary,
    GraphIntegritySummary, HealthReport, HealthScore, ModuleBreakdown, RepoOverview,
    ScoreDeduction, TestCoverageSummary, UnreferencedSymbol,
};
