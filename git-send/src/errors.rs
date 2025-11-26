use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitSendError {
    #[error("Git command failed: {0}")]
    GitCommandFailed(String),

    #[error("Not a git repository")]
    NotGitRepository,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid configuration value for key '{key}': {value}")]
    InvalidConfigValue { key: String, value: String },
}
