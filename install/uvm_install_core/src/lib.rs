#[no_mangle]
#[macro_export]

use log::*;
use std::fs::DirBuilder;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fs, io};

pub mod error;
pub mod installer;
mod loader;

mod sys;
pub mod utils;
use self::installer::*;
use error::*;

pub use self::loader::{Loader, ProgressHandler, InstallManifest};
pub use self::sys::*;
pub use ssri::Integrity;

pub struct UnityModule;
pub struct UnityEditor;

pub trait InstallHandler {
    fn install_handler(&self) -> Result<()>;

    fn install(&self) -> Result<()> {
        self.before_install()
            .chain_err(|| "pre install step failed")?;
        self.install_handler()
            .map_err(|err| {
                self.error_handler();
                err
            })
            .chain_err(|| "installation failed")?;
        self.after_install()
            .chain_err(|| "post install step failed")
    }

    fn error_handler(&self) {}

    fn before_install(&self) -> Result<()> {
        Ok(())
    }

    fn after_install(&self) -> Result<()> {
        Ok(())
    }
}
