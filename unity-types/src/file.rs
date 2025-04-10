use std::path::PathBuf;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize, Serialize)]
pub struct ExtractedPathRename {
    pub from: PathBuf,
    pub to: PathBuf,
    #[serde(rename = "__typename")]
    typename: Option<String>
}