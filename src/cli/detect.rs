use std::path::PathBuf;
use serde::de::Deserialize;


#[derive(Debug, Deserialize)]
pub struct DetectOptions {
    arg_project_path: Option<PathBuf>,
    flag_recursive: bool,
    flag_verbose: bool,
}

impl DetectOptions {
    pub fn project_path(&self) -> Option<&PathBuf> {
        self.arg_project_path.as_ref()
    }
    pub fn recursive(&self) -> bool {
        self.flag_recursive
    }

    pub fn verbose(&self) -> bool {
        self.flag_verbose
    }
}
