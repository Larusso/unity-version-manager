use thiserror::Error;

use thiserror_context::{Context, impl_context};

pub type InstallerResult<T> = std::result::Result<T, InstallerError>;

#[derive(Error, Debug)]
pub enum InstallerErrorInner {
    #[error("checksum verification failed")]
    ChecksumVerificationFailed,

    #[error("unknown installer {1}. Expected installer {0}")]
    UnknownInstaller(String, String),

    #[error("missing destination {0}")]
    MissingDestination(String),

    #[error("missing command {0}")]
    MissingCommand(String),

    #[error("unable to create installer: {0}")]
    InstallerCreateFailed(String),

    #[error("failed to copy {0}\n{1}")]
    CopyFailed(String, String, String),

    #[error("failed to install {0}\n{1}")]
    InstallationFailed(String, String),

    #[error("Failed to parse URL: {0}")]
    URLParseFailed(#[from] url::ParseError),

    #[error("Failed to unarchive zip: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("io error: {0}")]
    IO(#[from]#[source] std::io::Error),
}

impl_context!(InstallerError(InstallerErrorInner));