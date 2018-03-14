use std::fs;
use std::io;
use std::path::Path;
use unity::Installation;

const UNITY_INSTALL_LOCATION: &'static str = "/Applications";

pub fn list() -> io::Result<Vec<Installation>> {
    let install_location = Path::new(UNITY_INSTALL_LOCATION);
    let mut versions: Vec<Installation> = Vec::new();
    let files = fs::read_dir(install_location)?;
    for entry in files {
        let entry = entry.expect("Error while reading Directory entry.");
        let file_name_s = entry.file_name();

        let file_name = file_name_s.to_str().expect("Error while reading filename");
        if file_name.starts_with("Unity-") {
            if let Ok(i) = Installation::new(entry.path()) {
                versions.push(i)
            }
        }
    }
    versions.sort();
    Ok(versions)
}
