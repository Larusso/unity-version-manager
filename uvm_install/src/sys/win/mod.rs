use std::process::{Command, Stdio};
use crate::*;
use crate::install::error::{InstallerErrorInner, InstallerResult};
use crate::install::error::InstallerErrorInner::Other;
use crate::install::installer::{Installer, ModulePoInstaller, ModuleZipInstaller};
use crate::install::InstallHandler;
use self::exe::*;
use self::msi::ModuleMsiInstaller;
mod exe;
mod msi;

pub fn create_installer<P, I, M>(
    base_install_path: P,
    installer: I,
    module: &M,
) -> InstallerResult<Box<dyn InstallHandler>>
where
    P: AsRef<Path>,
    I: AsRef<Path>,
    M: InstallManifest,
{
    let installer = installer.as_ref();
    let base_install_path = base_install_path.as_ref();
    let rename = module.install_rename_from_to(base_install_path);

    if module.is_editor() {
        parse_editor_installer(installer, base_install_path, rename, Some("-UI=reduced".to_string()))
    } else {
        let destination = module.install_destination(base_install_path);
        parse_module_installer(installer, destination, rename, None)
    }
}

pub fn parse_editor_installer<P, D, R>(
    installer: P,
    destination: D,
    rename: Option<(R, R)>,
    cmd: Option<String>,
) -> InstallerResult<Box<dyn InstallHandler>>
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
        _ => Err(InstallerErrorInner::UnknownInstaller(
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
) -> InstallerResult<Box<dyn InstallHandler>>
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
                Err(InstallerErrorInner::MissingDestination("zip".to_string()).into())
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
                Err(InstallerErrorInner::MissingDestination("po".to_string()).into())
            }
        }

        Some(ext) if ext == "msi" => {
            if let Some(cmd) = cmd {
                Ok(Box::new(ModuleMsiInstaller::new(installer, cmd, rename)))
            } else {
                Err(InstallerErrorInner::MissingCommand("msi".to_string()).into())
            }
        }

        _ => Err(InstallerErrorInner::UnknownInstaller(
            installer.display().to_string(),
            ".exe".to_string(),
        )
        .into()),
    }
}

impl<V, T, I> Installer<V, T, I> {
    fn install_from_temp_command<P>(&self, command: P) -> InstallerResult<()>
    where
        P: AsRef<Path>,
    {
        let command = command.as_ref();
        let install_process = Command::new(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let output = install_process.wait_with_output()?;
        if !output.status.success() {
            return Err(Other(format!(
                "failed to install:\
                 {}",
                String::from_utf8_lossy(&output.stderr)
            ))
            .into());
        }
        Ok(())
    }
}
