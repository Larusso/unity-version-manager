use super::ColorOption;

#[derive(Debug, Deserialize)]
pub struct ListOptions {
    flag_verbose: bool,
    flag_path: bool,
    flag_color: ColorOption
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

    fn color(&self) -> &ColorOption {
        &self.flag_color
    }
}
