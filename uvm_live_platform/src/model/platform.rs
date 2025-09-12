use std::fmt;
use std::fmt::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnityReleaseDownloadArchitecture {
    X86_64,
    Arm64
}

impl Default for UnityReleaseDownloadArchitecture {
    fn default() -> Self {
        if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            Self::X86_64
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            Self::Arm64
        } else {
            Self::X86_64
        }
    }
}

impl Display for UnityReleaseDownloadArchitecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use UnityReleaseDownloadArchitecture::*;
        let s = match self {
            X86_64 => "x86_64",
            Arm64 => "arm64",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
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

impl Display for UnityReleaseDownloadPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use UnityReleaseDownloadPlatform::*;
        let s = match self {
            MacOs => "macOS",
            Linux => "Linux",
            Windows => "Windows",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnityReleaseStream {
    Lts,
    Beta,
    Alpha,
    Tech,
    Supported,
}

impl Default for UnityReleaseStream {
    fn default() -> Self {
        Self::Lts
    }
}

impl Display for UnityReleaseStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use UnityReleaseStream::*;
        let s = match self {
            Lts => "LTS",
            Beta => "Beta",
            Alpha => "Alpha",
            Tech => "Tech Preview",
            Supported => "Supported",
        };
        write!(f, "{}", s)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Deserialize, Serialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnityReleaseEntitlement {
    Xlts,
    U7Alpha,
}

impl Display for UnityReleaseEntitlement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use UnityReleaseEntitlement::*;
        let s = match self {
            Xlts => "XLTS",
            U7Alpha => "U7 Alpha",
        };
        write!(f, "{}", s)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Deserialize, Serialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
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