use crate::error::*;
use crate::*;
use uvm_core::unity::{Component, Module};

use self::exe::*;
use self::msi::ModuleMsiInstaller;
mod exe;
mod msi;

pub fn create_installer<P, I>(
    base_install_path: P,
    installer: I,
    module: &Module,
) -> Result<Box<dyn InstallHandler>>
where
    P: AsRef<Path>,
    I: AsRef<Path>,
{
    let installer = installer.as_ref();
    let base_install_path = base_install_path.as_ref();
    let rename = module.install_rename_from_to(base_install_path);
    let cmd = module.cmd.clone();

    match module.id {
        Component::Editor => parse_editor_installer(installer, base_install_path, rename, cmd),
        _ => {
            let destination = module.install_destination(base_install_path);
            parse_module_installer(installer, destination, rename, cmd)
        }
    }
    .chain_err(|| ErrorKind::InstallerCreationFailed)
}

pub fn parse_editor_installer<P, D, R>(
    installer: P,
    destination: D,
    rename: Option<(R, R)>,
    cmd: Option<String>,
) -> Result<Box<dyn InstallHandler>>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
    R: AsRef<Path>,
{
    let installer = installer.as_ref();
    match installer.extension() {
        Some(ext) if ext == "exe" => Ok(Box::new(EditorExeInstaller::new(
            installer,
            destination,
            cmd,
            rename,
        ))),
        _ => Err(ErrorKind::UnknownInstallerType(
            installer.display().to_string(),
            ".exe".to_string(),
        )
        .into()),
    }
}

pub fn parse_module_installer<P, D, R>(
    installer: P,
    destination: Option<D>,
    rename: Option<(R, R)>,
    cmd: Option<String>,
) -> Result<Box<dyn InstallHandler>>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
    R: AsRef<Path>,
{
    let installer = installer.as_ref();
    match installer.extension() {
        Some(ext) if ext == "exe" => {
            if let Some(destination) = destination {
                Ok(Box::new(ModuleExeTargetInstaller::new(
                    installer,
                    destination,
                    cmd,
                    rename,
                )))
            } else {
                Ok(Box::new(ModuleExeInstaller::new(installer, cmd, rename)))
            }
        }

        Some(ext) if ext == "zip" => {
            if let Some(destination) = destination {
                Ok(Box::new(ModuleZipInstaller::new(
                    installer,
                    destination,
                    rename,
                )))
            } else {
                Err(ErrorKind::MissingDestination("zip".to_string()).into())
            }
        }

        Some(ext) if ext == "po" => {
            if let Some(destination) = destination {
                Ok(Box::new(ModulePoInstaller::new(
                    installer,
                    destination,
                    rename,
                )))
            } else {
                Err(ErrorKind::MissingDestination("po".to_string()).into())
            }
        }

        Some(ext) if ext == "msi" => {
            if let Some(cmd) = cmd {
                Ok(Box::new(ModuleMsiInstaller::new(installer, cmd, rename)))
            } else {
                Err(ErrorKind::MissingCommand("msi".to_string()).into())
            }
        }

        _ => Err(ErrorKind::UnknownInstallerType(
            installer.display().to_string(),
            ".exe".to_string(),
        )
        .into()),
    }
}

impl<V, T, I> Installer<V, T, I> {
    fn install_from_temp_command<P>(&self, command: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let command = command.as_ref();
        let install_process = Command::new(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| {
                error!("error while spawning installer");
                err
            })?;
        let output = install_process.wait_with_output()?;
        if !output.status.success() {
            return Err(format!(
                "failed to install:\
                 {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Ok(())
    }
}
