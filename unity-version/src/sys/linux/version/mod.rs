use crate::error::VersionError;
use crate::Version;
use log::{debug, trace};
use std::convert::AsRef;
use std::io;
use std::path::Path;
use std::str::FromStr;

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
                    VersionError::Other {
                        source: io::Error::new(io::ErrorKind::Other, "Unknown").into(),
                        msg: "Unknown".to_string(),
                    }
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
        .spawn()
        .map_err(|e| VersionError::Other {
            source: e.into(),
            msg: "failed to spawn strings".to_string(),
        })?;

    let output = child.wait_with_output().map_err(|e| VersionError::Other {
        source: e.into(),
        msg: "failed to spawn strings".to_string(),
    })?;

    if !output.status.success() {
        return Err(VersionError::ExecutableContainsNoVersion(
            path.to_path_buf(),
        ));
    }

    let version = Version::from_str(&String::from_utf8_lossy(&output.stdout))?;
    debug!("found version {}", &version);
    Ok(version)
}
