pub mod editors;
pub mod paths;

use std::io;
use crate::unity;
use thiserror::Error;
//

#[derive(Error, Debug)]
pub enum UvmHubError {
    #[error("api hub config: '{0}' is missing")]
    ConfigNotFound(String),

    #[error("Unity Hub config directory missing")]
    ConfigDirectoryNotFound,
   
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
        source: std::io::Error,
    },
    
    #[error("failed to create config file for config {config}")]
    FailedToCreateConfig {
        config: String,
        source: io::Error
    },

    #[error("Unity Hub editor install path not found")]
    InstallPathNotFound,
}

type Result<T> = std::result::Result<T, UvmHubError>;

pub fn list_installations() -> Result<unity::Installations> {
    let install_path = paths::install_path()
        .ok_or_else(|| UvmHubError::InstallPathNotFound)?;

    debug!("api hub install path: {}", install_path.display());

    let editors = editors::Editors::load()?;
    debug!("raw editors map: {:?}", editors);
    let editors = unity::Installations::from(editors);
    if let Ok(installations) = unity::list_installations_in_dir(&install_path) {
        let iter = installations.chain(editors);
        return Ok(unity::Installations(Box::new(iter)));
    }

    Ok(editors)
}
