extern crate uvm;
extern crate console;

use uvm::cmd::list::list;
use uvm::unity::Installation;
use console::style;
use console::Term;
use console::pad_str;

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
    match installations
        .iter()
        .map(|i| i.version().to_string().len())
        .max()
    {
        Some(l) => l,
        None => 0,
    }
}

fn main() {
    let o = uvm::cli::get_list_options(USAGE);
    let term = Term::stdout();
    if let Ok(installations) = list() {
        let verbose = o.unwrap_or(uvm::cli::ListOptions { verbose: false }).verbose;
        let longest_version = longest_version(&installations);
        for installation in installations {
            let line = if verbose {
                format!(
                    "{version:>width$} - {path}",
                    version = style(installation.version().to_string()).cyan(),
                    width = longest_version,
                    path = style(installation.path().display()).italic().green()
                )
            } else {
                format!(
                    "{version:>width$}",
                    version = style(installation.version().to_string()).cyan(),
                    width = longest_version
                )
            };
            term.write_line(&line);
        }
    }
}
