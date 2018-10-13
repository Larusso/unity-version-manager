extern crate uvm_core;
#[macro_use]
extern crate log;
extern crate regex;

use std::fmt;
use std::io;
use uvm_core::brew;
use uvm_core::unity::Version;
use uvm_core::unity::VersionType;
use std::path::PathBuf;
use std::path::Path;
use regex::Regex;

pub mod installer;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum InstallVariant {
    Android,
    Ios,
    WebGl,
    Linux,
    Windows,
    WindowsMono,
    Editor,
}

impl fmt::Display for InstallVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &InstallVariant::Android => write!(f, "android"),
            &InstallVariant::Ios => write!(f, "ios"),
            &InstallVariant::WebGl => write!(f, "webgl"),
            &InstallVariant::Linux => write!(f, "linux"),
            &InstallVariant::Windows => write!(f, "windows"),
            &InstallVariant::WindowsMono => write!(f, "windows-mono"),
            _ => write!(f, "editor"),
        }
    }
}

fn fetch_download_path_from_output(output: &Vec<u8>) -> Option<PathBuf> {
    let url_pattern = Regex::new(r"^==> Success! Downloaded to -> (.*)$").unwrap();
    String::from_utf8_lossy(output).lines().find(|line| {
        url_pattern.is_match(line)
    }).map(|line| {
        let caps = url_pattern.captures(line).unwrap();
        Path::new(&caps.get(1).unwrap().as_str()).to_path_buf()
    })
}

pub fn download_installer(variant: InstallVariant, version: &Version) -> io::Result<PathBuf> {
    debug!("download installer for variant: {} and version: {}", variant, version);
    let cask = cask_name_for_type_version(variant, version);
    let child = brew::cask::fetch(vec!(cask), true)?;
    let o = child.wait_with_output()?;

    if !o.status.success() {
        //error!("{}", String::from_utf8_lossy(&o.stderr));
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to download installer"));
    }

    trace!("stderr:\n{}", String::from_utf8_lossy(&o.stderr));
    trace!("stdout:\n{}", String::from_utf8_lossy(&o.stdout));

    fetch_download_path_from_output(&o.stdout).ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, format!("Failed to fetch installer url \n{}", String::from_utf8_lossy(&o.stdout)))
    })
}

pub fn ensure_tap_for_version(version: &Version) -> io::Result<()> {
    ensure_tap_for_version_type(version.release_type())
}

pub fn ensure_tap_for_version_type(version_type: &VersionType) -> io::Result<()> {
    let tap = match version_type {
        VersionType::Final => "wooga/unityversions",
        VersionType::Beta => "wooga/unityversions-beta",
        VersionType::Patch => "wooga/unityversions-patch",
    };
    debug!("ensure brew tap {}", tap);
    brew::tap::ensure(tap)
}


pub fn cask_name_for_type_version(variant: InstallVariant, version: &Version) -> brew::cask::Cask {
    debug!("fetch cask name for variant {} and version {}", variant, version.to_string());
    let base_name = if variant == InstallVariant::Editor {
        String::from("unity")
    } else {
        format!("unity-{}-support-for-editor", variant)
    };

    let result = String::from(format!("{}@{}", base_name, version.to_string()));
    debug!("{}", result);
    result
}
