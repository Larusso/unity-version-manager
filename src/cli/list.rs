use serde::de::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ListOptions {
    flag_verbose: bool,
    flag_path: bool
}

impl ListOptions {
    pub fn verbose(&self) -> bool {
        self.flag_verbose
    }

    pub fn path_only(&self) -> bool {
        self.flag_path
    }
}
