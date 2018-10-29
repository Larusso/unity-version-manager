pub mod editors;
pub mod paths;

use std::io;
use unity;
//

pub fn list_installations() -> unity::Result<unity::Installations> {
    let install_path = paths::install_path().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "install path not found")
    })?;
    debug!("unity hub install path: {}", install_path.display());

    let editors = editors::Editors::load()?;
    debug!("raw editors map: {:?}", editors);
    let editors = unity::Installations::from(editors);
    if let Ok(installations) = unity::list_installations_in_dir(&install_path) {
        let iter = installations.chain(editors);
        return Ok(unity::Installations(Box::new(iter)))
    }

    Ok(editors)
}
