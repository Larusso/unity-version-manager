use crate::unity::VersionError;
use crate::unity::Version;
use plist::serde::deserialize;
use std::convert::AsRef;
use std::fs::File;
use std::path::Path;
use std::str::FromStr;

pub mod module;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AppInfo {
    pub c_f_bundle_version: String,
    pub unity_build_number: String,
}

pub fn read_version_from_path<P: AsRef<Path>>(path: P) -> Result<Version, VersionError> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(VersionError::PathContainsNoVersion(format!(
            "Provided Path does not exist. {}",
            path.display()
        )));
    }

    if path.is_dir() {
        //check for the `Unity.app` package
        let info_plist_path = path.join("Unity.app/Contents/Info.plist");
        let file = File::open(info_plist_path)?;
        let info: AppInfo = deserialize(file).map_err(|source| VersionError::Other(source.into()))?;
        let version = Version::from_str(&info.c_f_bundle_version)?;
        return Ok(version);
    }

    Err(VersionError::PathContainsNoVersion(
        path.display().to_string(),
    ))
}
