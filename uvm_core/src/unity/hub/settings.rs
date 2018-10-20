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
