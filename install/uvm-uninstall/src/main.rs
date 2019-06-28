
use uvm_cli;
use uvm_uninstall;

#[macro_use]
extern crate log;

use console::style;
use console::Term;
use std::process;
use uvm_uninstall::UninstallOptions;

const USAGE: &str = "
uvm-uninstall - Uninstall specified unity version.

Usage:
  uvm-uninstall [options] <version>
  uvm-uninstall (-h | --help)

Options:
  -a, --all         uninstall all support packages
  --android         uninstall android support for editor
  --ios             uninstall ios support for editor
  --webgl           uninstall webgl support for editor
  --mobile          uninstall mobile support (android, ios, webgl)
  --linux           uninstall linux support for editor
  --windows         uninstall windows support for editor
  --desktop         uninstall desktop support (linux, windows)
  -v, --verbose     print more output
  -d, --debug       print debug output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
";

fn main() -> std::io::Result<()> {
    let stdout = Term::stderr();
    let options: UninstallOptions = uvm_cli::get_options(USAGE)?;
    uvm_uninstall::UvmCommand::new()
        .exec(&options)
        .unwrap_or_else(|err| {
            let message = "Failure during deinstallation";
            stdout.write_line(&format!("{}", style(message).red())).ok();
            info!("{}", &format!("{}", style(err).red()));
            process::exit(1);
        });

    stdout.write_line(&format!("{}", style("Finish").green().bold()))
}
