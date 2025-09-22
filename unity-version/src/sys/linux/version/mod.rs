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
                .or_else(|_| Version::find_version_in_file(executable_path));
        }
    }

    Err(VersionError::PathContainsNoVersion(
        path.display().to_string(),
    ))
}

