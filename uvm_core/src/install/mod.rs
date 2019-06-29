use std::path::PathBuf;
use crate::unity::Version;
pub mod error;
mod installer;
mod variant;

pub use self::installer::Loader;

pub use self::error::{UvmInstallError, UvmInstallErrorKind, ResultExt, Result};


pub use self::installer::install_editor;
pub use self::installer::install_module;
pub use self::variant::InstallVariant;

pub fn download_installer(variant: InstallVariant, version: &Version) -> Result<PathBuf> {
    let d = Loader::new(variant, version.to_owned());
    d.download()
}
