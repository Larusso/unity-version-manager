use crate::error::*;
use crate::*;
pub struct Po;
pub type ModulePoInstaller = Installer<UnityModule, Po, InstallerWithDestination>;

impl<V, I> Installer<V, Po, I> {
    fn install_language_po_file(&self, po: &Path, destination: &Path) -> Result<()> {
        debug!("Copy po file {} to {}", po.display(), destination.display());
        fs::copy(po, destination).chain_err(|| "unable to copy po file to destination")?;
        Ok(())
    }
}

impl InstallHandler for ModulePoInstaller {
    fn install_handler(&self) -> Result<()> {
        let po = self.installer();
        let destination = self.destination();

        let destination_file = po
            .file_name()
            .map(|name| destination.join(name))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Unable to read filename from path {}", po.display()),
                )
            })?;

        let destination_already_existed = if destination.exists() {
            false
        } else {
            DirBuilder::new().recursive(true).create(&destination)?;
            true
        };

        self.install_language_po_file(po, &destination_file)
            .map_err(|err| {
                self.cleanup_file_failable(&destination_file);
                if destination_already_existed {
                    self.cleanup_directory_failable(&destination)
                }
                err
            })
    }

    fn after_install(&self) -> Result<()> {
        if let Some((from, to)) = &self.rename() {
            uvm_move_dir::move_dir(from, to).chain_err(|| "failed to rename installed module")?;
        }
        Ok(())
    }
}
