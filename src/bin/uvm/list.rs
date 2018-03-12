#[macro_use]
extern crate uvm;
use uvm::cmd::list::list;

const USAGE: &'static str = "
uvm-list - List installed unity versions.

Usage:
  uvm-list [options]
  uvm-list (-h | --help)

Options:
  -v, --verbose     print more output
  -h, --help        show this help message and exit
";

fn main() {
  let o = uvm::cli::get_list_options(USAGE);
  if let Ok(installations) = list() {
    let verbose = o.unwrap_or(uvm::cli::ListOptions{verbose:false}).verbose;
    for installation in installations {
        if verbose {
            println!("{} - {}", installation.version, installation.path.display());
        }
        else {
            println!("{}", installation.version);
        }
    }
  }
}
