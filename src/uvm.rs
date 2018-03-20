extern crate docopt;
extern crate regex;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
#[macro_use]
extern crate proptest;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate tempdir;

#[macro_export]
macro_rules! cargo_version {
    // `()` indicates that the macro takes no argument.
    () => (
        // The macro will expand into the contents of this block.
        format!("{}.{}.{}{}",
          env!("CARGO_PKG_VERSION_MAJOR"),
          env!("CARGO_PKG_VERSION_MINOR"),
          env!("CARGO_PKG_VERSION_PATCH"),
          option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""));
    )
}

pub mod cli;
pub mod unity;

pub use self::unity::list_installations;
pub use self::unity::current_installation;

pub use self::unity::Installation;
pub use self::unity::CurrentInstallation;
pub use self::unity::Version;

use std::io;
use std::fs;
use std::path::Path;
use std::os::unix;

pub fn is_active(version: &Version) -> bool {
    if let Ok(current) = current_installation() {
        current.version() == version
    } else {
        false
    }
}

pub fn find_installation(version: &Version) -> io::Result<Installation> {
    let mut installations = list_installations()?;
    installations
        .find(|i| i.version() == version)
        .ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Unable to find Unity version {}", version)
        ))
}

pub fn activate(ref installation: Installation) -> io::Result<()> {
    let active_path = Path::new("/Applications/Unity");
    if active_path.exists() {
        fs::remove_file(active_path)?;
    }
    unix::fs::symlink(installation.path(), active_path)?;
    Ok(())
}
