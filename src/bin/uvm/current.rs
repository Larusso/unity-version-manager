extern crate console;
extern crate uvm;

use console::style;
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
    let error_term = Term::stderr();
    let out_term = Term::stdout();

    if let Ok(installation) = uvm::cmd::current::current() {
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
        out_term.write_line(&line).is_ok();
    } else {
        error_term.write_line("No active version").is_ok();
    }
}
