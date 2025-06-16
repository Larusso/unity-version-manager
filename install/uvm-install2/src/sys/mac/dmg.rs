use crate::install::error::InstallerErrorInner::CopyFailed;
use crate::install::error::{InstallerError, InstallerErrorInner, InstallerResult};
use crate::install::installer::{BaseInstaller, Installer, InstallerWithDestination};
use crate::install::{InstallHandler, UnityModule};
use log::{debug, info};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fs, io};
use thiserror_context::Context;

pub struct Dmg;
pub type ModuleDmgWithDestinationInstaller = Installer<UnityModule, Dmg, InstallerWithDestination>;
pub type ModuleDmgInstaller = Installer<UnityModule, Dmg, BaseInstaller>;

impl<V, I> Installer<V, Dmg, I> {
    // TODO use fs_extra or similar
    // Maybe this is mac specific?
    fn copy_dir<P, D>(&self, source: P, destination: D) -> InstallerResult<()>
    where
        P: AsRef<Path>,
        D: AsRef<Path>,
    {
        let source = source.as_ref();
        let destination = destination.as_ref();

        debug!("Copy {} to {}", source.display(), destination.display());
        let child = Command::new("cp")
            .arg("-a")
            .arg(source)
            .arg(destination)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(CopyFailed(
                source.display().to_string(),
                destination.display().to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            )
            .into());
        }
        Ok(())
    }

    fn find_file_in_dir<P, F>(&self, dir: P, predicate: F) -> InstallerResult<PathBuf>
    where
        P: AsRef<Path>,
        F: FnMut(&std::fs::DirEntry) -> bool,
    {
        let dir = dir.as_ref();
        debug!("find file in directory {}", dir.display());
        fs::read_dir(dir)
            .and_then(|read_dir| {
                read_dir
                    .filter_map(io::Result::ok)
                    .find(predicate)
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("can't locate file in {}", &dir.display()),
                        )
                    })
                    .map(|entry| entry.path())
            })
            .map_err(|err| InstallerErrorInner::IO(err).into())
    }

    fn install_module_from_dmg(&self, dmg_file: &Path, destination: &Path) -> InstallerResult<()> {
        use ::dmg::Attach;

        debug!(
            "install from dmg {} to {}",
            dmg_file.display(),
            destination.display()
        );
        let volume = Attach::new(dmg_file).with()?;
        debug!("installer mounted at {}", volume.mount_point.display());

        let app_path = self
            .find_file_in_dir(&volume.mount_point, |entry| {
                entry.file_name().to_str().unwrap().ends_with(".app")
            })
            .context("failed to find .app in package")?;

        self.copy_dir(app_path, destination)
            .context("failed to copy .app contents to destination")?;
        Ok(())
    }
}

impl InstallHandler for ModuleDmgInstaller {
    fn install_handler(&self) -> InstallerResult<()> {
        let installer = self.installer();
        let destination = Path::new("/Applications");
        self.install_module_from_dmg(installer, destination)
    }

    fn installer(&self) -> &Path {
        self.installer()
    }

    fn after_install(&self) -> InstallerResult<()> {
        if let Some((from, to)) = &self.rename() {
            uvm_move_dir::move_dir(from, to).context("failed to rename installed module")?;
        }
        Ok(())
    }
}

impl InstallHandler for ModuleDmgWithDestinationInstaller {
    fn install_handler(&self) -> InstallerResult<()> {
        let installer = self.installer();
        let destination = self.destination();
        self.install_module_from_dmg(installer, destination)
    }

    fn installer(&self) -> &Path {
        self.installer()
    }

    fn after_install(&self) -> InstallerResult<()> {
        if let Some((from, to)) = &self.rename() {
            uvm_move_dir::move_dir(from, to).context("failed to rename installed module")?;
        }
        Ok(())
    }

    fn before_install(&self) -> InstallerResult<()> {
        if self.destination().exists() {
            if self.destination().is_dir() {
                info!("Destination directory {} already exists, removing it", self.destination().display());
                fs::remove_dir_all(self.destination()).context("failed to remove the existing destination directory")?;
            } else {
                info!("Destination file {} already exists, removing it", self.destination().display());
                fs::remove_file(self.destination()).context("failed to remove the existing destination file")?;
            }
        }
        Ok(())
    }
}
