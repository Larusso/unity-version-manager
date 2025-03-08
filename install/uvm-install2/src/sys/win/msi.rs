use crate::error::*;
use crate::*;
use std::io::Write;
use tempfile::Builder;

pub struct Msi;
pub type ModuleMsiInstaller = Installer<UnityModule, Msi, InstallerWithCommand>;

impl InstallHandler for ModuleMsiInstaller {
    fn install_handler(&self) -> Result<()> {
        let installer = self.installer();

        debug!("install api module from installer msi");
        let mut install_helper = Builder::new().suffix(".cmd").rand_bytes(20).tempfile()?;

        info!(
            "create install helper script {}",
            install_helper.path().display()
        );

        {
            let script = install_helper.as_file_mut();

            let install_command = self
                .cmd()
                .replace("/i", &format!(r#"/i "{}""#, installer.display()));

            trace!("install helper script content:");
            writeln!(script, "ECHO OFF")?;
            trace!("{}", &install_command);
            writeln!(script, "{}", install_command)?;
        }

        info!("install {}", installer.display());

        let installer_script = install_helper.into_temp_path();
        self.install_from_temp_command(&installer_script)?;
        installer_script.close()?;
        Ok(())
    }

    fn after_install(&self) -> Result<()> {
        if let Some((from, to)) = &self.rename() {
            uvm_move_dir::move_dir(from, to).chain_err(|| "failed to rename installed module")?;
        }
        Ok(())
    }
}
