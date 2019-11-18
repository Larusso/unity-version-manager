use self::dmg::{ModuleDmgInstaller, ModuleDmgWithDestinationInstaller};
use self::pkg::{EditorPkgInstaller, ModulePkgInstaller, ModulePkgNativeInstaller};
use crate::error::*;
use crate::installer::{ModulePoInstaller, ModuleZipInstaller};
use crate::*;
use std::path::Path;
use uvm_core::unity::Component;
use uvm_core::unity::Module;
mod dmg;
mod pkg;

pub fn create_installer<P, I>(
    base_install_path: P,
    installer: I,
    module: &Module,
) -> Result<Box<dyn InstallHandler>>
where
    P: AsRef<Path>,
    I: AsRef<Path>,
{
    let base_install_path = base_install_path.as_ref();
    let rename = module.install_rename_from_to(base_install_path);

    match module.id {
        Component::Editor => parse_editor_installer(installer, base_install_path, rename),
        _ => {
            let destination = module.install_destination(base_install_path);
            parse_module_installer(installer, destination, rename)
        }
    }
    .chain_err(|| ErrorKind::InstallerCreationFailed)
}

fn parse_editor_installer<P, D, R>(
    installer: P,
    destination: D,
    rename: Option<(R, R)>,
) -> Result<Box<dyn InstallHandler>>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
    R: AsRef<Path>,
{
    let installer = installer.as_ref();
    match installer.extension() {
        Some(ext) if ext == "pkg" => Ok(Box::new(EditorPkgInstaller::new(
            installer,
            destination,
            rename,
        ))),
        _ => Err(ErrorKind::UnknownInstallerType(
            installer.display().to_string(),
            ".pkg".to_string(),
        )
        .into()),
    }
}

fn parse_module_installer<P, D, R>(
    installer: P,
    destination: Option<D>,
    rename: Option<(R, R)>,
) -> Result<Box<dyn InstallHandler>>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
    R: AsRef<Path>,
{
    let installer = installer.as_ref();

    match installer.extension() {
        Some(ext) if ext == "pkg" => {
            if let Some(destination) = destination {
                Ok(Box::new(ModulePkgInstaller::new(
                    installer.to_path_buf(),
                    destination.as_ref().to_path_buf(),
                    rename,
                )))
            } else {
                let i = ModulePkgNativeInstaller::new(installer, rename);
                Ok(Box::new(i))
            }
        }

        Some(ext) if ext == "zip" => {
            if let Some(destination) = destination {
                Ok(Box::new(ModuleZipInstaller::new(
                    installer.to_path_buf(),
                    destination.as_ref().to_path_buf(),
                    rename,
                )))
            } else {
                Err(ErrorKind::MissingDestination("zip".to_string()).into())
            }
        }

        Some(ext) if ext == "po" => {
            if let Some(destination) = destination {
                Ok(Box::new(ModulePoInstaller::new(
                    installer.to_path_buf(),
                    destination.as_ref().to_path_buf(),
                    rename,
                )))
            } else {
                Err(ErrorKind::MissingDestination("po".to_string()).into())
            }
        }

        Some(ext) if ext == "dmg" => {
            if let Some(destination) = destination {
                Ok(Box::new(ModuleDmgWithDestinationInstaller::new(
                    installer.to_path_buf(),
                    destination.as_ref().to_path_buf(),
                    rename,
                )))
            } else {
                Ok(Box::new(ModuleDmgInstaller::new(
                    installer.to_path_buf(),
                    rename,
                )))
            }
        }
        _ => Err(ErrorKind::UnknownInstallerType(
            installer.display().to_string(),
            ".pkg, .zip, .dmg or .po".to_string(),
        )
        .into()),
    }
}
