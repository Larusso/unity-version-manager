use error::*;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use unity::version;
use unity::InstalledComponents;
use unity::Version;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AppInfo {
    pub c_f_bundle_version: String,
    pub unity_build_number: String,
}

pub trait UnityInstallation: Eq + Ord {
    fn path(&self) -> &PathBuf;

    fn version(&self) -> &Version;

    #[cfg(target_os = "windows")]
    fn location(&self) -> PathBuf {
        self.path().join("Editor/Unity.exe")
    }

    #[cfg(target_os = "macos")]
    fn location(&self) -> PathBuf {
        self.path().join("Unity.app")
    }

    #[cfg(target_os = "windows")]
    fn exec_path(&self) -> PathBuf {
        self.location()
    }

    #[cfg(target_os = "macos")]
    fn exec_path(&self) -> PathBuf {
        self.path().join("Unity.app/Contents/MacOS/Unity")
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Installation {
    version: Version,
    path: PathBuf,
}

impl UnityInstallation for Installation {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn version(&self) -> &Version {
        &self.version
    }
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

#[cfg(target_os = "macos")]
fn adjust_path(path:&Path) -> Option<&Path> {
    // if the path points to a file it could be the executable
    if path.is_file() {
        if let Some(name) = path.file_name() {
            if name == "Unity" {
                path.parent()
                    .and_then(|path| path.parent())
                    .and_then(|path| path.parent())
                    .and_then(|path| path.parent())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
fn adjust_path(path:&Path) -> Option<&Path> {
    if path.is_file() {
        if let Some(name) = path.file_name() {
            if name == "Unity.exe" {
                path.parent().and_then(|path| path.parent())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn adjust_path(path:&Path) -> Option<&Path> {
    None
}

impl Installation {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Installation> {
        let path = path.as_ref();
        let path = if let Some(p) = adjust_path(path) {
            p
        } else {
            path
        };
        version::read_version_from_path(&path)
            .map(|version| Installation {
                version,
                path: path.to_path_buf(),
            })
            .map_err(|err| err.into())
    }

    //TODO remove clone()
    pub fn installed_components(&self) -> InstalledComponents {
        InstalledComponents::new(self.clone())
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn version_owned(&self) -> Version {
        self.version.to_owned()
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    #[cfg(target_os = "windows")]
    pub fn location(&self) -> PathBuf {
        self.path().join("Editor/Unity.exe")
    }

    #[cfg(target_os = "macos")]
    pub fn location(&self) -> PathBuf {
        self.path().join("Unity.app")
    }

    #[cfg(target_os = "windows")]
    pub fn exec_path(&self) -> PathBuf {
        self.location()
    }

    #[cfg(target_os = "macos")]
    pub fn exec_path(&self) -> PathBuf {
        self.path().join("Unity.app/Contents/MacOS/Unity")
    }
}

impl From<crate::unity::hub::editors::EditorInstallation> for Installation {
    fn from(editor: crate::unity::hub::editors::EditorInstallation) -> Self {
        Installation {
            version: editor.version().to_owned(),
            path: editor.location().to_path_buf(),
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use plist::serde::serialize_to_xml;
    use std::fs;
    use std::fs::File;
    use std::path::Path;
    use std::str::FromStr;
    use tempfile::Builder;

    fn create_unity_installation(base_dir: &PathBuf, version: &str) -> PathBuf {
        let path = base_dir.join("Unity");
        let mut dir_builder = fs::DirBuilder::new();
        dir_builder.recursive(true);
        dir_builder.create(&path).unwrap();

        let info_plist_path = path.join("Unity.app/Contents/Info.plist");
        let exec_path = path.join("Unity.app/Contents/MacOS/Unity");
        dir_builder
            .create(info_plist_path.parent().unwrap())
            .unwrap();

        dir_builder
            .create(exec_path.parent().unwrap())
            .unwrap();

        let info = AppInfo {
            c_f_bundle_version: String::from_str(version).unwrap(),
            unity_build_number: String::from_str("ssdsdsdd").unwrap(),
        };

        let file = File::create(info_plist_path).unwrap();
        File::create(exec_path).unwrap();

        serialize_to_xml(file, &info).unwrap();

        path
    }

    macro_rules! prepare_unity_installation {
        ($version:expr) => {{
            let test_dir = Builder::new()
                .prefix("installation")
                .rand_bytes(5)
                .tempdir()
                .unwrap();
            let unity_path = create_unity_installation(&test_dir.path().to_path_buf(), $version);
            (test_dir, unity_path)
        }};
    }

    #[test]
    fn create_installtion_from_path() {
        let (_t, path) = prepare_unity_installation!("2017.1.2f5");
        let subject = Installation::new(path).unwrap();

        assert_eq!(subject.version.to_string(), "2017.1.2f5");
    }

    #[test]
    fn create_installation_from_executable_path() {
        let(_t, path) = prepare_unity_installation!("2017.1.2f5");
        let installation = Installation::new(path).unwrap();
        let subject = Installation::new(installation.exec_path()).unwrap();

        assert_eq!(subject.version.to_string(), "2017.1.2f5");
    }

    proptest! {
        #[test]
        fn doesnt_crash(ref s in "\\PC*") {

            Installation::new(Path::new(s).to_path_buf()).is_ok();
        }

        #[test]
        fn parses_all_valid_versions(ref s in r"[0-9]{1,4}\.[0-9]{1,4}\.[0-9]{1,4}[fpb][0-9]{1,4}") {
            let (_t, path) = prepare_unity_installation!(s);
            Installation::new(path).unwrap();
        }
    }
}
