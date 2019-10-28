use crate::sys::shared::installer::*;
use std::ffi::OsStr;
use std::fs;
use std::fs::DirBuilder;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::{Path,PathBuf};
use std::process::{Command, Stdio};

pub fn install_editor<P, D>(installer: P, destination: Option<D>) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    let installer = installer.as_ref();
    let destination = destination.ok_or(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Missing destination path",
    ))?;

    let destination = destination.as_ref();

    _install_editor(installer, destination).map_err(|err| {
        if destination.exists() {
            debug!(
                "Delete destination directory after failure {}",
                destination.display()
            );
            fs::remove_dir_all(destination).unwrap_or_else(|err| {
                error!("Failed to cleanup destination {}", destination.display());
                error!("{}", err);
            })
        }
        err
    })
}

fn _install_editor<P, D>(installer: P, destination: D) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    let installer = installer.as_ref();
    let destination = destination.as_ref();
    debug!(
        "install editor to destination: {} with installer: {}",
        destination.display(),
        installer.display()
    );

    if installer.extension() == Some(OsStr::new("zip")) {
        debug!("install editor from zip archive");
        clean_directory(destination)?;
        deploy_zip(installer, destination)?;
        return Ok(());
    } else if installer.extension() == Some(OsStr::new("xz")) {
        debug!("install editor from xz archive");
        clean_directory(destination)?;
        untar(installer, destination)?;
        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        format!(
            "Wrong installer. Expect .zip or .xz {:?} - {}",
            &installer.extension(),
            &installer.display()
        ),
    ))
}

pub fn install_module<P, D>(installer: P, destination: Option<D>) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    _install_module(installer, destination)
}

fn _install_module<P, D>(installer: P, destination: Option<D>) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    let installer = installer.as_ref();
    let destination = match destination {
        Some(ref d) => Some(d.as_ref()),
        _ => None,
    };

    debug!("install component {}", installer.display(),);
    if let Some(destination) = destination {
        debug!("to {}", destination.display());
    }

    match installer.extension() {
        Some(ext) if ext == "zip" => {
            let destination = destination.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing destination path for zip intaller",
                )
            })?;

            install_module_from_zip(installer, destination).map_err(|err| {
                cleanup_directory_failable(destination);
                err
            })
        }

        Some(ext) if ext == "xz" => {
            let destination = destination.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing destination path for xz intaller",
                )
            })?;

            install_module_from_xz(installer, destination).map_err(|err| {
                cleanup_directory_failable(destination);
                err
            })
        },

        Some(ext) if ext == "po" => {
            let destination = destination.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing destination path for po module",
                )
            })?;
            install_po_file(installer, destination)
        }

        Some(ext) if ext == "pkg" => {
            let destination = destination.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing destination path for pkg intaller",
                )
            })?;

            install_module_from_pkg(installer, destination).map_err(|err| {
                cleanup_directory_failable(destination);
                err
            })
        }

        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Wrong installer. Expect .pkg, .zip, .xz or .po {}",
                &installer.display()
            ),
        )),
    }
}

fn install_module_from_pkg<P, D>(installer: P, destination: D) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    let installer = installer.as_ref();
    let destination = destination.as_ref();

    debug!(
        "install module from pkg {} to {}",
        installer.display(),
        destination.display()
    );

    let tmp_destination = destination.join("tmp");
    DirBuilder::new().recursive(true).create(&tmp_destination)?;

    xar_pkg(installer, &tmp_destination)?;
    untar_pkg(&tmp_destination, destination)?;
    cleanup_pkg(&tmp_destination)?;
    Ok(())
}

fn install_module_from_xz<P, D>(installer: P, destination: D) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    let installer = installer.as_ref();
    let destination = destination.as_ref();

    debug!(
        "install module from xz archive {} to {}",
        installer.display(),
        destination.display()
    );

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
    untar(installer, destination)?;
    return Ok(());
}

fn xar_pkg<P, D>(installer: P, destination: D) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    let installer = installer.as_ref();
    let destination = destination.as_ref();

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

fn untar_pkg<P, D>(base_payload_path: P, destination: D) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    let base_payload_path = base_payload_path.as_ref();
    let destination = destination.as_ref();

    let payload = find_payload(base_payload_path)?;
    debug!("extract payload at {}", payload.display());

    let tar_child = if payload.file_name() == Some(OsStr::new("Payload~")) {
        let mut cpio = Command::new("cpio")
            .arg("-iu")
            .current_dir(destination)
            .stdin(Stdio::piped())
            .spawn()?;
        {
            let stdin = cpio.stdin.as_mut().expect("stdin");
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
            let stdin = cpio.stdin.as_mut().expect("stdin");
            let gzip_std_out = gzip.stdout.as_mut().expect("stdout");
            let mut buffer = Vec::new();
            gzip_std_out.read_to_end(&mut buffer)?;
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

fn untar<P, D>(source: P, destination: D) -> io::Result<()>
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
