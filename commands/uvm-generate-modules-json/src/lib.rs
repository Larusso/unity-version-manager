use serde_derive::Deserialize;
use uvm_cli;
use uvm_cli::ColorOption;
use uvm_core::unity::Version;

#[derive(Debug, Deserialize)]
pub struct Options {
    arg_version: Option<Version>,
    flag_verbose: bool,
    flag_debug: bool,
    flag_all: bool,
    flag_color: ColorOption,
}

impl Options {
    pub fn version(&self) -> &Option<Version> {
        &self.arg_version
    }

    pub fn all(&self) -> bool {
        self.flag_all
    }
}

impl uvm_cli::Options for Options {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }

    fn debug(&self) -> bool {
        self.flag_debug
    }

    fn color(&self) -> &ColorOption {
        &self.flag_color
    }
}
