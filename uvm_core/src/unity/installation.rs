use std::path::PathBuf;
use unity::Version;
use std::cmp::Ordering;
use std::str::FromStr;
use unity::InstalledComponents;
use std::io;
use result;
use UvmError;
use unity::version;

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
         return self.path().join("Unity.app")
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

impl Installation {
    pub fn new(path: PathBuf) -> result::Result<Installation> {
        version::read_version_from_path(&path)
            .map(|version| Installation { version: version, path: path.clone() })
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

    pub fn exec_path(&self) -> PathBuf {
        self.path.join("Unity.app/Contents/MacOS/Unity")
    }
}

impl From<crate::unity::hub::editors::EditorInstallation> for Installation {
    fn from(editor: crate::unity::hub::editors::EditorInstallation) -> Self {
        Installation {version: editor.version().to_owned(), path: editor.location().to_path_buf()}
    }
}

#[cfg(all(test, target_os="macos"))]
mod tests {
    use std::fs;
    use std::fs::OpenOptions;
    use std::path::Path;
    use tempfile::Builder;
    use plist::serde::serialize_to_xml;
    use super::*;

    fn create_unity_installation(base_dir:&PathBuf, version: &str) -> PathBuf {
        let path = base_dir.join("Unity");
        let mut dir_builder = fs::DirBuilder::new();
        dir_builder.recursive(true);
        dir_builder.create(&path).unwrap();

        let info_plist_path = path.join("Unity.app/Contents/Info.plist");
        dir_builder.create(info_plist_path.parent().unwrap()).unwrap();

        let info = AppInfo {
            c_f_bundle_version: String::from_str(version).unwrap(),
            unity_build_number: String::from_str("ssdsdsdd").unwrap()
        };

        let file = File::create(info_plist_path).unwrap();
        serialize_to_xml(file, &info).unwrap();

        path
    }

    macro_rules! prepare_unity_installation {
        ($version:expr) => {
            {
                let test_dir = Builder::new()
                                .prefix("installation")
                                .rand_bytes(5)
                                .tempdir()
                                .unwrap();
                let unity_path = create_unity_installation(&test_dir.path().to_path_buf(), $version);
                (test_dir, unity_path)
            }
        };
    }

    #[test]
    fn create_installtion_from_path() {
        let (t , path) = prepare_unity_installation!("2017.1.2f5");
        let subject = Installation::new(path).unwrap();

        assert_eq!(subject.version.to_string(), "2017.1.2f5");
    }

    proptest! {
        #[test]
        fn doesnt_crash(ref s in "\\PC*") {

            Installation::new(Path::new(s).to_path_buf()).is_ok();
        }

        #[test]
        fn parses_all_valid_versions(ref s in r"[0-9]{1,4}\.[0-9]{1,4}\.[0-9]{1,4}[fpb][0-9]{1,4}") {
            let (t , path) = prepare_unity_installation!(s);
            Installation::new(path).unwrap();
        }
    }
}
