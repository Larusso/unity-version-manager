use std::path::PathBuf;
use unity;

const UNITY_HUB_DEFAULT_LOCATION: &'static str = "/Applications/Unity/Hub/Editors";

pub struct Settings {
    secondary_install_path: Option<PathBuf>,
    default_editor: Option<unity::Version>,
}

impl Settings {
    pub fn new() -> Settings {
        Settings {secondary_install_path: None, default_editor: None}
    }
}

#[derive(Serialize,Deserialize,Debug,PartialEq)]
pub struct EditorValue {
    version: unity::Version,
    #[serde(with = "unity::hub::settings::editor_value_location")]
    location: PathBuf,
    manual: bool
}

impl EditorValue {
    fn new(version: unity::Version, location:PathBuf) -> EditorValue {
        let manual = true;
        EditorValue {version, location, manual}
    }
}

pub mod editor_value_location {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use std::path::{Path, PathBuf};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
        where D: Deserializer<'de>
    {
        let paths:Vec<String> = Vec::deserialize(deserializer)?;
        let path = paths.first().ok_or_else(|| serde::de::Error::invalid_length(0,&"1"))?;
        Ok(Path::new(&paths[0]).to_path_buf())
    }

    pub fn serialize<S>(location: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let s = format!("{}", location.display());
        serializer.serialize_str(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use serde_json;
    use std::collections::HashMap;

    #[test]
    fn parse_editors() {
        let data = r#"{
                        "2018.2.5f1": { "version": "2018.2.5f1", "location": ["/Applications/Unity-2018.2.5f1/Unity.app"], "manual": true },
                        "2017.1.0f3": { "version": "2017.1.0f3", "location": ["/Applications/Unity-2017.1.0f3/Unity.app"], "manual": true }
                  }"#;

        let editors:HashMap<String, EditorValue> = serde_json::from_str(data).unwrap();

        let v = unity::Version::new(2018,2,5,unity::VersionType::Final,1);
        let p = Path::new("/Applications/Unity-2018.2.5f1/Unity.app");
        assert_eq!(editors.get("2018.2.5f1").unwrap(), &EditorValue::new(v, p.to_path_buf()));
    }
}
