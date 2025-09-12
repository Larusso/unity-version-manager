use std::fs::DirBuilder;
use self::pkg::ModulePkgInstaller;
use self::xz::{EditorXzInstaller, ModuleXzInstaller};
use self::zip::EditorZipInstaller;
use crate::*;
use std::path::Path;
use crate::install::error::{InstallerErrorInner, InstallerResult};
use crate::install::installer::{Installer, ModulePoInstaller, ModuleZipInstaller};
use crate::install::InstallHandler;

mod pkg;
mod xz;
mod zip;

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
    let rename = module.install_rename_from_to(base_install_path);

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
        Some(ext) if ext == "zip" => Ok(Box::new(EditorZipInstaller::new(
            installer,
            destination,
            rename,
        ))),
        Some(ext) if ext == "xz" => Ok(Box::new(EditorXzInstaller::new(
            installer,
            destination,
            rename,
        ))),
        _ => Err(InstallerErrorInner::UnknownInstaller(
            installer.display().to_string(),
            ".zip, .xz".to_string(),
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
        Some(ext) if ext == "xz" => {
            if let Some(destination) = destination {
                Ok(Box::new(ModuleXzInstaller::new(
                    installer.to_path_buf(),
                    destination.as_ref().to_path_buf(),
                    rename,
                )))
            } else {
                Err(InstallerErrorInner::MissingDestination("xz".to_string()).into())
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

        Some(ext) if ext == "pkg" => {
            if let Some(destination) = destination {
                Ok(Box::new(ModulePkgInstaller::new(
                    installer.to_path_buf(),
                    destination.as_ref().to_path_buf(),
                    rename,
                )))
            } else {
                Err(InstallerErrorInner::MissingDestination("po".to_string()).into())
            }
        }
        _ => Err(InstallerErrorInner::UnknownInstaller(
            installer.display().to_string(),
            ".pkg, .zip, .xz or .po".to_string(),
        )
        .into()),
    }
}

impl<V, T, I> Installer<V, T, I> {
    pub fn clean_directory<P: AsRef<Path>>(&self, dir: P) -> InstallerResult<()> {
        let dir = dir.as_ref();
        debug!("clean output directory {}", dir.display());
        if dir.exists() && dir.is_dir() {
            debug!(
                "directory exists, delete directory and create empty directory at {}",
                dir.display()
            );
            fs::remove_dir_all(dir)?;
        }
        DirBuilder::new().recursive(true).create(dir)?;
        Ok(())
    }
}
