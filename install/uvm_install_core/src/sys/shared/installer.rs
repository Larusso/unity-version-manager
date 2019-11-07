use std::path::{Path, PathBuf};
use std::{fs, io};
use std::fs::DirBuilder;
use log::*;

pub fn install_po_file<P, D>(po: P, destination: D) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    let po = po.as_ref();
    let destination = destination.as_ref();

    let destination_file = po
        .file_name()
        .map(|name| destination.join(name))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Unable to read filename from path {}", po.display()),
            )
        })?;

    let destination_already_existed = if destination.exists() {
        false
    } else {
        DirBuilder::new().recursive(true).create(&destination)?;
        true
    };

    install_language_po_file(po, &destination_file).map_err(|err| {
        cleanup_file_failable(&destination_file);
        if destination_already_existed {
            cleanup_directory_failable(&destination)
        }
        err
    })

}

fn install_language_po_file<P, D>(po: P, destination: D) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    let po = po.as_ref();
    let destination = destination.as_ref();
    debug!("Copy po file {} to {}", po.display(), destination.display());
    fs::copy(po, destination)?;
    Ok(())
}

pub fn install_module_from_zip<P, D>(installer: P, destination: D) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    let installer = installer.as_ref();
    let destination = destination.as_ref();

    debug!(
        "install module from zip archive {} to {}",
        installer.display(),
        destination.display()
    );

    debug!("deploy zip archive to {}", destination.display());
    deploy_zip(installer, destination)?;
    Ok(())
}

pub fn deploy_zip<P: AsRef<Path>, D: AsRef<Path>>(installer: P, destination: D) -> io::Result<()> {
    use std::fs::File;

    let installer = installer.as_ref();
    let destination = destination.as_ref();

    let file = File::open(installer)?;

    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = destination.join(file.sanitized_name());

        {
            let comment = file.comment();
            if !comment.is_empty() {
                trace!("File {} comment: {}", i, comment);
            }
        }

        if (&*file.name()).ends_with('/') {
            debug!("File {} extracted to \"{}\"", i, outpath.as_path().display());
            std::fs::DirBuilder::new().recursive(true).create(&outpath)?;
        } else {
            debug!("File {} extracted to \"{}\" ({} bytes)", i, outpath.as_path().display(), file.size());
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::DirBuilder::new().recursive(true).create(&p)?;
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

pub fn find_payload<P>(dir: P) -> io::Result<PathBuf>
where P: AsRef<Path> {
    let dir = dir.as_ref();
    debug!("find paylod in unpacked installer {}", dir.display());
    let mut files = fs::read_dir(dir).and_then(|read_dir| {
        Ok(read_dir.filter_map(io::Result::ok))
    }).map_err(|_err| {
        io::Error::new(
            io::ErrorKind::Other,
            format!(
                "can't iterate files in extracted payload {}",
                &dir.display()
            ),
        )
    })?;

    files.find(|entry| {
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
    .and_then(|path| {
        match path.file_name() {
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
        }
    })
    .map(|path| {
        debug!("Found payload {}", path.display());
        path
    })
}

pub fn clean_directory<P: AsRef<Path>>(dir: P) -> io::Result<()> {
    let dir = dir.as_ref();
    debug!("clean output directory {}", dir.display());
    if dir.exists() && dir.is_dir() {
        debug!(
            "directory exists, delete directory and create empty directory at {}",
            dir.display()
        );
        fs::remove_dir_all(dir)?;
    }
    DirBuilder::new().recursive(true).create(dir)
}

pub fn cleanup_file_failable<P: AsRef<Path>>(file: P) {
    let file = file.as_ref();
    if file.exists() && file.is_file() {
        debug!("cleanup file {}", &file.display());
        fs::remove_file(file).unwrap_or_else(|err| {
            error!("failed to cleanup file {}", &file.display());
            error!("{}", err);
        });
    }
}

pub fn cleanup_directory_failable<P: AsRef<Path>>(dir: P) {
    let dir = dir.as_ref();
    if dir.exists() && dir.is_dir() {
        debug!("cleanup directory {}", dir.display());
        fs::remove_dir_all(dir).unwrap_or_else(|err| {
            error!("failed to cleanup directory {}", dir.display());
            error!("{}", err);
        })
    }
}

pub fn cleanup_pkg<D: AsRef<Path>>(tmp_destination: D) -> io::Result<()> {
    let tmp_destination = tmp_destination.as_ref();
    debug!("cleanup {}", &tmp_destination.display());
    fs::remove_dir_all(tmp_destination)
}
