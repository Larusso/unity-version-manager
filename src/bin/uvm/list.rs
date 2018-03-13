#[macro_use]
extern crate uvm;
use uvm::cmd::list::list;
use uvm::cmd::list::Installation;

const USAGE: &'static str = "
uvm-list - List installed unity versions.

Usage:
  uvm-list [options]
  uvm-list (-h | --help)

Options:
  -v, --verbose     print more output
  -h, --help        show this help message and exit
";

fn longest_version(installations: &Vec<Installation>) -> usize {
    match installations.iter().map(|i| i.version.to_string().len()).max() {
        Some(l) => l,
        None => 0,
    }
}

fn main() {
  let o = uvm::cli::get_list_options(USAGE);
  if let Ok(installations) = list() {
    let verbose = o.unwrap_or(uvm::cli::ListOptions{verbose:false}).verbose;
    let longest_version = longest_version(&installations);
    for installation in installations {
        if verbose {
            println!("{version:>width$} - {path}", version = installation.version.to_string(), width = longest_version, path = installation.path.display());
        }
        else {
            println!("{version:>width$}", version = installation.version.to_string(), width = longest_version);
        }
    }
  }
}
