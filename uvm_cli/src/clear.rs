#[derive(Debug, Deserialize)]
pub struct ClearOptions {
    flag_verbose: bool,
}

impl super::Options for ClearOptions {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }
}
