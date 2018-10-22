use error::UvmError;
use result::Result;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Write;
use std::iter::Iterator;
use std::path::PathBuf;
use unity;
use unity::hub::paths;
use unity::installation::UnityInstallation;

mod cmp;
mod convert;

pub struct Editors(HashMap<unity::Version, EditorValue>);

impl Editors {
    fn load() -> Result<Editors> {
        let path = paths::editors_config_path().ok_or_else(|| {
            UvmError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "Unable to find path for unity hub editors.json",
            ))
        })?;

        let file = File::open(path)?;
        let editors: HashMap<unity::Version, EditorValue> = serde_json::from_reader(file)?;
        Ok(Editors(editors))
    }

    fn add(&mut self, editor: EditorValue) -> Option<EditorValue> {
        self.0.insert(editor.version.clone(), editor.clone())
    }

    fn remove(&mut self, editor: &EditorValue) -> Option<EditorValue> {
        self.0.remove(&editor.version)
    }

    fn remove_version(&mut self, version: &unity::Version) -> Option<EditorValue> {
        self.0.remove(&version)
    }

    fn flush(&self) -> Result<()> {
        let path = paths::editors_config_path().ok_or_else(|| {
            UvmError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "Unable to find path for unity hub editors.json",
            ))
        })?;

        let mut file = File::create(path)?;
        let j = serde_json::to_string(&self.0)?;
        write!(file, "{}", &j);
        Ok(())
    }
}

impl Iterator for Editors {
    type Item = EditorValue;

    fn next(&mut self) -> Option<Self::Item> {
        for n in &mut self.0 {
            return Some(n.1.to_owned());
        }
        None
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct EditorValue {
    version: unity::Version,
    #[serde(with = "unity::hub::editors::editor_value_location")]
    location: PathBuf,
    manual: bool,
}

impl UnityInstallation for EditorValue {
    fn path(&self) -> &PathBuf {
        &self.location()
    }

    fn version(&self) -> &unity::Version {
        &self.version
    }
}

impl EditorValue {
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
    use std::path::{Path, PathBuf};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        let paths: Vec<String> = Vec::deserialize(deserializer)?;
        let path = paths
            .first()
            .ok_or_else(|| serde::de::Error::invalid_length(0, &"1"))?;
        let location = Path::new(&paths[0]).parent()
            .ok_or_else(|| serde::de::Error::invalid_value(Unexpected::Other("location with empty parent"), &"valid unity location"))?;
        Ok(location.to_path_buf())
    }

    pub fn serialize<S>(location: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", location.display());
        serializer.serialize_str(&s)
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

        let editors: HashMap<unity::Version, EditorValue> = serde_json::from_str(data).unwrap();

        let v = unity::Version::new(2018, 2, 5, unity::VersionType::Final, 1);
        let p = Path::new("/Applications/Unity-2018.2.5f1");
        assert_eq!(
            editors.get(&v).unwrap(),
            &EditorValue{ version: v, location: p.to_path_buf(), manual: true}
        );
    }
}
