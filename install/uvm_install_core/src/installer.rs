use std::fs;
use std::fs::DirBuilder;
use std::io;

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::ffi::OsStr;

pub fn install_editor(installer:&PathBuf, destination:&PathBuf) -> io::Result<()> {
    let tmp_destination = destination.join("tmp");

    if installer.extension() == Some(OsStr::new("pkg")) {
        DirBuilder::new()
            .recursive(true)
            .create(&tmp_destination)?;
        xar_pkg(installer, &tmp_destination)?;
        untar_pkg(&tmp_destination, destination)?;
        cleanup_editor_pkg(destination)?;
        cleanup_pkg(&tmp_destination)?;
        return Ok(())
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        format!(
            "Wrong installer. Expect .pkg {}",
            &installer.display()
        )))
}

pub fn install_module(installer:&PathBuf, destination:&PathBuf) -> io::Result<()> {
    let tmp_destination = destination.join("tmp");

    if installer.ends_with(".pkg") {
        DirBuilder::new()
            .recursive(true)
            .create(&tmp_destination)?;
        xar_pkg(installer, &tmp_destination)?;
        untar_pkg(&tmp_destination, destination)?;
        cleanup_pkg(&tmp_destination)?;
        return Ok(())
    }
    Err(io::Error::new(
        io::ErrorKind::Other,
        format!(
            "Wrong installer. Expect .pkg {}",
            &installer.display()
        )))
}

fn cleanup_pkg(tmp_destination:&PathBuf) -> io::Result<()> {
    println!("{}", "cleanup_pkg");
    fs::remove_dir_all(tmp_destination)
}

fn cleanup_editor_pkg(destination:&PathBuf) -> io::Result<()> {
    let tmp_unity_directory = destination.join("Unity");
    if !tmp_unity_directory.exists() {
        return Err(io::Error::new(io::ErrorKind::Other, "error whole extracting installer"))
    }

    move_files(&tmp_unity_directory, &destination)?;
    fs::remove_dir_all(&tmp_unity_directory)
}

fn xar_pkg(installer: &PathBuf, destination: &PathBuf) -> io::Result<()> {
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
        ))
    }
    Ok(())
}

fn find_payload(dir: &PathBuf) -> io::Result<PathBuf> {
    let read_dir = fs::read_dir(dir)?;
    read_dir
        .filter_map(io::Result::ok)
        .find(|entry| entry.file_name().to_str().unwrap().ends_with(".pkg.tmp"))
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::Other,
            format!(
                "can't locate *.pkg.tmp directory in extracted installer at {}",
                &dir.display()
            )))
        .map(|entry| entry.path())
        .and_then(|path| Ok(path.join("Payload")))
        .and_then(|path| match path.exists() {
            true => Ok(path),
            false => Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "can't locate Payload directory in extracted installer at {}",
                    &dir.display()
                )))
            }
        )
}

fn untar_pkg(base_payload_path: &PathBuf, destination: &PathBuf) -> io::Result<()> {
    let payload = find_payload(&base_payload_path)?;
    tar(&payload, destination)
}

fn tar (source: &PathBuf, destination: &PathBuf) -> io::Result<()> {
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
        ))
    }

    Ok(())
}

fn move_files(source: &PathBuf, destination:&PathBuf) -> io::Result<()> {
    for entry in fs::read_dir(&source)?.filter_map(io::Result::ok) {
        let new_location = destination.join(entry.file_name());
        if new_location.exists() && new_location.is_dir() {
            fs::remove_dir_all(&new_location)?;
        }

        fs::rename(entry.path(), &new_location)?;
    }
    Ok(())
}
