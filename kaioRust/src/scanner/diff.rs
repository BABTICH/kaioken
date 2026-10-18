use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub file_path: PathBuf,
    pub hunks: Vec<DiffHunk>,
}
