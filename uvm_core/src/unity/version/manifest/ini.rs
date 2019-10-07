use super::MD5;
use crate::unity::Component;
use reqwest::Url;
use std::collections::HashMap;
use std::fmt;

#[derive(Deserialize, Debug)]
pub struct IniData {
    pub title: String,
    pub description: String,
    pub url: String,
    #[serde(skip)]
    pub download_url: Option<Url>,
    pub size: u64,
    pub installedsize: u64,
    pub md5: Option<MD5>,
    #[serde(with = "de::bool")]
    #[serde(default = "de::bool::default")]
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "eulaurl1")]
    pub eula_url_1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "eulalabel1")]
    pub eula_label_1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "eulamessage")]
    pub eula_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<Component>,
    #[serde(flatten)]
    pub other: HashMap<String, String>,
}

impl fmt::Display for IniData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.title, self.url)
    }
}

mod de {
    pub mod bool {
        use serde::{Deserialize, Deserializer};
        use std::result;

        pub fn default() -> bool {
            false
        }

        pub fn deserialize<'de, D>(deserializer: D) -> result::Result<bool, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            if s.is_empty() {
                Ok(false)
            } else {
                match s.as_str() {
                    "true" => Ok(true),
                    "false" => Ok(true),
                    _ => Ok(false),
                }
            }
        }
    }
}
