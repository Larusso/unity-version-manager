use anyhow::Context;
use crate::error::*;
use crate::*;
use crate::install::error::{InstallerError, InstallerErrorInner, InstallerResult};
use crate::install::installer::Installer;

pub struct Pkg;

impl<V, I> Installer<V, Pkg, I> {
    pub fn cleanup<D: AsRef<Path>>(&self, tmp_destination: D) -> InstallerResult<()> {
        let tmp_destination = tmp_destination.as_ref();
        debug!("cleanup {}", &tmp_destination.display());
        fs::remove_dir_all(tmp_destination).map_err(|err| InstallerErrorInner::from(err).into())
    }

    pub fn find_payload<P>(&self, dir: P) -> InstallerResult<PathBuf>
    where
        P: AsRef<Path>,
    {
        let dir = dir.as_ref();
        debug!("find paylod in unpacked installer {}", dir.display());
        let mut files = fs::read_dir(dir)
            .and_then(|read_dir| Ok(read_dir.filter_map(io::Result::ok)))
            .map_err(|_err| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "can't iterate files in extracted payload {}",
                        &dir.display()
                    ),
                )
            })?;

        files
            .find(|entry| {
                if let Some(file_name) = entry.file_name().to_str() {
                    file_name.ends_with(".pkg.tmp") || file_name == "Payload~"
                } else {
                    false
                }
            })
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "can't locate *.pkg.tmp directory or Payload~ in extracted installer at {}",
                        &dir.display()
                    ),
                )
            })
            .map(|entry| entry.path())
            .and_then(|path| match path.file_name() {
                Some(name) if name == "Payload~" => Ok(path),
                _ => {
                    let payload_path = path.join("Payload");
                    if payload_path.exists() {
                        Ok(payload_path)
                    } else {
                        Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!(
                                "can't locate Payload directory in extracted installer at {}",
                                &dir.display()
                            ),
                        ))
                    }
                }
            })
            .map(|path| {
                debug!("Found payload {}", path.display());
                path
            }).map_err(|err| InstallerErrorInner::IO(err).into())
    }
}
