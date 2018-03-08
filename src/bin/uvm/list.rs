extern crate uvm;
use uvm::cmd::list::list;

const USAGE: &'static str = "
uvm-list - List installed unity versions.

Usage:
  uvm-list
  uvm-list (-h | --help)

Options:
  -h, --help        show this help message and exit
";

fn main() {
  uvm::cli::get_options(USAGE);
  list();
}