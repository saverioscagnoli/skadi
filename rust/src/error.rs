use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SkadiError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Failed to find configuration directory")]
    PathNotFound,

    #[error(
        "Configuration file not specified, you must provide at most one of the following: {0:?}"
    )]
    ConfigNotSpecified(Vec<PathBuf>),

    #[error("Failed to parse configuration located at {0}: {1}")]
    ConfigParsing(PathBuf, String),

    #[error("Failed to create application window: {0}")]
    WindowCreation(String),

    #[error("Failed to setup server: {0}")]
    ServerError(String),
}
