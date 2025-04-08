use crate::Version;
use crate::error::VersionError;
use std::convert::AsRef;
use std::io;
use std::str::FromStr;
use std::path::Path;
use log::{debug, trace};

pub fn read_version_from_path<P: AsRef<Path>>(path: P) -> Result<Version, VersionError> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(VersionError::PathContainsNoVersion(format!(
            "Provided Path does not exist. {}",
            path.display()
        )));
    }

    if path.is_dir() {
        //check for the `Unity` executable
        let executable_path = path.join("Editor/Unity");
        trace!(
            "executable_path {} exists: {}",
            executable_path.display(),
            executable_path.exists()
        );

        if executable_path.exists() {
            return path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    debug!("Unable to read filename from path {}", path.display());
                    VersionError::Other(io::Error::new(io::ErrorKind::Other, "Unknown").into())
                })
                .and_then(|path| Version::from_str(path))
                .or_else(|_| find_version_in_file(executable_path));
        }
    }

    Err(VersionError::PathContainsNoVersion(
        path.display().to_string(),
    ))
}

pub fn find_version_in_file<P: AsRef<Path>>(path: P) -> Result<Version, VersionError> {
    use std::process::{Command, Stdio};

    let path = path.as_ref();
    debug!("find api version in Unity executable {}", path.display());

    let child = Command::new("strings")
        .arg("--")
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(VersionError::ExecutableContainsNoVersion(
            path.display().to_string(),
        ));
    }

    let version = Version::from_str(&String::from_utf8_lossy(&output.stdout))?;
    debug!("found version {}", &version);
    Ok(version)
}
