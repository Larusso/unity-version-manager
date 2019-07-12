use super::Component;
use std::path::{Path, PathBuf};

pub fn installpath(_component:Component) -> Option<PathBuf> {
    None
}

pub fn install_location(_component:Component) -> Option<PathBuf> {
    None
}
