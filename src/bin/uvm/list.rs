extern crate uvm;
extern crate console;

use console::Style;
use console::Term;
use uvm::cli::ListOptions;

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
    let options:ListOptions = uvm::cli::get_options(USAGE).unwrap();
    let term = Term::stdout();
    let current_version = uvm::current_installation().ok();

    if let Ok(installations) = uvm::list_installations() {
        Term::stderr().write_line("Installed Unity versions:").is_ok();
        let verbose = options.verbose();

        let output = installations.fold(String::new(), |out, installation| {
            let mut out_style = Style::new().cyan();
            let mut path_style = Style::new().italic().green();

            if let &Some(ref current) = &current_version {
                if current == &installation {
                    out_style = out_style.yellow().bold();
                    path_style = path_style.italic().yellow();
                }
            }
            let mut new_line = out + &format!("{}", out_style.apply_to(installation.version().to_string()));
            if verbose {
                new_line += &format!(" - {}", path_style.apply_to(installation.path().display()));
            }
            new_line += "\n";
            new_line
        });

        term.write_line(&output).is_ok();

        // for installation in installations {
        //     out_style = Style::new().cyan();
        //     path_style = Style::new().italic().green();
        //     if let &Some(ref current) = &current_version {
        //         if current == &installation {
        //             out_style = out_style.yellow().bold();
        //             path_style = Style::new().italic().yellow();
        //         }
        //     }
        //
        //     let line = if verbose {
        //         format!(
        //             "{version:>width$} - {path}",
        //             version = out_style.apply_to(installation.version().to_string()),
        //             width = longest_version,
        //             path = path_style.apply_to(installation.path().display())
        //         )
        //     } else {
        //         format!(
        //             "{version:>width$}",
        //             version = out_style.apply_to(installation.version().to_string()),
        //             width = longest_version
        //         )
        //     };
        //     term.write_line(&line).is_ok();
        // }
    }
}
