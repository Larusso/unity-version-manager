#[derive(Debug, Deserialize)]
pub struct ListOptions {
    flag_verbose: bool,
    flag_path: bool
}

impl ListOptions {
    pub fn path_only(&self) -> bool {
        self.flag_path
    }
}

impl super::Options for ListOptions {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }
}
