use std::io;
use std::path::PathBuf;
use thiserror::Error;
use unity_version::error::VersionError;
pub use crate::unity::error::*;
#[derive(Error, Debug)]
pub enum UnityHubError {
    #[error("Unity Version error: {0}")]
    VersionReadError(#[from] VersionError),

    #[error("api hub config: '{0}' is missing")]
    ConfigNotFound(String),

    #[error("Unity Hub config directory missing")]
    ConfigDirectoryNotFound,

    #[error("Failed to locate application directory")]
    ApplicationDirectoryNotFound {
        #[source]
        source: io::Error,
    },

    #[error("failed to read Unity Hub config {config}")]
    ReadConfigError {
        config: String,
        source: anyhow::Error,
    },

    #[error("can't write Unity Hub config: '{config}'")]
    WriteConfigError {
        config: String,
        source: anyhow::Error,
    },

    #[error("failed to create config directory")]
    FailedToCreateConfigDirectory {
        source: io::Error,
    },

    #[error("failed to create config file for config {config}")]
    FailedToCreateConfig {
        config: String,
        source: io::Error
    },

    #[error("Unity Hub editor install path not found")]
    InstallPathNotFound,

    #[error("Failed to list installations at path {path}")]
    FailedToListInstallations {
        path: PathBuf,
        source: io::Error
    },

    #[error("Installation with version {version} not found")]
    InstallationNotFound {
        version: String,
        #[source]
        source: io::Error,
    },

}