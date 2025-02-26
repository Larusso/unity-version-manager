use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize};
use ssri::Integrity;
use crate::model::digital::DigitalValue;
use crate::model::platform::{UnityReleaseCategory, UnityReleaseDownloadArchitecture, UnityReleaseDownloadPlatform, UnityReleaseSkuFamily, UnityReleaseStream};
use crate::model::file::{ExtractedPathRename, FileType};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub version: String,
    pub product_name: String,
    pub release_date: String,
    pub release_notes: ReleaseNotes,
    pub stream: UnityReleaseStream,
    pub downloads: Vec<Download>,
    pub sku_family: UnityReleaseSkuFamily,
    pub recommended: bool,
    pub unity_hub_deep_link: String,
    pub short_revision: String,

}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub url: String,
    #[serde(deserialize_with = "deserialize_sri")]
    pub integrity: Option<Integrity>,
    #[serde(rename = "type")]
    pub file_type: FileType,
    pub platform: UnityReleaseDownloadPlatform,
    pub architecture: UnityReleaseDownloadArchitecture,
    pub modules: Vec<Module>,
    pub download_size: DigitalValue,
    pub installed_size: DigitalValue,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    pub url: String,
    #[serde(deserialize_with = "deserialize_sri")]
    pub integrity: Option<Integrity>,
    #[serde(rename = "type")]
    pub file_type: FileType,
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub category: UnityReleaseCategory,
    pub sub_modules: Vec<Module>,
    pub required: bool,
    pub hidden: bool,
    pub pre_selected: bool,
    pub destination: Option<String>,
    pub extracted_path_rename: Option<ExtractedPathRename>,
    pub download_size: DigitalValue,
    pub installed_size: DigitalValue,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseNotes {
    pub url: String,
    #[serde(deserialize_with = "deserialize_sri")]
    pub integrity: Option<Integrity>,
    #[serde(rename = "type")]
    pub file_type: FileType,
}

fn deserialize_sri<'de, D>(deserializer: D) -> Result<Option<Integrity>, D::Error>
where
    D: Deserializer<'de>,
{
    let sri_str: Option<String> = Option::deserialize(deserializer)?;

    match sri_str {
        Some(s) => match Integrity::from_str(&s) {
            Ok(integrity) => Ok(Some(integrity)),
            Err(_) => Ok(None), // If parsing fails (e.g., MD5 hash), ignore it
        },
        None => Ok(None),
    }
}
