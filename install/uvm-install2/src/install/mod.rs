use log::*;
use std::fs::DirBuilder;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fs, io};

pub mod error;
pub mod installer;
mod loader;
pub mod utils;

use self::installer::*;

pub use self::loader::{Loader, ProgressHandler, InstallManifest};
pub use crate::sys::*;
pub use ssri::Integrity;
use error::InstallerError;
use thiserror_context::Context;

pub struct UnityModule;
pub struct UnityEditor;

pub trait InstallHandler {
    fn install_handler(&self) -> Result<(), InstallerError>;

    fn install(&self) -> Result<(), InstallerError> {
        self.before_install()?;
            //.context("pre install step failed")?;
        self.install_handler()
            .map_err(|err| {
                self.error_handler();
                err
            })?;
            //.context("installation failed")?;
        self.after_install()
            //.context("post install step failed")
    }

    fn error_handler(&self) {}

    fn before_install(&self) -> Result<(), InstallerError> {
        Ok(())
    }

    fn after_install(&self) -> Result<(), InstallerError> {
        Ok(())
    }
}
