use crate::error::*;
use crate::*;

pub type EditorPkgInstaller = Installer<UnityEditor, Pkg, InstallerWithDestination>;
pub type ModulePkgNativeInstaller = Installer<UnityModule, Pkg, BaseInstaller>;
pub type ModulePkgInstaller = Installer<UnityModule, Pkg, InstallerWithDestination>;

impl<V, I> Installer<V, Pkg, I> {
    fn move_files<P: AsRef<Path>, D: AsRef<Path>>(&self, source: P, destination: D) -> Result<()> {
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

    fn xar<P: AsRef<Path>, D: AsRef<Path>>(&self, installer: P, destination: D) -> Result<()> {
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
            return Err(format!(
                "failed to extract installer from pkg package:/n{}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Ok(())
    }

    fn untar<P: AsRef<Path>, D: AsRef<Path>>(
        &self,
        base_payload_path: P,
        destination: D,
    ) -> Result<()> {
        let base_payload_path = base_payload_path.as_ref();
        let payload = self.find_payload(&base_payload_path)?;
        debug!("untar payload at {}", payload.display());
        self.tar(&payload, destination)
    }

    fn tar<P: AsRef<Path>, D: AsRef<Path>>(&self, source: P, destination: D) -> Result<()> {
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
            return Err(format!(
                "failed to untar payload:/n{}",
                String::from_utf8_lossy(&tar_output.stderr)
            )
            .into());
        }

        Ok(())
    }
}

impl EditorPkgInstaller {
    fn cleanup_editor<D: AsRef<Path>>(&self, destination: D) -> Result<()> {
        use std::fs;
        let destination = destination.as_ref();
        let tmp_unity_directory = destination.join("Unity");
        if !tmp_unity_directory.exists() {
            return Err("error extracting installer".into());
        }

        self.move_files(&tmp_unity_directory, &destination)?;
        fs::remove_dir_all(&tmp_unity_directory)?;
        Ok(())
    }
}

impl ModulePkgInstaller {
    fn cleanup_ios_support<D: AsRef<Path>>(&self, destination: D) -> Result<()> {
        use std::fs;
        let destination = destination.as_ref();
        debug!("cleanup ios support package at {}", destination.display());

        let tmp_ios_support_directory = destination.join("iOSSupport");
        if tmp_ios_support_directory.exists() {
            debug!("move ios files from {} to {}", tmp_ios_support_directory.display(), destination.display());
            self.move_files(&tmp_ios_support_directory, &destination)?;
            fs::remove_dir_all(&tmp_ios_support_directory)?;
        }
        Ok(())
    }
}

impl InstallHandler for EditorPkgInstaller {
    fn install_handler(&self) -> Result<()> {
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
}

impl InstallHandler for ModulePkgInstaller {
    fn install_handler(&self) -> Result<()> {
        let destination = self.destination();
        let installer = self.installer();

        debug!(
            "install module from pkg {} to {}",
            installer.display(),
            destination.display()
        );

        let tmp_destination = destination.join("tmp");
        DirBuilder::new().recursive(true).create(&tmp_destination)?;
        self.xar(installer, &tmp_destination)?;
        self.untar(&tmp_destination, destination)?;
        self.cleanup_ios_support(destination)?;
        self.cleanup(&tmp_destination)?;
        Ok(())
    }

    fn after_install(&self) -> Result<()> {
        if let Some((from, to)) = &self.rename() {
            uvm_move_dir::move_dir(from, to).chain_err(|| "failed to rename installed module")?;
        }
        Ok(())
    }

    fn error_handler(&self) {
        self.cleanup_directory_failable(&self.destination());
    }
}

impl InstallHandler for ModulePkgNativeInstaller {
    fn install_handler(&self) -> Result<()> {
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
            return Err(format!(
                "failed to install {}\n{}",
                installer.display(),
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Ok(())
    }

    fn after_install(&self) -> Result<()> {
        if let Some((from, to)) = &self.rename() {
            uvm_move_dir::move_dir(from, to).chain_err(|| "failed to rename installed module")?;
        }
        Ok(())
    }
}
