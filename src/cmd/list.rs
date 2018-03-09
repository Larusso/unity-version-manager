use std::path::Path;
use std::fs;
use regex::Regex;
// struct Version {
//     major: u32,
//     minor: u32,
//     patch: u32,
// }

// enum UnityVersion {
//     Final {version: Version, revision: u32},
//     Beta {version: Version, revision: u32},
//     Patch {version: Version, revision: u32},
// }

// struct Unity {
//     path: String,
//     version: String,
//     active: bool,
// }

const UNITY_INSTALL_LOCATION: &'static str = "/Applications";

pub fn list() {
    let install_location = Path::new(UNITY_INSTALL_LOCATION);

    let version_pattern = Regex::new(r"((\d+)\.(\d+)\.(\d+)((f|p|b)(\d+))?)$").unwrap();

    if let Ok(entries) = fs::read_dir(install_location) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.starts_with("Unity-") {
                        if let Some(caps) = version_pattern.captures(file_name) {
                            let version = caps.get(1).map_or("", |m| m.as_str());

                            let major = caps.get(2).map_or("", |m| m.as_str());
                            let minor = caps.get(3).map_or("", |m| m.as_str());
                            let patch = caps.get(4).map_or("", |m| m.as_str());
                            let r_type = caps.get(6).map_or("", |m| m.as_str());
                            let revision = caps.get(7).map_or("", |m| m.as_str());

                            println!(
                                "{:?} M:{} M:{} P:{} T:{} R:{}",
                                version, major, minor, patch, r_type, revision
                            );
                        }
                    }
                }
            }
        }
    }
}
