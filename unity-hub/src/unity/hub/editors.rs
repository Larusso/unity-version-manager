use std::cmp::Ordering;
use super::*;
use crate::unity;
use crate::unity::hub::paths;
use crate::unity::installation::{UnityInstallation, Installation, FromInstallation};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::iter::{IntoIterator, Iterator};
use std::path::PathBuf;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use unity_version::Version;

#[derive(Debug)]
pub struct Editors {
    map: HashMap<Version, EditorInstallation>,
}

impl Editors {
    pub fn load() -> Result<Editors> {
        let path = paths::editors_config_path()
            .ok_or_else(|| (UnityHubError::ConfigDirectoryNotFound))?;

        let map: HashMap<Version, EditorInstallation> = if path.exists() {
            debug!("load hub editors from file: {}", path.display());
            File::open(path)
                .map_err(|source| {
                    UnityHubError::ReadConfigError{ config: "editors.json".to_string(), source: source.into() }
                })
                .and_then(|f| {
                    serde_json::from_reader(f).map_err(|source| {
                        UnityHubError::ReadConfigError { config: "editors.json".to_string(), source: source.into()}
                    })
                })?
        } else {
            debug!("hub editors file doesn't exist return empty map");
            HashMap::new()
        };
        trace!("loaded editors map: {:?}", map);
        Ok(Editors::create(map))
    }

    pub fn create(mut map: HashMap<Version, EditorInstallation>) -> Editors {
        trace!("create Editors map");
        map.retain(|version, installation| {
            trace!(
                "filter: version: {} - installaton: {:?}",
                version,
                installation
            );
            let check_installation = UnityInstallation::new(installation.location.to_path_buf());
            if let Ok(check_installation) = check_installation {
                trace!(
                    "Found api installation at with version {} at location: {}",
                    check_installation.version(),
                    installation.location.display()
                );
                trace!(
                    "Installation has correct version: {}",
                    check_installation.version() == version
                );
                check_installation.version() == version
            } else {
                trace!(
                    "No installtion found at location: {}",
                    installation.location.display()
                );
                false
            }
        });
        Editors { map }
    }

    pub fn add(&mut self, editor: &EditorInstallation) -> Option<EditorInstallation> {
        self.map.insert(editor.version.clone(), editor.clone())
    }

    pub fn remove(&mut self, editor: &EditorInstallation) -> Option<EditorInstallation> {
        self.map.remove(&editor.version)
    }

    pub fn remove_version(&mut self, version: &Version) -> Option<EditorInstallation> {
        self.map.remove(&version)
    }

    pub fn flush(&self) -> Result<()> {
        let config_path =
            paths::config_path().ok_or_else(|| (UnityHubError::ConfigDirectoryNotFound))?;

        let path = paths::editors_config_path()
            .ok_or_else(|| UnityHubError::ConfigNotFound("editors.json".to_string()))?;

        fs::create_dir_all(config_path).map_err(|source| UnityHubError::FailedToCreateConfigDirectory{source})?;
        let mut file = File::create(path).map_err(|source| UnityHubError::FailedToCreateConfig{config: "editors.json".to_string(), source})?;

        let j = serde_json::to_string(&self.map).map_err(|source| {
            UnityHubError::WriteConfigError {config: "editors.json".to_string(), source: source.into()}
        })?;
        write!(file, "{}", &j).map_err(|source| {
            UnityHubError::WriteConfigError {config: "editors.json".to_string(), source: source.into()}
        })
    }
}

impl IntoIterator for Editors {
    type Item = EditorInstallation;
    type IntoIter = ::std::vec::IntoIter<EditorInstallation>;

