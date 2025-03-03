use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::convert::TryFrom;
// use crate::unity::InstalledComponents;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AppInfo {
    pub c_f_bundle_version: String,
    pub unity_build_number: String,
}

pub trait Installation: Eq + Ord {
    fn path(&self) -> &PathBuf;

    fn version(&self) -> &Version;

    #[cfg(target_os = "windows")]
    fn location(&self) -> PathBuf {
        self.path().join("Editor\\Unity.exe")
    }

    #[cfg(target_os = "macos")]
    fn location(&self) -> PathBuf {
        self.path().join("Unity.app")
    }

    #[cfg(target_os = "linux")]
    fn location(&self) -> PathBuf {
        self.path().join("Editor/Unity")
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    fn exec_path(&self) -> PathBuf {
        self.location()
    }

    #[cfg(target_os = "macos")]
    fn exec_path(&self) -> PathBuf {
        self.path().join("Unity.app/Contents/MacOS/Unity")
    }

    fn installed_modules(&self) -> Result<impl IntoIterator<Item = Module>, UnityError> {
        let modules_json_path = self.path().join("modules.json");
        let file_content = fs::read_to_string(&modules_json_path)?;
        let modules: Vec<Module> = serde_json::from_str(&file_content)?;
        let installed_modules = modules.into_iter().filter(|m| m.is_installed);
        Ok(installed_modules)
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UnityInstallation {
    version: Version,
    path: PathBuf,
}

impl Installation for UnityInstallation {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn version(&self) -> &Version {
        &self.version
    }
}

impl Ord for UnityInstallation {
    fn cmp(&self, other: &UnityInstallation) -> Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialOrd for UnityInstallation {
    fn partial_cmp(&self, other: &UnityInstallation) -> Option<Ordering> {
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

#[cfg(target_os = "linux")]
fn adjust_path(path:&Path) -> Option<&Path> {
    if path.is_file() {
        if let Some(name) = path.file_name() {
            if name == "Unity" {
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

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn adjust_path(path:&Path) -> Option<&Path> {
    None
}

impl UnityInstallation {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<UnityInstallation, UnityHubError> {
        let path = path.as_ref();
        let path = if let Some(p) = adjust_path(path) {
            p
        } else {
            path
        };

        let version = Version::try_from(path)?;
        Ok(UnityInstallation {
            version,
            path: path.to_path_buf(),
        })
    }

    // //TODO remove clone()
    // pub fn installed_modules(&self) -> InstalledComponents {
    //     InstalledComponents::new(self.clone())
    // }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn into_version(self) -> Version {
        self.version
    }

    pub fn version_owned(&self) -> Version {
        self.version.to_owned()
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    #[cfg(target_os = "windows")]
    pub fn location(&self) -> PathBuf {
        self.path().join("Editor\\Unity.exe")
    }

    #[cfg(target_os = "macos")]
    pub fn location(&self) -> PathBuf {
        self.path().join("Unity.app")
    }

    #[cfg(target_os = "linux")]
    pub fn location(&self) -> PathBuf {
        self.path().join("Editor/Unity")
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    pub fn exec_path(&self) -> PathBuf {
        self.location()
    }

    #[cfg(target_os = "macos")]
    pub fn exec_path(&self) -> PathBuf {
        self.path().join("Unity.app/Contents/MacOS/Unity")
    }
}

use std::{fmt, fs};
use serde::{Deserialize, Serialize};
use unity_version::Version;
use crate::error::UnityHubError;
use crate::unity::error::UnityError;
use crate::unity::hub::module::Module;

impl fmt::Display for UnityInstallation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.version, self.path.display())
    }
}

pub trait FromInstallation<T:Sized> {
    fn from_installation(value: T) -> Self;
}

impl<I> FromInstallation<I> for UnityInstallation
where
    I: Installation,
    I: Sized, // Ensures this does not apply to trait objects
{
    fn from_installation(value: I) -> UnityInstallation {
        UnityInstallation {
            path: value.location().to_path_buf(),
            version: value.version().clone(),
        }
    }
}

// impl<I> From<I> for UnityInstallation
// where I: Installation {
//     fn from(value: I) -> Self {
//         UnityInstallation {
//             version: value.version().to_owned(),
//             path: value.location().to_path_buf(),
//         }
//     }
// }

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use plist::to_writer_xml;
    use std::fs;
    use std::fs::File;
    use std::path::Path;
    use std::str::FromStr;
    use proptest::proptest;
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

        to_writer_xml(&file, &info).unwrap();
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
        let subject = UnityInstallation::new(path).unwrap();

        assert_eq!(subject.version.to_string(), "2017.1.2f5");
    }

    #[test]
    fn create_installation_from_executable_path() {
        let(_t, path) = prepare_unity_installation!("2017.1.2f5");
        let installation = UnityInstallation::new(path).unwrap();
        let subject = UnityInstallation::new(installation.exec_path()).unwrap();

        assert_eq!(subject.version.to_string(), "2017.1.2f5");
    }

    proptest! {
        #[test]
        fn doesnt_crash(ref s in "\\PC*") {
            let _ = UnityInstallation::new(Path::new(s).to_path_buf()).is_ok();
        }

        #[test]
        fn parses_all_valid_versions(ref s in r"[0-9]{1,4}\.[0-9]{1,4}\.[0-9]{1,4}[fpb][0-9]{1,4}") {
            let (_t, path) = prepare_unity_installation!(s);
            UnityInstallation::new(path).unwrap();
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod linux_tests {
    use std::fs;
    use std::fs::{create_dir_all, File};
    use std::path::PathBuf;
    use crate::Installation;
    use crate::unity::Component;

    macro_rules! prepare_unity_installation {
        ($version:expr) => {{
            let test_dir = tempfile::Builder::new()
                .prefix("installation")
                .rand_bytes(5)
                .tempdir()
                .unwrap();
            let unity_path = create_unity_installation(&test_dir.path().to_path_buf(), $version);
            (test_dir, unity_path)
        }};
    }

    fn create_unity_installation(base_dir: &PathBuf, version: &str) -> PathBuf {
        let path = base_dir.join(version);
        let mut dir_builder = fs::DirBuilder::new();
        dir_builder.recursive(true);
        dir_builder.create(&path).unwrap();

        let exec_path = path.join("Editor/Unity");
        dir_builder
            .create(exec_path.parent().unwrap())
            .unwrap();
        File::create(exec_path).unwrap();
        path
    }

    #[test]
    fn installation_recognizes_installed_webgl_module() {
        let(_t, path) = prepare_unity_installation!("2021.3.35f1");
        //Create WegGL module directory, so that the installation thinks its installed
        create_dir_all(path.join("Editor/Data/PlaybackEngines/WebGLSupport")).unwrap();
        let installation = Installation::new(path).unwrap();
        let mut components = installation.installed_components();
        let has_webgl_component = components.any(|c| c == Component::WebGl);

        assert_eq!(has_webgl_component, true);
    }

    #[test]
    fn installation_recognizes_non_installed_webgl_module() {
        let(_t, path) = prepare_unity_installation!("2021.3.35f1");
        //Create WegGL module directory, so that the installation thinks its installed
        let installation = Installation::new(path).unwrap();
        let mut components = installation.installed_components();
        let has_webgl_component = components.any(|c| c == Component::WebGl);

        assert_eq!(has_webgl_component, false);
    }
}
