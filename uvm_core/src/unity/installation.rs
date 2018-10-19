use std::path::PathBuf;
use unity::Version;
use std::cmp::Ordering;
use std::str::FromStr;
use unity::InstalledComponents;
use std::io;
use result;
use UvmError;
use plist::serde::deserialize;
use std::fs::File;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AppInfo {
    pub c_f_bundle_version: String,
    pub unity_build_number: String,
}

#[derive(PartialEq, Eq, Debug, Clone)]
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
    pub fn new(path: PathBuf) -> result::Result<Installation> {
        //on macOS the unity installation is a directory
        if !path.exists() {
            return Err(UvmError::IoError(io::Error::new(io::ErrorKind::InvalidInput, format!("Provided Path does not exist. {}", path.display()))))
        }
        if path.is_dir() {
            //check for the `Unity.app` package
            let info_plist_path = path.join("Unity.app/Contents/Info.plist");
            let file = File::open(info_plist_path)?;
            let info:AppInfo = deserialize(file)?;
            let version = Version::from_str(&info.c_f_bundle_version)?;

            Ok(Installation {
                version: version,
                path: path.clone()
            })
        } else {
            Err(UvmError::IoError(io::Error::new(io::ErrorKind::InvalidInput, "Provided Path is not a Unity installtion.")))
        }
    }

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

#[cfg(test)]
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
