use thiserror::Error;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct LivePlatformError(#[from] ErrorRepr);

impl LivePlatformError {
    /// Create an error with a custom message (e.g., wrapping `anyhow::Error`).
    pub fn new<E>(msg: &str, source: E) -> Self
    where
        E: Into<anyhow::Error>,
    {
        ErrorRepr::Unexpected {
            msg: msg.to_string(),
            source: source.into(),
        }
            .into()
    }
}

#[derive(Error, Debug)]
pub enum ErrorRepr {
    #[error("FetchRelease error: {0}")]
    FetchReleaseError(#[from] FetchReleaseError),

    #[error("ListVersions error: {0}")]
    ListVersionsError(#[from] ListVersionsError),

    #[error("Unexpected error: {msg}")]
    Unexpected {
        msg: String,
        #[source]
        source: anyhow::Error,
    },
}

#[derive(Error, Debug)]
pub enum FetchReleaseError {
    #[error("Failed to fetch release: {0}")]
    FetchFailed(String),

    #[error("Invalid JSON response")]
    JsonError(#[source] reqwest::Error),

    #[error("Release not found for version: {0} platform: {1} architecture: {2} stream: {3}")]
    NotFound(String, String, String, String),

    #[error("Network error: {0}")]
    NetworkError(#[source] reqwest::Error),
}

#[derive(Error, Debug)]
pub enum ListVersionsError {
    #[error("Failed to fetch version list: {0}")]
    FetchFailed(String),

    #[error("Invalid JSON response")]
    JsonError(#[source] reqwest::Error),

    #[error("Network error: {0}")]
    NetworkError(#[source] reqwest::Error),
}