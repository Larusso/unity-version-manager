extern crate console;
extern crate uvm;

use std::process;
use std::io::Error;
use console::style;
use uvm::cli::UseOptions;

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
    let options:UseOptions = uvm::cli::get_options(USAGE).unwrap();
    if uvm::is_active(&options.version()) {
        let message = format!("Version {} already active", &options.version());
        eprintln!("{}", style(message).red());
        process::exit(1);
    }

    uvm::find_installation(&options.version())
        .and_then(uvm::activate)
        .unwrap_or_else(|err| {
            eprintln!("{}", style(err).red());
            process::exit(1);
        });

    let message = format!("Using {}", &options.version());
    eprintln!("{}", style(message).green().bold());
}
