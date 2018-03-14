extern crate uvm;
extern crate console;

use std::path::Path;
use uvm::unity::{Installation, Version};
use console::style;
use std::str::FromStr;
use console::Term;

const USAGE: &'static str = "
uvm-current - Prints current activated version of unity.

Usage:
  uvm-current [options]
  uvm-current (-h | --help)

Options:
  -v, --verbose     print more output
  -h, --help        show this help message and exit
";

fn main() {
    let o = uvm::cli::get_list_options(USAGE);
    let path = Path::new("/Applications/Unity");
    let errorTerm = Term::stderr();
    let outTerm = Term::stdout();
    if let Ok(metadata) = path.symlink_metadata() {
        if metadata.file_type().is_symlink() {
            let linked_file = path.read_link().unwrap();
            let installation = Installation::new(linked_file).expect("Can't read current version");
            let verbose = o.unwrap_or(uvm::cli::ListOptions { verbose: false }).verbose;
            let line = if verbose {
                format!(
                    "{version} - {path}",
                    version = style(installation.version().to_string()).cyan(),
                    path = style(installation.path().display()).italic().green()
                )
            } else {
                format!(
                    "{version}",
                    version = style(installation.version().to_string()).cyan(),
                )
            };
            outTerm.write_line(&line);

        }
        else {
            errorTerm.write_line("/Applications/Unity is not a symlink");
        }
    }
    else {
        errorTerm.write_line("No active version");
    }

}
