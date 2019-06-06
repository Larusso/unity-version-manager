use std::io;
use std::path::PathBuf;
use std::fs::File;
use std::fs::DirBuilder;
use std::ffi::{OsString, OsStr};
use std::fs;
use unzip::Unzipper;
use std::process::{Command, Stdio};
use std::io::Write;
use std::io::Read;

pub fn install_editor(installer: &PathBuf, destination: &PathBuf) -> io::Result<()> {
    _install_editor(installer, destination)
    .map_err(|err| {
        if destination.exists() {
            debug!("Delete destination directory after failure {}", destination.display());
            fs::remove_dir_all(destination).unwrap_or_else(|err| {
                error!("Failed to cleanup destination {}", destination.display());
                error!("{}", err);
            })
        }
        err
    })
}

fn _install_editor(installer: &PathBuf, destination: &PathBuf) -> io::Result<()> {
    debug!(
        "install editor to destination: {} with installer: {}",
        destination.display(),
        installer.display()
    );

    if installer.extension() == Some(OsStr::new("zip")) {
        debug!("install editor from zip archive");
        cleanDirectory(destination)?;
        deploy_zip(installer, destination)?;
        return Ok(());
    } else if installer.extension() == Some(OsStr::new("xz")) {
        debug!("install editor from xz archive");
        cleanDirectory(destination)?;
        untar(installer, destination)?;
        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("Wrong installer. Expect .zip or .xz {:?} - {}", &installer.extension(), &installer.display()),
    ))
}

pub fn install_module(installer: &PathBuf, destination: &PathBuf) -> io::Result<()> {
    _install_module(installer, destination)
    .map_err(|err| {
        if destination.exists() {
            debug!("Delete destination directory after failure {}", destination.display());
            fs::remove_dir_all(destination).unwrap_or_else(|err| {
                error!("Failed to cleanup destination {}", destination.display());
                error!("{}", err);
            })
        }
        err
    })
}

fn _install_module(installer: &PathBuf, destination: &PathBuf) -> io::Result<()> {
    debug!(
        "install component {} to {}",
        &installer.display(),
        &destination.display()
    );

    if installer.extension() == Some(OsStr::new("zip")) {
        debug!("install component from zip archive");
        cleanDirectory(destination)?;
        deploy_zip(installer, destination)?;
        return Ok(());
    } else if installer.extension() == Some(OsStr::new("xz")) {
        let destination = if destination.ends_with("Editor/Data/PlaybackEngines") {
            destination.parent()
                .and_then(|f| f.parent())
                .and_then(|f| f.parent())
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Can't determine destination for {} and destination {}", &installer.display(), destination.display()),
                ))?
        } else {
            destination
        };

        DirBuilder::new().recursive(true).create(destination)?;
        untar(installer, &destination.to_path_buf())?;
        return Ok(());
    } else if installer.extension() == Some(OsStr::new("po")) {
        debug!("install component po file");
        let destination = destination.join(installer.file_name().unwrap().to_str().unwrap());
        fs::copy(installer, destination)?;
        return Ok(());
    } else if installer.extension() == Some(OsStr::new("pkg")) {
        debug!("install component from pkg archive");
        let tmp_destination = destination.join("tmp");
        DirBuilder::new().recursive(true).create(&tmp_destination)?;
        xar_pkg(installer, &tmp_destination)?;
        untar_pkg(&tmp_destination, destination)?;
        cleanup_pkg(&tmp_destination)?;
        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("Wrong installer. Expect .pkg, .zip, .xz or .po {}", &installer.display()),
    ))
}

fn xar_pkg(installer: &PathBuf, destination: &PathBuf) -> io::Result<()> {
    debug!(
        "unpack installer {} to temp destination {}",
        installer.display(),
        destination.display()
    );

    let child = Command::new("7z")
        .arg("x")
        .arg("-y")
        .arg(format!("-o{}", destination.display()))
        .arg(installer)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "failed to extract installer:/n{}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }
    Ok(())
}

fn cleanup_pkg(tmp_destination: &PathBuf) -> io::Result<()> {
    debug!("cleanup {}", &tmp_destination.display());
    fs::remove_dir_all(tmp_destination)
}

fn find_payload(dir: &PathBuf) -> io::Result<PathBuf> {
    debug!("find paylod in unpacked installer {}", dir.display());
    let mut files = fs::read_dir(dir).and_then(|read_dir| {
        Ok(read_dir.filter_map(io::Result::ok))
    }).map_err(|err| {
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
            if file_name.ends_with(".pkg.tmp") || file_name == "Payload~" {
                true
            } else {
                false
            }
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
        if path.file_name() == Some(OsStr::new("Payload~")) {
            Ok(path)
        } else {
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
    })
}

fn untar_pkg(base_payload_path: &PathBuf, destination: &PathBuf) -> io::Result<()> {
    let payload = find_payload(&base_payload_path)?;
    debug!("extract payload at {}", payload.display());

    let tar_child = if payload.file_name() == Some(OsStr::new("Payload~")) {
        let mut cpio = Command::new("cpio")
            .arg("-iu")
            .current_dir(destination)
            .stdin(Stdio::piped())
            .spawn()?;
        {
            let mut stdin = cpio.stdin.as_mut().expect("stdin");
            let mut file = File::open(payload)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            stdin.write(&buffer)?;
        }
        cpio
    } else {
        let mut gzip = Command::new("gzip")
            .arg("-dc")
            .arg(payload)
            .stdout(Stdio::piped())
            .spawn()?;

            let mut cpio = Command::new("cpio")
                .arg("-iu")
                .current_dir(destination)
                .stdin(Stdio::piped())
                .spawn()?;
            {
                let mut stdin = cpio.stdin.as_mut().expect("stdin");
                let mut gzipStdOut = gzip.stdout.as_mut().expect("stdout");
                let mut buffer = Vec::new();
                gzipStdOut.read_to_end(&mut buffer)?;
                stdin.write(&buffer)?;
            }
            cpio
    };

    let tar_output = tar_child.wait_with_output()?;
    if !tar_output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "failed to untar payload:/n{}",
                String::from_utf8_lossy(&tar_output.stderr)
            ),
        ));
    }

    Ok(())
}

fn cleanDirectory(path: &PathBuf) -> io::Result<()> {
    debug!("clean output directory {}", path.display());
    if path.exists() {
        debug!("directory exists. Delete directory and create empty directory at {}", path.display());
        fs::remove_dir_all(path)?;
    }
    DirBuilder::new().recursive(true).create(path)
}

fn deploy_zip(installer: &PathBuf, destination: &PathBuf) -> io::Result<()> {
    let file = File::open(installer)?;
    let unzipper = Unzipper::new(file, destination);
    unzipper.unzip()?;

    Ok(())
}

fn untar(source: &PathBuf, destination: &PathBuf) -> io::Result<()> {
    debug!("untar archive {} to {}", source.display(), destination.display());
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
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "failed to untar payload:/n{}",
                String::from_utf8_lossy(&tar_output.stderr)
            ),
        ));
    }

    Ok(())
}
