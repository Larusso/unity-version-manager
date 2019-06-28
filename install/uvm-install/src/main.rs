

use uvm_cli;
use uvm_install;

#[macro_use]
extern crate log;

use console::style;
use console::Term;
use std::process;

const USAGE: &str = "
uvm-install - Install specified unity version.

Usage:
  uvm-install [options] <version> [<destination>]
  uvm-install (-h | --help)

Options:
  -a, --all         install all support packages
  --android         install android support for editor
  --ios             install ios support for editor
  --webgl           install webgl support for editor
  --mobile          install mobile support (android, ios, webgl)
  --linux           install linux support for editor
  --windows         install windows support for editor
  --desktop         install desktop support (linux, windows)
  --verify          verify installer
  --no-verify       skip installer verification
  -v, --verbose     print more output
  -d, --debug       print debug output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit

Arguments:
  <version>         The unity version to install in the form of `2018.1.0f3`
  <destination>     A directory to install the requested version to
";

fn main() -> std::io::Result<()> {
    let stdout = Term::stderr();
    let options: uvm_install::Options = uvm_cli::get_options(USAGE)?;

    uvm_install::UvmCommand::new()
        .exec(&options)
        .unwrap_or_else(|err| {
            let message = "Failure during installation";
            stdout.write_line(&format!("{}", style(message).red())).ok();
            info!("{}", &format!("{}", style(err).red()));
            process::exit(1);
        });

    stdout.write_line(&format!("{}", style("Finish").green().bold()))
}
