pub mod editors;
pub mod paths;
pub mod module;

use std::io;
use log::debug;
use crate::unity;
use thiserror::Error;
use crate::error::UnityHubError;
use crate::unity::Installations;
//

type Result<T> = std::result::Result<T, UnityHubError>;

pub fn list_installations() -> Result<Installations> {
    let install_path = paths::install_path()
        .ok_or_else(|| UnityHubError::InstallPathNotFound)?;

    debug!("api hub install path: {}", install_path.display());

    let editors = editors::Editors::load()?;
    debug!("raw editors map: {:?}", editors);
    let editors = crate::unity::Installations::from(editors);
    if let Ok(installations) = unity::list_installations_in_dir(&install_path) {
        let iter = installations.chain(editors);
        return Ok(unity::Installations(Box::new(iter)));
    }

    Ok(editors)
}
