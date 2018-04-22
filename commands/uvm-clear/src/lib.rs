#[macro_use]
extern crate serde_derive;
extern crate uvm_cli;
extern crate uvm_core;
extern crate console;

use console::style;
use console::Term;
use std::fs;
use std::io;
use std::path::Path;
use uvm_cli::ColorOption;
use uvm_cli::Options;

#[derive(Debug, Deserialize)]
pub struct ClearOptions {
    flag_verbose: bool,
    flag_color: ColorOption
}

impl uvm_cli::Options for ClearOptions {
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

const UNITY_CURRENT_LOCATION: &'static str = "/Applications/Unity";

impl UvmCommand {
    pub fn new() -> UvmCommand {
        UvmCommand {
            stdout: Term::stdout(),
            stderr: Term::stderr(),
        }
    }

    pub fn exec(&self, options:ClearOptions) -> io::Result<()>
    {
        let active_path = Path::new(UNITY_CURRENT_LOCATION);
        if !active_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No active unity version"));
        }

        if options.verbose() {
            let installation = uvm_core::current_installation()?;
            self.stderr.write_line(&format!(
                "Clear active unity version: {} at: {}",
                style(installation.version().to_string()).yellow(),
                style(installation.path().display()).green(),
            ))?;
        }

        fs::remove_file(active_path)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to clear active version"))?;
        self.stderr.write_line(&format!("{}", style("success").green()))?;
        Ok(())
    }
}
