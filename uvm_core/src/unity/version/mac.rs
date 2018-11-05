use plist::serde::deserialize;
use std::io;
use std::convert::AsRef;
use std::path::Path;
use std::fs::File;
use super::*;
use crate::result::Result;
use crate::error::UvmError;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AppInfo {
    pub c_f_bundle_version: String,
    pub unity_build_number: String,
}

pub fn read_version_from_path<P : AsRef<Path>>(path:P) -> Result<Version> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(UvmError::IoError(io::Error::new(io::ErrorKind::InvalidInput, format!("Provided Path does not exist. {}", path.display()))))
    }
    if path.is_dir() {
        //check for the `Unity.app` package
        let info_plist_path = path.join("Unity.app/Contents/Info.plist");
        let file = File::open(info_plist_path)?;
        let info:AppInfo = deserialize(file)?;
        let version = Version::from_str(&info.c_f_bundle_version)?;
        Ok(version)
    } else {
        Err(UvmError::IoError(io::Error::new(io::ErrorKind::InvalidInput, "Provided Path is not a Unity installation.")))
    }
}
