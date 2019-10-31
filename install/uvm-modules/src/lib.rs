use std::ops::Deref;
use serde::Deserialize;
use uvm_core::Version;
use uvm_core::error::*;
use uvm_core::unity::{Manifest, Modules, Category};
use uvm_cli::ColorOption;

#[derive(Debug, Deserialize)]
pub struct Options {
    arg_version: Version,
    flag_category: Option<Vec<Category>>,
    flag_verbose: bool,
    flag_debug: bool,
    flag_all: bool,
    flag_show_sync_modules: bool,
    flag_color: ColorOption,
}

impl Options {
    pub fn version(&self) -> &Version {
        &self.arg_version
    }

    pub fn category(&self) -> Option<&Vec<Category>> {
        self.flag_category.as_ref()
    }

    pub fn all(&self) -> bool {
        self.flag_all
    }

    pub fn show_sync_modules(&self) -> bool {
        self.flag_show_sync_modules
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

pub struct Module<'a> {
    base: &'a uvm_core::unity::Module,
    children: Vec<Module<'a>>,
}

impl<'a> Deref for Module<'_> {
    type Target = uvm_core::unity::Module;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'a> Module<'a> {
    pub fn new(module:&'a uvm_core::unity::Module, lookup:&'a [uvm_core::unity::Module]) -> Self {
        let mut children = Vec::new();
        let base = module;

        for m in lookup.iter() {
            match m.sync {
                Some(id) if id == base.id => children.push(Module::new(m, &lookup)),
                _ => ()
            }
        }

        Module {base, children}
    }

    pub fn children(&self) -> &Vec<Module<'a>> {
        &self.children
    }
}

pub fn load_modules<V: AsRef<Version>>(version:V) -> Result<Modules> {
    let version = version.as_ref();
    let manifest = Manifest::load(version)?;
    let modules:Modules = manifest.into_modules();
    Ok(modules)
}
