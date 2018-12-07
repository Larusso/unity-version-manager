use super::*;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::iter::{IntoIterator, Iterator};
use std::path::PathBuf;
use unity;
use unity::hub::paths;
use unity::installation::UnityInstallation;
mod cmp;
mod convert;

#[derive(Debug)]
pub struct Editors {
    map: HashMap<unity::Version, EditorInstallation>,
}

impl Editors {
    pub fn load() -> Result<Editors> {
        let path = paths::editors_config_path()
            .ok_or_else(|| (UvmHubErrorKind::ConfigDirectoryNotFound))?;

        let map: HashMap<unity::Version, EditorInstallation> = if path.exists() {
            debug!("load hub editors from file: {}", path.display());
            File::open(path)
                .map_err(|err| {
                    UvmHubError::with_chain(
                        err,
                        UvmHubErrorKind::ReadConfigError("editors.json".to_string()),
                    )
                })
                .and_then(|f| {
                    serde_json::from_reader(f).map_err(|err| {
                        UvmHubError::with_chain(
                            err,
                            UvmHubErrorKind::ReadConfigError("editors.json".to_string()),
                        )
                    })
                })?
        } else {
            debug!("hub editors file doesn't exist return empty map");
            HashMap::new()
        };
        Ok(Editors::create(map))
    }

    pub fn create(mut map: HashMap<unity::Version, EditorInstallation>) -> Editors {
        trace!("create Editors map");
        map.retain(|version, installation| {
            trace!(
                "filter: version: {} - installaton: {:?}",
                version,
                installation
            );
            let check_installation = unity::Installation::new(installation.location.to_path_buf());
            if let Ok(check_installation) = check_installation {
                trace!(
                    "Found unity installation at with version {} at location: {}",
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

    pub fn remove_version(&mut self, version: &unity::Version) -> Option<EditorInstallation> {
        self.map.remove(&version)
    }

    pub fn flush(&self) -> Result<()> {
        let config_path =
            paths::config_path().ok_or_else(|| (UvmHubErrorKind::ConfigDirectoryNotFound))?;

        let path = paths::editors_config_path()
            .ok_or_else(|| UvmHubErrorKind::ConfigNotFound("editors.json".to_string()))?;

        fs::create_dir_all(config_path)?;
        let mut file = File::create(path)?;

        let j = serde_json::to_string(&self.map).map_err(|err| {
            UvmHubError::with_chain(
                err,
                UvmHubErrorKind::WriteConfigError("editors.json".to_string()),
            )
        })?;
        write!(file, "{}", &j).map_err(|err| {
            UvmHubError::with_chain(
                err,
                UvmHubErrorKind::WriteConfigError("editors.json".to_string()),
            )
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
    version: unity::Version,
    #[serde(with = "unity::hub::editors::editor_value_location")]
    location: PathBuf,
    manual: bool,
}

impl UnityInstallation for EditorInstallation {
    fn path(&self) -> &PathBuf {
        &self.location()
    }

    fn version(&self) -> &unity::Version {
        &self.version
    }
}

impl EditorInstallation {
    pub fn new(version: unity::Version, location: PathBuf) -> EditorInstallation {
        EditorInstallation {
            version,
            location,
            manual: true,
        }
    }

    pub fn version(&self) -> &unity::Version {
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
                if cfg!(target_os = "windows") {
                    return location.parent();
                }
                Some(location)
            }).ok_or_else(|| {
                serde::de::Error::invalid_value(
                    Unexpected::Other("location with empty parent"),
                    &"valid unity location",
                )
            })?;
        Ok(location.to_path_buf())
    }

    pub fn serialize<S>(location: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(1))?;
        seq.serialize_element(&location.join("Unity.app"))?;
        seq.end()
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn parse_editors() {
        let data = r#"{
                        "2018.2.5f1": { "version": "2018.2.5f1", "location": ["/Applications/Unity-2018.2.5f1/Unity.app"], "manual": true },
                        "2017.1.0f3": { "version": "2017.1.0f3", "location": ["/Applications/Unity-2017.1.0f3/Unity.app"], "manual": true }
                  }"#;

        let editors: HashMap<unity::Version, EditorInstallation> =
            serde_json::from_str(data).unwrap();

        let v = unity::Version::new(2018, 2, 5, unity::VersionType::Final, 1);
        let p = Path::new("/Applications/Unity-2018.2.5f1");
        assert_eq!(
            &editors[&v],
            &EditorInstallation {
                version: v,
                location: p.to_path_buf(),
                manual: true
            }
        );
    }
}
