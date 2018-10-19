use unity::Installation;
use std::path::{Path,PathBuf};
use result::Result;

pub type CurrentInstallation = Installation;
const UNITY_CURRENT_LOCATION: &'static str = "/Applications/Unity";

impl CurrentInstallation {
    fn current(path: PathBuf) -> Result<Installation> {
        let linked_file = path.read_link()?;
        CurrentInstallation::new(linked_file)
    }
}

pub fn current_installation() -> Result<CurrentInstallation> {
    let active_path = Path::new(UNITY_CURRENT_LOCATION);
    CurrentInstallation::current(active_path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::str::FromStr;
    use std::path::Path;
    use std::path::PathBuf;
    use rand;
    use tempfile::TempDir;
    use tempfile::Builder;
    use super::*;
    use std::os::unix;
    use std::fs::File;
    use unity::installation::AppInfo;
    use plist::serde::{deserialize, serialize_to_xml};

    fn create_test_path(base_dir:&PathBuf, version: &str) -> PathBuf {
        let path = base_dir.join(format!("Unity-{version}", version = version));
        let mut dir_builder = fs::DirBuilder::new();
        dir_builder.recursive(true);
        dir_builder.create(&path).unwrap();

        let info_plist_path = path.join(Path::new("Unity.app/Contents/Info.plist"));
        dir_builder.create(info_plist_path.parent().unwrap()).unwrap();

        let info = AppInfo {
            c_f_bundle_version: String::from_str(version).unwrap(),
            unity_build_number: String::from_str("ssdsdsdd").unwrap()
        };
        let file = File::create(info_plist_path).unwrap();
        serialize_to_xml(file, &info).unwrap();

        path
    }

    macro_rules! prepare_unity_installations {
        ($($input:expr),*) => {
            {
                let test_dir = Builder::new().prefix("current_installations").tempdir().unwrap();
                $(
                    create_test_path(&test_dir.path().to_path_buf(), $input);
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
            "5.6.0p3",
            "2017.1.0p1",
            "2017.2.0f2"
        ];

        let dir = test_dir.path().join("Unity");
        let src = test_dir.path().join("Unity-2017.1.0p1");
        unix::fs::symlink(&src, &dir).unwrap();
        dir.read_link().unwrap();
        let subject = CurrentInstallation::current(dir).unwrap();
        assert_eq!(subject.path(), &src);
    }
}
