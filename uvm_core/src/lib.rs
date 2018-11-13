extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate serde_ini;
extern crate semver;
extern crate reqwest;

#[macro_use]
extern crate log;

#[cfg(test)]
#[macro_use]
extern crate proptest;
#[cfg(test)]
extern crate rand;
#[cfg(any(test,windows))]
extern crate tempfile;
extern crate plist;
#[macro_use]
extern crate serde_derive;
extern crate dirs;
#[macro_use]
extern crate itertools;

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

pub mod unity;
pub mod brew;
pub mod error;
pub mod result;
pub mod install;

pub use self::unity::list_installations;
pub use self::unity::list_all_installations;
pub use self::unity::current_installation;
pub use self::result::Result;
pub use self::unity::Installation;
pub use self::unity::CurrentInstallation;
pub use self::unity::Version;

use self::error::UvmError;
use std::io;
use std::fs;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;

use std::os;

use std::str::FromStr;
use std::convert::AsRef;


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
        .ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Unable to find Unity version {}", version),
        ).into())
}

pub fn activate(ref installation: Installation) -> Result<()> {
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
    match project_version.exists() {
        true => Ok(project_version),
        false => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "directory {} is not a Unity project",
                base_dir.as_ref().display()
            ),
        )),
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
