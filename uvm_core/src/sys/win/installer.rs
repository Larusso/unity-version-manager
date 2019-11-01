use crate::sys::shared::installer::*;
use std::io;
use std::io::Write as IoWrite;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::Builder;
use crate::utils;

pub fn install_editor<P, D>(
    installer: P,
    destination: Option<D>,
    cmd: Option<&str>,
) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    let installer = installer.as_ref();
    let destination = match destination {
        Some(ref d) => Some(utils::prepend_long_path_support(d)),
        _ => None,
    };
    let destination = destination.as_ref();

    debug!("install editor {}", installer.display(),);
    if let Some(destination) = destination {
        debug!("to {}", destination.display());
    }

    install_from_exe(installer, destination, cmd)
}

pub fn install_module<P, D>(
    installer: P,
    destination: Option<D>,
    cmd: Option<&str>,
) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    _install_module(installer, destination, cmd)
}

fn _install_module<P, D>(installer: P, destination: Option<D>, cmd: Option<&str>) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    let installer = installer.as_ref();
    let destination = match destination {
        Some(ref d) => Some(utils::prepend_long_path_support(d)),
        _ => None,
    };
    let destination = destination.as_ref();

    debug!("install component {}", installer.display(),);
    if let Some(destination) = destination {
        debug!("to {}", destination.display());
    }

    match installer.extension() {
        Some(ext) if ext == "exe" => install_from_exe(installer, destination, cmd),
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
        Some(ext) if ext == "msi" => {
            let cmd = cmd.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing cmd parameter path for msi intaller",
                )
            })?;

            install_from_msi(installer, cmd)
        }
        Some(ext) if ext == "po" => {
            let destination = destination.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing destination path for po module",
                )
            })?;
            install_po_file(installer, destination)
        }

        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Wrong installer. Expect .exe, .msi, .zip or .po {}",
                &installer.display()
            ),
        )),
    }
}

fn install_from_exe<P, D>(installer: P, destination: Option<D>, cmd: Option<&str>) -> io::Result<()>
where
    P: AsRef<Path>,
    D: AsRef<Path>,
{
    let installer = installer.as_ref();
    let destination = match destination {
        Some(ref d) => Some(d.as_ref()),
        _ => None,
    };

    debug!("install unity from installer exe");
    let mut install_helper = Builder::new().suffix(".cmd").rand_bytes(20).tempfile()?;

    info!(
        "create install helper script {}",
        install_helper.path().display()
    );

    {
        let script = install_helper.as_file_mut();
        let parameter_option = match cmd {
            Some(parameters) => parameters,
            _ => "/S",
        };

        let destination_option = match destination {
            Some(destination) => format!("/D={}", destination.display()),
            _ => "".to_string(),
        };

        let install_command = format!(
            r#"CALL "{installer}" {parameters} {destination}"#,
            installer = installer.display(),
            parameters = parameter_option,
            destination = destination_option
        );

        trace!("install helper script content:");
        writeln!(script, "ECHO OFF")?;
        trace!("{}", &install_command);
        writeln!(script, "{}", install_command)?;
    }

    info!("install {}", installer.display());
    if let Some(destination) = destination {
        info!("to {}", destination.display());
    }

    let installer_script = install_helper.into_temp_path();
    install_from_temp_command(&installer_script)?;
    installer_script.close()?;
    Ok(())
}

fn install_from_temp_command<P>(command: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let command = command.as_ref();
    let install_process = Command::new(command)
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
        ));
    }
    Ok(())
}

fn install_from_msi<P>(installer: P, cmd: &str) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let installer = installer.as_ref();

    debug!("install unity module from installer msi");
    let mut install_helper = Builder::new().suffix(".cmd").rand_bytes(20).tempfile()?;

    info!(
        "create install helper script {}",
        install_helper.path().display()
    );

    {
        let script = install_helper.as_file_mut();

        let install_command = cmd.replace("/i", &format!(r#"/i "{}""#, installer.display()));

        trace!("install helper script content:");
        writeln!(script, "ECHO OFF")?;
        trace!("{}", &install_command);
        writeln!(script, "{}", install_command)?;
    }

    info!("install {}", installer.display());

    let installer_script = install_helper.into_temp_path();
    install_from_temp_command(&installer_script)?;
    installer_script.close()?;
    Ok(())
}
