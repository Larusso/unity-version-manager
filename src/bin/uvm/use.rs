extern crate console;
extern crate uvm;

use std::process;
use std::io::Error;
use console::style;

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
    let o = uvm::cli::get_use_options(USAGE).unwrap();
    if uvm::is_active(&o.version) {
        let message = format!("Version {} already active", &o.version);
        eprintln!("{}", style(message).red());
        process::exit(1);
    }

    uvm::find_installation(&o.version)
        .and_then(uvm::activate)
        .unwrap_or_else(|err| {
            eprintln!("{}", style(err).red());
            process::exit(1);
        });

    let message = format!("Using {}", &o.version);
    eprintln!("{}", style(message).green().bold());
}
