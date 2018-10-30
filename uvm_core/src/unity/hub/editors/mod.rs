use error::UvmError;
use result::Result;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Write;
use std::iter::{Iterator, IntoIterator};
use std::path::PathBuf;
use unity;
use unity::hub::paths;
use unity::installation::UnityInstallation;
use  std::fs;
mod cmp;
mod convert;

#[derive(Debug)]
pub struct Editors {
    map: HashMap<unity::Version, EditorInstallation>,
}

impl Editors {
    pub fn load() -> Result<Editors> {
        let path = paths::editors_config_path().ok_or_else(|| {
            UvmError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "Unable to find path for unity hub editors.json",
            ))
        })?;

        let map = if path.exists() {
            debug!("load hub editors from file: {}", path.display());
            let file = File::open(path)?;
            let editors: HashMap<unity::Version, EditorInstallation> = serde_json::from_reader(file)?;
            editors
        } else {
            debug!("hub editors file doesn't exist return empty map");
            let editors:HashMap<unity::Version, EditorInstallation> = HashMap::new();
            editors
        };
        Ok(Editors::create(map))
    }

    pub fn create(mut map:HashMap<unity::Version, EditorInstallation>) -> Editors {
        trace!("create Editors map");
        map.retain(|version, installation| {
            trace!("filter: version: {} - installaton: {:?}", version, installation);
            let check_installation = unity::Installation::new(installation.location.to_path_buf());
            if let Ok(check_installation) = check_installation {
                trace!("Found unity installation at with version {} at location: {}", check_installation.version(), installation.location.display());
                trace!("Installation has correct version: {}", check_installation.version() == version);
                check_installation.version() == version
            } else {
                trace!("No installtion found at location: {}", installation.location.display());
                false
            }
        });
        Editors{map}
    }

    pub fn add(&mut self, editor: EditorInstallation) -> Option<EditorInstallation> {
        self.map.insert(editor.version.clone(), editor.clone())
    }

    pub fn remove(&mut self, editor: &EditorInstallation) -> Option<EditorInstallation> {
        self.map.remove(&editor.version)
    }

    pub fn remove_version(&mut self, version: &unity::Version) -> Option<EditorInstallation> {
        self.map.remove(&version)
    }

    pub fn flush(&self) -> Result<()> {
        let config_path = paths::config_path().ok_or_else(|| {
            UvmError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "Unable to find path for unity hub editors.json",
            ))
        })?;

        let path = paths::editors_config_path().ok_or_else(|| {
            UvmError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "Unable to find path for unity hub editors.json",
            ))
        })?;

        fs::create_dir_all(config_path)?;
        let mut file = File::create(path)?;
        let j = serde_json::to_string(&self.map)?;
        write!(file, "{}", &j);
        Ok(())
    }
}

impl IntoIterator for Editors {
    type Item = EditorInstallation;
    type IntoIter = ::std::vec::IntoIter<EditorInstallation>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.values()
            .map(|installation| installation.clone())
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
    pub fn new(version:unity::Version, location: PathBuf) -> EditorInstallation {
        EditorInstallation {version: version, location: location, manual: true}
    }

    pub fn version(&self) -> &unity::Version {
        &self.version
    }

    pub fn location(&self) -> &PathBuf {
        &self.location
    }
}

pub mod editor_value_location {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use serde::de::Unexpected;
    use serde::ser::SerializeSeq;
    use std::path::{Path, PathBuf};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        let paths: Vec<String> = Vec::deserialize(deserializer)?;
        let path = paths
            .first()
            .ok_or_else(|| serde::de::Error::invalid_length(0, &"1"))?;
        let location = Path::new(&path).parent()
            .ok_or_else(|| serde::de::Error::invalid_value(Unexpected::Other("location with empty parent"), &"valid unity location"))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn parse_editors() {
        let data = r#"{
                        "2018.2.5f1": { "version": "2018.2.5f1", "location": ["/Applications/Unity-2018.2.5f1/Unity.app"], "manual": true },
                        "2017.1.0f3": { "version": "2017.1.0f3", "location": ["/Applications/Unity-2017.1.0f3/Unity.app"], "manual": true }
                  }"#;

        let editors: HashMap<unity::Version, EditorInstallation> = serde_json::from_str(data).unwrap();

        let v = unity::Version::new(2018, 2, 5, unity::VersionType::Final, 1);
        let p = Path::new("/Applications/Unity-2018.2.5f1");
        assert_eq!(
            editors.get(&v).unwrap(),
            &EditorInstallation{ version: v, location: p.to_path_buf(), manual: true}
        );
    }
}
