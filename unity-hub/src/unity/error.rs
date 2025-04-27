use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UnityError {
    #[error("modules.json not found")]
    ModulesJsonNotFound {
        #[from]
        source: io::Error,
    },

    #[error("Failed to parse modules json")]
    ModulesJsonParseError(#[from] serde_json::Error),
}