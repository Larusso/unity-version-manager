use std::io;
use thiserror::Error;
use unity_version::error::VersionError;

#[derive(Error, Debug)]
pub enum UnityError {
    #[error("modules.json not found")]
    ModulesJsonNotFound(#[from] io::Error),

    #[error("Failed to parse modules json")]
    ModulesJsonParseError(#[from] serde_json::Error),
}