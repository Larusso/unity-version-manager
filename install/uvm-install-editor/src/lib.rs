#[macro_use]
extern crate serde_derive;
extern crate uvm_cli;
extern crate uvm_core;
extern crate console;

use std::path::PathBuf;
use console::style;
use console::Term;
use std::io;
use uvm_cli::ColorOption;
use uvm_core::install;

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
    stdout: Term,
    stderr: Term
}

impl UvmCommand {
    pub fn new() -> UvmCommand {
        UvmCommand {
            stdout: Term::stdout(),
            stderr: Term::stderr(),
        }
    }

    pub fn exec(&self, options:Options) -> io::Result<()> {
        install::install_editor(options.installer(), options.destination())?;
        self.stderr.write_line(&format!("{}", style("success").green().bold()))
    }
}
