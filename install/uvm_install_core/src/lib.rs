use uvm_core::unity::v2::Manifest;
use uvm_core::unity::Version;
use std::path::PathBuf;

pub mod error;

#[cfg(unix)]
macro_rules! lock_process {
    ($lock_path:expr) => {
        let lock_file = fs::File::create($lock_path)?;
        let _lock = uvm_core::utils::lock_process_or_wait(&lock_file)?;
    };
}

#[cfg(windows)]
macro_rules! lock_process {
    ($lock_path:expr) => {};
}

mod installer;
mod variant;
mod sys;
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
