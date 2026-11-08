pub mod diff;
pub mod file_inventory;
pub mod git;
pub mod languages;
pub mod lock;
pub mod tree_sitter;

pub use diff::{DiffHunk, FileDiff};
pub use file_inventory::{
    DiscoveredFile, FileInventory, FileInventoryOptions, InventorySummary, ParseStatus,
    DEFAULT_MAX_FILE_SIZE_BYTES,
};
pub use git::{ChangeType, ChangedFile, GitCommitInfo, GitScanner};
pub use languages::Language;
pub use lock::{LockGuard, ScanLock};
pub use tree_sitter::{AstAnalysisResult, AstParser, ExtractedSymbol, SymbolKind};
