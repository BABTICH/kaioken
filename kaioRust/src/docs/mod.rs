pub mod cascade;
pub mod frontmatter;
pub mod generator;
pub mod markdown;
pub mod stale;

pub use cascade::{StalenessCascadeEngine, StalenessCascadeReport};
pub use frontmatter::DocFrontmatter;
pub use generator::{DocBatchSummary, DocGenerateOptions, DocGenerationResult, DocGenerator};
pub use markdown::{CommitDocEntry, FileDocTemplate, SymbolDocEntry};
pub use stale::{
    DocStalenessStatus, DocVerificationItem, DocVerificationReport, StalenessDetector,
};
