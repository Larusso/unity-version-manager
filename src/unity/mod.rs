mod installation;
mod version;

pub use self::installation::Installation;
pub use self::version::Version;

use std::fs;
use std::path::Path;

const UNITY_INSTALL_LOCATION: &'static str = "/Applications";

pub struct Installations {
    iter: Box<Iterator<Item = Installation>>,
}

impl Installations {
    fn new(install_location: &Path) -> Result<Installations, ()> {
        if let Ok(rd) = fs::read_dir(install_location) {
            Ok(Installations {
                iter: Box::new(
                    rd.filter_map(|f| f.ok())
                        .filter_map(|entry| match entry.file_name().to_str() {
                            Some(name) => {
                                if name.starts_with("Unity-") {
                                    return Some(entry);
                                } else {
                                    return None;
                                }
                            }
                            None => None,
                        })
                        .filter_map(|entry| Installation::new(entry.path()).ok()),
                ),
            })
        } else {
            Err(())
        }
    }
}

impl Iterator for Installations {
    type Item = Installation;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub fn list_installations() -> Result<Installations, ()> {
    let install_location = Path::new(UNITY_INSTALL_LOCATION);
    Installations::new(install_location)
}
