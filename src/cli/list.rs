use serde::de::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ListOptions {
    flag_verbose: bool
}

impl ListOptions {
    pub fn verbose(&self) -> bool {
        self.flag_verbose
    }
}
