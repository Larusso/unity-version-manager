use std::env;
use std::path::{Path, PathBuf};

const UNITY_HUB_CONFIG_PATH: &'static str = "/Library/Application Support/UnityHub";

pub fn config_path() -> Option<PathBuf> {
    env::home_dir().map(|path| path.join(UNITY_HUB_CONFIG_PATH))
}

pub fn editors_config_path() -> Option<PathBuf> {
    config_path().map(|path| path.join("editors.json"))
}

pub fn secondary_install_path_config_path() -> Option<PathBuf> {
    config_path().map(|path| path.join("secondaryInstallPath.json"))
}

pub fn default_editor_config_path() -> Option<PathBuf> {
    config_path().map(|path| path.join("defaultEditor.json"))
}
