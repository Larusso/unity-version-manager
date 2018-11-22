use result::Result;
use std::path::Path;
use unity::Installation;

pub type CurrentInstallation = Installation;
const UNITY_CURRENT_LOCATION: &str = "/Applications/Unity";

impl CurrentInstallation {
    fn current<P: AsRef<Path>>(path: P) -> Result<Installation> {
        let path = path.as_ref();
        let linked_file = path.read_link()?;
        CurrentInstallation::new(linked_file)
    }
}

pub fn current_installation() -> Result<CurrentInstallation> {
    let active_path = Path::new(UNITY_CURRENT_LOCATION);
    CurrentInstallation::current(active_path.to_path_buf())
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use plist::serde::serialize_to_xml;
    use std::fs;
    use std::fs::File;
    use std::os;
    use std::path::Path;
    use std::path::PathBuf;
    use std::str::FromStr;
    use tempfile::Builder;
    use unity::installation::AppInfo;

    fn create_test_path(base_dir: &PathBuf, version: &str) -> PathBuf {
        let path = base_dir.join(format!("Unity-{version}", version = version));
        let mut dir_builder = fs::DirBuilder::new();
        dir_builder.recursive(true);
        dir_builder.create(&path).unwrap();

        let info_plist_path = path.join(Path::new("Unity.app/Contents/Info.plist"));
        dir_builder
            .create(info_plist_path.parent().unwrap())
            .unwrap();

        let info = AppInfo {
            c_f_bundle_version: String::from_str(version).unwrap(),
            unity_build_number: String::from_str("ssdsdsdd").unwrap(),
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
        let test_dir = prepare_unity_installations!["5.6.0p3", "2017.1.0p1", "2017.2.0f2"];

        let dir = test_dir.path().join("Unity");
        let src = test_dir.path().join("Unity-2017.1.0p1");

        #[cfg(unix)]
        os::unix::fs::symlink(&src, &dir).unwrap();
        #[cfg(windows)]
        os::windows::fs::symlink_dir(&src, &dir).unwrap();
        dir.read_link().unwrap();
        let subject = CurrentInstallation::current(dir).unwrap();
        assert_eq!(subject.path(), &src);
    }
}
