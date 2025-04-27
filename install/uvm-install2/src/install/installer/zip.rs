use crate::error::*;
use crate::install::error::InstallerResult;
use crate::install::installer::{Installer, InstallerWithDestination};
use crate::install::{InstallHandler, UnityModule};
use crate::*;
use ::zip;
use std::fs::File;
use thiserror_context::Context;

pub struct Zip;
pub type ModuleZipInstaller = Installer<UnityModule, Zip, InstallerWithDestination>;

impl<V, I> Installer<V, Zip, I> {
    pub fn deploy_zip(&self, installer: &Path, destination: &Path) -> InstallerResult<()> {
        self.deploy_zip_with_rename(installer, destination, |p| p.to_path_buf())
    }

    fn deploy_zip_with_rename<F>(
        &self,
        installer: &Path,
        destination: &Path,
        rename_handler: F,
    ) -> InstallerResult<()>
    where
        F: Fn(&Path) -> PathBuf,
    {
        let file = File::open(installer).context("failed to open zip file")?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).expect("expect file entry at index 0");
            let output_path = rename_handler(&destination.join(file.mangled_name()));
            {
                let comment = file.comment();
                if !comment.is_empty() {
                    trace!("File {} comment: {}", i, comment);
                }
            }

            if (&*file.name()).ends_with('/') {
                debug!(
                    "File {} extracted to \"{}\"",
                    i,
                    output_path.as_path().display()
                );
                fs::DirBuilder::new()
                    .recursive(true)
                    .create(&output_path)
                    .context(format!(
                        "failed to create output path {}",
                        output_path.display()
                    ))?;
            } else {
                debug!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    output_path.as_path().display(),
                    file.size()
                );
                if let Some(p) = output_path.parent() {
                    if !p.exists() {
                        fs::DirBuilder::new()
                            .recursive(true)
                            .create(&p)
                            .context(format!(
                                "failed to create parent directory {} for output path {}",
                                p.display(),
                                output_path.display()
                            ))?;
                    }
                }
                let mut outfile = fs::File::create(&output_path)?;
                io::copy(&mut file, &mut outfile).context(format!(
                    "failed to copy file {} to output path {}",
                    file.name(),
                    output_path.display()
                ))?;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&output_path, fs::Permissions::from_mode(mode)).context(
                        format!(
                            "failed to set permissions on file {}",
                            output_path.display()
                        ),
                    )?;
                }
            }
        }

        Ok(())
    }
}

impl InstallHandler for ModuleZipInstaller {
    fn install_handler(&self) -> InstallerResult<()> {
        let rename = self.rename();

        let rename_handler = |path: &Path| match rename {
            Some((from, to)) => path.strip_prefix(from).map(|p| to.join(p)).unwrap(),
            None => path.to_path_buf(),
        };

        let installer = self.installer();
        let destination = self.destination();

        debug!(
            "install module from zip archive {} to {}",
            installer.display(),
            destination.display()
        );

        self.deploy_zip_with_rename(installer, destination, rename_handler)
    }

    fn error_handler(&self) {
        self.cleanup_directory_failable(self.destination());
    }

    fn installer(&self) -> &Path {
        self.installer()
    }
}
