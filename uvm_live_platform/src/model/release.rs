use crate::model::digital::DigitalValue;
use crate::model::file::{ExtractedPathRename, FileType};
use crate::model::platform::{
    UnityReleaseCategory, UnityReleaseDownloadArchitecture, UnityReleaseDownloadPlatform,
    UnityReleaseSkuFamily, UnityReleaseStream,
};
use crate::Size;
use derive_getters::Getters;
use serde::{Deserialize, Deserializer, Serialize};
use ssri::Integrity;
use std::fmt::format;
use std::fs::File;
use std::str::FromStr;

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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub third_party_notices: Vec<UnityThirdPartyNotice>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    #[serde(flatten)]
    pub release_file: UnityReleaseFile,
    pub platform: UnityReleaseDownloadPlatform,
    pub architecture: UnityReleaseDownloadArchitecture,
    pub modules: Vec<Module>,
    pub download_size: Size,
    pub installed_size: Size,
}

impl Download {
    pub fn iter_modules(&self) -> impl Iterator<Item = &Module> {
        ModuleIterator::new(&self.modules)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Getters)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    #[serde(rename = "__typename")]
    #[getter(skip)]
    type_name: String,
    #[serde(flatten)]
    release_file: UnityReleaseFile,
    id: String,
    #[getter(skip)]
    slug: String,
    #[getter(skip)]
    name: String,
    description: String,
    category: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    sub_modules: Vec<Module>,
    #[getter(skip)]
    required: bool,
    hidden: bool,
    pre_selected: bool,
    #[getter(skip)]
    destination: Option<String>,
    extracted_path_rename: Option<ExtractedPathRename>,
    #[getter(skip)]
    pub download_size: Size,
    #[getter(skip)]
    pub installed_size: Size,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    eula: Vec<Eula>,
}

impl Module {
    pub fn destination(&self) -> Option<String> {
        if &self.id == "ios" {
            self.destination
                .as_ref()
                .map(|d| format!("{}/iOSSupport", d.to_string()).to_string())
        } else {
            self.destination.clone()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Eula {
    #[serde(flatten)]
    pub release_file: UnityReleaseFile,
    pub label: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnityThirdPartyNotice {
    #[serde(flatten)]
    release_file: UnityReleaseFile,
    original_file_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnityReleaseFile {
    pub url: String,
    #[serde(deserialize_with = "deserialize_sri")]
    pub integrity: Option<Integrity>,
    #[serde(rename = "type")]
    pub file_type: FileType,
}

pub type ReleaseNotes = UnityReleaseFile;

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

fn null_to_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

pub struct ModuleIterator<'a> {
    stack: Vec<&'a Module>,
}

impl<'a> ModuleIterator<'a> {
    pub fn new(modules: &'a [Module]) -> Self {
        Self {
            stack: modules.iter().rev().collect(),
        }
    }
}

impl<'a> Iterator for ModuleIterator<'a> {
    type Item = &'a Module;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.stack.pop()?;
        self.stack.extend(next.sub_modules().iter().rev());
        Some(next)
    }
}
