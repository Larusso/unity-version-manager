#[macro_use]
extern crate serde_derive;
extern crate uvm_cli;
extern crate uvm_core;
extern crate console;

use console::style;
use console::Term;
use std::io;
use uvm_cli::ColorOption;
use uvm_cli::Options;

#[derive(Debug, Deserialize)]
pub struct CurrentOptions {
    flag_verbose: bool,
    flag_path: bool,
    flag_color: ColorOption
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

    pub fn exec(&self, options:CurrentOptions) -> io::Result<()>
    {
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
            self.stdout.write_line(&line).is_ok();
        } else {
            self.stderr.write_line("No active version").is_ok();
        }

        Ok(())
    }
}
