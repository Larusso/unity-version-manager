#[macro_use]
extern crate serde_derive;

use uvm_cli;
use uvm_core;
#[macro_use]
extern crate error_chain;

use console::style;
use console::Term;
use std::fs;
use std::path::Path;
use uvm_cli::ColorOption;
use uvm_cli::Options;

mod error {
    use uvm_core::error::{UvmError, UvmErrorKind};

    error_chain! {
        foreign_links {
            Fmt(::std::fmt::Error);
            Io(::std::io::Error);
        }

        links {
            Uvm(UvmError, UvmErrorKind);
        }
    }
}

use self::error::*;

#[derive(Debug, Deserialize)]
pub struct ClearOptions {
    flag_verbose: bool,
    flag_color: ColorOption,
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
    stderr: Term,
}

const UNITY_CURRENT_LOCATION: &str = "/Applications/Unity";

impl Default for UvmCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl UvmCommand {
    pub fn new() -> UvmCommand {
        UvmCommand {
            stdout: Term::stdout(),
            stderr: Term::stderr(),
        }
    }

    pub fn exec(&self, options: &ClearOptions) -> Result<()> {
        let active_path = Path::new(UNITY_CURRENT_LOCATION);
        if !active_path.exists() {
            return Err("No active unity version".into());
        }

        if options.verbose() {
            let installation = uvm_core::current_installation()?;
            self.stderr.write_line(&format!(
                "Clear active unity version: {} at: {}",
                style(installation.version().to_string()).yellow(),
                style(installation.path().display()).green(),
            ))?;
        }

        fs::remove_file(active_path).chain_err(|| "Failed to clear active version")?;
        self.stdout
            .write_line(&format!("{}", style("success").green()))?;
        Ok(())
    }
}
