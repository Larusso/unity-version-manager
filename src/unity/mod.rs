mod installation;
mod version;

pub use self::installation::Installation;
pub use self::version::Version;

use std::fs;
use std::path::Path;
use std::io;
use std::convert::From;

const UNITY_INSTALL_LOCATION: &'static str = "/Applications";

pub struct Installations(Box<Iterator<Item = Installation>>);
pub struct Versions(Box<Iterator<Item = Version>>);

impl Installations {
    fn new(install_location: &Path) -> io::Result<Installations> {
        let read_dir = fs::read_dir(install_location)?;
        let iter = read_dir
            .filter_map(|entry| entry.ok())
            .filter_map(check_dir_entry)
            .map(|entry| entry.path())
            .map(Installation::new)
            .filter_map(|i| i.ok());
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

pub fn list_installations() -> io::Result<Installations> {
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

        assert_eq!(subject.next().unwrap().version().to_string(), "2017.1.2f3");
        assert_eq!(subject.next().unwrap().version().to_string(), "2017.2.3f4");
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

        assert_eq!(subject.next().unwrap().to_string(), "2017.1.2f3");
        assert_eq!(subject.next().unwrap().to_string(), "2017.2.3f4");
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

        assert_eq!(subject.next().unwrap().to_string(), "2017.1.2f3");
        assert_eq!(subject.next().unwrap().to_string(), "2017.2.3f4");
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
