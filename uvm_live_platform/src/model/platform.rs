use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnityReleaseDownloadArchitecture {
    X86_64,
    Arm64
}

impl Default for UnityReleaseDownloadArchitecture {
    fn default() -> Self {
        if cfg!(target_arch = "x86_64") {
            Self::X86_64
        } else if cfg!(target_arch = "aarch64") {
            Self::Arm64
        } else {
            panic!("Not supported on current architecture")
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnityReleaseDownloadPlatform {
    MacOs,
    Linux,
    Windows
}

impl Default for UnityReleaseDownloadPlatform {
    fn default() -> Self {
        if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else {
            panic!("Not supported on current OS")
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnityReleaseStream {
    Lts,
    Beta,
    Alpha,
    Tech,
}

impl Default for UnityReleaseStream {
    fn default() -> Self {
        Self::Lts
    }
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

impl fmt::Display for UnityReleaseCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use UnityReleaseCategory::*;
        let s = match self {
            DevTool => "Dev tools",
            Plugin => "Plugins",
            Documentation => "Documentation",
            Component => "Components",
            Platform => "Platform",
            LanguagePack => "Language packs (Preview)",
        };
        write!(f, "{}", s)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnityReleaseSkuFamily {
    Classic,
    Dots
}