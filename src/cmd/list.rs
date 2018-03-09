use std::path::Path;
use std::fs;
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

const UNITY_INSTALL_LOCATION : &'static str = "/Applications";

pub fn list() {
    let install_location = Path::new(UNITY_INSTALL_LOCATION);
    if let Ok(entries) = fs::read_dir(install_location) {
        for entry in entries {
            if let Ok(entry) = entry {
                if entry.file_name().to_str().unwrap().starts_with("Unity-") {
                    println!("{:?}", entry.file_name());
                }
            }
        }
    }
}