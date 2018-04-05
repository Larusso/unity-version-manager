extern crate console;
extern crate uvm_cli;
extern crate uvm_core;

use std::fs;
use std::io;
use std::io::{Error, ErrorKind};
use console::style;
use console::Term;
use uvm_cli::ClearOptions;
use uvm_cli::Options;
use std::path::Path;

const USAGE: &'static str = "
uvm-clear - Remove the link so you can install a new version without overwriting.

Usage:
  uvm-clear [options]
  uvm-clear (-h | --help)

Options:
  -v, --verbose     print more output
  -h, --help        show this help message and exit
";

const UNITY_CURRENT_LOCATION: &'static str = "/Applications/Unity";

fn exec(_out: Term, err: Term) -> io::Result<()> {
    let options: ClearOptions = uvm_cli::get_options(USAGE).unwrap();

    let active_path = Path::new(UNITY_CURRENT_LOCATION);
    if !active_path.exists() {
        return Err(Error::new(ErrorKind::NotFound, "No active unity version"));
    }

    if options.verbose() {
        let installation = uvm_core::current_installation()?;
        err.write_line(&format!(
            "Clear active unity version: {} at: {}",
            style(installation.version().to_string()).yellow(),
            style(installation.path().display()).green(),
        ))?;
    }

    fs::remove_file(active_path)
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to clear active version"))?;
    err.write_line(&format!("{}", style("success").green()))?;
    Ok(())
}

fn main() {
    exec(Term::stdout(), Term::stderr()).unwrap_or_else(uvm_cli::print_error_and_exit);
}
