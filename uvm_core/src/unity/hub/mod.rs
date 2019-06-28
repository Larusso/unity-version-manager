pub mod editors;
pub mod paths;

use std::io;
use crate::unity;
//

error_chain! {
    types {
        UvmHubError, UvmHubErrorKind, ResultExt, Result;
    }

    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
    }

    errors {
        ConfigNotFound(config_name: String) {
            description("unity hub config missing"),
            display("unity hub config: '{}' is missing", config_name)
        }

        ConfigDirectoryNotFound {
            description("unity hub config directory missing")
        }

        ReadConfigError(config_name: String) {
            description("error reading unity hub config"),
            display("error reading unity hub config: '{}'", config_name)
        }
        WriteConfigError(config_name: String) {
            description("error writing unity hub config"),
            display("error writing unity hub config: '{}'", config_name)
        }
    }
}

pub fn list_installations() -> Result<unity::Installations> {
    let install_path = paths::install_path()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "install path not found"))?;
    debug!("unity hub install path: {}", install_path.display());

    let editors = editors::Editors::load()?;
    debug!("raw editors map: {:?}", editors);
    let editors = unity::Installations::from(editors);
    if let Ok(installations) = unity::list_installations_in_dir(&install_path) {
        let iter = installations.chain(editors);
        return Ok(unity::Installations(Box::new(iter)));
    }

    Ok(editors)
}
