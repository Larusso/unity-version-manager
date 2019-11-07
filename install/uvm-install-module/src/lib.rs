#[macro_use]
extern crate serde_derive;

use uvm_cli;

use console::style;
use console::Term;
use std::io;
use std::path::PathBuf;
use uvm_cli::ColorOption;
use uvm_install_core as install;

#[derive(Debug, Deserialize)]
pub struct Options {
    arg_installer: PathBuf,
    arg_destination: PathBuf,
    flag_verbose: bool,
    flag_color: ColorOption,
}

impl Options {
    pub fn installer(&self) -> &PathBuf {
        &self.arg_installer
    }

    pub fn destination(&self) -> &PathBuf {
        &self.arg_destination
    }
}

impl uvm_cli::Options for Options {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }

    fn color(&self) -> &ColorOption {
        &self.flag_color
    }
}

pub struct UvmCommand {
    stderr: Term,
}

impl Default for UvmCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl UvmCommand {
    pub fn new() -> UvmCommand {
        UvmCommand {
            stderr: Term::stderr(),
        }
    }

    pub fn exec(&self, options: &Options) -> io::Result<()> {
        #[cfg(windows)]
        install::install_module(options.installer(), Some(options.destination()), None)?;
        #[cfg(unix)]
        install::install_module(options.installer(), Some(options.destination()))?;

        self.stderr
            .write_line(&format!("{}", style("success").green().bold()))
    }
}
