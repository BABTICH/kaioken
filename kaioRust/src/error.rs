use thiserror::Error;

#[derive(Error, Debug)]
pub enum KaiokenError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Memory/Database error: {0}")]
    MemoryError(String),

    #[error("Scanner error: {0}")]
    ScannerError(String),

    #[error("Documentation error: {0}")]
    DocError(String),

    #[error("Lock error: {0}")]
    LockError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, KaiokenError>;
