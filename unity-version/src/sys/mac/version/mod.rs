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
        let info: AppInfo = plist::from_file(&info_plist_path).map_err(|source| VersionError::Other {
            msg: "Failed to parse version from Info.plist".to_string(),
            source: source.into(),
        })?;
        let version = Version::from_str(&info.c_f_bundle_version)?;
        return Ok(version);
    }

    Err(VersionError::PathContainsNoVersion(
        path.display().to_string(),
    ))
}
