extern crate uvm_cli;
extern crate uvm_current;

use uvm_current::CurrentOptions;

const USAGE: &str = "
uvm-current - Prints current activated version of unity.

Usage:
  uvm-current [options]
  uvm-current (-h | --help)

Options:
  -p, --path        print only the path to the current version
  -v, --verbose     print more output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
";

fn main() {
    let options: CurrentOptions = uvm_cli::get_options(USAGE).unwrap();
    uvm_current::UvmCommand::new().exec(&options).unwrap();
}
