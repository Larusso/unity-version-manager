use std::path::PathBuf;
use unity::Version;
use std::cmp::Ordering;
use std;
use std::str::FromStr;
use std::io::{Error, ErrorKind};

#[derive(PartialEq, Eq, Debug)]
pub struct Installation {
    version: Version,
    path: PathBuf,
}

impl Ord for Installation {
    fn cmp(&self, other: &Installation) -> Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialOrd for Installation {
    fn partial_cmp(&self, other: &Installation) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Installation {
    pub fn new(path: PathBuf) -> std::io::Result<Installation> {
        if path.is_dir() {
            let name = path.file_name().expect("Error reading filename");
            let name = name.to_str().unwrap();
            match Version::from_str(name) {
                Ok(v) => {
                    return Ok(Installation {
                        version: v,
                        path: path.clone()
                    })
                }
                Err(_) => return Err(Error::new(ErrorKind::InvalidInput, "Can't parse Unity version"))
            }
        }
        Err(Error::new(ErrorKind::InvalidInput, "Provided Path is not a directory."))
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn version_owned(self) -> Version {
        self.version
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}
