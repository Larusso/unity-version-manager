use crate::*;
use crate::install::installer::{Installer, InstallerWithDestination, Zip};
use crate::install::{InstallHandler, UnityEditor};
use crate::install::error::InstallerResult;

pub type EditorZipInstaller = Installer<UnityEditor, Zip, InstallerWithDestination>;

impl InstallHandler for EditorZipInstaller {
    fn before_install(&self) -> InstallerResult<()> {
        self.clean_directory(self.destination())
    }

    fn install_handler(&self) -> InstallerResult<()> {
        debug!("install editor from zip archive");
        self.deploy_zip(self.installer(), self.destination())
    }
}
