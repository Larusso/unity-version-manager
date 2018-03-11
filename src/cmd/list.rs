use std::path::Path;
use std::fs;
use regex::Regex;
use unity::Version;
use std::str::FromStr;

const UNITY_INSTALL_LOCATION: &'static str = "/Applications";

pub fn list() {
    let install_location = Path::new(UNITY_INSTALL_LOCATION);

    let version_pattern = Regex::new(r"((\d+)\.(\d+)\.(\d+)((f|p|b)(\d+))?)$").unwrap();

    if let Ok(entries) = fs::read_dir(install_location) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.starts_with("Unity-") {
                        if let Ok(v) = Version::from_str(file_name) {
                            println!("{}", v);
                        }
                    }
                }
            }
        }
    }
}
