use crate::unity::UvmVersionErrorKind;
use crate::unity::UvmVersionErrorResult as Result;
use crate::unity::Version;
use std::convert::AsRef;
use std::io;
use std::path::Path;
use std::str::FromStr;

pub mod module;

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
        //check for the `Unity` executable
        let executable_path = path.join("Editor/Unity");
        trace!(
            "executable_path {} exists: {}",
            executable_path.display(),
            executable_path.exists()
        );

        if executable_path.exists() {
            return path.file_name().and_then(|name| name.to_str()).ok_or_else(|| {
                debug!("Unable to read filename from path {}", path.display());
                UvmVersionErrorKind::FailedToReadVersion(path.display().to_string())
            }).and_then(|path| {
                Version::from_str(path).map_err(|err| err.into())
            }).or_else(|_| {
                Version::find_version_in_file(executable_path).into()
            })
        }
    }

    Err(UvmVersionErrorKind::NotAUnityInstalltion(path.display().to_string()).into())
}
