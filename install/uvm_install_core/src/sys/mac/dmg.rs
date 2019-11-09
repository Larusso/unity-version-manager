use crate::*;

pub struct Dmg;
pub type ModuleDmgWithDestinationInstaller = Installer<UnityModule, Dmg, InstallerWithDestination>;
pub type ModuleDmgInstaller = Installer<UnityModule, Dmg, BaseInstaller>;

impl<V, I> Installer<V, Dmg, I> {
    // TODO use fs_extra or similar
    // Maybe this is mac specific?
    fn copy_dir<P, D>(&self, source: P, destination: D) -> Result<()>
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
            return Err(format!(
                "failed to copy {} to {}\n{}",
                source.display(),
                destination.display(),
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Ok(())
    }

    fn find_file_in_dir<P, F>(&self, dir: P, predicate: F) -> Result<PathBuf>
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
            .chain_err(|| "failed to find file")
    }

    fn install_module_from_dmg(&self, dmg_file: &Path, destination: &Path) -> Result<()> {
        use ::dmg::Attach;

        debug!(
            "install from dmg {} to {}",
            dmg_file.display(),
            destination.display()
        );
        let volume = Attach::new(dmg_file).with()?;
        debug!("installer mounted at {}", volume.mount_point.display());

        let app_path = self.find_file_in_dir(&volume.mount_point, |entry| {
            entry.file_name().to_str().unwrap().ends_with(".app")
        })?;

        self.copy_dir(app_path, destination)?;
        Ok(())
    }
}

impl InstallHandler for ModuleDmgInstaller {
    fn install_handler(&self) -> Result<()> {
        let installer = self.installer();
        let destination = Path::new("/Applications");
        self.install_module_from_dmg(installer, destination)
    }

    fn after_install(&self) -> Result<()> {
        if let Some((from, to)) = &self.rename() {
            uvm_move_dir::move_dir(from, to).chain_err(|| "failed to rename installed module")?;
        }
        Ok(())
    }
}

impl InstallHandler for ModuleDmgWithDestinationInstaller {
    fn install_handler(&self) -> Result<()> {
        let installer = self.installer();
        let destination = self.destination();
        self.install_module_from_dmg(installer, destination)
    }

    fn after_install(&self) -> Result<()> {
        if let Some((from, to)) = &self.rename() {
            uvm_move_dir::move_dir(from, to).chain_err(|| "failed to rename installed module")?;
        }
        Ok(())
    }
}
