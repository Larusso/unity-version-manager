use unity::Installation;
use std::path::PathBuf;
use std::path::Path;
use std::io;

pub type CurrentInstallation = Installation;
const UNITY_CURRENT_LOCATION: &'static str = "/Applications/Unity";

impl CurrentInstallation {
    fn current(path: PathBuf) -> io::Result<Installation> {
        let linked_file = path.read_link()?;
        CurrentInstallation::new(linked_file)
    }
}

pub fn current_installation() -> io::Result<CurrentInstallation> {
    let active_path = Path::new(UNITY_CURRENT_LOCATION);
    CurrentInstallation::current(active_path.to_path_buf())
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
    use std::os::unix;

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
    fn current_installation_fails_when_path_is_not_a_symlink() {
        let test_dir = prepare_unity_installations!["Unity"];
        let dir = test_dir.path().join("Unity");

        assert!(CurrentInstallation::current(dir).is_err());
    }

    #[test]
    fn current_installation_returns_active_installation() {
        let test_dir = prepare_unity_installations![
            "Unity-5.6.0p3",
            "Unity-2017.1.0p1",
            "Unity-2017.2.0f2"
        ];

        let dir = test_dir.path().join("Unity");
        let src = test_dir.path().join("Unity-2017.1.0p1");
        unix::fs::symlink(&src, &dir);

        let subject = CurrentInstallation::current(dir).unwrap();
        assert_eq!(subject.path(), &src);
    }
}
