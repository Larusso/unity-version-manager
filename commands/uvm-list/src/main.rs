extern crate uvm_cli;
extern crate uvm_list;

use uvm_list::ListOptions;

const USAGE: &str = "
uvm-list - List installed unity versions.

Usage:
  uvm-list [options]
  uvm-list (-h | --help)

Options:
  -p, --path        print only the path to the current version
  -v, --verbose     print more output
  -d, --debug       print debug output
  --hub             print unity hub installations
  --all             print all unity installations
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
";

fn main() {
    let options: ListOptions = uvm_cli::get_options(USAGE).unwrap();
    uvm_list::UvmCommand::new().exec(&options).unwrap();
}
