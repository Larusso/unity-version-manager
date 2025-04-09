use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VersionError {
    #[error("Failed to parse unity version string: {0}")]
    ParsingFailed(String),

    #[error("Provided Path does not exist: {0}")]
    PathContainsNoVersion(String),

    #[error("Failed to parse unity version from path: {0}")]
    FetchVersionFromPathFailed(PathBuf),

    #[error("Executable at {0} contains no version information")]
    ExecutableContainsNoVersion(PathBuf),

    #[error("Unexpected error: {msg}")]
    Other {
        msg: String,
        #[source]
        source: anyhow::Error,
    },
}
