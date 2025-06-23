use std::fs::File;
use std::io;
use std::io::{Cursor, Error, Read};
use std::path::Path;
use std::str::FromStr;
use log::{info, warn};
use mach_object::{get_arch_name_from_types, OFile};
use sysctl::Sysctl;
use unity_hub::unity::{Installation, UnityInstallation};
use unity_version::Version;
use thiserror::Error;

#[derive(Error, Debug)]
enum ArchError {
    #[error("Unknown architecture")]
    UnknownArchitecture,

    #[error("IO error")]
    IoError(#[from] io::Error),

    #[error("Mach file type not supported")]
    MachFileTypeError,

    #[error("Sysctl error")]
    SysCtlError(#[from] sysctl::SysctlError)
}

fn fetch_architectures_from_binary<P: AsRef<Path>>(path: P) -> Result<Vec<String>, ArchError> {
    info!(
        "Reading binary architecture for the file path: {}",
        path.as_ref().display()
    );

    let mut f = File::open(path)?;
    let mut buf = Vec::new();
    let size = f.read_to_end(&mut buf)?;
    let mut cur = Cursor::new(&buf[..size]);
    let o_file =
        OFile::parse(&mut cur).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    match o_file {
        OFile::MachFile {
            ref header,
            ref commands,
        } => {
            let arch = get_arch_name_from_types(header.cputype, header.cpusubtype).ok_or(
                ArchError::UnknownArchitecture
            )?;
            info!("File is a mach file with architecture: {}", arch);
            Ok(vec![arch.to_string()])
        }
        OFile::FatFile {
            magic: _,
            ref files,
        } => {
            info!("File is a Fat file with architectures:");
            let architectures = files
                .into_iter()
                .map(|(fat_arch, _)| {
                    let arch =
                        get_arch_name_from_types(fat_arch.cputype, fat_arch.cpusubtype).ok_or(
                            ArchError::UnknownArchitecture
                        ).unwrap_or("unknown");
                    info!("{}", arch);
                    arch.to_string()
                })
                .collect();
            Ok(architectures)
        }
        _ => {
            warn!("Not a supported file type");
            Err(ArchError::MachFileTypeError)
        }
    }
}

fn fetch_system_architecture() -> Result<String, ArchError> {
    let ctl = sysctl::Ctl::new("hw.machine")?;
    let value = ctl.value()?;
    Ok(value.to_string())
}

pub fn ensure_installation_architecture_is_correct<I: Installation>(installation: &I) -> io::Result<bool> {
    if installation.version() >= &Version::from_str("2021.2.0f1").expect("a valid version") {
        let architectures = fetch_architectures_from_binary(installation.exec_path()).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        let system_architectures = fetch_system_architecture().map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        if architectures.contains(&system_architectures) {
            return Ok(true);
        }
        warn!(
            "The binary architecture of the Unity installation is not equal to the system architecture. The binary architecture is: {}. The system architecture is: {}.",
            architectures.join(", "),
            system_architectures
        );
        return Ok(false);
    } else {
        info!("The installation version is lower than 2021.2.0f1. The architecture check will be skipped.");
    }
    Ok(true)
}