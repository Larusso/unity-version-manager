use thiserror::Error;
use std::io;
use std::result;

#[derive(Error, Debug)]
pub enum VersionError {
    #[error("unable to read version from path. {0}")]
    PathContainsNoVersion(String),

    #[error("failed to read version from executable {0}")]
    ExecutableContainsNoVersion(String),

    #[error("failed to parse version '{0}'")]
    ParsingFailed(String),

    #[error("failed to parse unkown version type {0}")]
    VersionTypeParsingFailed(String),

    #[error("no version found for match with req {0}")]
    NoMatch(String),

    #[error("failed to fetch version hash for version {version}")]
    HashMissing {
        source: super::hash::UnityHashError,
        version: String,
    },

    #[error("failed to read version")]
    Io {
        #[from]
        source: io::Error,
    },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = result::Result<T, VersionError>;