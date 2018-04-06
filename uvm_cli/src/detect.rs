use std::path::PathBuf;
use super::ColorOption;

#[derive(Debug, Deserialize)]
pub struct DetectOptions {
    arg_project_path: Option<PathBuf>,
    flag_recursive: bool,
    flag_verbose: bool,
    flag_color: ColorOption
}

impl DetectOptions {
    pub fn project_path(&self) -> Option<&PathBuf> {
        self.arg_project_path.as_ref()
    }
    pub fn recursive(&self) -> bool {
        self.flag_recursive
    }
}

impl super::Options for DetectOptions {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }

    fn color(&self) -> &ColorOption {
        &self.flag_color
    }
}
