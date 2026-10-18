pub mod frontmatter;
pub mod markdown;
pub mod stale;
pub mod generator;
pub mod cascade;

pub use frontmatter::DocFrontmatter;
pub use markdown::{FileDocTemplate, SymbolDocEntry, CommitDocEntry};
pub use stale::{StalenessDetector, DocVerificationReport, DocVerificationItem, DocStalenessStatus};
pub use generator::{DocGenerator, DocGenerateOptions, DocGenerationResult, DocBatchSummary};
pub use cascade::{StalenessCascadeEngine, StalenessCascadeReport};
