extern crate uvm_cli;
extern crate uvm_core;
extern crate console;

use console::Style;
use console::Term;
use uvm_cli::ListOptions;
use uvm_cli::Options;

const USAGE: &'static str = "
uvm-list - List installed unity versions.

Usage:
  uvm-list [options]
  uvm-list (-h | --help)

Options:
  -p, --path        print only the path to the current version
  -v, --verbose     print more output
  -h, --help        show this help message and exit
";

fn main() {
    let options:ListOptions = uvm_cli::get_options(USAGE).unwrap();
    let term = Term::stdout();
    let current_version = uvm_core::current_installation().ok();

    if let Ok(installations) = uvm_core::list_installations() {
        Term::stderr().write_line("Installed Unity versions:").is_ok();
        let verbose = options.verbose();
        let path_only = options.path_only();

        let output = installations.fold(String::new(), |out, installation| {
            let mut out_style = Style::new().cyan();
            let mut path_style = Style::new().italic().green();

            if let &Some(ref current) = &current_version {
                if current == &installation {
                    out_style = out_style.yellow().bold();
                    path_style = path_style.italic().yellow();
                }
            }
            let mut new_line = out;

            if path_only == false {
                new_line += &format!("{}", out_style.apply_to(installation.version().to_string()));
            }

            if verbose {
                new_line += " - ";
            }

            if verbose || path_only {
                new_line += &format!("{}", path_style.apply_to(installation.path().display()));
            }
            new_line += "\n";
            new_line
        });

        term.write_line(&output).is_ok();
    }
}