    fn into_iter(self) -> Self::IntoIter {
        self.map
            .values()
            .cloned()
            .collect::<Vec<Self::Item>>()
            .into_iter()
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct EditorInstallation {
    version: Version,
    #[serde(with = "editor_value_location")]
    location: PathBuf,
    manual: bool,
}

trait EditorInstallationMarker {}

impl EditorInstallationMarker for EditorInstallation {}

impl Installation for EditorInstallation {
    fn path(&self) -> &PathBuf {
        &self.location()
    }

    fn version(&self) -> &Version {
        &self.version
    }
}

impl EditorInstallation {
    pub fn new(version: Version, location: PathBuf) -> EditorInstallation {
        trace!("create new editor installation: {} at {}", version, location.display());
        EditorInstallation {
            version,
            location,
            manual: true,
        }
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn location(&self) -> &PathBuf {
        &self.location
    }
}

pub mod editor_value_location {
    use serde::de::Unexpected;
    use serde::ser::SerializeSeq;
    use serde::{self, Deserialize, Deserializer, Serializer};
    use std::path::{Path, PathBuf};
    use log::trace;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        let paths: Vec<String> = Vec::deserialize(deserializer)?;
        let path = paths
            .first()
            .ok_or_else(|| serde::de::Error::invalid_length(0, &"1"))?;
        let location = Path::new(&path)
            .parent()
            .and_then(|location| {
                if cfg!(target_os = "windows") || cfg!(target_os = "linux") {
                    return location.parent();
                }
                Some(location)
            })
            .ok_or_else(|| {
                serde::de::Error::invalid_value(
                    Unexpected::Other("location with empty parent"),
                    &"valid api location",
                )
            })?;
        trace!("found Editor location: {}", location.display());
        Ok(location.to_path_buf())
    }

    pub fn serialize<S>(location: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(1))?;

        #[cfg(target_os = "windows")]
        seq.serialize_element(&location.join("Editors\\Unity.exe"))?;
        #[cfg(target_os = "linux")]
        seq.serialize_element(&location.join("Editors/Unity"))?;
        #[cfg(target_os = "macos")]
        seq.serialize_element(&location.join("Unity.app"))?;
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        seq.serialize_element(&location)?;

        seq.end()
    }
}

impl Ord for EditorInstallation {
    fn cmp(&self, other: &EditorInstallation) -> Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialOrd for EditorInstallation {
    fn partial_cmp(&self, other: &EditorInstallation) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// const INSTALLATION_BINARY: &str = "Unity.app";
//
// impl From<UnityInstallation> for EditorInstallation {
//     fn from(installation: UnityInstallation) -> Self {
//         let location = installation.path().join(INSTALLATION_BINARY);
//         let version = installation.into_version();
//         EditorInstallation {
//             version,
//             location,
//             manual: true,
//         }
//     }
// }

impl<I> FromInstallation<I> for EditorInstallation
where I: Installation + Sized {
    fn from_installation(value: I) -> Self {
        EditorInstallation {
            location: value.location().to_path_buf(),
            version: value.version().clone(),
            manual: true,
        }
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;
    use unity_version::{ReleaseType, Version};
    use crate::unity::hub::editors::EditorInstallation;

    #[test]
    fn parse_editors() {
        #[cfg(target_os = "macos")]
        let data = r#"{
                        "2018.2.5f1": { "version": "2018.2.5f1", "location": ["/Applications/Unity-2018.2.5f1/Unity.app"], "manual": true },
                        "2017.1.0f3": { "version": "2017.1.0f3", "location": ["/Applications/Unity-2017.1.0f3/Unity.app"], "manual": true }
                  }"#;

        #[cfg(target_os = "windows")]
        let data = r#"{
                      "2018.2.5f1": { "version": "2018.2.5f1", "location": ["C:\\Program Files\\Unity-2018.2.5f1\\Editor\\Unity.exe"], "manual": true },
                      "2017.1.0f3": { "version": "2017.1.0f3", "location": ["C:\\Program Files\\Unity-2017.1.0f3\\Editor\\Unity.exe"], "manual": true }
                }"#;

        #[cfg(target_os = "linux")]
        let data = r#"{
                        "2018.2.5f1": { "version": "2018.2.5f1", "location": ["/homce/ci/.local/share/Unity-2018.2.5f1/Editor/Unity"], "manual": true },
                        "2017.1.0f3": { "version": "2017.1.0f3", "location": ["/homce/ci/.local/share/Unity-2017.1.0f3/Editor/Unity"], "manual": true }
                  }"#;

        let editors: HashMap<Version, EditorInstallation> =
            serde_json::from_str(data).unwrap();

        let v = Version::new(2018, 2, 5, ReleaseType::Final, 1);

        #[cfg(target_os = "macos")]
        let p = Path::new("/Applications/Unity-2018.2.5f1");
        #[cfg(target_os = "windows")]
        let p = Path::new("C:\\Program Files\\Unity-2018.2.5f1");
        #[cfg(target_os = "linux")]
        let p = Path::new("/homce/ci/.local/share/Unity-2018.2.5f1");

        assert_eq!(
            &editors[&v],
            &EditorInstallation {
                version: v,
                location: p.to_path_buf(),
                manual: true
            }
        );
    }

    #[test]
    fn write_editors() {
        let v = Version::new(2018, 2, 5, ReleaseType::Final, 1);

        #[cfg(target_os = "macos")]
        let p = Path::new("/Applications/Unity-2018.2.5f1");
        #[cfg(target_os = "windows")]
        let p = Path::new("C:\\Program Files\\Unity-2018.2.5f1");
        #[cfg(target_os = "linux")]
        let p = Path::new("/homce/ci/.local/share/Unity-2018.2.5f1");

        #[cfg(target_os = "macos")]
        let expected_result = r#"{"2018.2.5f1":{"version":"2018.2.5f1","location":["/Applications/Unity-2018.2.5f1/Unity.app"],"manual":true}}"#;

        #[cfg(target_os = "windows")]
        let expected_result = r#"{"2018.2.5f1":{"version":"2018.2.5f1","location":["C:\\Program Files\\Unity-2018.2.5f1\\Editor\\Unity.exe"],"manual":true}}"#;

        #[cfg(target_os = "linux")]
        let expected_result = r#"{"2018.2.5f1":{"version":"2018.2.5f1","location":["/homce/ci/.local/share/Unity-2018.2.5f1/Editor/Unity"],"manual":true}}"#;

        let i = EditorInstallation {
            version: v.clone(),
            location: p.to_path_buf(),
            manual: true,
        };

        let expected_editors: HashMap<Version, EditorInstallation> =
            serde_json::from_str(&expected_result).unwrap();

        let mut editors: HashMap<Version, EditorInstallation> = HashMap::new();
        editors.insert(v, i);
        let json = serde_json::to_string(&editors).expect("convert editors map to json");
        let written_editors: HashMap<Version, EditorInstallation> =
            serde_json::from_str(&json).unwrap();

        assert_eq!(
            written_editors,
            expected_editors
        );
    }
}
