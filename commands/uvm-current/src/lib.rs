use uvm_cli;
use uvm_core;
use serde_derive::Deserialize;

use console::style;
use std::io;
use uvm_cli::{ColorOption, Options};

#[derive(Debug, Deserialize)]
pub struct CurrentOptions {
    flag_verbose: bool,
    flag_path: bool,
    flag_color: ColorOption,
}

impl CurrentOptions {
    pub fn path_only(&self) -> bool {
        self.flag_path
    }
}

impl uvm_cli::Options for CurrentOptions {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }

    fn color(&self) -> &ColorOption {
        &self.flag_color
    }
}

pub struct UvmCommand {}

impl Default for UvmCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl UvmCommand {
    pub fn new() -> UvmCommand {
        UvmCommand {}
    }

    pub fn exec(&self, options: &CurrentOptions) -> io::Result<()> {
        if let Ok(installation) = uvm_core::current_installation() {
            let verbose = options.verbose();
            let line = if verbose {
                format!(
                    "{version} - {path}",
                    version = style(installation.version().to_string()).cyan(),
                    path = style(installation.path().display()).italic().green()
                )
            } else {
                format!(
                    "{version}",
                    version = style(installation.version().to_string()).cyan(),
                )
            };
            println!("{}", &line);
        } else {
            println!("No active version");
        }

        Ok(())
    }
}
