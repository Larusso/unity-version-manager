use crate::*;
use std::fs::DirBuilder;
pub struct Xz;
pub type EditorXzInstaller = Installer<UnityEditor, Xz, InstallerWithDestination>;
pub type ModuleXzInstaller = Installer<UnityModule, Xz, InstallerWithDestination>;

impl<V, I> Installer<V, Xz, I> {
    fn untar<P, D>(&self, source: P, destination: D) -> Result<()>
    where
        P: AsRef<Path>,
        D: AsRef<Path>,
    {
        let source = source.as_ref();
        let destination = destination.as_ref();

        debug!(
            "untar archive {} to {}",
            source.display(),
            destination.display()
        );
        let tar_child = Command::new("tar")
            .arg("-C")
            .arg(destination)
            .arg("-amxf")
            .arg(source)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
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

impl InstallHandler for EditorXzInstaller {
    fn before_install(&self) -> Result<()> {
        self.clean_directory(self.destination())
    }

    fn install_handler(&self) -> Result<()> {
        debug!("install editor from xz archive");
        self.untar(self.installer(), self.destination())
    }
}

impl InstallHandler for ModuleXzInstaller {
    fn install_handler(&self) -> Result<()> {
        debug!(
            "install module from xz archive {} to {}",
            self.installer().display(),
            self.destination().display()
        );

        let destination = self.destination();
        let installer = self.installer();

        let destination = if destination.ends_with("Editor/Data/PlaybackEngines") {
            destination
                .parent()
                .and_then(|f| f.parent())
                .and_then(|f| f.parent())
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!(
                            "Can't determine destination for {} and destination {}",
                            &installer.display(),
                            destination.display()
                        ),
                    )
                })?
        } else {
            destination
        };

        DirBuilder::new().recursive(true).create(destination)?;
        self.untar(installer, destination)
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
