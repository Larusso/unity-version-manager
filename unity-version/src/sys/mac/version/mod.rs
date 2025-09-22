use crate::error::VersionError;
use crate::Version;
use serde::{Deserialize, Serialize};
use std::convert::AsRef;
use std::path::Path;
use std::str::FromStr;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AppInfo {
    pub c_f_bundle_version: String,
    pub unity_build_number: String,
}

pub fn read_version_from_path<P: AsRef<Path>>(path: P) -> Result<Version, VersionError> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(VersionError::PathContainsNoVersion(
            path.display().to_string(),
        ));
    }

    if path.is_dir() {
        //check for the `Unity.app` package
        let info_plist_path = path.join("Unity.app/Contents/Info.plist");
        
        // Try to read from Info.plist first (preferred method on macOS)
        if let Ok(info) = plist::from_file::<_, AppInfo>(&info_plist_path) {
            if let Ok(version) = Version::from_str(&info.c_f_bundle_version) {
                return Ok(version);
            }
        }
        
        // Fallback: try to find Unity executable and use strings command
        let unity_executable = path.join("Unity.app/Contents/MacOS/Unity");
        if unity_executable.exists() {
            if let Ok(version) = Version::find_version_in_file(&unity_executable) {
                return Ok(version);
            }
        }
    }

    Err(VersionError::PathContainsNoVersion(
        path.display().to_string(),
    ))
}
