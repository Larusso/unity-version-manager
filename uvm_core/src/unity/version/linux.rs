use super::*;
use std::convert::AsRef;
use std::io;
use std::path::Path;

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
        //check for the `Unity.exe`
        let executable_path = path.join("Editor/Unity");
        trace!(
            "executable_path {} exists: {}",
            executable_path.display(),
            executable_path.exists()
        );

        if executable_path.exists() {
            let path_name = path.file_name().and_then(|name| name.to_str()).ok_or_else(|| {
                debug!("Unable to read filename from path {}", path.display());
                UvmVersionErrorKind::FailedToReadVersion(path.display().to_string())
            })?;

            return Version::from_str(path_name);
        }
    }

    Err(UvmVersionErrorKind::NotAUnityInstalltion(path.display().to_string()).into())
}
