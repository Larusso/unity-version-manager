use serde::Deserialize;
use uvm_core::Version;
use uvm_core::error::*;
use uvm_core::unity::{Installations, Manifest, Modules, Category};
use uvm_cli::ColorOption;
#[derive(Debug, Deserialize)]
pub struct Options {
    arg_version: Version,
    flag_category: Option<Vec<Category>>,
    flag_verbose: bool,
    flag_debug: bool,
    flag_color: ColorOption,
}

impl Options {
    pub fn version(&self) -> &Version {
        &self.arg_version
    }

    pub fn category(&self) -> Option<&Vec<Category>> {
        self.flag_category.as_ref()
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

pub fn load_modules<V: AsRef<Version>>(version:V) -> Result<Modules> {
    let version = version.as_ref();
    let manifest = Manifest::load(version)?;
    let modules:Modules = manifest.into();
    Ok(modules)
}
