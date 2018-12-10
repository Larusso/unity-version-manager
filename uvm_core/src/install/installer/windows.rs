use std::io;
use std::io::Write as IoWrite;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::Builder;

pub fn install_editor(installer: &PathBuf, destination: &PathBuf) -> io::Result<()> {
    debug!(
        "install editor to destination: {} with installer: {}",
        &destination.display(),
        &installer.display()
    );
    install_from_exe(installer, destination)
}

pub fn install_module(installer: &PathBuf, destination: &PathBuf) -> io::Result<()> {
    debug!(
        "install component {} to {}",
        &installer.display(),
        &destination.display()
    );
    install_from_exe(installer, destination)
}

fn install_from_exe(installer: &PathBuf, destination: &PathBuf) -> io::Result<()> {
    debug!("install unity from installer exe");
    let mut install_helper = Builder::new().suffix(".cmd").rand_bytes(20).tempfile()?;

    info!(
        "create install helper script {}",
        install_helper.path().display()
    );
    {
        let script = install_helper.as_file_mut();
        let install_command = format!(
            r#"CALL "{}" /S /D={}"#,
            installer.display(),
            destination.display()
        );
        trace!("install helper script content:");
        writeln!(script, "ECHO OFF");
        trace!("{}", &install_command);
        writeln!(script, "{}", install_command);
    }

    info!(
        "install {} to {}",
        installer.display(),
        destination.display()
    );
    let installer_script = install_helper.into_temp_path();
    let install_process = Command::new(&installer_script)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            error!("error while spawning installer");
            err
        })?;
    let output = install_process.wait_with_output()?;
    installer_script.close()?;
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "failed to install:\
                 {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }
    Ok(())
}
