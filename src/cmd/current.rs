use std::io::{Result, Error, ErrorKind};
use std::path::Path;
use unity::Installation;

const UNITY_CURRENT_LOCATION: &'static str = "/Applications/Unity";

pub fn current() -> Result<Installation> {
    let active_path = Path::new(UNITY_CURRENT_LOCATION);
    if let Ok(metadata) = active_path.symlink_metadata() {
        if metadata.file_type().is_symlink() {
            let linked_file = active_path.read_link().unwrap();
            let installation = Installation::new(linked_file).expect("Can't read current version");
            return Ok(installation)
        }
        else {
            return Err(Error::new(ErrorKind::Other, "Version at path is not a symlink"))
        }
    }
    Err(Error::new(ErrorKind::Other, "Can't read directory metadata"))
}
