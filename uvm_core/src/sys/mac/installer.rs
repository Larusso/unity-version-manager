use crate::sys::shared::installer::*;
use std::ffi::OsStr;
use std::fs;
use std::fs::DirBuilder;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn install_editor(installer: &PathBuf, destination: &PathBuf) -> io::Result<()> {
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

    let tmp_destination = destination.join("tmp");

    if installer.extension() == Some(OsStr::new("pkg")) {
        DirBuilder::new().recursive(true).create(&tmp_destination)?;
        xar_pkg(installer, &tmp_destination)?;
        untar_pkg(&tmp_destination, destination)?;
        cleanup_editor_pkg(destination)?;
        cleanup_pkg(&tmp_destination)?;
        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("Wrong installer. Expect .pkg {}", &installer.display()),
    ))
}

pub fn install_module(installer: &PathBuf, destination: &PathBuf) -> io::Result<()> {
    _install_module(installer, destination)
}

fn _install_module<P, D>(installer: P, destination: D) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    let installer = installer.as_ref();
    let destination = destination.as_ref();

    debug!(
        "install component {} to {}",
        installer.display(),
        destination.display()
    );

    match installer.extension() {
        Some(ext) if ext == "pkg" => {
            install_module_from_pkg(installer, destination).map_err(|err| {
                cleanup_directory_failable(destination);
                err
            })
        }

        Some(ext) if ext == "zip" => {
            install_module_from_zip(installer, destination).map_err(|err| {
                cleanup_directory_failable(destination);
                err
            })
        }

        Some(ext) if ext == "po" => install_po_file(installer, destination),

        Some(ext) if ext == "dmg" => install_module_from_dmg(installer, destination),

        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Wrong installer. Expect .pkg, .zip, .dmg or .po {}",
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
    cleanup_ios_support_pkg(destination)?;
    cleanup_pkg(&tmp_destination)?;
    Ok(())
}

fn install_module_from_dmg<P, D>(installer: P, destination: D) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    use dmg::Attach;

    let installer = installer.as_ref();
    let destination = destination.as_ref();

    debug!(
        "install from dmg {} to {}",
        installer.display(),
        destination.display()
    );
    let volume = Attach::new(installer).with()?;
    debug!("installer mounted at {}", volume.mount_point.display());

    let app_path = find_file_in_dir(&volume.mount_point, |entry| {
        entry.file_name().to_str().unwrap().ends_with(".app")
    })?;

    copy_dir(app_path, destination)?;
    Ok(())
}

fn copy_dir<P, D>(source: P, destination: D) -> io::Result<()>
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
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "failed to copy {} to {}\n{}",
                source.display(),
                destination.display(),
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }
    Ok(())
}

fn cleanup_ios_support_pkg<D: AsRef<Path>>(destination: D) -> io::Result<()> {
    let destination = destination.as_ref();
    let tmp_ios_support_directory = destination.join("iOSSupport");
    if tmp_ios_support_directory.exists() {
        move_files(&tmp_ios_support_directory, &destination)?;
        fs::remove_dir_all(&tmp_ios_support_directory)?;
    }
    Ok(())
}

fn cleanup_editor_pkg<D: AsRef<Path>>(destination: D) -> io::Result<()> {
    let destination = destination.as_ref();
    let tmp_unity_directory = destination.join("Unity");
    if !tmp_unity_directory.exists() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "error whole extracting installer",
        ));
    }

    move_files(&tmp_unity_directory, &destination)?;
    fs::remove_dir_all(&tmp_unity_directory)
}

fn xar_pkg<P: AsRef<Path>, D: AsRef<Path>>(installer: P, destination: D) -> io::Result<()> {
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

fn find_file_in_dir<P, F>(dir: P, predicate: F) -> io::Result<PathBuf>
where
    P: AsRef<Path>,
    F: FnMut(&std::fs::DirEntry) -> bool,
{
    let dir = dir.as_ref();
    debug!("find file in directory {}", dir.display());
    fs::read_dir(dir).and_then(|read_dir| {
        read_dir
            .filter_map(io::Result::ok)
            .find(predicate)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("can't locate file in {}", &dir.display()),
                )
            })
            .map(|entry| entry.path())
    })
}

fn untar_pkg<P: AsRef<Path>, D: AsRef<Path>>(
    base_payload_path: P,
    destination: D,
) -> io::Result<()> {
    let base_payload_path = base_payload_path.as_ref();
    let payload = find_payload(&base_payload_path)?;
    debug!("untar payload at {}", payload.display());
    tar(&payload, destination)
}

fn tar<P: AsRef<Path>, D: AsRef<Path>>(source: P, destination: D) -> io::Result<()> {
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

fn move_files<P: AsRef<Path>, D: AsRef<Path>>(source: P, destination: D) -> io::Result<()> {
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
