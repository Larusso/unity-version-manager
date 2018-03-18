mod installation;
mod version;

pub use self::installation::Installation;
pub use self::version::Version;

use std::fs;
use std::path::Path;
use std::io;

const UNITY_INSTALL_LOCATION: &'static str = "/Applications";

pub struct Installations(Box<Iterator<Item = Installation>>);

fn check_dir_entry(entry: fs::DirEntry) -> Option<fs::DirEntry> {
    match entry.file_name().to_str() {
        Some(name) => {
            if name.starts_with("Unity-") {
                return Some(entry);
            };
            None
        }
        None => None,
    }
}

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
}

impl Iterator for Installations {
    type Item = Installation;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
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
    fn list_installations_in_directory() {
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
    fn list_installations_in_directory_returns_error() {
        let test_dir = prepare_unity_installations![];
        assert!(Installations::new(test_dir.path()).is_ok());
    }
}
