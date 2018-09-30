extern crate uvm_core;

use std::fmt;
use std::io;
use uvm_core::brew;
use uvm_core::unity::Version;
use uvm_core::unity::VersionType;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum InstallVariant {
    Android,
    Ios,
    WebGl,
    Linux,
    Windows,
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
            _ => write!(f, "editor"),
        }
    }
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
    brew::tap::ensure(tap)
}


pub fn cask_name_for_type_version(variant: InstallVariant, version: &Version) -> brew::cask::Cask {
    let base_name = if variant == InstallVariant::Editor {
        String::from("unity")
    } else {
        format!("unity-{}-support-for-editor", variant)
    };

    String::from(format!("{}@{}", base_name, version.to_string()))
}
