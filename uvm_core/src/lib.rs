#![recursion_limit = "1024"]
#[cfg(unix)]
extern crate cluFlock;
extern crate md5;
extern crate regex;
extern crate reqwest;
extern crate semver;
extern crate serde;
extern crate serde_ini;
extern crate serde_json;
extern crate serde_yaml;
#[macro_use]
extern crate log;

#[cfg(test)]
#[macro_use]
extern crate proptest;
extern crate plist;
#[cfg(test)]
extern crate rand;
extern crate tempfile;
#[macro_use]
extern crate serde_derive;
extern crate dirs_2;
extern crate itertools;

#[macro_use]
extern crate error_chain;

pub mod utils;

#[macro_export]
#[cfg(unix)]
macro_rules! lock_process {
    ($lock_path:expr) => {
        let lock_file = fs::File::create($lock_path)?;
        let _lock = ::utils::lock_process_or_wait(&lock_file)?;
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
pub mod install;
pub mod unity;

pub use self::error::*;
pub use self::unity::current_installation;
pub use self::unity::list_all_installations;
pub use self::unity::list_hub_installations;
pub use self::unity::list_installations;
pub use self::unity::CurrentInstallation;
pub use self::unity::Installation;
pub use self::unity::Version;

use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};

use std::os;

use std::convert::AsRef;
use std::str::FromStr;

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
            ).into()
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

fn get_project_version<P: AsRef<Path>>(base_dir: P) -> io::Result<PathBuf> {
    let project_version = base_dir
        .as_ref()
        .join("ProjectSettings")
        .join("ProjectVersion.txt");
    if project_version.exists() {
        Ok(project_version)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "directory {} is not a Unity project",
                base_dir.as_ref().display()
            ),
        ))
    }
}

pub fn detect_unity_project_dir(dir: &Path, recur: bool) -> io::Result<PathBuf> {
    let error = Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Unable to find a Unity project",
    ));

    if dir.is_dir() {
        if get_project_version(dir).is_ok() {
            return Ok(dir.to_path_buf());
        } else if !recur {
            return error;
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let f = detect_unity_project_dir(&path, true);
                if f.is_ok() {
                    return f;
                }
            }
        }
    }
    error
}

pub fn dectect_project_version(project_path: &Path, recur: Option<bool>) -> io::Result<Version> {
    let project_version = detect_unity_project_dir(project_path, recur.unwrap_or(false))
        .and_then(get_project_version)?;

    let mut file = File::open(project_version)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Version::from_str(&contents)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Can't parse Unity version"))
}
