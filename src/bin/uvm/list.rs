extern crate uvm;
extern crate console;

use uvm::cmd::list::list;
use uvm::cmd::current::current;
use uvm::unity::Installation;
use console::Style;
use console::Term;

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
    let current_version = current().ok();

    if let Ok(installations) = list() {
        let verbose = o.unwrap_or(uvm::cli::ListOptions { verbose: false }).verbose;
        let longest_version = longest_version(&installations);
        let mut out_style;
        let mut path_style;

        for installation in installations {
            out_style = Style::new().cyan();
            path_style = Style::new().italic().green();
            if let &Some(ref current) = &current_version {
                if current == &installation {
                    out_style = out_style.yellow().bold();
                    path_style = Style::new().italic().yellow();
                }
            }

            let line = if verbose {
                format!(
                    "{version:>width$} - {path}",
                    version = out_style.apply_to(installation.version().to_string()),
                    width = longest_version,
                    path = path_style.apply_to(installation.path().display())
                )
            } else {
                format!(
                    "{version:>width$}",
                    version = out_style.apply_to(installation.version().to_string()),
                    width = longest_version
                )
            };
            term.write_line(&line);
        }
    }
}
