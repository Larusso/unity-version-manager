use serde::de::Deserialize;
use std::fmt;
use std::fmt::{Debug, Display};
use std::path::{PathBuf};

#[derive(Deserialize, Debug, Clone)]
pub enum UnityPlatform {
    Win32,
    Win64,
    OSX,
    Linux,
    Linux64,
    IOS,
    Android,
    Web,
    WebStreamed,
    WebGl,
    XboxOne,
    PS4,
    PSP2,
    WsaPlayer,
    Tizen,
    SamsungTV,
}

impl Display for UnityPlatform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let raw = format!("{:?}", self).to_lowercase();
        write!(f, "{}", raw)
    }
}

#[derive(Debug, Deserialize)]
pub struct LaunchOptions {
    arg_project_path: Option<PathBuf>,
    flag_platform: Option<UnityPlatform>,
    flag_force_project_version: bool,
    flag_recursive: bool,
    flag_verbose: bool
}

impl LaunchOptions {
    pub fn project_path(&self) -> Option<&PathBuf> {
        self.arg_project_path.as_ref()
    }

    pub fn platform(&self) -> Option<&UnityPlatform> {
        self.flag_platform.as_ref()
    }

    pub fn force_project_version(&self) -> bool {
        self.flag_force_project_version
    }

    pub fn recursive(&self) -> bool {
        self.flag_recursive
    }
}

impl super::Options for LaunchOptions {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }
}
