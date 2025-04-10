use std::fs::File;
use crate::error::*;
use crate::*;
use ::zip;
use crate::install::installer::{Installer, InstallerWithDestination};
use crate::install::{InstallHandler, UnityModule};
use crate::install::error::InstallerResult;

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
        let file = File::open(installer)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let outpath = rename_handler(&destination.join(file.mangled_name()));
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
                    outpath.as_path().display()
                );
                fs::DirBuilder::new()
                    .recursive(true)
                    .create(&outpath)?;
            } else {
                debug!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    outpath.as_path().display(),
                    file.size()
                );
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::DirBuilder::new().recursive(true).create(&p)?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
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
}
