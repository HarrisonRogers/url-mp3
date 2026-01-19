use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Binary not found: {0}")]
    BinaryNotFound(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to initialize binaries: {0}")]
    BinaryInit(String),
}
