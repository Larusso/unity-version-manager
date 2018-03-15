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

#[cfg(test)]
use std::env;
#[cfg(test)]
use std::fs::DirBuilder;
#[cfg(test)]
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;

    fn createTestPath(version: &str) -> PathBuf {
        let base_dir = env::temp_dir();
        let path = &format!("{base_dir:?}/Unity-{version}", base_dir = base_dir, version = version);
        let mut dir_builder = DirBuilder::new();
        dir_builder.recursive(true);
        dir_builder.create(path).unwrap();
        Path::new(path).to_path_buf()
    }

    #[test]
    fn create_installtion_from_path() {
        let path = createTestPath("2017.1.2f5");
        let subject = Installation::new(path).unwrap();

        assert_eq!(subject.version.to_string(), "2017.1.2f5");
    }

    proptest! {
        #[test]
        fn doesnt_crash(ref s in "\\PC*") {
            Installation::new(Path::new(s).to_path_buf());
        }

        #[test]
        fn parses_all_valid_versions(ref s in r"[0-9]{1,4}\.[0-9]{1,4}\.[0-9]{1,4}[fpb][0-9]{1,4}") {
            let path = createTestPath(s);
            let subject = Installation::new(path).unwrap();
        }
    }
}
