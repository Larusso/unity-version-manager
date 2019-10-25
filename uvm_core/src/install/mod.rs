use crate::unity::v2::Manifest;
use crate::unity::Version;
use std::path::PathBuf;
pub mod error;
mod installer;
mod variant;

pub use self::installer::Loader;

pub use self::error::{Result, ResultExt, UvmInstallError, UvmInstallErrorKind};

pub use self::installer::install_editor;
pub use self::installer::install_module;
pub use self::variant::InstallVariant;

pub fn download_installer(variant: InstallVariant, version: &Version) -> Result<PathBuf> {
    let manifest: Result<Manifest> =
        Manifest::load(version).map_err(|_| UvmInstallErrorKind::ManifestLoadFailed.into());
    let manifest = manifest?;
    let d = Loader::new(variant, &manifest);
    d.download()
}
