extern crate console;
extern crate uvm_cli;
extern crate uvm_version;
#[macro_use]
extern crate log;

use console::style;
use console::Term;
use std::process;
use uvm_version::FetchOptions;

const USAGE: &str = "
uvm-fetch-version - Fetch a version based on a filter.

Usage:
  uvm-version [options] latest [<release-type>]
  uvm-version [options] matching <version-req> [<release-type>]
  uvm-version (-h | --help)

Options:
  -v, --verbose     print more output
  -d, --debug       print debug output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
";

fn main() -> std::io::Result<()> {
    let stdout = Term::stderr();
    let options: FetchOptions = uvm_cli::get_options(USAGE)?;

    uvm_version::exec(&options).unwrap_or_else(|err| {
        let message = "No version found.";
        stdout.write_line(&format!("{}", style(message).red())).ok();
        info!("{}", &format!("{}", style(err).red()));
        process::exit(1);
    });

    Ok(())
}
