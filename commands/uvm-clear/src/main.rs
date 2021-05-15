
use uvm_clear;
use uvm_cli;


use uvm_clear::ClearOptions;

const USAGE: &str = "
uvm-clear - [DEPRECATED] Remove the link so you can install a new version without overwriting.

Usage:
  uvm-clear [options]
  uvm-clear (-h | --help)

Options:
  -v, --verbose     print more output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
";

fn main() {
    let options: ClearOptions = uvm_cli::get_options(USAGE).unwrap();
    uvm_clear::UvmCommand::new().exec(&options).unwrap();
}
