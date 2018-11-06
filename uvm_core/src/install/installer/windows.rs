use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::ffi::OsStr;

pub fn install_editor(installer:&PathBuf, destination:&PathBuf) -> io::Result<()> {
    debug!("install editor to destination: {} with installer: {}",
        &destination.display(), &installer.display());
    install_from_exe(installer, destination)
}

pub fn install_module(installer:&PathBuf, destination:&PathBuf) -> io::Result<()> {
    debug!("install component {} to {}",
        &installer.display(), &destination.display());
    install_from_exe(installer, destination)
}

fn install_from_exe(installer:&PathBuf, destination:&PathBuf) -> io::Result<()>
{
    debug!("install unity from installer exe");
    let destination_arg = &format!("/D={}", destination.display());
    let args = vec!["/S", destination_arg];

    info!("install {} to {} with arguments {}", installer.display(), destination.display(), args.join(" "));
    let install_process = Command::new(installer)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            error!("error while spawning installer");
            err
        })?;

    let output = install_process.wait_with_output()?;
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "failed to install:\
                {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ))
    }
    Ok(())
}
