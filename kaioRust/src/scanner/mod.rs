pub mod languages;
pub mod file_inventory;
pub mod git;
pub mod diff;
pub mod tree_sitter;
pub mod lock;

pub use languages::Language;
pub use file_inventory::{
    FileInventory, DiscoveredFile, FileInventoryOptions, InventorySummary, ParseStatus,
    DEFAULT_MAX_FILE_SIZE_BYTES,
};
pub use git::{GitScanner, GitCommitInfo, ChangedFile, ChangeType};
pub use diff::{DiffHunk, FileDiff};
pub use tree_sitter::{AstParser, ExtractedSymbol, AstAnalysisResult, SymbolKind};
pub use lock::{ScanLock, LockGuard};
