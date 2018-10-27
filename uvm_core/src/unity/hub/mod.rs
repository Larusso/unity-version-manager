pub mod editors;
pub mod paths;

use std::io;
use unity;
//

pub fn list_installations() -> unity::Result<unity::Installations> {
    let install_path = paths::install_path().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "install path not found")
    })?;
    let editors = editors::Editors::load()?;
    let installations = unity::list_installations_in_dir(&install_path)?;
    let iter = installations.chain(unity::Installations::from(editors));
    Ok(unity::Installations(Box::new(iter)))
}
