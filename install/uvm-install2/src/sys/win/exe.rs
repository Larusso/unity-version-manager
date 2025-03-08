use crate::error::*;
use crate::*;
use std::io::Write;
use tempfile::Builder;

pub struct Exe;
pub type EditorExeInstaller =
    Installer<UnityEditor, Exe, InstallerWithDestinationAndOptionalCommand>;
pub type ModuleExeTargetInstaller =
    Installer<UnityModule, Exe, InstallerWithDestinationAndOptionalCommand>;
pub type ModuleExeInstaller = Installer<UnityModule, Exe, InstallerWithOptionalCommand>;

impl<V> InstallHandler for Installer<V, Exe, InstallerWithDestinationAndOptionalCommand> {
    fn install_handler(&self) -> Result<()> {
        let installer = self.installer();
        let destination = self.destination();

        debug!("install api from installer exe");
        let mut install_helper = Builder::new().suffix(".cmd").rand_bytes(20).tempfile()?;

        info!(
            "create install helper script {}",
            install_helper.path().display()
        );

        {
            let script = install_helper.as_file_mut();
            let parameter_option = match self.cmd() {
                Some(parameters) => parameters,
                _ => "/S",
            };

            let destination_option = format!("/D={}", destination.display());

            let install_command = format!(
                r#"CALL "{installer}" {parameters} {destination}"#,
                installer = installer.display(),
                parameters = parameter_option,
                destination = destination_option,
            );

            trace!("install helper script content:");
            writeln!(script, "ECHO OFF")?;
            trace!("{}", &install_command);
            writeln!(script, "{}", install_command)?;
        }

        info!("install {}", installer.display());
        info!("to {}", destination.display());
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

impl InstallHandler for ModuleExeInstaller {
    fn install_handler(&self) -> Result<()> {
        let installer = self.installer();

        debug!("install api from installer exe");
        let mut install_helper = Builder::new().suffix(".cmd").rand_bytes(20).tempfile()?;

        info!(
            "create install helper script {}",
            install_helper.path().display()
        );

        {
            let script = install_helper.as_file_mut();
            let parameter_option = match self.cmd() {
                Some(parameters) => parameters,
                _ => "/S",
            };

            let install_command = format!(
                r#"CALL "{installer}" {parameters}"#,
                installer = installer.display(),
                parameters = parameter_option,
            );

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
