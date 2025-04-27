use crate::install::error::InstallerErrorInner::{InstallationFailed, InstallerCreateFailed};
use crate::install::error::{InstallerError, InstallerResult};
use crate::install::installer::{BaseInstaller, Installer, InstallerWithDestination, Pkg};
use crate::install::{InstallHandler, UnityEditor, UnityModule};
use log::{debug, warn};
use std::fs::DirBuilder;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{fs, io};
use thiserror_context::Context;

pub type EditorPkgInstaller = Installer<UnityEditor, Pkg, InstallerWithDestination>;
pub type ModulePkgNativeInstaller = Installer<UnityModule, Pkg, BaseInstaller>;
pub type ModulePkgInstaller = Installer<UnityModule, Pkg, InstallerWithDestination>;

impl<V, I> Installer<V, Pkg, I> {
    fn move_files<P: AsRef<Path>, D: AsRef<Path>>(
        &self,
        source: P,
        destination: D,
    ) -> InstallerResult<()> {
        let source = source.as_ref();
        let destination = destination.as_ref();
        debug!(
            "move all files from {} into {}",
            source.display(),
            destination.display()
        );
        for entry in fs::read_dir(&source)?.filter_map(io::Result::ok) {
            let new_location = destination.join(entry.file_name());
            debug!(
                "move {} to {}",
                entry.path().display(),
                new_location.display()
            );
            if new_location.exists() && new_location.is_dir() {
                warn!(
                    "target directory already exists. {}",
                    new_location.display()
                );
                warn!("delete directory: {}", new_location.display());
                fs::remove_dir_all(&new_location)?;
            }

            fs::rename(entry.path(), &new_location)?;
        }
        Ok(())
    }

    fn xar<P: AsRef<Path>, D: AsRef<Path>>(
        &self,
        installer: P,
        destination: D,
    ) -> InstallerResult<()> {
        let installer = installer.as_ref();
        let destination = destination.as_ref();

        debug!(
            "unpack installer {} to temp destination {}",
            installer.display(),
            destination.display()
        );
        let child = Command::new("xar")
            .arg("-x")
            .arg("-f")
            .arg(installer)
            .arg("-C")
            .arg(destination)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(InstallerCreateFailed(format!(
                "failed to extract installer from pkg package:/n{}",
                String::from_utf8_lossy(&output.stderr)
            ))
            .into());
        }
        Ok(())
    }

    fn untar<P: AsRef<Path>, D: AsRef<Path>>(
        &self,
        base_payload_path: P,
        destination: D,
    ) -> Result<(), InstallerError> {
        let base_payload_path = base_payload_path.as_ref();
        let payload = self.find_payload(&base_payload_path).context("unable to find payload in package")?;
        debug!("untar payload at {}", payload.display());
        self.tar(&payload, destination)
    }

    fn tar<P: AsRef<Path>, D: AsRef<Path>>(
        &self,
        source: P,
        destination: D,
    ) -> InstallerResult<()> {
        let source = source.as_ref();
        let destination = destination.as_ref();

        let tar_child = Command::new("tar")
            .arg("-C")
            .arg(destination)
            .arg("-zmxf")
            .arg(source)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let tar_output = tar_child.wait_with_output()?;
        if !tar_output.status.success() {
            return Err(InstallerCreateFailed(format!(
                "failed to untar payload:/n{}",
                String::from_utf8_lossy(&tar_output.stderr)
            ))
            .into());
        }

        Ok(())
    }
}

impl EditorPkgInstaller {
    fn cleanup_editor<D: AsRef<Path>>(&self, destination: D) -> InstallerResult<()> {
        use std::fs;
        let destination = destination.as_ref();
        let tmp_unity_directory = destination.join("Unity");
        if !tmp_unity_directory.exists() {
            return Err(InstallerCreateFailed(
                "Failed to create temp unity install directory".to_string(),
            )
            .into());
        }

        self.move_files(&tmp_unity_directory, &destination)?;
        fs::remove_dir_all(&tmp_unity_directory)?;
        Ok(())
    }
}

impl ModulePkgInstaller {
    fn cleanup_ios_support<D: AsRef<Path>>(&self, destination: D) -> InstallerResult<()> {
        use std::fs;
        let destination = destination.as_ref();
        debug!("cleanup ios support package at {}", destination.display());

        let tmp_ios_support_directory = destination.join("iOSSupport");
        if tmp_ios_support_directory.exists() {
            debug!(
                "move ios files from {} to {}",
                tmp_ios_support_directory.display(),
                destination.display()
            );
            self.move_files(&tmp_ios_support_directory, &destination)?;
            fs::remove_dir_all(&tmp_ios_support_directory)?;
        }
        Ok(())
    }
}

impl InstallHandler for EditorPkgInstaller {
    fn install_handler(&self) -> InstallerResult<()> {
        let destination = self.destination();
        let installer = self.installer();

        debug!(
            "install editor from pkg {} to {}",
            installer.display(),
            destination.display()
        );

        let tmp_destination = destination.join("tmp");
        DirBuilder::new().recursive(true).create(&tmp_destination)?;
        self.xar(installer, &tmp_destination)?;
        self.untar(&tmp_destination, destination)?;
        self.cleanup_editor(destination)?;
        self.cleanup(&tmp_destination)?;

        Ok(())
    }

    fn error_handler(&self) {
        self.cleanup_directory_failable(&self.destination());
    }

    fn installer(&self) -> &Path {
        self.installer()
    }
}

impl InstallHandler for ModulePkgInstaller {
    fn install_handler(&self) -> InstallerResult<()> {
        let destination = self.destination();
        let installer = self.installer();

        debug!(
            "install module from pkg {} to {}",
            installer.display(),
            destination.display()
        );

        let tmp_destination = destination.join("tmp");
        DirBuilder::new().recursive(true).create(&tmp_destination).context("failed to create temp install directory")?;
        self.xar(installer, &tmp_destination)?;
        self.untar(&tmp_destination, destination).context("failed to unpack the payload")?;
        self.cleanup_ios_support(destination).context("failed to cleanup ios support destination")?;
        self.cleanup(&tmp_destination).context("failed to cleanup temp files")?;
        Ok(())
    }

    fn after_install(&self) -> InstallerResult<()> {
        if let Some((from, to)) = &self.rename() {
            uvm_move_dir::move_dir(from, to)?;
        }
        Ok(())
    }

    fn error_handler(&self) {
        self.cleanup_directory_failable(&self.destination());
    }

    fn installer(&self) -> &Path {
        self.installer()
    }
}

impl InstallHandler for ModulePkgNativeInstaller {
    fn install_handler(&self) -> InstallerResult<()> {
        let installer = self.installer();
        debug!("install from pkg {}", installer.display());

        let child = Command::new("sudo")
            .arg("installer")
            .arg("-package")
            .arg(installer)
            .arg("-target")
            .arg("/")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(InstallationFailed(
                installer.display().to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            )
            .into());
        }
        Ok(())
    }

    fn installer(&self) -> &Path {
        self.installer()
    }

    fn after_install(&self) -> InstallerResult<()> {
        if let Some((from, to)) = &self.rename() {
            uvm_move_dir::move_dir(from, to)?
            //.context("failed to rename installed module")?;
        }
        Ok(())
    }
}
