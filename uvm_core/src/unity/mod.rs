mod installation;
mod version;
mod current_installation;

pub use self::installation::Installation;
pub use self::version::Version;
pub use self::version::ParseVersionError;
pub use self::version::VersionType;
pub use self::version::unity_version_format;
pub use self::current_installation::CurrentInstallation;
pub use self::current_installation::current_installation;

use std::fs;
use std::path::Path;
use std::io;
use std::convert::From;
use result::Result;

const UNITY_INSTALL_LOCATION: &'static str = "/Applications";

pub struct Installations(Box<Iterator<Item = Installation>>);
pub struct Versions(Box<Iterator<Item = Version>>);

impl Installations {
    fn new(install_location: &Path) -> Result<Installations> {
        let read_dir = fs::read_dir(install_location)?;
        let iter = read_dir
            .filter_map(|dir_entry| dir_entry.ok())
            .filter_map(check_dir_entry)
            .map(|entry| entry.path())
            .map(Installation::new)
            .filter_map(Result::ok);
        Ok(Installations(Box::new(iter)))
    }

    pub fn versions(self) -> Versions {
        self.into()
    }
}

impl Iterator for Installations {
    type Item = Installation;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl Iterator for Versions {
    type Item = Version;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl From<Installations> for Versions {
    fn from(installations:Installations) -> Self {
        let iter = installations.map(|i| i.version_owned());
        Versions(Box::new(iter))
    }
}

fn check_dir_entry(entry:fs::DirEntry) -> Option<fs::DirEntry> {
    let name = entry.file_name();
    if name.to_str().unwrap_or("").starts_with("Unity-") {
        return Some(entry);
    };
    None
}

pub fn list_installations() -> Result<Installations> {
    let install_location = Path::new(UNITY_INSTALL_LOCATION);
    Installations::new(install_location)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use rand;
    use tempdir::TempDir;
    use std::str::FromStr;
    use super::*;

    macro_rules! prepare_unity_installations {
        ($($input:expr),*) => {
            {
                let test_dir = TempDir::new("list_installations_in_directory").unwrap();
                let mut dir_builder = fs::DirBuilder::new();
                $(
                    let dir = test_dir.path().join($input);
                    dir_builder.create(dir).unwrap();
                )*
                test_dir
            }
        };
    }

    #[test]
    fn list_installations_in_directory_filters_unity_installations() {
        let test_dir = prepare_unity_installations![
            "Unity-2017.1.2f3",
            "some_random_name",
            "Unity-2017.2.3f4"
        ];

        let mut subject = Installations::new(test_dir.path()).unwrap();

        let r1 = Version::from_str("2017.1.2f3").unwrap();
        let r2 = Version::from_str("2017.2.3f4").unwrap();
        for installation in subject {
            let version = installation.version_owned();
            assert!(version == r1 || version == r2);
        }
    }

    #[test]
    fn list_installations_in_empty_directory_returns_no_error() {
        let test_dir = prepare_unity_installations![];
        assert!(Installations::new(test_dir.path()).is_ok());
    }

    #[test]
    fn installations_can_be_converted_to_versions() {
        let test_dir = prepare_unity_installations![
            "Unity-2017.1.2f3",
            "some_random_name",
            "Unity-2017.2.3f4"
        ];

        let installations = Installations::new(test_dir.path()).unwrap();
        let mut subject = installations.versions();

        let r1 = Version::from_str("2017.1.2f3").unwrap();
        let r2 = Version::from_str("2017.2.3f4").unwrap();
        for version in subject {
            assert!(version == r1 || version == r2);
        }
    }

    #[test]
    fn versions_can_be_created_from_intallations() {
        let test_dir = prepare_unity_installations![
            "Unity-2017.1.2f3",
            "some_random_name",
            "Unity-2017.2.3f4"
        ];

        let installations = Installations::new(test_dir.path()).unwrap();
        let mut subject = Versions::from(installations);

        let r1 = Version::from_str("2017.1.2f3").unwrap();
        let r2 = Version::from_str("2017.2.3f4").unwrap();
        for version in subject {
            assert!(version == r1 || version == r2);
        }
    }

    proptest! {
        #[test]
        fn doesnt_crash(ref s in "\\PC*") {
            Installations::new(Path::new(s))
        }

        #[test]
        fn parses_all_valid_versions(ref s in r"[0-9]{1,4}\.[0-9]{1,4}\.[0-9]{1,4}[fpb][0-9]{1,4}") {
            let test_dir = prepare_unity_installations![
                format!("Unity-{}",s)
            ];
            let mut subject = Installations::new(test_dir.path()).unwrap();
            assert_eq!(subject.count(), 1);
        }
    }
}
