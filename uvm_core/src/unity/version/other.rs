use std::convert::AsRef;
use std::path::Path;
use crate::result::Result;
use crate::error::IllegalOperationError;

pub fn read_version_from_path<P : AsRef<Path>>(path:P) -> Result<Version> {
    Err(IllegalOperationError::new("fn 'read_version_from_path' not supported on current platform"))
}
