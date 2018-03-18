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
}
