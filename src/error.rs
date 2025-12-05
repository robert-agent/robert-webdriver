use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Failed to connect to Chrome: {0}")]
    ConnectionFailed(String),

    #[error("Failed to launch Chrome: {0}")]
    LaunchFailed(String),

    #[error("Navigation failed: {0}")]
    NavigationFailed(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("No page available")]
    NoPage,

    #[error("CDP error: {0}")]
    CdpError(#[from] chromiumoxide::error::CdpError),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, BrowserError>;
