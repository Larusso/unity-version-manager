#![recursion_limit = "1024"]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[cfg(test)]
#[macro_use]
extern crate proptest;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate error_chain;

pub mod platform;
pub mod progress;
pub mod sys;
pub mod utils;

#[macro_export]
#[cfg(unix)]
macro_rules! lock_process {
    ($lock_path:expr) => {
        let lock_file = fs::File::create($lock_path)?;
        let _lock = crate::utils::lock_process_or_wait(&lock_file)?;
    };
}

#[macro_export]
#[cfg(windows)]
macro_rules! lock_process {
    ($lock_path:expr) => {};
}

#[macro_export]
macro_rules! cargo_version {
    // `()` indicates that the macro takes no argument.
    () => {
        // The macro will expand into the contents of this block.
        format!(
            "{}.{}.{}{}",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH"),
            option_env!("CARGO_PKG_VERSION_PRE").unwrap_or("")
        );
    };
}

pub mod error;
pub mod unity;

pub use self::error::*;
pub use self::unity::current_installation;
pub use self::unity::list_all_installations;
pub use self::unity::list_hub_installations;
pub use self::unity::list_installations;
pub use self::unity::CurrentInstallation;
pub use self::unity::Installation;
pub use self::unity::Version;
pub use unity::project::{dectect_project_version, detect_unity_project_dir};
use std::fs;
use std::io;
use std::path::Path;
use std::os;


pub fn is_active(version: &Version) -> bool {
    if let Ok(current) = current_installation() {
        current.version() == version
    } else {
        false
    }
}

pub fn find_installation(version: &Version) -> Result<Installation> {
    let mut installations = list_all_installations()?;
    installations
        .find(|i| i.version() == version)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Unable to find Unity version {}", version),
            )
            .into()
        })
}

pub fn activate(installation: &Installation) -> Result<()> {
    let active_path = Path::new("/Applications/Unity");
    if active_path.exists() {
        fs::remove_file(active_path)?;
    }

    #[cfg(unix)]
    os::unix::fs::symlink(installation.path(), active_path)?;
    #[cfg(windows)]
    os::windows::fs::symlink_dir(installation.path(), active_path)?;

    Ok(())
}
