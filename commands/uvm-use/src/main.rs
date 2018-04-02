extern crate console;
extern crate uvm_cli;
extern crate uvm_core;

use std::process;
use console::style;
use uvm_cli::UseOptions;

const USAGE: &'static str = "
uvm-use - Use specific version of unity.

Usage:
  uvm-use [options] <version>
  uvm-use (-h | --help)

Options:
  -v, --verbose     print more output
  -h, --help        show this help message and exit
";

fn main() {
    let options:UseOptions = uvm_cli::get_options(USAGE).unwrap();
    if uvm_core::is_active(options.version()) {
        let message = format!("Version {} already active", options.version());
        eprintln!("{}", style(message).red());
        process::exit(1);
    }

    uvm_core::find_installation(&options.version())
        .and_then(uvm_core::activate)
        .unwrap_or_else(|err| {
            eprintln!("{}", style(err).red());
            process::exit(1);
        });

    let message = format!("Using {}", &options.version());
    eprintln!("{}", style(message).green().bold());
}
