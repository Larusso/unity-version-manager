use crate::unity::UvmVersionErrorKind;
use crate::unity::UvmVersionErrorResult as Result;
use crate::unity::UvmVersionErrorResultExt;
use crate::unity::Version;
use plist::serde::deserialize;
use std::convert::AsRef;
use std::fs::File;
use std::io;
use std::path::Path;
use std::str::FromStr;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AppInfo {
    pub c_f_bundle_version: String,
    pub unity_build_number: String,
}

pub fn read_version_from_path<P: AsRef<Path>>(path: P) -> Result<Version> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Provided Path does not exist. {}", path.display(),),
        )
        .into());
    }

    if path.is_dir() {
        //check for the `Unity.app` package
        let info_plist_path = path.join("Unity.app/Contents/Info.plist");
        let file = File::open(info_plist_path).chain_err(|| "unable to open Info.plist")?;
        let info: AppInfo = deserialize(file).chain_err(|| "unable to read Info.plist")?;
        let version = Version::from_str(&info.c_f_bundle_version)?;
        return Ok(version);
    }

    Err(UvmVersionErrorKind::NotAUnityInstalltion(path.display().to_string()).into())
}
