use super::ColorOption;

#[derive(Debug, Deserialize)]
pub struct ClearOptions {
    flag_verbose: bool,
    flag_color: ColorOption
}

impl super::Options for ClearOptions {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }

    fn color(&self) -> &ColorOption {
        &self.flag_color
    }
}
