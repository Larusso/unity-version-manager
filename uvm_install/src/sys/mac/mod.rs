use self::dmg::{ModuleDmgInstaller, ModuleDmgWithDestinationInstaller};
use self::pkg::{EditorPkgInstaller, ModulePkgInstaller, ModulePkgNativeInstaller};
use crate::install::error::{InstallerErrorInner, InstallerResult};
use crate::install::installer::{ModulePoInstaller, ModuleZipInstaller};
use crate::install::InstallHandler;
use crate::InstallManifest;
use log::{info, warn};
use mach_object::{get_arch_name_from_types, OFile};
use std::fs::File;
use std::io;
use std::io::{Cursor, Read};
use std::path::Path;
use std::str::FromStr;
use sysctl::Sysctl;
use unity_hub::unity::UnityInstallation;
use unity_version::Version;

mod dmg;
mod pkg;
mod arch;
pub use arch::ensure_installation_architecture_is_correct;

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
    let base_install_path = base_install_path.as_ref();
    let rename = module.install_rename_from_to(&base_install_path);

    if module.is_editor() {
        parse_editor_installer(installer, &base_install_path, rename)
    } else {
        let destination = module.install_destination(&base_install_path);
        parse_module_installer(installer, destination, rename)
    }
}

fn parse_editor_installer<P, D, R>(
    installer: P,
    destination: D,
    rename: Option<(R, R)>,
) -> InstallerResult<Box<dyn InstallHandler>>
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
        _ => Err(InstallerErrorInner::UnknownInstaller(
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
) -> InstallerResult<Box<dyn InstallHandler>>
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
                Err(InstallerErrorInner::MissingDestination("zip".to_string()).into())
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
                Err(InstallerErrorInner::MissingDestination("po".to_string()).into())
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
        _ => Err(InstallerErrorInner::UnknownInstaller(
            installer.display().to_string(),
            ".pkg, .zip, .dmg or .po".to_string(),
        )
        .into()),
    }
}
