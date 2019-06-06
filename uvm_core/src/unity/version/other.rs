use super::*;
use std::convert::AsRef;
use std::path::Path;

pub fn read_version_from_path<P: AsRef<Path>>(path: P) -> Result<Version> {
    Err(
        UvmErrorKind::IllegalOperationError("fn 'read_version_from_path' not supported on current platform")
        .into(),
    )
}
