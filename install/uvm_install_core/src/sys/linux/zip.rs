use crate::*;

pub type EditorZipInstaller = Installer<UnityEditor, Zip, InstallerWithDestination>;

impl InstallHandler for EditorZipInstaller {
    fn before_install(&self) -> Result<()> {
        self.clean_directory(self.destination())
    }

    fn install_handler(&self) -> Result<()> {
        debug!("install editor from zip archive");
        self.deploy_zip(self.installer(), self.destination())
    }
}
