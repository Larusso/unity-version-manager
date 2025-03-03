use serde::{Deserialize, Serialize};
use unity_types::digital::{DigitalValue, Size};
use unity_types::file::ExtractedPathRename;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    pub url: String,
    pub integrity: Option<String>,
    #[serde(rename = "type")]
    pub file_type: FileType,
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub category: String,
    #[serde(default)]
    pub sub_modules: Vec<Module>,
    pub required: bool,
    pub hidden: bool,
    pub pre_selected: bool,
    pub destination: Option<String>,
    pub extracted_path_rename: Option<ExtractedPathRename>,
    pub download_size: Size,
    pub installed_size: Size,
    #[serde(default)]
    pub is_installed: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FileType {
    Text,
    TarGz,
    TarXz,
    Zip,
    Pkg,
    Exe,
    Po,
    Dmg,
    Lzma,
    Lz4,
    Md,
    Pdf
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnityReleaseCategory {
    Documentation,
    Platform,
    LanguagePack,
    DevTool,
    Plugin,
    Component,
}

pub enum SizeOrDigitalValue {
    Size(usize),
    Value(DigitalValue),
}