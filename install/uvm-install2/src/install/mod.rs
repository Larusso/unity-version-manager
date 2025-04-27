pub mod error;
pub mod installer;
mod loader;
pub mod utils;

pub use self::loader::{InstallManifest, Loader, ProgressHandler};
pub use crate::sys::*;
use error::InstallerError;
use log::error;
use std::path::Path;
use thiserror_context::Context;

pub struct UnityModule;
pub struct UnityEditor;

pub trait InstallHandler {
    fn install_handler(&self) -> Result<(), InstallerError>;

    fn install(&self) -> Result<(), InstallerError> {
        self.before_install().context("pre install step failed")?;
        self.install_handler()
            .map_err(|err| {
                error!("installation of {} failed", self.installer().display());
                self.error_handler();
                err
            })
            .context(format!(
                "installation of {} failed",
                self.installer().display()
            ))?;
        self.after_install().context("post install step failed")?;
        Ok(())
    }

    fn installer(&self) -> &Path;

    fn error_handler(&self) {}

    fn before_install(&self) -> Result<(), InstallerError> {
        Ok(())
    }

    fn after_install(&self) -> Result<(), InstallerError> {
        Ok(())
    }
}
