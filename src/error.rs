use std::path::PathBuf;

/// Application-level error types.
///
/// # Examples
///
/// ```text
/// Chrome is running, please close it first
/// ```
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Low-level I/O failure.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// JSON parsing or serialization failure.
    #[error("JSON processing failed: {0}")]
    InvalidJson(String),

    /// Unsupported OS type.
    #[error("Unsupported operating system: {0}")]
    UnsupportedOs(String),

    /// Missing required environment variable.
    #[error("Missing environment variable: {0}")]
    MissingEnv(String),

    /// Chrome is still running.
    #[error("Chrome is running, please close it first")]
    ChromeRunning,

    /// Chrome still has not exited successfully.
    #[error("Chrome is still running, please confirm it has been fully closed")]
    ChromeStillRunning,

    /// Chrome configuration file not found.
    #[error("Chrome configuration file not found: {0}")]
    ConfigNotFound(PathBuf),

    /// Backup file not found.
    #[error("Backup file not found: {0}")]
    BackupNotFound(PathBuf),

    /// Invalid file path.
    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    /// External command failure.
    #[error("Command execution failed: {command} ({details})")]
    CommandFailed { command: String, details: String },
}

/// Application-level Result type alias.
///
/// # Examples
///
/// ```text
/// let result: AppResult<()> = Ok(());
/// ```
pub type AppResult<T> = std::result::Result<T, AppError>;
