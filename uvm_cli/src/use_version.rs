use super::ColorOption;
use uvm_core::unity::Version;
use uvm_core;

#[derive(Debug, Deserialize)]
pub struct UseOptions {
    arg_version: Version,
    flag_verbose: bool,
    flag_color: ColorOption
}

impl UseOptions {
    pub fn version(&self) -> &Version {
        &self.arg_version
    }
}

impl super::Options for UseOptions {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }

    fn color(&self) -> &ColorOption {
        &self.flag_color
    }
}
